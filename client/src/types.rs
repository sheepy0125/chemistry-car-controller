/*!
 * Types for the client
 * Created by sheepy0125 | MIT License | 2023-02-19
 */

/***** Setup *****/
// Imports
use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

// Constants
pub const BAUD_RATE: u32 = 115200_u32;

/***** Error *****/
#[derive(ThisError, Debug)]
pub enum Error {
    #[error("There was an error with parsing: {0}")]
    ParseError(String),
}
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::ParseError(value.to_string())
    }
}

/***** Commands *****/
/* These here are Rust bindings to the commands, arguments, and responses documented
 * in `readme_data_transmission.md` in the project root.
 * The commands listed below are between the client to the server and do not interface
 * with the R41Z, only through. */

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Ping,   // PingArguments, PingResponse
    Start,  // StartArguments, StartResponse
    Stop,   // StopArguments, StopResponse
    Status, // StatusArguments, StatusResponse
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PingArguments {
    /// Unix epoch timestamp
    pub time: f64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct PingResponse {
    /// Unix epoch timestamp
    pub time: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartArguments {
    /// The distance to travel in centimeters
    pub distance: f64,
    /// Whether to spin the motors forward or backward in order to propel the car
    pub forward: bool,
    /// Whether to reverse the motors to brake
    pub reverse_brake: bool,
}
pub type StartResponse = ();

pub type StopArguments = ();
pub type StopResponse = ();

pub type StatusArguments = ();
#[derive(Serialize, Deserialize, Debug)]
pub struct StatusResponse {
    /// Whether the car is running and the motors a turnin'
    pub running: bool,
    /// The amount of time the server software has been running for in seconds
    pub uptime: usize,
    /// The amount of the time the car has been running for in seconds.
    /// If not `running`, this is `0`.
    pub runtime: usize,
    /// The distance traveled in seconds.
    /// If not `running`, this is `0.0`.
    pub distance: f64,
    /// The latest readings from the accelerometer
    pub accelerometer_readings: AccelerometerReadings,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct AccelerometerReadings {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
