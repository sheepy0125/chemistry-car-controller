# Cool Chemistry Car Controller Circuitry

## SBC components

### List

- 1x USB Volume Cotrol (sic)
- 1x Raspbery Pi 3B
- 1x 3.5" screen
- 1x R41Z Evaluation Kit

### Diagram

`Raspberry Pi <-> Bluetooth L.E. and R41Z middleman <-> Arduino controller`

### Purposes

#### USB Volume Cotrol portion

- Be misspelled
- Be attached ot the side of the Raspbery Pi and connected through USB
- Interact with the Raspberry Pi UI

#### Raspberry Pi portion

- Have a 3.5" screen attached
- Provide a U.I. for the user that is interacted with through the USB Volume Cotrol
- Relay information to and from the Arduino through the R41Z middleman

#### R41Z portion

- Be connected to the Arduino on the car
- Act as a middleman to transmit signals to and from the Raspberry Pi (repeat RX and TX)

#### Arduino portion

- Be connected to the car
- Be hooked up to the circuitry of the car
- Provide status to the Raspberry Pi and get the distance to travel from it through the R41Z 
- Check for how much distance has traveled through sensors
- Act as a relay to cut power to the motors if the distance traveled approaches or exceeds the maximum distance

## Hardware (much more general)

### List

- 1x 6-12V Motor
- 1x 3-5V 80mm Fan 
- 6x 40mm x 40mm Thermoelectric Generators
- 1x Breadboard
- Many Wires (low gauge)
- 1x 0.1 Ohm Resistor