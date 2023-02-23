# Cool Chemistry Car Controller Circuitry

## SBC components

### List

- 1x USB Volume Cotrol (sic)
- 1x Raspbery Pi 3B
- 1x 3.5" Screen
- 1x R41Z Evaluation Kit
- 1x Raspberry Pi 1 Model B
- 1x Hall Effect Digital Sensor

### Diagram

`Raspberry Pi 3B <-> Bluetooth L.E. <-> R41Z Evaluation Kit <-> Raspberry PI Model B`

### Purposes

#### USB Volume Cotrol portion

- Be misspelled
- Be attached to the side of the Raspbery Pi 3B and connected through USB
- Interact with the Raspberry Pi 3B UI

#### Raspberry Pi 3B portion

**See `readme_data_transmission.md` in the project root**

- Have a 3.5" screen attached
- Provide a U.I. for the user that is interacted with through the USB Volume Cotrol
- Relay information to and from the other Raspberry Pi through bluetooth and the R41Z middleman

#### R41Z portion 

**See `readme_serial_bluetooth_bridge.md` in the project root**

- Be connected to the Raspberry Pi 1B on the car
- Act as a serial bridge

### Hall effect sensor portion

- Be connected to the Raspberry Pi 1B on the car
- Detect if the magnet(s) on the wheel is nearby to this fixed sensor

#### Raspberry Pi Model B portion

*Also known as Raspberry Pi 1B*

**See `readme_data_transmission.md` in the project root**

- Be connected to the car
- Be hooked up to the circuitry of the car
- Provide status to the Raspberry Pi 3B and get the distance to travel from it through the R41Z middleman
- Calculate distance traveled through the reed switch (magnet detector) 
- Act as a relay to cut power to the motors if the distance traveled approaches or exceeds the maximum distance

## Hardware (much more general)

### List

- 1x 6-12V Motor
- 1x 3-5V 80mm Fan 
- 6x 40mm x 40mm Thermoelectric Generators
- 1x Breadboard
- Many Wires (high gauge - 18+)
- 1x 0.1 Ohm Resistor
- 2x USB Battery Banks
- At-least 1 Small Magnet(s)