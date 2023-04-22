/*!
 * Events for the client
 * Created by sheepy0125 | MIT license | 2023-02-23
 */

use std::time::{SystemTime, UNIX_EPOCH};

/***** Setup *****/
// Imports
use crate::bindings::{
    ClientError, Command, ErrorResponse, MetaData, PingResponse, StartResponse,
    StaticStatusResponse, StatusResponse, StopResponse, TransitMode, TransitType,
};
use either::Either;
use serde::{Deserialize, Serialize};
use serde_json::{from_str as serde_from_str, to_string as serde_to_string};
use serialport::SerialPort;

/***** Event *****/

pub struct Event<S>
where
    S: Serialize + for<'a> Deserialize<'a> + Default,
{
    pub command: Command,
    pub transit_mode: TransitMode,
    pub transit_type: TransitType,
    pub value: S,
    pub metadata: MetaData,
}

/// Context of events that have been sent and received
pub struct EventContext {
    /// A buffer for the serial RX incase there is no `\n` available yet
    pub serial_rx_buffer: String,
}

/// A serial event propagator
///
/// This will connect to the serial connection and await for events
pub struct SerialEventPropagator {
    pub serial: Box<dyn SerialPort>,
}
impl SerialEventPropagator {
    pub fn new(serial: Box<dyn SerialPort>) -> Self {
        Self { serial }
    }

    /// Read from the serial connection
    ///
    /// If there are no bytes to be read, this will return `None`.
    /// Otherwise, it will block until a newline is present!
    pub fn read_from_serial(&mut self) -> Option<Result<String, ClientError>> {
        match self
            .serial
            .bytes_to_read()
            .map_err(|e| ClientError::SerialError(e.to_string()))
        {
            Ok(0) => return None,
            Err(e) => return Some(Err(e)),
            _ => (),
        }

        let mut return_string = String::new();
        let mut character_buffer = [0_u8; 1];
        loop {
            character_buffer[0] = 0_u8;
            match self.serial.read_exact(&mut character_buffer) {
                Ok(()) => (),
                Err(_) => break,
            };
            let character = character_buffer[0] as char;
            match character {
                '\r' => break,
                '\n' => break,
                character => return_string.push(character),
            }
        }

        if return_string.trim().is_empty() {
            return None;
        }

        Some(Ok(return_string))
    }

    /// Write a command to the serial connection
    pub fn write_to_serial<S>(&mut self, command: Command, data: S) -> Result<(), ClientError>
    where
        S: Serialize + for<'a> Deserialize<'a> + Sized + Default,
    {
        let prefix = TransitMode::ClientToServerRequest as u8 as char;
        let stringified_data =
            serde_to_string(&data).map_err(|e| ClientError::ParseError(e.to_string()))?;
        let stringified_data = match stringified_data.as_str() {
            "null" => "{}",
            stringified => stringified,
        };

        let metadata = MetaData {
            time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| ClientError::UnknownError(e.to_string()))?
                .as_secs_f32(),
        };
        let stringified_metadata =
            serde_to_string(&metadata).map_err(|e| ClientError::ParseError(e.to_string()))?;
        writeln!(
            self.serial,
            "{prefix}{command}${stringified_data}${stringified_metadata}"
        )
        .map_err(|e| ClientError::SerialError(e.to_string()))
    }

    /// Parse response
    pub fn parse_response<S>(
        data: &String,
    ) -> Result<Result<Event<S>, Event<ErrorResponse>>, ClientError>
    where
        S: Serialize + for<'a> Deserialize<'a> + Default,
    {
        println!("0 {data}");

        let is_error;

        // Find the command
        let split_data = data.split('$').collect::<Vec<_>>();
        let command = Command::try_from(split_data[0][1..].to_string())?;

        // Find the metadata
        let metadata_data = split_data[2];
        // Parse the metadata
        let metadata = serde_from_str::<MetaData>(metadata_data)?;
        println!("1 {metadata:?}");

        // Get the response
        let response_data = split_data[1];
        println!("2 {response_data}");

        // Parse the response (if error then parse that)
        is_error = command == Command::Unknown;
        if !is_error {
            // Ensure the value is non-empty, or else Serde will think it's a map
            let value = if response_data == "{}" {
                S::default()
            } else {
                serde_from_str::<S>(response_data)?
            };

            Ok(Ok(Event {
                command,
                transit_mode: TransitMode::ServerToClientResponse,
                transit_type: TransitType::Response,
                value,
                metadata,
            }))
        } else {
            let value = serde_from_str::<ErrorResponse>(response_data)?;
            Ok(Err(Event {
                command,
                transit_mode: TransitMode::ServerToClientResponse,
                transit_type: TransitType::Response,
                value,
                metadata,
            }))
        }
    }
}
