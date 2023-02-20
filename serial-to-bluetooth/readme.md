# Serial <-> Bluetooth bridge portion

## Purpose

This exists to map a bluetooth connection between the GUI to a serial connection with the R41Z-EVAL as a middleman.

The R41Z middleman code can be found in the `r41z-code` directory at the project root.

## Modus Operandi

The R41Z-EVAL will be plugged into the Raspberry Pi 1B on the car and give commands over serial.

The R41Z-EVAL will connect to the Raspberry Pi 3B (where the R41Z is a peripheral device / master and the Raspberry Pi 3B is a central device / slave) and have a write-only characteristic for Tx and a read-only characteristic for Rx, as seen by the Raspberry Pi 3B (the slave).

Below is a list of the useful GATT attributes as shown by `bluetoothctl` in `menu gatt` mode by running `list-attributes`, with some comments denoted by `#`:
```
Primary Service (Handle 0x0000)
	/org/bluez/hci0/dev_00_60_37_E9_0B_6F/service0011               # Service for TX and RX
	01ff0100-ba5e-f4ee-5ca1-eb1e5e4b1ce0
	Vendor specific
Characteristic (Handle 0x0000)
	/org/bluez/hci0/dev_00_60_37_E9_0B_6F/service0011/char0012      # Write-only (TX)
	01ff0101-ba5e-f4ee-5ca1-eb1e5e4b1ce0
	Vendor specific
Characteristic (Handle 0x0000)
	/org/bluez/hci0/dev_00_60_37_E9_0B_6F/service0011/char0015      # Read-only (RX)
	01ff0101-ba5e-f4ee-5ca1-eb1e5e4b1ce1
	Vendor specific
```

Below also is a table corroborating with the `bluetoothctl` output above:

|Attribute UUID|Permissions|Size|Type|Role from client (RPi3B) view|
|---|---|---|---|---|
|01ff0101-ba5e-f4ee-5ca1-eb1e5e4b1ce0|Writable|1|ASCII|TX|
|01ff0101-ba5e-f4ee-5ca1-eb1e5e4b1ce1|Readable|244|ASCII|RX|

