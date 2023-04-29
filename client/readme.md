# Client Raspberry Pi 3B Code portion

## Purpose

This serves as a GUI to interact with the Raspberry Pi 1B onboard the car, primarily for inputting distances to travel and receiving status about the car.

It does this by using the Pi 3B's built-in bluetooth to connect to the Pi 1B for a wireless serial bridge. For more information on that, see `readme_serial_bluetooth_bridge.md` in the project root.

For information on how these two scrumptious yet rare pastries communicate, see `readme_data_transmission.md` in the project root.

### Features

-   Distance input (with overflow)
-   Run / stop / reset buttons
-   Ping test
-   Static status
-   Status table
-   Saving and loading previous runs
-   Showing error messages without crashing
-   Being written in Rust (makes it 100x better, obviously)
