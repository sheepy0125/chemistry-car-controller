/*!
 * Serial to bluetooth bridge for wireless UART on the client side
 * Created by sheepy0125 | MIT License | 2023-02-19
 */

/***** Setup *****/
// Imports
use bluer::{gatt::remote::Characteristic, Adapter, AdapterEvent, Address, Device};
use futures::{pin_mut, StreamExt};
use log::{error, info};
use serialport::{new as new_serialport, SerialPort};
use std::{io::Write, time::Duration};
use tokio::time::sleep;

mod gatt;
mod types;
use gatt::{RX_CHARACTERISTIC_SIZE, RX_CHARACTERISTIC_UUID, SERVICE_UUID, TX_CHARACTERISTIC_UUID};
use types::{
    BluetoothError::*,
    Error::{self, *},
    POLL_DELAY,
};

use crate::gatt::TX_CHARACTERISTIC_SIZE;

// Constants
const BAUD_RATE: u32 = 115200;

/***** Helper functions *****/

/// Helper function to search through the characteristics after we have connected
/// This is needed so we can disconnect on error while also using the idiomatic `?`
async fn already_connected_find_serial_characteristics(
    device: &Device,
) -> Result<SerialCharacteristics, Error> {
    // Find the service again
    info!("\tEnumerating services...");
    let mut service = None;
    for service_iter in device.services().await? {
        let uuid = service_iter.uuid().await?;
        info!("\tService UUID: {}", &uuid);
        info!("\tService data: {:?}", service_iter.all_properties().await?);
        match uuid {
            SERVICE_UUID => {
                service = Some(service_iter);
                break;
            }
            _ => continue,
        }
    }
    let service = match service {
        Some(service) => service,
        None => Err(BluetoothError(MissingService))?,
    };
    info!("\tFound our service!");

    // Find serial characteristics
    let mut rx_characteristic = None;
    let mut tx_characteristic = None;
    error!("service.characteristics()");
    for char in service.characteristics().await? {
        error!("char.uuid()");
        let uuid = char.uuid().await?;
        info!("\tCharacteristic UUID: {}", &uuid);
        error!("char.all_properties()");
        // info!("\tCharacteristic data: {:?}", char.all_properties().await?);
        match uuid {
            RX_CHARACTERISTIC_UUID => {
                info!("\tFound the RX characteristic!");
                rx_characteristic = Some(char);
            }
            TX_CHARACTERISTIC_UUID => {
                info!("\tFound the TX characteristic!");
                tx_characteristic = Some(char);
            }
            _ => (),
        }

        // Are we done?
        if rx_characteristic.is_some() && tx_characteristic.is_some() {
            return Ok(SerialCharacteristics {
                // Safety: We know both of them are `Some` variants
                rx_characteristic: rx_characteristic.unwrap(),
                tx_characteristic: tx_characteristic.unwrap(),
            });
        }
    }
    Err(BluetoothError(MissingCharacteristic))
}

/// Helper function to find if the scanned device is the one we are looking for
async fn find_serial_characteristics(device: &Device) -> Result<SerialCharacteristics, Error> {
    // Get GAP information of the device
    let addr = device.address();

    // Get GATT information of the device without connecting
    let uuids = device.uuids().await?.unwrap_or_default();
    let md = device.manufacturer_data().await?;

    info!("Discovered device {} with service UUIDs {:?}", addr, &uuids);
    info!("\tManufacturer data: {:x?}", &md);

    // Determine if it is our device (has the serial service)
    if !uuids.contains(&SERVICE_UUID) {
        Err(BluetoothError(MissingService))?;
    }
    info!("\tDevice provides the serial service!");

    // Attempt to connect since it is our device
    if !device.is_connected().await? {
        info!("\tConnecting...");
        device.connect().await?;
        info!("\tConnected");
    } else {
        info!("\tAlready connected");
    }

    match already_connected_find_serial_characteristics(device).await {
        Ok(characteristics) => Ok(characteristics),
        Err(e) => {
            device.disconnect().await?;
            Err(e)
        }
    }
}

fn flush_stdout() -> Result<(), Error> {
    std::io::stdout().flush()?;
    Ok(())
}

/***** Structs *****/
pub struct SerialCharacteristics {
    pub rx_characteristic: Characteristic,
    pub tx_characteristic: Characteristic,
}

pub struct WirelessUartDevice {
    pub address: Address,
    pub device: Device,
    pub serial_characteristics: SerialCharacteristics,
}

struct SerialBluetoothBridge {
    pub serial: Box<dyn SerialPort>,
    pub wireless_uart_device: WirelessUartDevice,
    previous_rx_value: Vec<u8>,
}

impl SerialBluetoothBridge {
    fn new(serial: Box<dyn SerialPort>, wireless_uart_device: WirelessUartDevice) -> Self {
        Self {
            serial,
            wireless_uart_device,
            previous_rx_value: Vec::with_capacity(RX_CHARACTERISTIC_SIZE),
        }
    }

    /***** Bluetooth handlers *****/

    /// Intialize the bluetooth adapter
    pub async fn intialize_bluetooth_adapter() -> Result<Adapter, Error> {
        let session = bluer::Session::new().await?;

        let adapter = session.default_adapter().await?;
        adapter.set_powered(true).await?;

        info!(
            "Discovering on Bluetooth adapter {} with address {}\n",
            adapter.name(),
            adapter.address().await?
        );

        Ok(adapter)
    }

