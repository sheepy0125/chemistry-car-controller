/*!
 * Events for the client
 * Created by sheepy0125 | MIT license | 2023-02-23
 */

use std::time::{Instant, SystemTime, UNIX_EPOCH};

/***** Setup *****/
// Imports
use crate::{
    bindings::{
        ClientError, Command, ErrorResponse, MetaData, PingResponse, Response, ServerError,
        StartResponse, StaticStatusResponse, StatusResponse, StopResponse, TransitMode,
        TransitType,
    },
    shared::SERIAL_DELAY_TIME,
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{from_str as serde_from_str, to_string as serde_to_string};
use serialport::SerialPort;
use smart_default::SmartDefault;
use std::mem::take;
/// Event encapsulating a request or response
pub struct Event<S>
where
    S: Serialize + for<'a> Deserialize<'a>,
{
    pub command: Command,
    pub transit_mode: TransitMode,
    pub transit_type: TransitType,
    pub value: S,
    pub metadata: MetaData,
}
/// Run data
#[derive(SmartDefault)]
pub struct RunData {
    pub ping_status_response: Option<(Box<Event<PingResponse>>, f64)>,
    pub static_status_response: Option<Box<Event<StaticStatusResponse>>>,
    pub status_responses: Vec<Event<StatusResponse>>,
    pub other_responses: Vec<Response>,
    #[default = false]
    pub running: bool,
}

/// A serial event propagator
///
/// This will connect to the serial connection and await for events
pub struct SerialEventPropagator {
    pub serial: Box<dyn SerialPort>,
    pub last_get_time: Instant,
    rx_data: String,
}
impl SerialEventPropagator {
    pub fn new(serial: Box<dyn SerialPort>) -> Self {
        Self {
            serial,
            rx_data: String::new(),
            last_get_time: Instant::now(),
        }
    }

    /// Read from the serial connection until a newline is hit
    ///
    /// If there are no bytes to be read, or if it was unfinished, this will
    /// return `None`.
    ///
    /// The serial connection has a timeout, therefore if we stop receiving before
    /// a newline is present, then the data will temporarily be written to
    /// `self.rx_data` and this will pick back up where it left off
    pub fn read_from_serial(&mut self) -> Result<Option<String>, ClientError> {
        let max_bytes_to_read = match self
            .serial
            .bytes_to_read()
            .map_err(|e| ClientError::Serial(e.to_string()))
        {
            Ok(0) => return Ok(None),
            Ok(bytes) => bytes,
            Err(e) => return Err(e),
        };

        let mut character_buffer = [0_u8; 1];
        for _ in 0..max_bytes_to_read {
            character_buffer[0] = 0_u8;
            match self.serial.read_exact(&mut character_buffer) {
                Ok(()) => (),
                Err(_) => break,
            };
            let character = character_buffer[0] as char;
            self.rx_data.push(character);
            match character {
                '\r' => break, // 'Tis what `scip` does
                '\n' => break, // 'Tis what the server does
                _ => (),
            }
        }

        if self.rx_data.trim().is_empty() || !self.rx_data.ends_with(['\n', '\r']) {
            return Ok(None);
        }

        // Remove the ending \r or \n
        self.rx_data.pop();

        Ok(Some(take(&mut self.rx_data)))
    }

    /// Write a command to the serial connection
    pub fn write_to_serial<S>(&mut self, command: Command, data: S) -> Result<(), ClientError>
    where
        S: Serialize + for<'a> Deserialize<'a> + Sized,
    {
        let prefix = TransitMode::ClientToServerRequest as u8 as char;
        let stringified_data =
            serde_to_string(&data).map_err(|e| ClientError::Parse(e.to_string()))?;
        let stringified_data = match stringified_data.as_str() {
            "null" => "{}",
            stringified => stringified,
        };

        let metadata = MetaData {
            time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| ClientError::Unknown(e.to_string()))?
                .as_secs_f64(),
        };
        let stringified_metadata =
            serde_to_string(&metadata).map_err(|e| ClientError::Parse(e.to_string()))?;
        writeln!(
            self.serial,
            "{prefix}{command}${stringified_data}${stringified_metadata}"
        )
        .map_err(|e| ClientError::Serial(e.to_string()))
    }

    /// Helper function to encapsulate a *response* into an event
    fn encapsulate_response_to_event<S>(command: Command, metadata: MetaData, value: S) -> Event<S>
    where
        S: Serialize + for<'a> Deserialize<'a> + Sized,
    {
        Event {
            command,
            transit_mode: TransitMode::ServerToClientResponse,
            transit_type: TransitType::Response,
            value,
            metadata,
        }
    }

    /// Parse response
    pub fn parse_response(data: &str) -> Result<Response, ClientError> {
        // Sanity check (to prevent out of range panics)
        if data.len() < 5 {
            Err(ClientError::Parse(
                "Too short to be a valid response".to_owned(),
            ))?;
        }

        // Find the command
        let split_data = data.split('$').collect::<Vec<_>>();
        let command = Command::try_from(split_data[0][1..].to_string())?;

        // Find the metadata
        let metadata_data = split_data[2];
        // Parse the metadata
        let metadata = serde_from_str::<MetaData>(metadata_data)?;

        // Get the response
        // XXX: Serde thinks "{}" is a map while "null" is perfectly plausible
        // pertaining proportionally for paragliding pedantically in terms of
        // parsing plainly a plain struct free of frills -- fields
        let response_data = match split_data[1] {
            "{}" => "null",
            non_empty => non_empty,
        };

        // Parse the response (if error then parse that)
        match command {
            // An unknown command
            Command::Error => {
                let value = serde_from_str::<ErrorResponse>(response_data)?;
                Ok(Response::Error(Self::encapsulate_response_to_event(
                    command, metadata, value,
                )))
            }
            _ => {
                use Command::*;
                Ok(match command {
                    Ping => Response::Ping(Self::encapsulate_response_to_event(
                        command,
                        metadata,
                        serde_from_str::<PingResponse>(response_data)?,
                    )),
                    Start => Response::Start(Self::encapsulate_response_to_event(
                        command,
                        metadata,
                        serde_from_str::<StartResponse>(response_data)?,
                    )),
                    Stop => Response::Stop(Self::encapsulate_response_to_event(
                        command,
                        metadata,
                        serde_from_str::<StopResponse>(response_data)?,
                    )),
                    Status => Response::Status(Self::encapsulate_response_to_event(
                        command,
                        metadata,
                        serde_from_str::<StatusResponse>(response_data)?,
                    )),
                    StaticStatus => Response::StaticStatus(Self::encapsulate_response_to_event(
                        command,
                        metadata,
                        serde_from_str::<StaticStatusResponse>(response_data)?,
                    )),
                    _ => unreachable!(),
                })
            }
        }
    }
}
