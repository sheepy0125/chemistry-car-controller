# Data transmission between the client and server

Client = Raspberry Pi 3B ; GUI

Server = Raspberry Pi 1B ; Controls everything and is onboard the car

## Modus operandi

The data transfer occurs through a request-response model through serial; a command is inputted from the client and the server responds to it. Newlines (`\n`) determine the end of a request or response.

### Requests (commands)

A command can be broken up into the following: `<prefix><command>$<arguments>`

The `<prefix>` placeholder determines where the command is intended for:

`?` => Client -> Server

`@` => Server -> R41Z

The `<command>` placeholder determines the command and subsequently the type to parse the arguments in. It must be uppercase and a valid command.

The `<arguments>` placeholder is a JSON object of the arguments, guaranteed to be parsible into the argument type inferred from the command.

The following is an example command: `?PING${"time": 1676868616.121098}`

### Responses

A response can be broken up into the following: `<prefix><command>$<response>`

The `<prefix>` placeholder determines where the response came from:

`~` => Server -> Client

`#` => R41Z -> Server

`&` => Server -> R41Z

The `<command>` placeholder determines which command was being responsded to. It must be uppercase and a valid command.

The `<response>` placeholder is a JSON object of the response, guaranteed to be parsible into the response type inferred from the command.

The following is an example response: `~PING${"time": 1676870077.3424761}`

## Requests and responses between the client and server

|Command|Arguments type|Arguments with placeholders|Response type|Response with placeholders|Notes|
|---|---|---|---|---|---|
|PING|PingArguments|`{"time": <float>}`|PingResponse|`{"time": <float>}`|This is used to test the latency between the client and the server and mainly used as a test to ensure communication is working.|
|START|StartArguments|`{"distance": <unsigned float>, "forward": <bool>, "reverse_brake": <bool>}`|None|`<none>`|This will start the car by allowing power to flow to the motors. Distance is in centimeters. The `forward` parameter determines whether the car should have the propelling direction to be with the motors forward or backwards. The `reverse_brake` parameter determines whether the car should brake through reversing the motors.|
|STOP|None|`<none>`|None|`<none>`|This will stop the car by cutting power to the motors. No reversing of the motor direction will be done.|
|STATUS|None|`<none>`|StatusResponse|`{"running": <bool>, "uptime": <unsigned integer>, "runtime": <unsigned integer>, "distance": <unsigned float>, "accelerometer_readings": {"x": <signed float>, "y": <signed float>, "z": <signed float>}}`|This will return the status of the car. The `uptime` is the number of seconds since the server software started running. The `runtime`, on the other hand, is the amount of time the car has been traveling for. If `running` is `false`, then `runtime` is `0`. The distance traveled is in centimeters. The `accelerometer_readings` are whatever the last accelerometer readings were.|

## Requests and responses between the server and R41Z

|Command|Arguments type|Arguments with placeholders|Response type|Response with placeholders|Notes|
|---|---|---|---|---|---|
|PING|PingArguments|`{"time": <float>}`|PingResponse|`{"time": <float>}`|This is used to test the latency between the server and the accelerometer. This is critical for the motor reversing and distance traveled calculations. |
