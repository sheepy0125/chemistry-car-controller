"""
Types for the server
Created by sheepy0125 and Brushfire257
MIT License | 2023-02-20
"""

### Setup ###
# Imports
from json import dumps

### Enums ###


class Enum:
    @classmethod
    def lookup_by_prefix(cls, prefix: str) -> int | None:
        ...

    @classmethod
    def lookup_by_variant(cls, variant: int) -> str | None:
        ...


class TransitMode(Enum):
    ClientToServerRequest = 0
    ServerToClientResponse = 1
    ServerToR41ZRequest = 2
    R41ZToServerResponse = 3

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
            case "#":
                return cls.R41ZToServerResponse
            case "@":
                return cls.ServerToR41ZRequest
            case _:
                return None

    @classmethod
    def lookup_by_variant(cls, variant: int) -> str | None:
        match variant:
            case cls.ClientToServerRequest:
                return "?"
            case cls.ServerToClientResponse:
                return "~"
            case cls.R41ZToServerResponse:
                return "#"
            case cls.ServerToR41ZRequest:
                return "@"
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
            case "@":
                return cls.Request
            case "#":
                return cls.Response
            case "~":
                return cls.Response
            case "&":
                return cls.Response
            case _:
                return None


class Command(Enum):
    Ping = 0
    Start = 1
    Stop = 2
    Status = 3

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
            case "STATUS":
                return cls.Status
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
            case _:
                return None


### Structs ###


class SerializableStruct:
    def __init__(self, *_args, **_kwargs):
        ...

    def __dict__(self) -> dict:
        return {}

    def __str__(self) -> str:
        return dumps(self.__dict__())


# Ping


class PingArguments(SerializableStruct):
    def __init__(self, time: float):
        self.time = float(time)

    def __dict__(self) -> dict:
        return {"time": self.time}


class PingResponse(SerializableStruct):
    def __init__(self, time: float):
        self.time = float(time)

    def __dict__(self) -> dict:
        return {"time": self.time}


# Start


class StartArguments(SerializableStruct):
    def __init__(self, distance: float, forward: bool, reverse_brake: bool):
        self.distance = float(distance)
        self.forward = bool(forward)
        self.reverse_brake = bool(reverse_brake)

    def __dict__(self) -> dict:
        return {
            "distance": self.distance,
            "forward": self.foward,
            "reverse_brake": self.reverse_brake,
        }


class StartResponse(SerializableStruct):
    ...


# Stop


class StopArguments(SerializableStruct):
    ...


class StopResponse(SerializableStruct):
    ...


# Status


class AccelerometerReadings(SerializableStruct):
    def __init__(self, x: float, y: float, z: float):
        self.x = float(x)
        self.y = float(y)
        self.z = float(z)

    def __dict__(self) -> dict:
        return {"x": self.x, "y": self.y, "z": self.z}


class StatusArguments(SerializableStruct):
    ...


class StatusResponse(SerializableStruct):
    def __init__(
        self,
        running: bool,
        uptime: int,
        runtime: int,
        distance: float,
        accelerometer_readings: AccelerometerReadings,
    ):
        self.running = bool(running)
        self.uptime = int(uptime)
        self.runtime = int(runtime)
        self.distance = float(distance)
        self.accelerometer_readings = dict(accelerometer_readings)

    def __dict__(self) -> dict:
        return {
            "running": self.running,
            "uptime": self.uptime,
            "runtime": self.runtime,
            "distance": self.distance,
            "accelerometer_readings": self.accelerometer_readings,
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
}
