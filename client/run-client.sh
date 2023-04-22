#!/bin/bash

pushd ~/car-control/

echo "=== Starting socat"
nohup socat -d -d pty,raw,echo=0 pty,raw,echo=0 > socat.log &
SOCAT_PROCESS=$!

echo "=== Getting PTYs from socat"
(tail -f socat.log &) | grep -q "starting data transfer"
SERIAL_DEVS_STR=$(cat socat.log | sed -n -e 's/^.*N PTY is //p')
readarray -t SERIAL_DEVS_ARR <<< $SERIAL_DEVS_STR
BRIDGE_PTY="${SERIAL_DEVS_ARR[0]}"
CLIENT_PTY="${SERIAL_DEVS_ARR[1]}"

echo "=== Starting serial bridge"
pushd serial-to-bluetooth
nohup cargo run $BRIDGE_PTY 2>/dev/null > bridge.log &
BRIDGE_PROCESS=$!
(tail -f bridge.log &) | grep -q "Connecting to the Wireless UART device... done!"
popd

echo "=== Starting client"
pushd client
cargo run $CLIENT_PTY
popd

echo "=== Cleaning up"
kill $SOCAT_PROCESS
kill $BRIDGE_PROCESS
popd