# Client Raspberry Pi 3B Code portion

## Purpose

This serves as a GUI to interact with the Raspberry Pi 1B onboard the car.

It does this by using the Pi 3B's built-in bluetooth to connect to the Pi 1B for a wireless serial bridge. For more information on that, see `readme_serial_bluetooth_bridge.md` in the project root.

The menus are controlled through a `USB Volume Cotrol` (sic) which is attached to the Pi and just gives volume up / volume down keypresses (yes, I know it is customizable and even homebrewable, but it's not needed for anything major.)

For information on how these two scrumptious yet rare pastries communicate, see `reamde_data_transmission.md` in the project root.