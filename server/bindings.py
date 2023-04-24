"""
Types for the server
Created by sheepy0125 and Brushfire257
MIT License | 2023-02-20
"""

### Setup ###
# Imports
from json import dumps


### Helper functions ###


def unsigned_float(value: float | int | str) -> float:
    """A non-negative float value.

    :raises ServerException: If the value is not a valid float
    :raises ServerException: If the value is negative
    """

    if not isinstance(value, float):
        try:
            value = float(value)
        except Exception as e:
            raise ServerException(
                enum_variant=Error.MalformedResponseTypeError, inner=e
            )

    if value < 0.0:
        raise ServerException(
            enum_variant=Error.MalformedResponseTypeError,
            inner=TypeError(f"Negative unsigned float: {value}"),
        )

    return value


def unsigned_int(value: int | float | str) -> int:
    """A non-negative integer value.

    :raises ServerException: If the value is not a valid integer
    :raises ServerException: If the value is negative
    """

    if not isinstance(value, int):
        try:
            value = int(value)
        except Exception as e:
            raise ServerException(
                enum_variant=Error.MalformedResponseTypeError, inner=e
            )

    if value < 0:
        raise ServerException(
            enum_variant=Error.MalformedRequestTypeError,
            inner=TypeError(f"Negative unsigned integer: {value}"),
        )

    return value


### Classes ###


class ServerException(Exception):
    """A custom server exception"""

    def __init__(self, *args, enum_variant: int, inner: Exception):
        super().__init__(*args)

        self.enum_variant = enum_variant
        self.inner = inner


### Enums ###


class Enum:
    @classmethod
    def lookup_by_prefix(cls, prefix: str) -> int | None:
        ...

    @classmethod
    def lookup_by_variant(cls, variant: int) -> str | None:
        ...


class Error(Enum):
    """This enum is non serializable"""

    MalformedRequestFailedPrefixParsing = 0
    MalformedRequestFailedCommandParsing = 1
    MalformedRequestFailedSeparatorParsing = 2
    MalformedRequestFailedArgumentsParsing = 3
    MalformedRequestFailedMetadataParsing = 4
    MalformedRequestTypeError = 5
    MalformedRequestOtherError = 6
    MalformedResponseTypeError = 10
    MalformedResponseOtherError = 11
    FailedToStartAlreadyStarted = 21
    FailedToStartMagnetOdometerFailed = 22
    FailedToStartMotorControlFailed = 23
    FailedToStartCouldNotAcquireDistanceLock = 24
    FailedToStopNotStarted = 25
    FailedToStopStartThreadWouldNotRespond = 26
    FailedStatusCouldNotAcquireDistanceLock = 27
    FailedPingNegativeLatency = 27
    AnyOtherError = 99


class Direction(Enum):
    """This enum is non serializable"""

    Backward = -1
    Stopped = 0
    Forward = 1


class GPIOPin(Enum):
    """This enum is non serializable"""

    MotorControllerForwardPositive = 17  # GPIO_GEN0
    MotorControllerForwardNegative = 27  # GPIO_GEN2
    MotorControllerBackwardPositive = 23  # GPIO_GEN4
    MotorControllerBackwardNegative = 24  # GPIO_GEN5
    MagnetHallEffectSensor = 25  # GPIO_GEN6


class TransitMode(Enum):
    ClientToServerRequest = 0
    ServerToClientResponse = 1

    @classmethod
    def lookup_by_prefix(cls, prefix: str) -> int | None:
        """Lookup the transit mode's variant by its prefix

        :raises ValueError: If the prefix is not 1-lengthed
        """

        # Ensure the prefix is valid
        if len(prefix) != 1:
            raise ValueError("Prefix is not 1-lengthed")

        match prefix:
            case "?":
                return cls.ClientToServerRequest
            case "~":
                return cls.ServerToClientResponse
            case _:
                return None

    @classmethod
    def lookup_by_variant(cls, variant: int) -> str | None:
        match variant:
            case cls.ClientToServerRequest:
                return "?"
            case cls.ServerToClientResponse:
                return "~"
            case _:
                return None


class TransitType(Enum):
    Request = 0
    Response = 1

    @classmethod
    def lookup_by_prefix(cls, prefix: str) -> int | None:
        """Lookup the transit type's variant by its prefix

        :raises ValueError: If the prefix is not 1-lengthed
        """

        # Ensure the prefix is valid
        if len(prefix) != 1:
            raise ValueError("Prefix is not 1-lengthed")

        match prefix:
            case "?":
                return cls.Request
            case "~":
                return cls.Response
            case _:
                return None


