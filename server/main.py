"""
Server software to run on the Raspberry Pi 1B onboard the car
Created by sheepy0125 and Brushfire257
MIT License | 2023-02-20
"""

### Setup ###
# Imports
from serial import Serial
from asyncio import run
from threading import Thread, Event as ThreadEvent, Lock as ThreadLock
from time import time as unix_epoch
from sys import argv
from utils import Logger
from events import SerialEventPropagator, SerialEvent, GPIOEventPropagator, GPIOEvent
from bindings import (
    Command,
    Direction,
    GPIOPin,
    MetaData,
    PingArguments,
    PingResponse,
    StartArguments,
    StartResponse,
    StopArguments,
    StopResponse,
    StaticStatusArguments,
    StaticStatusResponse,
    StatusArguments,
    StatusResponse,
    DistanceInformation,
    ServerException,
    Error,
    TransitMode,
    TransitType,
    unsigned_float,
)
from shared import (
    BACKWARD_LEEWAY_DISTANCE_CENTIMETERS,
    FORWARD_LEEWAY_DISTANCE_CENTIMETERS,
    NUMBER_OF_MAGNETS,
    STATUS_POLL_DURATION_SECONDS,
    WHEEL_DIAMETER_CENTIMETERS,
    WHEEL_DIAMETER_INCHES,
    WHEEL_CIRCUMFERENCE_CENTIMETERS,
)
from motor_controller import Motor
from RPi import GPIO


### Constants ###
START_UPTIME: float = unix_epoch()


### Classes ###
class GlobalEventPropagators:
    """Event propagators"""

    gpio: GPIOEventPropagator | None = None
    serial: SerialEventPropagator | None = None


class MutexStartData:
    """Shared thread-safe values for the start event"""

    started_time: float | None = None  # Unix epoch
    start_thread: Thread | None = None
    started_flag: ThreadEvent = ThreadEvent()
    magnet_hit_flag: ThreadEvent = ThreadEvent()
    e_stop_flag: ThreadEvent = ThreadEvent()
    distance_information_lock: ThreadLock = ThreadLock()
    distance: DistanceInformation | None = None


### Event callbacks ###
def magnet_event(event: GPIOEvent) -> None:
    """An event from the magnet hall effect sensor"""

    # Don't bother if we aren't running
    if not MutexStartData.started_flag.is_set():
        return

    if not event.value:
        if MutexStartData.magnet_hit_flag.is_set():
            Logger.warn("Magnet hit event but flag is already set!!!")
        MutexStartData.magnet_hit_flag.set()
    else:
        MutexStartData.magnet_hit_flag.clear()


def ping(event: SerialEvent) -> PingResponse:
    ping_request: PingArguments = event.value

    metadata: MetaData = event.metadata

    time = unix_epoch()

    latency = unsigned_float(time - metadata.time)
    Logger.verbose(f"One-way latency is {round(latency * 100, 5)}ms")

    return PingResponse(ping_request.time)


def start(event: SerialEvent) -> StartResponse:
    # Ensure not already started
    if MutexStartData.started_flag.is_set():
        raise ServerException(
            enum_variant=Error.FailedToStartAlreadyStarted,
            inner=ValueError("Already started!"),
        )

    arguments: StartArguments = event.value

    # Reset flags
    MutexStartData.started_flag.set()
    MutexStartData.magnet_hit_flag.clear()
    MutexStartData.e_stop_flag.clear()
    MutexStartData.started_time = unix_epoch()
    MutexStartData.distance = DistanceInformation(
        distance=0, velocity=0.0, magnet_hit_counter=0
    )

    # Run threads
    MutexStartData.start_thread = Thread(target=start_thread, args=(arguments,))
    MutexStartData.start_thread.start()

    return StartResponse()


