"""
Shared types for the server
Created by sheepy0125 | MIT License | 2023-02-20
"""

from math import pi
from RPi import GPIO

GPIO.setmode(GPIO.BCM)
del GPIO

VERBOSE: bool = False
SERIAL_DELAY_TIME_SECONDS: float = 0.10
STATUS_POLL_DURATION_SECONDS: float = 1 / 4
NUMBER_OF_MAGNETS: int = 2
WHEEL_DIAMETER_INCHES: float = 2.5
WHEEL_DIAMETER_CENTIMETERS: float = WHEEL_DIAMETER_INCHES * 2.54
WHEEL_CIRCUMFERENCE_CENTIMETERS: float = pi * WHEEL_DIAMETER_CENTIMETERS
SERIAL_BAUD_RATE: int = 115200
SERIAL_BYTE_SIZE: int = 8
SERIAL_STOP_BITS: int = 1
SERIAL_PARITY: int | str = "N"  # None
GPIO_DEBOUNCE_TIME_MS: int = 10
MOTOR_CONTROLLER_SAFETY_DELAY_SECONDS: float = 0.1
FORWARD_LEEWAY_DISTANCE_CENTIMETERS: float = (
    WHEEL_DIAMETER_CENTIMETERS / NUMBER_OF_MAGNETS
) / 2
BACKWARD_LEEWAY_DISTANCE_CENTIMETERS: float = (
    WHEEL_CIRCUMFERENCE_CENTIMETERS / NUMBER_OF_MAGNETS
) / 2
MAGNET_FREE_STALL_FOR_SECONDS: float = 3.0
CAUTIOUS_REVERSE_STALL_FOR_SECONDS: float = 1.0
BACKWARD_COOLDOWN_SECONDS: float = 0.10

del pi
