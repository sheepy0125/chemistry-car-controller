# Motor controller portion

## Purpose

This allows the car's motor to drive and reverse with an external DC power source.

## Modus operandi

There are four parts to this:

1. Power source
2. Breadboard
3. 4-Channel relay
4. Motor

The power source plugs into the breadboard on one set of power rails.

Then, two sets of positive and negative wires go from that set of power rails to two relay commons (alternating).

Two sets of positive and negative wires corresponding to the common ones go from the relay normally open to the other power rails on the breadboard. However, one set (for relays 3 and 4) are inverted so that the positive wire goes into the negative hole and vice versa.

Finally, the motor is plugged into the other power rail where the relay normally opens are connected to, so that it propels the car forward when relays 1 and 2 are enabled.

## Modes of travel

```
Forward  POS = Relay 1
Forward  NEG = Relay 2
Backward POS = Relay 3
Backward NEG = Relay 4
```

### Forward

```
Backward POS -> LOW
Backward NEG -> LOW
Wait...
Forward  POS -> HIGH
Forward  NEG -> HIGH
```

### Backward

```
Forward  POS -> LOW
Forward  NEG -> LOW
Wait...
Backward POS -> HIGH
Backward NEG -> HIGH
```

### Stop

```
Forward  POS -> LOW
Forward  NEG -> LOW
Backward POS -> LOW
Backward NEG -> LOW
```
