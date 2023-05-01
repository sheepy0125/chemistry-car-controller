/*!
 * Serial to bluetooth bridge for wireless UART on the client side
 * Created by sheepy0125 | MIT License | 2023-02-19
 */

/***** Setup *****/
// Imports
use bluer::{gatt::remote::Characteristic, Adapter, AdapterEvent, Address, Device};
use futures::{pin_mut, StreamExt};
use log::error;
use serialport::{new as new_serialport, SerialPort};
use std::{
    env::args,
    io::Write,
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time::{sleep, Instant};
mod gatt;
mod types;
use bindings::Command;
use gatt::{
    RX_CHARACTERISTIC_SIZE, RX_CHARACTERISTIC_UUID, SERVICE_UUID, TX_CHARACTERISTIC_SIZE,
    TX_CHARACTERISTIC_UUID,
};
use types::{
    BluetoothError::*,
    Error::{self, *},
    Request, POLL_DELAY, SCAN_TIMEOUT,
};

// Constants
const BAUD_RATE: u32 = 115200;

/***** Helper functions *****/

/// Helper function to search through the characteristics after we have connected
/// This is needed so we can disconnect on error while also using the idiomatic `?`
async fn already_connected_find_serial_characteristics(
    device: &Device,
) -> Result<SerialCharacteristics, Error> {
    // Find the service again
    println!("\tEnumerating services...");
    let mut service = None;
    for service_iter in device.services().await? {
        let uuid = service_iter.uuid().await?;
        println!("\tService UUID: {}", &uuid);
        println!("\tService data: {:?}", service_iter.all_properties().await?);
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
    println!("\tFound our service!");

    // Find serial characteristics
    let mut rx_characteristic = None;
    let mut tx_characteristic = None;
    for char in service.characteristics().await? {
        let uuid = char.uuid().await?;
        // This line crashes, WTF? \/
        // println!("\tCharacteristic data: {:?}", char.all_properties().await?);
        match uuid {
            RX_CHARACTERISTIC_UUID => {
                println!("\tFound the RX characteristic!");
                rx_characteristic = Some(char);
            }
            TX_CHARACTERISTIC_UUID => {
                println!("\tFound the TX characteristic!");
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

    if addr != Address::from_str("00:60:37:E9:0B:6F").unwrap() {
        Err(BluetoothError(MissingService))?;
    }

    // Get GATT information of the device without connecting
    let uuids = device.uuids().await?.unwrap_or_default();
    let md = device.manufacturer_data().await?;

    println!("Discovered device {} with service UUIDs {:?}", addr, &uuids);
    println!("\tManufacturer data: {:x?}", &md);

    // Determine if it is our device (has the serial service)
    if !uuids.contains(&SERVICE_UUID) {
        Err(BluetoothError(MissingService))?;
    }
    println!("\tDevice provides the serial service!");

    // Attempt to connect since it is our device
    if !device.is_connected().await? {
        println!("\tConnecting...");
        device.connect().await?;
        println!("\tConnected");
    } else {
        println!("\tAlready connected");
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
    pub connected: bool,
    pub serial: Box<dyn SerialPort>,
    pub wireless_uart_device: Option<WirelessUartDevice>,
    previous_rx_value: Vec<u8>,
}

impl SerialBluetoothBridge {
    fn new(serial: Box<dyn SerialPort>) -> Self {
        Self {
            serial,
            wireless_uart_device: None,
            connected: false,
            previous_rx_value: Vec::with_capacity(RX_CHARACTERISTIC_SIZE),
        }
    }

    /***** Bluetooth handlers *****/

    /// Initialize the bluetooth adapter
    pub async fn initialize_bluetooth_adapter() -> Result<Adapter, Error> {
        let session = bluer::Session::new().await?;

        let adapter = session.default_adapter().await?;
        adapter.set_powered(true).await?;

        println!(
            "Discovering on Bluetooth adapter {} with address {}\n",
            adapter.name(),
            adapter.address().await?
        );

        Ok(adapter)
    }

    /// De-initialize the bluetooth adapter
    pub async fn deinitialize_bluetooth_adapter() -> Result<(), Error> {
        let session = bluer::Session::new().await?;

        let adapter = session.default_adapter().await?;
        adapter.set_powered(false).await?;

        println!("No longer discovering on bluetooth adapter");

        Ok(())
    }

    /// Connect to the wireless UART device
    pub async fn connect_to_device(
        adapter: &mut Adapter,
    ) -> Result<Option<WirelessUartDevice>, Error> {
        let discover = adapter.discover_devices().await?;
        pin_mut!(discover);

        let start_time = Instant::now();
        let wireless_uart_device = loop {
            let adapter_event = discover
                .next()
                .await
                .ok_or(BluetoothError(MissingAdapterEvent))?;

            if start_time.elapsed().as_millis() > SCAN_TIMEOUT as u128 {
                break None;
            }

            match adapter_event {
                AdapterEvent::DeviceAdded(address) => {
                    let device = adapter.device(address)?;
                    match find_serial_characteristics(&device).await {
                        Ok(serial_characteristics) => {
                            break Some(WirelessUartDevice {
                                address,
                                device,
                                serial_characteristics,
                            });
                        }
                        Err(e) => {
                            error!("Failed to find the serial characteristics for {device:?}: {e}");
                        }
                    }
                }
                AdapterEvent::DeviceRemoved(address) => {
                    println!("Device {address} removed");
                }
                AdapterEvent::PropertyChanged(property) => {
                    println!("Property change: {property:?}");
                }
            }
        };

        Ok(wireless_uart_device)
    }

    /// Read the Rx data from the wireless UART device
    pub async fn read_from_bluetooth_device(&mut self) -> Result<Option<String>, Error> {
        let raw_buffer = self
            .wireless_uart_device
            .as_ref()
            .ok_or_else(|| BluetoothError(NotConnected))?
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

        println!("Wireless UART Device: Got {string_buffer}");

        // Update the previous buffer
        self.previous_rx_value = raw_buffer;

        Ok(Some(string_buffer))
    }

    /// Write the Tx data to the wireless UART device,
    /// returning the number of bytes written
    ///
    /// Assumes the character fits in a `u8`
    pub async fn write_to_bluetooth_device(&mut self, data: String) -> Result<usize, Error> {
        println!("Writing {data} to bluetooth device");

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
                .as_mut()
                .ok_or_else(|| BluetoothError(NotConnected))?
                .serial_characteristics
                .tx_characteristic
                .write(&buffer)
                .await?;
        }

        Ok(characters_count)
    }

    /***** Serial handlers *****/

    /// Initialize the serial port
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

        println!("Reading {bytes_available} bytes from serial port");

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
                print!("{character} ");
                *character as char
            })
            .collect::<String>();
        println!();

        // Flush the serial Tx queue (this will NOT flush incoming Rx)
        self.serial.flush()?;

        println!("Local serial connection: Got {string_buffer}");

        Ok(Some(string_buffer))
    }

    /// Write the Rx data to the serial connection,
    /// returning the number of bytes written
    pub fn write_to_serial(&mut self, data: String) -> Result<usize, Error> {
        let bytes_written = self.serial.write(&data.into_bytes())?;
        Ok(bytes_written)
    }

    /***** Events *****/

    /// Parse request
    pub fn parse_request(data: &str) -> Result<Request, Error> {
        // Sanity check (to prevent out of range panics)
        if data.len() < 5 {
            Err(RequestError("Too short to be a valid response".to_owned()))?;
        }

        // Find the command
        let split_data = data.trim().split('$').collect::<Vec<_>>();
        println!("{split_data:?}");
        let command = Command::try_from(split_data[0][1..].to_string())
            .map_err(|e| RequestError(e.to_string()))?;

        // Hey, none of the commands need anything more than the command
        // In fact, the only reason why we have anything else is because it'd be
        // easier to make the GUI send a full thing with no data at all
        // XXX
        Ok(match command {
            Command::BluetoothStatus => Request::BluetoothStatus,
            Command::Connect => Request::Connect,
            Command::Disconnect => Request::Disconnect,
            _ => unreachable!(),
        })
    }

    pub async fn handle_command(&mut self, data: &str) -> Result<(), Error> {
        println!("Handling command from {data}");

        let request = Self::parse_request(data)?;

        use Request::*;
        match request {
            Connect => {
                println!("Connecting");
                // Terminate current handle
                self.connected = false;
                self.wireless_uart_device = None;
                self.previous_rx_value.clear();
                // Restart adapter
                Self::deinitialize_bluetooth_adapter().await?;
                let mut adapter = Self::initialize_bluetooth_adapter().await?;
                // Connect
                self.wireless_uart_device = Self::connect_to_device(&mut adapter).await?;
                self.connected = self.wireless_uart_device.is_some();
            }
            Disconnect => {
                println!("Disconnecting");
                // Terminate current handle
                self.connected = false;
                self.wireless_uart_device = None;
                // Turn off adapter
                Self::deinitialize_bluetooth_adapter().await?;
            }
            BluetoothStatus => {
                println!("Returning bluetooth status");
                // Time crunch, therefore this is the best I am willing to do :)
                // This is a *really, REALLY* bad way of doing it, and is prone to error. FIXME: XXX
                writeln!(
                    self.serial,
                    "&BLUETOOTHSTATUS${{\"connected\": {connected}}}${{\"time\":{time}}}",
                    connected = self.connected,
                    time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|duration| duration.as_secs_f64())
                        .unwrap_or(0.0),
                )?;
            }
        }

        Ok(())
    }
}

async fn loop_iteration(serial_bridge: &mut SerialBluetoothBridge) -> Result<(), Error> {
    // Receive
    if serial_bridge.connected {
        let rx = serial_bridge.read_from_bluetooth_device().await?;
        if let Some(rx) = rx {
            serial_bridge.write_to_serial(rx)?;
        }
    }

    // Transmit
    let tx = serial_bridge.read_from_serial_port()?;
    if let Some(tx) = tx {
        // Handle a command meant for us
        if tx.starts_with('^') {
            if let Err(e) = serial_bridge.handle_command(&tx).await {
                error!("Error handling command: {}", e);
            };
        } else if serial_bridge.connected {
            serial_bridge.write_to_bluetooth_device(tx).await?;
        }
    }

    // Delay
    sleep(Duration::from_millis(POLL_DELAY)).await;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let serial_port = args().nth(1_usize).expect(
        "Please enter the serial port device (e.g. `./serial-to-bluetooth.x64 /dev/pts/17`",
    );

    print!("Initializing the serial port... ");
    flush_stdout()?;
    let serial = SerialBluetoothBridge::initialize_serial_port(serial_port)?;
    println!("done!");

    let mut serial_bridge = SerialBluetoothBridge::new(serial);

    // Serial handles
    loop {
        if let Err(e) = loop_iteration(&mut serial_bridge).await {
            println!("Error: {e}");
            let _ = SerialBluetoothBridge::deinitialize_bluetooth_adapter().await;
            serial_bridge.wireless_uart_device = None;
            serial_bridge.connected = false;
        }
    }
}
