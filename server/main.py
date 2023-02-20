"""
Server software to run on the Raspberry Pi 1B onboard the car
Created by sheepy0125 and Brushfire257
MIT License | 2023-02-20
"""

### Setup ###
# Imports
from serial import Serial
from asyncio import run
from time import time as unix_epoch
from sys import argv
from utils import Logger
from events import SerialEventPropagator, Event
from bindings import (
    Command,
    PingArguments,
    PingResponse,
    StartArguments,
    StartResponse,
    StopArguments,
    StopResponse,
    StatusArguments,
    StatusResponse,
)


### Event callbacks ###
def ping(event: Event) -> PingResponse:
    arguments: PingArguments = event.value

    time = unix_epoch()

    return PingResponse(time)


def start(event: Event):
    arguments: StartArguments = event.value

    raise NotImplementedError


def stop(event: Event):
    arguments: StopArguments = event.value

    raise NotImplementedError


def status(event: Event):
    arguments: StatusArguments = event.value

    raise NotImplementedError


### Lookup tables ###
CALLBACKS_LUT = {
    Command.Ping: ping,
    Command.Start: start,
    Command.Stop: stop,
    Command.Status: status,
}


### Main ###
async def main():
    # Connect to serial port
    serial_port = argv[1]
    serial = Serial(
        serial_port, 115200, bytesize=8, parity="N", stopbits=1, timeout=None
    )
    Logger.info(f"Connected to serial port {serial_port}")

    # Create serial event handler
    serial_event_propagator = SerialEventPropagator(serial, callbacks_lut=CALLBACKS_LUT)
    await serial_event_propagator.main_loop()


if __name__ == "__main__":
    run(main())
