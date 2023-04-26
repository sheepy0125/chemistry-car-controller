"""
Controls the motor with relays safely
Created by sheepy0125 | MIT License | 2023-04-14
"""


from RPi import GPIO
from time import sleep
from shared import MOTOR_CONTROLLER_SAFETY_DELAY
from utils import Logger
from bindings import GPIOPin, Direction


class MotorDirectionRelayPins:
    """An important note: HIGH is off while LOW is on"""

    def __init__(self, pos: int, neg: int, debug_name: str) -> None:
        self.pos = pos
        self.neg = neg
        self.activated = False  # Assumed to be deactivated and normally open
        self.debug_name = debug_name

        # Setup as outputs at HIGH
        for pin in (pos, neg):
            GPIO.setup(pin, GPIO.OUT)
            GPIO.output(pin, True)

    def __str__(self) -> str:
        return self.debug_name

    def set(self, to: bool) -> None:
        """It is ***imperative*** to ensure that the other direction
        is deactivated before doing this. Otherwise the Pi will bake!
        """

        if self.activated == to:
            raise RuntimeError(f"Already set to {to}")
        self.activated = to

        for pin in (self.pos, self.neg):
            # Remember: LOW is on and HIGH is off, so this is backwards!
            GPIO.output(pin, not to)

        if not self.ensure(to):
            raise RuntimeError(f"Set to {to=} but is {not to}")

    def ensure(self, value: bool) -> bool:
        """Ensures both pins are actually set to :param:`value`"""

        for pin in (self.pos, self.neg):
            # Remember: LOW is on and HIGH is off, so this is backwards!
            if GPIO.input(pin) == value:
                return False
        return True


class MotorControllerRelayPins:
    """This class instantiates the forward and backward pins to use globally."""

    Forward = MotorDirectionRelayPins(
        pos=GPIOPin.MotorControllerForwardPositive,
        neg=GPIOPin.MotorControllerForwardNegative,
        debug_name="forward",
    )
    Backward = MotorDirectionRelayPins(
        pos=GPIOPin.MotorControllerBackwardPositive,
        neg=GPIOPin.MotorControllerBackwardNegative,
        debug_name="backward",
    )


class Motor:
    # current_direction = Direction.Stopped

    @staticmethod
    def _transmute_relays(
        first: MotorDirectionRelayPins,
        first_to: bool,
        second: MotorDirectionRelayPins,
        second_to: bool,
        wait: bool,
    ):
        """Transmutes the motors by affecting certain relays, ensuring
        both are at the correct value and also waiting for the safety
        pause duration if so desired.

        This is a helper function for `forward`, `backward`, and `stop`.
        """

        def _set_relays(relays: MotorDirectionRelayPins, to: bool):
            try:
                relays.set(to)
            except RuntimeError:
                # Already set
                return

        def _activating_or_deactivating(to: bool) -> str:
            return "Activating" if to else "Deactivating"

        Logger.verbose(f"{_activating_or_deactivating(first_to)} {first!s} relays")
        _set_relays(first, first_to)

        if wait:
            Logger.verbose("Pausing for safety")
            sleep(MOTOR_CONTROLLER_SAFETY_DELAY)

        # Extra ensure
        if not first.ensure(first_to):
            raise RuntimeError(
                f"{_activating_or_deactivating(first_to)} {first!s} did not affect"
            )

        Logger.verbose(f"{_activating_or_deactivating(second_to)} {second!s} relays")
        _set_relays(second, second_to)

    @classmethod
    def forward(cls):
        """
        Backward POS -> LOW
        Backward NEG -> LOW
        Wait...
        Forward  POS -> HIGH
        Forward  NEG -> HIGH
        """

        Motor.current_direction = Direction.Forward

        cls._transmute_relays(
            first=MotorControllerRelayPins.Backward,
            first_to=False,
            second=MotorControllerRelayPins.Forward,
            second_to=True,
            wait=True,
        )

    @classmethod
    def backward(cls):
        """
        Forward  POS -> LOW
        Forward  NEG -> LOW
        Wait...
        Backward POS -> HIGH
        Backward NEG -> HIGH
        """

        Motor.current_direction = Direction.Backward

        cls._transmute_relays(
            first=MotorControllerRelayPins.Forward,
            first_to=False,
            second=MotorControllerRelayPins.Backward,
            second_to=True,
            wait=True,
        )

    @classmethod
    def stop(cls):
        """
        Forward  POS -> LOW
        Forward  NEG -> LOW
        Backward POS -> LOW
        Backward NEG -> LOW
        """

        Motor.current_direction = Direction.Stopped

        cls._transmute_relays(
            first=MotorControllerRelayPins.Forward,
            first_to=False,
            second=MotorControllerRelayPins.Backward,
            second_to=False,
            wait=False,
        )
