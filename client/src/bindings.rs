/*!
 * Bindings for the client
 * Created by sheepy0125 | MIT License | 2023-02-19
 */

/***** Setup *****/
// Imports
use crate::events::Event;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, mem::transmute};
use thiserror::Error as ThisError;

// Constants
pub const BAUD_RATE: u32 = 115200_u32;

/***** Error *****/

/// An error from the client
#[derive(ThisError, Debug)]
pub enum ClientError {
    #[error("There was an error with parsing: {0}")]
    Parse(String),
    #[error("There was an error with running: {0}")]
    Run(String),
    #[error("There was an error with the serial connection: {0}")]
    Serial(String),
    #[error("An unknown error occurred: {0}")]
    Unknown(String),
    #[error("{0}")]
    Server(String),
}
impl From<serde_json::Error> for ClientError {
    fn from(value: serde_json::Error) -> Self {
        Self::Parse(value.to_string())
    }
}

/// An error returned by the server
#[repr(u8)]
#[derive(Deserialize, Serialize, Debug, FromPrimitive, Clone, Copy)]
pub enum ServerError {
    MalformedRequestFailedPrefixParsing = 0_u8,
    MalformedRequestFailedCommandParsing = 1_u8,
    MalformedRequestFailedSeparatorParsing = 2_u8,
    MalformedRequestFailedArgumentsParsing = 3_u8,
    MalformedRequestFailedMetadataParsing = 4_u8,
    MalformedRequestTypeError = 5_u8,
    MalformedRequestOtherError = 6_u8,
    Filler7 = 7_u8,
    Filler8 = 8_u8,
    Filler9 = 9_u8,
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
        Ok(unsafe { transmute(value) })
    }
}
impl ToString for ServerError {
    fn to_string(&self) -> String {
        match *self as u8 {
            0 => "Malformed request - Failed prefix parsing",
            1 => "Malformed request - Failed command parsing",
            2 => "Malformed request - Failed separator parsing",
            3 => "Malformed request - Failed arguments parsing",
            4 => "Malformed request - Failed metadata parsing",
            5 => "Malformed request - Type error",
            6 => "Malformed request - Other error",
            10 => "Malformed response - Type error",
            11 => "Malformed response - Other error",
            21 => "Failed to start - Already started",
            22 => "Failed to start - Magnet odometer failed",
            23 => "Failed to start - Motor control failed",
            24 => "Failed to start - Could not acquire distance mutex lock",
            25 => "Failed to stop - Not started",
            26 => "Failed to stop - Start thread would not respond",
            27 => "Failed status - Could not acquire distance mutex lock",
            28 => "Failed ping - Negative latency",
            _ => "Any other error",
        }
        .to_string()
    }
}

#[derive(Deserialize, Serialize)]
pub struct ErrorResponse {
    /// This is a u8 for Serde
    pub error_variant: u8,
    pub message: String,
}

/***** Generic bindings *****/

/// Metadata is sent alongside with the request and response
#[derive(Serialize, Deserialize, Debug)]
pub struct MetaData {
    pub time: f64,
}

/// The mode of transit
#[repr(u8)]
#[derive(FromPrimitive)]
pub enum TransitMode {
    ClientToServerRequest = b'?',
    ServerToClientResponse = b'~',
}

/// The type of transit
#[derive(Debug)]
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
    Error,
}
impl TryFrom<String> for Command {
    type Error = ClientError; /* Potential type collision */

    fn try_from(value: String) -> Result<Self, ClientError> {
        use Command::*;
        match value.to_ascii_uppercase().as_str() {
            "PING" => Ok(Ping),
            "START" => Ok(Start),
            "STOP" => Ok(Stop),
            "STATICSTATUS" => Ok(StaticStatus),
            "STATUS" => Ok(Status),
            "UNKNOWN" | "ERROR" => Ok(Error),
            _ => Err(ClientError::Parse(format!(
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
                Error => "ERROR",
            }
        )
    }
}

/***** Request and response bindings for commands *****/

/// Possible responses
pub enum Response {
    Ping(Event<PingResponse>),
    Start(Event<StartResponse>),
    Stop(Event<StopResponse>),
    Status(Event<StatusResponse>),
    StaticStatus(Event<StaticStatusResponse>),
    Error(Event<ErrorResponse>),
}

// Ping

#[derive(Serialize, Deserialize)]
pub struct PingArguments {
    pub time: f64,
}
#[derive(Deserialize, Serialize)]
pub struct PingResponse {
    pub sent_time: f64,
}

// Start

#[derive(Serialize, Deserialize)]
pub struct StartArguments {
    pub distance: f64,
    pub reverse_brake: bool,
}
#[derive(Deserialize, Serialize)]
pub struct StartResponse;

// Stop
#[derive(Serialize, Deserialize)]
pub struct StopArguments;
#[derive(Deserialize, Serialize)]
pub struct StopResponse;

// Static status

#[derive(Serialize, Deserialize)]
pub struct StaticStatusArguments; //             I don't have the Pi with me so I am
#[derive(Deserialize, Serialize)] //             pretending to be it :)
pub struct StaticStatusResponse {
    pub number_of_magnets: usize,
    pub wheel_diameter: f64,
}

// Regular (dynamic) status

#[derive(Deserialize, Serialize)]
pub struct DistanceInformation {
    /// Centimeters
    pub distance: f64,
    pub velocity: f64,
    pub magnet_hit_counter: usize,
}
#[derive(Serialize, Deserialize)]
pub struct StatusArguments;
#[derive(Deserialize, Serialize)]
pub struct StatusResponse {
    pub running: bool,
    pub uptime: usize,
    pub runtime: usize,
    pub distance: DistanceInformation,
}

/***** Client status *****/

#[repr(u8)]
#[derive(Default, Copy, Clone)]
pub enum ClientStatus {
    #[default]
    GatheringData = 0_u8,
    // Pinging
    SendingPing = 1_u8,
    ReceivingPing = 2_u8,
    // Static status
    RequestingStaticStatus = 3_u8,
    ReceivingStaticStatus = 4_u8,
    // Dynamic status
    RequestingStart = 5_u8,
    ReceivingStatus = 6_u8,
    // Stopping
    RequestingStop = 7_u8,
    Finished = 8_u8,
}
impl ToString for ClientStatus {
    fn to_string(&self) -> String {
        use ClientStatus::*;
        match self {
            GatheringData => "Gathering user input",
            SendingPing => "Pinging (send)",
            ReceivingPing => "Pinging (receive)",
            RequestingStaticStatus => "Getting car information (send)",
            ReceivingStaticStatus => "Getting car information (receive)",
            RequestingStart => "Starting the car (send)",
            ReceivingStatus => "Getting information about the car",
            RequestingStop => "Stopping the car (send)",
            Finished => "Finished",
        }
        .into()
    }
}
impl TryFrom<u8> for ClientStatus {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Self::Finished as u8 {
            Err(())?;
        }
        Ok(unsafe { transmute((Self::GatheringData as u8) + value) })
    }
}
impl ClientStatus {
    /// If at a boundary, this will return the same thing
    pub fn next(self) -> Self {
        Self::try_from((self as u8) + 1_u8).unwrap_or(Self::Finished)
    }
}
