"""
Handles all events sent and received through the serial interface through callbacks
Created by sheepy0125 | MIT License | 2023-02-20
"""

### Setup ###
# Imports
from serial import Serial
from json import loads, dumps
from asyncio import sleep
from bindings import (
    Command,
    SerializableStruct,
    TransitType,
    TransitMode,
    ServerException,
    Error,
    ErrorResponse,
    MetaData,
    COMMAND_SERIALIZABLE_LUT,
)
from motor_controller import Motor

from utils import Logger
from shared import (
    SERIAL_DELAY_TIME_SECONDS,
    SERIAL_BAUD_RATE,
    SERIAL_BYTE_SIZE,
    SERIAL_STOP_BITS,
    SERIAL_PARITY,
    GPIO_DEBOUNCE_TIME_MS,
)
from time import time as unix_epoch
import RPi.GPIO as GPIO

### Classes ###


class SerialEvent:
    """A serial event to be used for storing context

    :param command: The :class:`Command` enum variant of the command
    :type command: int
    :param transit_mode: The :class:`TransitMode` enum variant determining which mode it is in
    :type type: int
    :param transit_type: The :class:`TransitType` enum variant determining which type it is in
    :type type: int
    :param value: The value, if applicable, as an instance of a serializable struct type.
        Must be the same type as what is passed into type.
    :type value: SerializableStruct
    """

    def __init__(
        self,
        command: int,
        transit_type: int,
        transit_mode: int,
        value: SerializableStruct | None,
        metadata: MetaData,
    ):
        self.command = command
        self.transit_type = transit_type
        self.transit_mode = transit_mode
        self.value = value
        self.metadata = metadata


class SerialEventContext:
    """Context of events that have been sent and received"""

    # A buffer for the serial RX incase there is no `\n` available yet
    serial_rx_buffer: str = ""


