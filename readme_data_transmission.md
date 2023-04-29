# Data transmission between the client and server

Client = Raspberry Pi 3B ; GUI

Server = Raspberry Pi 1B ; Controls everything and is onboard the car

R41Z-EVAL = BLE wireless serial middleman

## Modus operandi

The data transfer occurs through a request-response model through serial; a command is inputted from the client and the server responds to it. Newlines (`\n`) determine the end of a request or response.

### Metadata

The metadata consists of the time in the Unix epoch format (`unsigned float`).

Here is an example of the metadata struct: `{"time": 1676870077.3424761}`

### Requests (commands)

A command can be broken up into the following: `<prefix><command>$<arguments>$<metadata>`

The `<prefix>` placeholder determines where the command is intended for:

```
? => Client -> Server
```

The `<command>` placeholder determines the command and subsequently the type to parse the arguments in. It must be uppercase and a valid command.

The `<arguments>` placeholder is a JSON object of the arguments, guaranteed to be parsable into the argument type inferred from the command.

The `<metadata>` placeholder is a JSON object of the metadata struct.

The following is an example command: `?PING${}${"time": 1676870077.3424761}`

### Responses

A response can be broken up into the following: `<prefix><command>$<response>$<metadata>`

The `<prefix>` placeholder determines where the response came from:

```
~ => Server -> Client
```

The `<command>` placeholder determines which command was being responded to. It must be uppercase and a valid command.

The `<response>` placeholder is a JSON object of the response, guaranteed to be parsable into the response type inferred from the command.

The `<metadata>` placeholder is a JSON object of the metadata struct.

The following is an example response: `~PING${}${"time": 1676870077.3424761}`

### Responding with errors

If an error occurred, then an `ErrorResponse` will be sent under the `Error` command, consisting of an error enum variant (`unsigned integer`) and an error message (`string`).

For the error enum variants, here is a lookup table:

```
00 => Malformed request - Failed prefix parsing
01 => Malformed request - Failed command parsing
02 => Malformed request - Failed separator parsing
03 => Malformed request - Failed arguments parsing
04 => Malformed request - Failed metadata parsing
05 => Malformed request - Type error
06 => Malformed request - Other error
10 => Malformed response - Type error
11 => Malformed response - Other error
21 => Failed to start - Already started
22 => Failed to start - Magnet odometer failed
23 => Failed to start - Motor control failed
24 => Failed to start - Could not acquire distance mutex lock
25 => Failed to stop - Not started
26 => Failed to stop - Start thread would not respond
27 => Failed status - Could not acquire distance mutex lock
28 => Failed ping - Negative latency
99 => Any other error
```

Here is the format of the `ErrorResponse` struct: `{"error_variant": <unsigned int>, "message": <string>}`

The following is an example of a very useful error: `~ERROR${"error_variant": 99, "message": "Something happened."}`

## Requests and responses between the client and server

| Command       | Arguments type | Arguments with placeholders                               | Response type        | Response with placeholders                                                                                                                                                                                                          | Notes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| ------------- | -------------- | --------------------------------------------------------- | -------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| PING          | PingArguments  | `{"time": <float>}`                                       | PingResponse         | `{"sent_time": <float>}`                                                                                                                                                                                                            | This is used to test the latency between the client and the server and mainly used as a test to ensure communication is working. The server simply returns the time sent to it.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| START         | StartArguments | `{"distance": <unsigned float>, "reverse_brake": <bool>}` | None                 | `<none>`                                                                                                                                                                                                                            | This will start the car by allowing power to flow to the motors. The `reverse_brake` parameter determines whether the car should brake through reversing the motors. **All distance units are in centimeters!**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| STOP          | None           | `<none>`                                                  | None                 | `<none>`                                                                                                                                                                                                                            | This will stop the car by cutting power to the motors and killing the start threads, acting as an emergency stop. No reversing of the motor direction will be done, even if the start arguments said it would.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| STATIC_STATUS | None           | `<none>`                                                  | StaticStatusResponse | `{"number_of_magnets": <unsigned integer>, "wheel_diameter": <unsigned float>}`                                                                                                                                                     | This will return the static status of the car (never-changing). It is used to supplement information returned from the (dynamic) status. The `wheel_diameter` is in `centimeters`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| STATUS        | None           | `<none>`                                                  | StatusResponse       | `{"running": <bool>, "uptime": <unsigned integer>, "runtime": <unsigned integer>, "distance": {"distance": <unsigned float>, "velocity": <unsigned float>, "magnet_hit_counter": <unsigned integer>}, "stage": <unsigned integer>}` | **Although this will return status when called by it's command, it is not intended to be used that way. Instead, the `start` command will continually give status until the car stops running. This design choice is to reduce latency and allow for quicker transfers.** This will return the status of the car. The `uptime` is the number of seconds since the server software started running. The `runtime`, on the other hand, is the amount of time the car has been traveling for. If `running` is `false`, then `runtime` is `0`. The distance traveled is in centimeters. The `distance` is what the distance readings are from the odometer, with `velocity` being the distance divided by the runtime. You can find more information about how this is measured from getting the `STATIC_STATUS` response. If `running` is `false`, then all of the values for `distance` are whatever it was last at when previously running, unless it has been cleared when the `START` event was called. If cleared or never started, then the values are `0.0`, `0.0`, and `0` respectively. The `stage` represents an enum variant of what stage it the car is currently in. **All distance units are in `centimeters`!** |
| UNKNOWN       | None           | `<none>`                                                  | ErrorResponse        | `{"error_variant": <unsigned integer>, "message": <string>}`                                                                                                                                                                        | See `Responding with errors`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