    /// Connect to the wireless UART device
    pub async fn connect_to_device(adapter: &mut Adapter) -> Result<WirelessUartDevice, Error> {
        let discover = adapter.discover_devices().await?;
        pin_mut!(discover);

        let wireless_uart_device = loop {
            let adapter_event = discover
                .next()
                .await
                .ok_or(BluetoothError(MissingAdapterEvent))?;

            match adapter_event {
                AdapterEvent::DeviceAdded(address) => {
                    let device = adapter.device(address)?;
                    match find_serial_characteristics(&device).await {
                        Ok(serial_characteristics) => {
                            break WirelessUartDevice {
                                address,
                                device,
                                serial_characteristics,
                            };
                        }
                        Err(e) => {
                            error!("Failed to find the serial characteristics for {device:?}: {e}");
                            // Disconnect if needed
                            // if device.is_connected().await? {
                            // info!("Disconnecting due to error...");
                            // device.disconnect().await?;
                            // }
                        }
                    }
                }
                AdapterEvent::DeviceRemoved(address) => {
                    info!("Device {address} removed");
                }
                AdapterEvent::PropertyChanged(property) => {
                    info!("Property change: {property:?}");
                }
            }
        };

        Ok(wireless_uart_device)
    }

    /// Read the Rx data from the wireless UART device
    pub async fn read_from_bluetooth_device(&mut self) -> Result<Option<String>, Error> {
        let raw_buffer = self
            .wireless_uart_device
            .serial_characteristics
            .rx_characteristic
            .read()
            .await?;

        // If it is the same thing we just read, then discard it
        if raw_buffer == self.previous_rx_value {
            return Ok(None);
        }

        let string_buffer = raw_buffer
            .iter()
            .map(|character| *character as char)
            .collect::<String>();

        info!("Wireless UART Device: Got {string_buffer}");

        // Update the previous buffer
        self.previous_rx_value = raw_buffer;

        Ok(Some(string_buffer))
    }

    /// Write the Tx data to the wireless UART device,
    /// returning the number of bytes written
    ///
    /// Assumes the character fits in a `u8`
    pub async fn write_to_bluetooth_device(&mut self, data: String) -> Result<usize, Error> {
        info!("Writing {data} to bluetooth device");

        // Chunk it
        let mut characters_count = 0_usize;
        let mut character_iterator = data.chars();
        let mut done = false;
        while !done {
            let mut buffer = [0_u8; TX_CHARACTERISTIC_SIZE];
            for idx in 0..TX_CHARACTERISTIC_SIZE {
                buffer[idx] = match character_iterator.next() {
                    Some(character) => {
                        characters_count += 1;
                        character as u8
                    }
                    None => {
                        done = true;
                        0_u8
                    }
                };
            }

            self.wireless_uart_device
                .serial_characteristics
                .tx_characteristic
                .write(&buffer)
                .await?;
        }

        Ok(characters_count)
    }

    /***** Serial handlers *****/

    /// Intialize the serial port
    pub fn initialize_serial_port(device: String) -> Result<Box<dyn SerialPort>, Error> {
        let serial = new_serialport(device, BAUD_RATE)
            .timeout(Duration::from_millis(500_u64))
            .open()?;
        Ok(serial)
    }

    /// Read data from the serial port to be transferred over (this is getting Tx)
    pub fn read_from_serial_port(&mut self) -> Result<Option<String>, Error> {
        // Get how many bytes can be read
        let bytes_available = self.serial.bytes_to_read()? as usize;
        if bytes_available == 0 {
            return Ok(None);
        }

        info!("Reading {bytes_available} bytes from serial port");

        // Read into buffer
        let mut vector_raw_buffer = Vec::with_capacity(bytes_available);
        for _ in 0..bytes_available {
            vector_raw_buffer.push(0_u8);
        }
        let raw_buffer = vector_raw_buffer.as_mut_slice();
        self.serial.read_exact(raw_buffer)?;
        let string_buffer = raw_buffer
            .iter()
            .map(|character| {
                info!("U8: {character}");
                *character as char
            })
            .collect::<String>();

        // Flush the serial Tx queue (this will NOT flush incoming Rx)
        self.serial.flush()?;

        info!("Local serial connection: Got {string_buffer} ({raw_buffer:?})");

        Ok(Some(string_buffer))
    }

    /// Write the Rx data to the serial connection,
    /// returning the number of bytes written
    pub fn write_to_serial(&mut self, data: String) -> Result<usize, Error> {
        let bytes_written = self.serial.write(&mut data.into_bytes())?;
        Ok(bytes_written)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    // env_logger::init();

    print!("Intializing the serial port... ");
    flush_stdout()?;
    let serial = SerialBluetoothBridge::initialize_serial_port("/dev/pts/7".into())?;
    print!("done!\n");
    print!("Initializing the bluetooth adapter... ");
    flush_stdout()?;
    let mut bluetooth_adapter = SerialBluetoothBridge::intialize_bluetooth_adapter().await?;
    print!("done!\n");
    print!("Connecting to the Wireless UART device... ");
    flush_stdout()?;
    let wireless_uart_device =
        SerialBluetoothBridge::connect_to_device(&mut bluetooth_adapter).await?;
    print!("done!\n");

    let mut serial_bridge = SerialBluetoothBridge::new(serial, wireless_uart_device);

    // Serial handles
    loop {
        // Receive
        let rx = serial_bridge.read_from_bluetooth_device().await?;
        if let Some(rx) = rx {
            serial_bridge.write_to_serial(rx)?;
        }

        // Transmit
        let tx = serial_bridge.read_from_serial_port()?;
        if let Some(tx) = tx {
            serial_bridge.write_to_bluetooth_device(tx).await?;
        }

        // Delay
        sleep(Duration::from_millis(POLL_DELAY)).await;
    }
}