class SerialEventPropagator:
    """A serial event propagator

    This will connect to the serial connection and await for events, parsing them
    and calling their respective callbacks for what to send back.

    :param callbacks_lut: A callback lookup table, with the key being a :class:`Command`
        enum variant and value being a callable. The callable must take in :class:`Event`
        as its only argument and return a :class:`SerializableStruct`, which will be sent
        back to the caller.
    :type callbacks_lut: Dict[int, Callable]
    """

    def __init__(self, serial_port: str, callbacks_lut: dict[int, callable]):
        try:
            self.serial = self.connect_to_serial(serial_port)
        except Exception:
            Logger.fatal("Failed to connect to the serial connection")
            raise
        Logger.info(f"Connected to serial port {serial_port}")
        self.callbacks_lut = callbacks_lut

    @staticmethod
    def connect_to_serial(serial_port: str) -> Serial:
        return Serial(
            serial_port,
            SERIAL_BAUD_RATE,
            bytesize=SERIAL_BYTE_SIZE,
            parity=SERIAL_PARITY,
            stopbits=SERIAL_STOP_BITS,
            timeout=None,
        )

    def read_from_serial(self) -> str | None:
        """Read Rx from the serial connection

        :raises UnicodeDecodeError: If the input is malformed
        """

        bytes_available = self.serial.in_waiting

        if bytes_available == 0:
            return None

        string_buffer = SerialEventContext.serial_rx_buffer
        for _ in range(bytes_available):
            byte = self.serial.read(1)

            # Don't do anything will NULL bytes
            if byte == b"\x00":
                continue

            try:
                character = byte.decode("utf-8")
            except Exception as e:
                Logger.fatal("Failed parsing character! Continuing...")
                Logger.log_error(e)
                continue
            # If we hit a newline, then stop reading
            # It will be in the *self.serial buffer* next time
            # Carriage returns are counted as newlines, as `scip` uses them upon enter
            if character == "\n" or character == "\r":
                break
            string_buffer += character
        else:  # no break
            # If we haven't hit a newline, then don't return anything yet
            # (the data isn't entirely ready yet)
            # Also store it in the buffer for the next time this method is called
            # Logger.verbose(
            # f'Received data from serial, but did not hit newline: "{string_buffer}"'
            # )
            SerialEventContext.serial_rx_buffer = string_buffer
            return None

        SerialEventContext.serial_rx_buffer = ""
        return string_buffer

    @staticmethod
    def parse_request(data: str) -> SerialEvent:
        """Parse the request data into an :class:`SerialEvent`

        :raises ServerException: If the input is malformed
        """

        # Find the prefix
        try:
            prefix = data[0]
        except IndexError:
            raise ServerException(
                enum_variant=Error.MalformedRequestFailedPrefixParsing,
                inner=ValueError("Data is 0-lengthed"),
            )

        transit_mode = TransitMode.lookup_by_prefix(prefix)
        if not transit_mode is not None:
            raise ServerException(
                enum_variant=Error.MalformedRequestFailedPrefixParsing,
                inner=ValueError("Could not determine transit mode"),
            )
        transit_type = TransitType.lookup_by_prefix(prefix)
        if not transit_type is not None:
            raise ServerException(
                enum_variant=Error.MalformedRequestFailedPrefixParsing,
                inner=ValueError("Could not determine transit type"),
            )

        # Find command
        split_data = data.split("$")
        if not len(split_data) == 3:
            raise ServerException(
                enum_variant=Error.MalformedRequestFailedSeparatorParsing,
                inner=ValueError(f"Expected 3 separators but found {len(split_data)}"),
            )
        command = split_data[0][1:]  # Ignore prefix
        if len(command) == 0:
            command = "{}"
        if not len(command) >= 2:
            raise ServerException(
                enum_variant=Error.MalformedRequestFailedArgumentsParsing,
                inner=ValueError("Arguments are too short"),
            )
        command = Command.lookup_by_name(command)
        if not command is not None:
            raise ServerException(
                enum_variant=Error.MalformedRequestFailedCommandParsing,
                inner=ValueError("Invalid command"),
            )

        # Find argument type
        argument_type = COMMAND_SERIALIZABLE_LUT[command][transit_type]

        # Parse value into a dict
        try:
            serializable_value_data = loads(split_data[1])
        except Exception as e:
            raise ServerException(
                enum_variant=Error.MalformedRequestFailedArgumentsParsing, inner=e
            )

        # Parse value into required type
        try:
            parsed_value = argument_type(**serializable_value_data)
        except Exception as e:
            raise ServerException(
                enum_variant=Error.MalformedRequestFailedArgumentsParsing, inner=e
            )

        # Parse metadata into a dict
        try:
            serializable_metadata_data = loads(split_data[2])
        except Exception as e:
            raise ServerException(
                enum_variant=Error.MalformedRequestFailedMetadataParsing, inner=e
            )

        # Parse the metadata into metadata type
        try:
            parsed_metadata = MetaData(**serializable_metadata_data)
        except Exception as e:
            raise ServerException(
                enum_variant=Error.MalformedRequestFailedMetadataParsing, inner=e
            )

        # Encapsulate everything in an :class:`SerialEvent`
        event = SerialEvent(
            command=command,
            transit_type=transit_type,
            transit_mode=transit_mode,
            value=parsed_value,
            metadata=parsed_metadata,
        )

        return event

    @staticmethod
    def parse_response(event: SerialEvent) -> str:
        """Parse the response into a str to be transmitted

        :raises ValueError: If the event is somehow malformed
        """

        # Find the prefix
        prefix = None
        # Find the inverse transit mode for the prefix
        # Only the possible values for the server are included
        transit_mode: TransitMode
        match event.transit_mode:
            case TransitMode.ClientToServerRequest:
                transit_mode = TransitMode.ServerToClientResponse
            case unreachable:
                raise ValueError(f"Unreachable reached: {unreachable} transit mode")

        prefix = TransitMode.lookup_by_variant(transit_mode)
        if prefix is None:
            raise ValueError(f"No prefix for {transit_mode}")

        # Find the command
        command = Command.lookup_by_variant(event.command)
        if command is None:
            raise ValueError(f"No command name for {event.command} variant")

        # Stringify the response
        response = dumps(event.value.__dict__)

        # Get new metadata
        metadata = dumps(MetaData(float(unix_epoch())).__dict__)

        # Encapsulate everything into a string
        return f"{prefix}{command}${response}${metadata}\r\n"

    async def main_loop(self):  # NOSONAR
        while True:
            error = None

            # Get
            rx = None
            try:
                rx = self.read_from_serial()
            except Exception as e:
                Logger.fatal(
                    "Failed getting the `rx` buffer from the serial connection!"
                )
                Logger.log_error(e)

            if rx is None:
                continue
            if len(rx) == 0:
                continue

            # Parse
            parsed_rx = None
            if rx is not None:
                try:
                    parsed_rx = self.parse_request(rx)
                    Logger.verbose(
                        f"{parsed_rx.command=} {parsed_rx.transit_type=} {parsed_rx.transit_mode=} {parsed_rx.value.__dict__=}"
                    )
                except ServerException as e:
                    Logger.fatal(f"Failed parsing {rx!r}: {e.inner!s}")
                    error = ErrorResponse(
                        error_variant=e.enum_variant, message=str(e.inner)
                    )
                except Exception as e:
                    Logger.fatal(f"Failed parsing {rx!r}, but not sure why!")
                    Logger.log_error(e)
                    error = ErrorResponse(
                        error_variant=Error.MalformedRequestOtherError, message=str(e)
                    )

            # Callback
            response = None
            if parsed_rx is not None:
                try:
                    response = self.callbacks_lut[parsed_rx.command](parsed_rx)
                    if not isinstance(response, SerializableStruct):
                        raise ServerException(
                            enum_variant=Error.MalformedResponseTypeError,
                            inner=ValueError(
                                "Callback for the event given did not respond with a valid struct"
                            ),
                        )
                    expected_struct_type = COMMAND_SERIALIZABLE_LUT[parsed_rx.command][
                        TransitType.Response
                    ]
                    if not isinstance(response, expected_struct_type):
                        raise ServerException(
                            enum_variant=Error.MalformedRequestTypeError,
                            inner=TypeError(
                                f"The response responded with {response.__class__!r} instead of "
                                f"{expected_struct_type.__class__!r}: {response.__dict__}"
                            ),
                        )
                except ServerException as e:
                    Logger.fatal(f"Callback failed for {rx!r}")
                    Logger.fatal(f"Error enum variant: {e.enum_variant}")
                    Logger.log_error(e)
                    Logger.log_error(e.inner)
                    error = ErrorResponse(e.enum_variant, str(e.inner))
                except Exception as e:
                    Logger.fatal(f"Callback failed for {rx!r} for an uncaught reason!")
                    Logger.log_error(e)
                    error = ErrorResponse(Error.AnyOtherError, str(e))
                finally:
                    # The response could be an error blocking the motors from stopping
                    # For safety, do it after *every* callback
                    Motor.stop()

            # Parse response
            to_send = None
            if response is not None:
                event = parsed_rx
                event.value = response
            else:
                if error is None:
                    error = ErrorResponse(
                        error_variant=Error.AnyOtherError,
                        message="No error but did not succeed",
                    )

                if parsed_rx is not None:
                    event = parsed_rx
                    event.command = Command.Unknown
                    event.value = error
                else:
                    event = SerialEvent(
                        command=Command.Unknown,
                        transit_type=TransitType.Response,  # Doesn't matter lol
                        transit_mode=TransitMode.ClientToServerRequest,  # Will be flopped
                        value=error,
                        metadata=MetaData(unix_epoch()),
                    )
            to_send = self.parse_response(event)

            # Transmit response
            try:
                self.serial.write(bytes(to_send, "utf-8"))
            except Exception as e:
                Logger.fatal(f"Failed to send {to_send!r}!")
                Logger.log_error(e)

            # Delay
            await sleep(SERIAL_DELAY_TIME_SECONDS)


