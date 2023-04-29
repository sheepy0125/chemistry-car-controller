# Cool Chemistry Car Controller Circuitry (C4 Circuitry)

## Purpose

This repo provides the source code behind a chem-car for a school project.

To summarize the related requirements and constraints of the project:

-   Make a car that goes a random distance up to 10 meters
-   Power said car with a chemical reaction
-   Use at least one electronic component

## General overview

There is circuitry onboard the car that controls the motor to go the required distance. There also is a GUI that connects wirelessly to that circuitry to input the distance and also receive some status.

## Modus operandi

Simplified, the mode of operation is:

1. Receive user input for the distance to travel
1. Start the motor
1. Keep track of distance using the magnet-based odometer
1. Wait until the car exceeds or meets the target distance
1. Turn the motor off
1. Wait until the car completely stops
1. Back the car up slowly (the least distance it can precisely travel) and then turn off the motor for it to stop
1. Repeat the above step until the car meets the target distance
1. Turn the motor off

## SBC components

### List

-   1x Raspberry Pi 3B
-   1x 3.5" Screen
-   1x R41Z Evaluation Kit
-   1x Raspberry Pi 1 Model B
-   1x Hall Effect Digital Sensor

### Diagram

GUI / Client <-> Serial to and fro bluetooth bridge <-> Onboard Bluetooth L.E. <-> R41Z Evaluation Kit <-> Serial over USB <-> Raspberry Pi Model B

### Purposes

#### Raspberry Pi 3B portion

**See `readme_data_transmission.md` in the project root**

-   Have a 3.5" screen attached
-   Provide a U.I. for the user that is interacted with through the touchscreen
-   Relay information to and from the other Raspberry Pi through bluetooth and the R41Z middleman

#### R41Z portion

**See `readme_serial_bluetooth_bridge.md` in the project root**

-   Be connected to the Raspberry Pi 1B on the car
-   Act as a serial bridge to transmit data with the GUI

### Hall effect sensor portion

-   Be connected to the Raspberry Pi 1B on the car
-   Detect if the magnet(s) on the wheel is nearby to this fixed sensor
-   Thusly providing the revolutions of the wheel to determine distance

#### Raspberry Pi Model B portion

_Also known as Raspberry Pi 1B_

**See `readme_data_transmission.md` in the project root**

-   Be hooked up to the circuitry of the car (hall effect sensor / relays / motor)
-   Provide status to the Raspberry Pi 3B and get the distance to travel from it through the R41Z middleman
-   Calculate distance traveled through the Hall Effect sensor (magnet detector)
-   Control the motor accordingly to travel the distance needed

## Hardware (much more general)

### List

-   1x 3-12V Motor
-   1x Breadboard
-   1x 4-Channel Relay
-   Many Wires (high gauge, probably 24)
-   2x USB Battery Banks
-   At-least 1 Small Magnet(s)

### Purposes

#### 3-12V Motor portion

-   Be connected to the "motor controller"
-   Allow the automobile to spin

#### Breadboard portion

-   Provide a place to connect to and from the "motor controller"

#### 4-Channel Relay

**See `readme_motor_controller.md` in the project root**

-   Be connected to the Raspberry 1B and the circuitry (breadboard / motor)
-   Act as a "motor controller"