def start_thread(arguments: StartArguments):
    should_reverse_brake = arguments.reverse_brake

    # Start the motors
    Motor.forward()

    next_status_poll = unix_epoch() + STATUS_POLL_DURATION_SECONDS

    def send_status():
        current_time = unix_epoch()
        to_send = GlobalEventPropagators.serial.parse_response(
            SerialEvent(
                command=Command.Status,
                transit_type=TransitType.Response,
                transit_mode=TransitMode.ClientToServerRequest,  # Will swap
                value=StatusResponse(
                    running=MutexStartData.started_flag.is_set(),
                    uptime=current_time - START_UPTIME,
                    runtime=current_time - MutexStartData.started_time,
                    distance=MutexStartData.distance,
                ),
                metadata=MetaData(unix_epoch()),
            )
        )
        GlobalEventPropagators.serial.serial.write(bytes(to_send, "utf-8"))

    if not MutexStartData.distance_information_lock.acquire(timeout=1.0):
        raise ServerException(
            enum_variant=Error.FailedToStartCouldNotAcquireDistanceLock,
            inner=RuntimeError("The distance lock simply was left acquired"),
        )

    while MutexStartData.started_flag.is_set():
        # E-STOP
        if MutexStartData.e_stop_flag.is_set():
            break

        # Status
        current_time = unix_epoch()
        if current_time >= next_status_poll:
            next_status_poll += STATUS_POLL_DURATION_SECONDS
            send_status()

        # Keep track of distance
        if MutexStartData.magnet_hit_flag.is_set():
            MutexStartData.magnet_hit_flag.clear()

            MutexStartData.distance.magnet_hit_counter += Motor.current_direction
            MutexStartData.distance.distance += (
                WHEEL_CIRCUMFERENCE_CENTIMETERS / NUMBER_OF_MAGNETS
            ) * Motor.current_direction
            MutexStartData.distance.velocity = MutexStartData.distance.distance / (
                unix_epoch() - MutexStartData.started_time
            )

        # Going forward
        if Motor.current_direction == Direction.Forward:
            # Exceeded distance
            if (
                MutexStartData.distance.distance + FORWARD_LEEWAY_DISTANCE_CENTIMETERS
                >= arguments.distance
            ):
                Motor.backward()

        # Reverse braking
        elif Motor.current_direction == Direction.Backward:
            # Done reverse braking
            if (
                MutexStartData.distance.distance - BACKWARD_LEEWAY_DISTANCE_CENTIMETERS
                <= arguments.distance
            ):
                Motor.stop()
                break

    stop(stop_start_thread=False)
    send_status()


def stop(*_args, stop_start_thread: bool = True, **_kwargs) -> StopResponse:
    MutexStartData.e_stop_flag.set()
    MutexStartData.started_flag.clear()
    if stop_start_thread:
        MutexStartData.start_thread.join()
    try:
        MutexStartData.distance_information_lock.release()
    except Exception:
        ...

    Motor.stop()

    return StopResponse()


def static_status(_: SerialEvent) -> StaticStatusResponse:
    """Return static status about the car"""

    return StaticStatusResponse(
        number_of_magnets=NUMBER_OF_MAGNETS, wheel_diameter=WHEEL_DIAMETER_CENTIMETERS
    )


def status(_: SerialEvent) -> StatusResponse:
    # Get distance information
    if not MutexStartData.distance_information_lock.acquire(timeout=1.0):
        return ServerException(
            enum_variant=Error.FailedStatusCouldNotAcquireDistanceLock,
            inner=RuntimeError("The distance information mutex lock was not acquired"),
        )
    distance = MutexStartData.distance
    MutexStartData.distance_information_lock.release()

    return StatusResponse(
        running=MutexStartData.started_flag.is_set(),
        runtime=(unix_epoch() - MutexStartData.started_time),
        uptime=(unix_epoch() - START_UPTIME),
        distance=distance,
    )


### Lookup tables ###
SERIAL_CALLBACKS_LUT = {
    Command.Ping: ping,
    Command.Start: start,
    Command.Stop: stop,
    Command.Status: status,
    Command.StaticStatus: static_status,
}
SETUP_GPIO_CALLBACKS_LUT = {
    GPIOPin.MagnetHallEffectSensor: lambda: GPIO.setup(
        GPIOPin.MagnetHallEffectSensor, GPIO.IN, pull_up_down=GPIO.PUD_UP
    )
}
GPIO_CALLBACKS_LUT = {
    GPIOPin.MagnetHallEffectSensor: magnet_event,
}


### Main ###
async def main():
    # Connect to serial port
    try:
        serial_port = argv[1]
    except IndexError:
        Logger.fatal(
            "You must supply the serial device as the first argument "
            "(e.g. `python3 main.py /dev/ttyACM0`)"
        )
        return

    # Create serial event handler
    try:
        GlobalEventPropagators.serial = SerialEventPropagator(
            serial_port=serial_port, callbacks_lut=SERIAL_CALLBACKS_LUT
        )
        # Create the GPIO event handler
        GlobalEventPropagators.gpio = GPIOEventPropagator(
            callbacks_lut=GPIO_CALLBACKS_LUT,
            setup_callbacks_lut=SETUP_GPIO_CALLBACKS_LUT,
        )
    except Exception as e:
        Logger.fatal("Creating event propagators failed")
        Logger.log_error(e)
        raise

    # Await and handle commands
    try:
        await GlobalEventPropagators.serial.main_loop()
    except Exception as e:
        Logger.fatal("Main loop failed")
        Logger.log_error(e)


if __name__ == "__main__":
    try:
        run(main())
    except Exception as e:
        Logger.fatal("Running failed")
        Logger.log_error(e)
    finally:
        Motor.stop()
        GPIO.cleanup()
