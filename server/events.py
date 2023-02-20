"""
Handles all events sent and received through the serial interface through callbacks
Created by sheepy0125 | MIT License | 2023-02-20
"""

### Setup ###
# Imports
from serial import Serial
from bindings import (
    Command,
    SerializableStruct,
    TransitType,
    TransitMode,
    COMMAND_SERIALIZABLE_LUT,
)
from utils import Logger
from json import loads, dumps
from asyncio import sleep
from shared import SERIAL_DELAY_TIME

### Classes ###


class Event:
    """An event to be used for storing context

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
    ):
        self.command = command
        self.transit_type = transit_type
        self.transit_mode = transit_mode
        self.value = value


class EventContext:
    """Context of events that have been sent and received"""

    command_sent: Event | None = None
    command_received: Event | None = None

    # A buffer for the serial RX incase there is no `\n` available yet
    serial_rx_buffer: str = ""


class SerialEventPropagator:
    """A serial event propagator

    This will connect to the serial connection and await for events, parsing them
    and calling their respective callbacks for what to send back.

    :param serial: The serial connection
    :type serial: serial.Serial
    :param callback_lut: A callback lookup table, with the key being a :class:`Command`
        enum variant and value being a callable. The callable must take in :class:`Event`
        as its only argument and return a :class:`SerializableStruct`, which will be sent
        back to the caller.
    :type callback_lut: Dict[int, Callable]
    """

    def __init__(self, serial: Serial, callbacks_lut: dict[int, callable]):
        self.serial = serial
        self.callbacks_lut = callbacks_lut

    def read_from_serial(self) -> str | None:
        """Read Rx from the serial connection

        :raises UnicodeDecodeError: If the input is malformed
        """

        bytes_available = self.serial.in_waiting
        if bytes_available == 0:
            return None

        string_buffer = EventContext.serial_rx_buffer
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
                hit_newline = True
                break
            string_buffer += character
        else:  # no break
            # If we haven't hit a newline, then don't return anything yet
            # (the data isn't entirely ready yet)
            # Also store it in the buffer for the next time this method is called
            EventContext.serial_rx_buffer = string_buffer
            return None

        EventContext.serial_rx_buffer = ""
        return string_buffer

    @staticmethod
    def parse_request(data: str) -> Event:
        """Parse the request data into an :class:`Event`

        :raises ValueError: If the input is malformed
        """

        # Find the prefix
        prefix = data[0]

        transit_mode = TransitMode.lookup_by_prefix(prefix)
        transit_type = TransitType.lookup_by_prefix(prefix)

        # Find command
        split_data = data.split("$")
        command = split_data[0][1:]  # Ignore prefix
        command = Command.lookup_by_name(command)

        # Find argument type
        argument_type = COMMAND_SERIALIZABLE_LUT[command][transit_type]

        # Find serializable data
        serializable_data = loads(split_data[1])

        # Parse into that type
        parsed = argument_type(**serializable_data)

        # Encapsulate everything in an :class:`Event`
        event = Event(
            command=command,
            transit_type=transit_type,
            transit_mode=transit_mode,
            value=parsed,
        )

        return event

    @staticmethod
    def parse_response(event: Event) -> str:
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
            case TransitMode.R41ZToServerResponse:
                transit_mode = TransitMode.R41ZToServerResponse
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
        response = dumps(event.value.__dict__())

        # Encapsulate it all
        return f"{prefix}{command}${response}"

    async def main_loop(self):
        while True:
            # Get
            try:
                rx = self.read_from_serial()
            except Exception as e:
                Logger.fatal(
                    "Failed getting the `rx` buffer from the serial connection!"
                )
                Logger.log_error(e)

            # Parse
            parsed_rx = None
            if rx is not None:
                try:
                    parsed_rx = self.parse_request(rx)
                    Logger.verbose(
                        f"{parsed_rx.command=} {parsed_rx.transit_type=} {parsed_rx.transit_mode=} {parsed_rx.value.__dict__()=}"
                    )
                except Exception as e:
                    Logger.fatal(f"Failed parsing {rx!r}!")
                    Logger.log_error(e)

            # Callback
            response = None
            if parsed_rx is not None:
                try:
                    response = self.callbacks_lut[parsed_rx.command](parsed_rx)
                except Exception as e:
                    Logger.fatal(f"Callback failed for {rx!r}")
                    Logger.log_error(e)

            # Transmit response
            if response is not None:
                event = parsed_rx
                event.value = response
                parsed_response = self.parse_response(event)
                self.serial.write(bytes(parsed_response, "utf-8"))

            # Delay
            await sleep(0.10)
