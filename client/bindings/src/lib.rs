/*!
 * Shared types for all client related stuff
 * Created by sheepy0125 | MIT License | 2023-02-19
 */

/***** Setup *****/
// Imports
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{convert::TryFrom, fmt::Display, mem::transmute};
use thiserror::Error as ThisError;

// Constants
pub const BAUD_RATE: u32 = 115200_u32;

/***** Events *****/

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
    #[error("Failed handling CSV file: {0}")]
    CSV(String),
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
    _RequestErrorUpperBound = 7_u8,
    _ResponseErrorLowerBound = 8_u8,
    MalformedResponseTypeError = 10_u8,
    MalformedResponseOtherError = 11_u8,
    _ResponseErrorUpperBound = 12_u8,
    _SpecificErrorLowerBound = 13_u8,
    FailedToStartAlreadyStarted = 21_u8,
    FailedToStartMagnetOdometerFailed = 22_u8,
    FailedToStartMotorControlFailed = 23_u8,
    FailedToStopNotStarted = 24_u8,
    FailedToStopStartThreadWouldNotRespond = 25_u8,
    FailedStatusCouldNotAcquireDistanceLock = 26_u8,
    FailedPingNegativeLatency = 27_u8,
    _SpecificErrorUpperBound = 28_u8,
    AnyOtherError = 99_u8,
}
impl TryFrom<u8> for ServerError {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        // Ensure not out of bounds
        if (value > 0 && value <= Self::_RequestErrorUpperBound as u8)
            || (value >= Self::_ResponseErrorLowerBound as u8
                && value <= Self::_ResponseErrorUpperBound as u8)
            || (value >= Self::_SpecificErrorLowerBound as u8
                && value <= Self::_SpecificErrorUpperBound as u8)
        {
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
    ClientToSerialBridgeRequest = b'^',
    SerialBridgeToClientResponse = b'&',
}
impl From<Command> for TransitMode {
    fn from(value: Command) -> Self {
        use Command::*;
        use TransitMode::*;
        match value {
            Ping => ClientToServerRequest,
            Start => ClientToServerRequest,
            Stop => ClientToServerRequest,
            Status => ClientToServerRequest,
            StaticStatus => ClientToServerRequest,
            Error => ClientToServerRequest,
            Connect => ClientToSerialBridgeRequest,
            Disconnect => ClientToSerialBridgeRequest,
            BluetoothStatus => ClientToSerialBridgeRequest,
        }
    }
}

/// The type of transit
#[derive(Debug)]
pub enum TransitType {
    Request,
    Response,
}

/***** Commands *****/

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Command {
    /* Server commands */
    Ping,
    Start,
    Stop,
    StaticStatus,
    Status,
    Error,
    /* Serial bridge commands */
    Connect,
    Disconnect,
    BluetoothStatus,
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
            "CONNECT" => Ok(Connect),
            "DISCONNECT" => Ok(Disconnect),
            "BLUETOOTHSTATUS" => Ok(BluetoothStatus),
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
                Connect => "CONNECT",
                Disconnect => "DISCONNECT",
                BluetoothStatus => "BLUETOOTHSTATUS",
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
    BluetoothStatus(Event<BluetoothStatusResponse>),
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
pub struct StaticStatusArguments;
#[derive(Deserialize, Serialize)]
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
#[repr(u8)]
#[derive(Deserialize_repr, Serialize_repr, Clone, Copy)]
pub enum StatusStage {
    Stopped = 0_u8,
    Finalized = 4_u8,
    VehementForward = 1_u8,
    StallOvershoot = 2_u8,
    CautiousBackward = 3_u8,
}
impl TryFrom<u8> for StatusStage {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Self::Finalized as u8 {
            Err(())?;
        }
        Ok(unsafe { transmute((Self::Stopped as u8) + value) })
    }
}
impl Display for StatusStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use StatusStage::*;
        writeln!(
            f,
            "{}",
            match *self {
                Stopped => "Stopped",
                Finalized => "Finalized",
                VehementForward => "Forward",
                StallOvershoot => "Coast",
                CautiousBackward => "Backward",
            }
        )
    }
}
#[derive(Serialize, Deserialize)]
pub struct StatusArguments;
#[derive(Deserialize, Serialize)]
pub struct StatusResponse {
    pub running: bool,
    pub uptime: usize,
    pub runtime: usize,
    pub stage: StatusStage,
    pub distance: DistanceInformation,
}

// Bluetooth connect

#[derive(Serialize, Deserialize)]
pub struct BluetoothConnectRequest;

#[derive(Serialize, Deserialize)]
pub struct BluetoothConnectResponse;

// Bluetooth disconnect

#[derive(Serialize, Deserialize)]
pub struct BluetoothDisconnectRequest;
#[derive(Serialize, Deserialize)]
pub struct BluetoothDisconnectResponse;

// Bluetooth status

#[derive(Serialize, Deserialize)]
pub struct BluetoothStatusRequest;
#[derive(Serialize, Deserialize)]
pub struct BluetoothStatusResponse {
    pub connected: bool,
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
