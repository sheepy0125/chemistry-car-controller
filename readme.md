# Cool Chemistry Car Controller Circuitry

## SBC components

### List

- 1x USB Volume Cotrol (sic)
- 1x Raspbery Pi 3B
- 1x 3.5" screen
- 1x R41Z Evaluation Kit
- 1x Raspberry Pi 1 Model B

### Diagram

`Raspberry Pi 3B <-> Bluetooth L.E. <-> R41Z Evaluation Kit <-> Raspberry PI Model B`

### Purposes

#### USB Volume Cotrol portion

- Be misspelled
- Be attached ot the side of the Raspbery Pi and connected through USB
- Interact with the Raspberry Pi UI

#### Raspberry Pi 3B portion

- Have a 3.5" screen attached
- Provide a U.I. for the user that is interacted with through the USB Volume Cotrol
- Relay information to and from the other Raspberry Pi through bluetooth and the R41Z middleman

#### R41Z portion

- Be connected to the Raspberry Pi 1B on the car
- Act as a serial bridge
- Provide accelerometer readings

#### Raspberry Pi Model B

- Be connected to the car
- Be hooked up to the circuitry of the car
- Provide status to the Raspberry Pi 3B and get the distance to travel from it through the R41Z middleman
- Calculate distance traveled through the accelerometer readings from the R41Z
- Act as a relay to cut power to the motors if the distance traveled approaches or exceeds the maximum distance

## Hardware (much more general)

### List

- 1x 6-12V Motor
- 1x 3-5V 80mm Fan 
- 6x 40mm x 40mm Thermoelectric Generators
- 1x Breadboard
- Many Wires (low gauge)
- 1x 0.1 Ohm Resistor
- 2x USB Battery Banks