class Command(Enum):
    Ping = 0
    Start = 1
    Stop = 2
    StaticStatus = 3
    Status = 4
    Unknown = Error = 99

    @classmethod
    def lookup_by_name(cls, name: str) -> int | None:
        """Lookup a command's variant by its name"""

        uppercase_name = name.upper()
        match uppercase_name:
            case "PING":
                return cls.Ping
            case "START":
                return cls.Start
            case "STOP":
                return cls.Stop
            case "STATICSTATUS":
                return cls.StaticStatus
            case "STATUS":
                return cls.Status
            case "ERROR" | "UNKNOWN":
                return cls.Error
            case _:
                return None

    @classmethod
    def lookup_by_variant(cls, variant: int) -> str | None:
        """Lookup a command's name by its variant"""

        match variant:
            case cls.Ping:
                return "PING"
            case cls.Start:
                return "START"
            case cls.Stop:
                return "STOP"
            case cls.Status:
                return "STATUS"
            case cls.StaticStatus:
                return "STATICSTATUS"
            case cls.Unknown:
                return "UNKNOWN"
            case _:
                return None


### Structs ###


class SerializableStruct:
    def __init__(self, *_args, **_kwargs):
        ...

    @property
    def __dict__(self) -> dict:
        return {}

    def __str__(self) -> str:
        return dumps(self.__dict__)


# Metadata


class MetaData:
    def __init__(self, time: float):
        self.time = unsigned_float(time)

    @property
    def __dict__(self) -> dict:
        return {"time": self.time}


# Error


class ErrorResponse(SerializableStruct):
    def __init__(self, error_variant: int, message: str):
        self.error_variant = error_variant
        self.message = message

    @property
    def __dict__(self) -> dict:
        return {"error_variant": self.error_variant, "message": self.message}


# Ping


class PingArguments(SerializableStruct):
    def __init__(self, time: float):
        self.time = time

    @property
    def __dict__(self) -> dict:
        return {"time": self.time}


class PingResponse(SerializableStruct):
    def __init__(self, sent_time: float):
        self.sent_time = sent_time

    @property
    def __dict__(self) -> dict:
        return {"sent_time": self.sent_time}


# Start


class StartArguments(SerializableStruct):
    def __init__(self, distance: float, reverse_brake: bool):
        self.distance = unsigned_float(distance)
        self.reverse_brake = bool(reverse_brake)

    @property
    def __dict__(self) -> dict:
        return {
            "distance": self.distance,
            "reverse_brake": self.reverse_brake,
        }


class StartResponse(SerializableStruct):
    ...


# Stop


class StopArguments(SerializableStruct):
    ...


class StopResponse(SerializableStruct):
    ...


# Static status


class StaticStatusArguments(SerializableStruct):
    ...


class StaticStatusResponse(SerializableStruct):
    def __init__(self, number_of_magnets: int, wheel_diameter: float):
        self.number_of_magnets = number_of_magnets
        self.wheel_diameter = wheel_diameter

    @property
    def __dict__(self) -> dict:
        return {
            "number_of_magnets": self.number_of_magnets,
            "wheel_diameter": self.wheel_diameter,
        }


# Dynamic (regular) status


class DistanceInformation(SerializableStruct):
    def __init__(
        self,
        distance: float,
        velocity: float,
        magnet_hit_counter: int,
    ):
        self.distance = unsigned_float(distance)
        self.velocity = unsigned_float(velocity)
        self.magnet_hit_counter = unsigned_int(magnet_hit_counter)

    @property
    def __dict__(self) -> dict:
        return {
            "distance": self.distance,
            "velocity": self.velocity,
            "magnet_hit_counter": self.magnet_hit_counter,
        }


class StatusArguments(SerializableStruct):
    ...


class StatusResponse(SerializableStruct):
    def __init__(
        self,
        running: bool,
        uptime: int,
        runtime: int,
        distance: DistanceInformation,
    ):
        self.running = bool(running)
        self.uptime = unsigned_int(uptime)
        self.runtime = unsigned_int(runtime)
        self.distance = distance.__dict__

    @property
    def __dict__(self) -> dict:
        return {
            "running": self.running,
            "uptime": self.uptime,
            "runtime": self.runtime,
            "distance": self.distance,
        }


### Lookup tables ###


COMMAND_SERIALIZABLE_LUT = {
    Command.Ping: {
        TransitType.Request: PingArguments,
        TransitType.Response: PingResponse,
    },
    Command.Start: {
        TransitType.Request: StartArguments,
        TransitType.Response: StartResponse,
    },
    Command.Stop: {
        TransitType.Request: StopArguments,
        TransitType.Response: StopResponse,
    },
    Command.Status: {
        TransitType.Request: StatusArguments,
        TransitType.Response: StatusResponse,
    },
    Command.StaticStatus: {
        TransitType.Request: StaticStatusArguments,
        TransitType.Response: StaticStatusResponse,
    },
}
