/*!
 * Helper types
 * Created by sheepy0125 | MIT License | 2022-02-19
 */

/***** Setup *****/
// Imports
use bluer::Error as BluerError;
use serialport::Error as SerialPortError;
use std::io::Error as IoError;
use thiserror::Error as ThisError;

// Constants
pub const POLL_DELAY: u64 = 20; // Milliseconds

/***** Enums *****/

/// Error
#[derive(Debug, ThisError)]
pub enum Error {
    #[error("A bluetooth error has occurred: {0}")]
    BluerError(BluerError),
    #[error("A bluetooth error has occurred: {0}")]
    BluetoothError(BluetoothError),
    #[error("A serial error has occurred: {0}")]
    SerialError(SerialPortError),
    #[error("An IO error has occurred: {0}")]
    IoError(IoError),
}

/// A bluetooth error that has not been propogated through Bluer
#[derive(Debug, ThisError)]
pub enum BluetoothError {
    #[error("The service needed could not be found")]
    MissingService,
    #[error("The characteristics needed could not be found")]
    MissingCharacteristic,
    #[error("Failed to get an adapter event")]
    MissingAdapterEvent,
}

impl From<BluerError> for Error {
    fn from(value: BluerError) -> Self {
        Self::BluerError(value)
    }
}
impl From<SerialPortError> for Error {
    fn from(value: SerialPortError) -> Self {
        Self::SerialError(value)
    }
}
impl From<IoError> for Error {
    fn from(value: IoError) -> Self {
        Self::IoError(value)
    }
}