class GPIOEvent:
    """A GPIO event used for storing context

    :param pin: An enum variant of the GPIO pin
    :type pin: int
    :param value: Whether the pin is high or low
    :type value: bool
    :param metadata: Metadata about the event
    :type metadata: MetaData
    """

    def __init__(self, pin: int, value: bool, metadata: MetaData):
        self.pin = pin
        self.value = value
        self.metadata = metadata


class GPIOEventPropagator:
    """A GPIO event propagator.

    This will set up the GPIO connections and send callbacks to them.

    :param setup_callbacks_lut: A callback lookup table, with the key being a :class:`GPIOPin`
        enum variant and value being a callable. The callable must take nothing for arguments
        and return nothing, and should be used to run `GPIO.setup(...)`.
    :type callbacks_lut: Dict[int, Callable]
    :param event_callbacks_lut: A callback lookup table, with the key being a :class:`GPIOPin`
        enum variant and the value being a callable. The callable must take in solely a
        :class:`GPIOEvent` and return nothing.
    """

    def __init__(
        self,
        setup_callbacks_lut: dict[int, callable],
        callbacks_lut: dict[int, callable],
    ):
        self.setup_callbacks_lut = setup_callbacks_lut
        self.callbacks_lut = callbacks_lut

        # Call setup callbacks
        for pin, callback in self.setup_callbacks_lut.items():
            try:
                callback()
                Logger.verbose(f"Called GPIO pin {pin}'s setup callback")
            except Exception:
                Logger.fatal(f"Failed calling setup callback for GPIO pin {pin}")
                raise

        # Set callbacks
        for pin in self.callbacks_lut.keys():
            try:
                GPIO.add_event_detect(
                    pin,
                    GPIO.BOTH,
                    callback=self.callback_propagator,
                    bouncetime=GPIO_DEBOUNCE_TIME_MS,
                )
            except Exception:
                Logger.fatal(f"Failed to add detection callback for GPIO pin {pin}")
                raise

    def callback_propagator(self, pin: int):
        """A callback propagator.

        The :mod:`GPIO` module already provides a on change callback,
        and this takes that callback and encapsulates it up in a :class:`GPIOEvent`.
        """

        # Read the value
        value = GPIO.input(pin)

        # Encapsulate everything into a :class:`GPIOEvent`
        event = GPIOEvent(pin=pin, value=value, metadata=MetaData(unix_epoch()))
        Logger.verbose(f'Callback for {pin}: {"HIGH" if value else "LOW"}')

        # Call the callback
        try:
            self.callbacks_lut[pin](event)
        except Exception as e:
            Logger.fatal(
                f'Failed to call callback for event on GPIO {pin}: {"HIGH" if value else "LOW"}'
            )
            Logger.log_error(e)
            raise
