/*!
 * Shared types for the client
 * Created by sheepy0125 | MIT license | 2023-02-23
 */

use std::time::Duration;

pub const WIDTH: f32 = 480.0;
pub const HEIGHT: f32 = 320.0;
pub const SERIAL_DELAY_TIME: f64 = 0.10;
pub const STATUS_POLL_DURATION: Duration = Duration::from_millis(1000);
pub const SHOULD_REVERSE_BRAKE: bool = true;
pub const MAX_DISTANCE_RANGE_CENTIMETERS: f64 = 1_000.0;
