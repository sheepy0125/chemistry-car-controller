/*!
 * Bindings for the client
 * Created by sheepy0125 | MIT License | 2023-02-19
 */

/***** Setup *****/
// Imports
use lazy_static::lazy_static;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, mem::transmute};
use thiserror::Error as ThisError;

// Constants
pub const BAUD_RATE: u32 = 115200_u32;

/***** Error *****/
/// An error from the client
#[derive(ThisError, Debug)]
pub enum ClientError {
    #[error("There was an error with parsing: {0}")]
    ParseError(String),
    #[error("There was an error with the serial connection: {0}")]
    SerialError(String),
    #[error("An unknown error occurred: {0}")]
    UnknownError(String),
}
impl From<serde_json::Error> for ClientError {
    fn from(value: serde_json::Error) -> Self {
        Self::ParseError(value.to_string())
    }
}

/// An error returned by the server
#[repr(u8)]
#[derive(Deserialize, Serialize, Debug, FromPrimitive)]
pub enum ServerError {
    MalformedRequestFailedPrefixParsing = 0_u8,
    MalformedRequestFailedCommandParsing = 1_u8,
    MalformedRequestFailedSeparatorParsing = 2_u8,
    MalformedRequestFailedArgumentsParsing = 3_u8,
    MalformedRequestFailedMetadataParsing = 4_u8,
    MalformedRequestTypeError = 5_u8,
    MalformedRequestOtherError = 6_u8,
    MalformedResponseTypeError = 10_u8,
    MalformedResponseOtherError = 11_u8,
    FailedToStartAlreadyStarted = 21_u8,
    FailedToStartMagnetOdometerFailed = 22_u8,
    FailedToStartMotorControlFailed = 23_u8,
    FailedToStopNotStarted = 24_u8,
    FailedToStopStartThreadWouldNotRespond = 25_u8,
    FailedStatusCouldNotAcquireDistanceLock = 26_u8,
    FailedPingNegativeLatency = 27_u8,
    _UpperBound = 28_u8,
    AnyOtherError = 99_u8,
}
impl TryFrom<u8> for ServerError {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        // Ensure not out of bounds
        if value >= Self::_UpperBound as u8 && value != 99_u8 {
            Err(())?;
        }
        // Safety: not out of bounds
        Ok(unsafe { transmute(value as u8) })
    }
}

#[derive(Deserialize, Serialize)]
pub struct ErrorResponse {
    /// This is a u8 for Serde
    pub error_variant: u8,
    pub message: String,
}
impl Default for ErrorResponse {
    fn default() -> Self {
        unreachable!()
    }
}

/***** General bindings *****/

/// Metadata is sent alongside with the request and response
#[derive(Serialize, Deserialize, Debug)]
pub struct MetaData {
    pub time: f32,
}

/// The mode of transit
#[repr(u8)]
#[derive(FromPrimitive)]
pub enum TransitMode {
    ClientToServerRequest = '?' as u8,
    ServerToClientResponse = '~' as u8,
}

/// The type of transit
pub enum TransitType {
    Request,
    Response,
}

/***** Commands *****/
#[derive(PartialEq, Eq)]
pub enum Command {
    Ping,
    Start,
    Stop,
    StaticStatus,
    Status,
    Unknown,
}
impl TryFrom<String> for Command {
    type Error = ClientError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        use Command::*;
        match value.to_ascii_uppercase().as_str() {
            "PING" => Ok(Ping),
            "START" => Ok(Start),
            "STOP" => Ok(Stop),
            "STATICSTATUS" => Ok(StaticStatus),
            "STATUS" => Ok(Status),
            "UNKNOWN" => Ok(Unknown),
            _ => Err(ClientError::ParseError(format!(
                "Failed to parse command from {value}"
            ))),
        }
    }
}
impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Command::*;
        write!(
            f,
            "{}",
            match *self {
                Ping => "PING",
                Start => "START",
                Stop => "STOP",
                StaticStatus => "STATICSTATUS",
                Status => "STATUS",
                Unknown => "UNKNOWN",
            }
        )
    }
}

// Ping

#[derive(Serialize, Deserialize)]
pub struct PingArguments;
#[derive(Deserialize, Serialize)]
pub struct PingResponse;

// Start

#[derive(Serialize, Deserialize)]
pub struct StartArguments {
    pub distance: f32,
    pub forward: bool,
    pub reverse_brake: bool,
}
impl Default for StartArguments {
    fn default() -> Self {
        unreachable!()
    }
}
#[derive(Deserialize, Serialize)]
pub struct StartResponse;
impl Default for StartResponse {
    fn default() -> Self {
        Self {}
    }
}

// Stop

#[derive(Serialize, Deserialize)]
pub struct StopArguments;
impl Default for StopArguments {
    fn default() -> Self {
        Self {}
    }
}
#[derive(Deserialize, Serialize)]
pub struct StopResponse;
impl Default for StopResponse {
    fn default() -> Self {
        Self {}
    }
}

// Static status

#[derive(Serialize, Deserialize)]
pub struct StaticStatusArguments;
impl Default for StaticStatusArguments {
    fn default() -> Self {
        Self {}
    }
}
#[derive(Deserialize, Serialize)]
pub struct StaticStatusResponse {
    pub number_of_magnets: usize,
    pub wheel_diameter: f32,
}
impl Default for StaticStatusResponse {
    fn default() -> Self {
        unreachable!()
    }
}

// Regular (dynamic) status

#[derive(Deserialize, Serialize)]
pub struct DistanceInformation {
    /// Centimeters
    pub distance: f32,
    pub velocity: f32,
    pub magnet_hit_counter: usize,
}
#[derive(Serialize, Deserialize)]
pub struct StatusArguments;
impl Default for StatusArguments {
    fn default() -> Self {
        unreachable!()
    }
}
#[derive(Deserialize, Serialize)]
pub struct StatusResponse {
    pub running: bool,
    pub uptime: usize,
    pub runtime: usize,
    pub distance: DistanceInformation,
}
impl Default for StatusResponse {
    fn default() -> Self {
        unreachable!()
    }
}
