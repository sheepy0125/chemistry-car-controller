from RPi import GPIO
from time import sleep

DEBUG = True
SAFETY_WAIT = 0.1


GPIO.setwarnings(False)
GPIO.setmode(GPIO.BCM)  # Broadcom SOC Channel


def dbg(*args, **kwargs):
    if not DEBUG:
        return
    kwargs["flush"] = True
    print(*args, **kwargs)


class StatusMessages:
    Success = "SUCCESS"
    Failed = "FAILED"
    Unchanged = "UNCHANGED"


class MotorDirectionRelayPins:
    """An important note: HIGH is off while LOW is on"""

    def __init__(self, pos: int, neg: int, dbg_name: str) -> None:
        self.pos = pos
        self.neg = neg
        self.activated = False  # Assumed to be deactivated and normally open
        self.dbg_name = dbg_name

        # Setup as outputs at HIGH
        for pin in (pos, neg):
            GPIO.setup(pin, GPIO.OUT)
            GPIO.output(pin, True)

    def __str__(self) -> str:
        return self.dbg_name

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


class RelayPins:
    Forward = MotorDirectionRelayPins(pos=17, neg=27, dbg_name="forward")
    Backward = MotorDirectionRelayPins(pos=23, neg=24, dbg_name="backward")


class Motor:
    @staticmethod
    def _transmute_relays(
        first: MotorDirectionRelayPins,
        first_to: bool,
        second: MotorDirectionRelayPins,
        second_to: bool,
        wait: bool,
    ):
        """Turns one set of relays off and the other set on, ensuring
        both are at the correct value and also waiting for the safety
        pause duration

        Comes with some debug prints!
        """

        def _set_relays(relays: MotorDirectionRelayPins, to: bool):
            try:
                relays.set(to)
                dbg(StatusMessages.Success)
            except RuntimeError:
                dbg(StatusMessages.Unchanged)
            except Exception as e:
                dbg(StatusMessages.Failed + "\n" + str(e))
                raise

        def _activating_or_deactivating(to: bool) -> str:
            return "Activating" if to else "Deactivating"

        dbg(f"{_activating_or_deactivating(first_to)} {first!s} relays... ", end="")
        _set_relays(first, first_to)

        if wait:
            dbg("Pausing for safety... ", end="")
            sleep(SAFETY_WAIT)
            dbg(StatusMessages.Success)

        # Extra ensure
        if not first.ensure(first_to):
            raise RuntimeError(
                f"{_activating_or_deactivating(first_to)} {first!s} did not affect"
            )

        dbg(f"{_activating_or_deactivating(second_to)} {second!s} relays... ", end="")
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

        cls._transmute_relays(
            first=RelayPins.Backward,
            first_to=False,
            second=RelayPins.Forward,
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

        cls._transmute_relays(
            first=RelayPins.Forward,
            first_to=False,
            second=RelayPins.Backward,
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

        cls._transmute_relays(
            first=RelayPins.Forward,
            first_to=False,
            second=RelayPins.Backward,
            second_to=False,
            wait=False,
        )
