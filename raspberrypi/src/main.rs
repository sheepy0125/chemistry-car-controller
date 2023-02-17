/*
 * The Raspberry PI GUI for the Arduino car controller
 * Created by sheepy0125
 * 2023-02-16
 */

use eframe::{egui::CentralPanel, epaint::vec2, run_native, App, NativeOptions};
/***** Setup *****/
// Imports
use serialport::{available_ports, new as new_serialport, SerialPort, TTYPort};
use std::{
    env::args,
    fmt::Display,
    mem::transmute,
    time::{Duration, Instant},
};
// #[path="../../shared/types.rs"]
mod types;
use types::*;

// Constants
const STATUS_POLL_DURATION: Duration = Duration::from_millis(250);

/***** Helper functions *****/
fn log_error<E>(e: E)
where
    E: Display,
{
    println!("{e}");
}

/// Block and read until there is a newline from the serial
fn read_serial_until_newline(serial: &mut Box<dyn SerialPort>) -> String {
    let mut return_string = String::new();
    let mut character_buffer = [0_u8; 1];
    loop {
        character_buffer[0] = 0_u8;
        serial.read_exact(&mut character_buffer).unwrap();
        let character = character_buffer[0] as char;
        match character {
            '\n' => break,
            character => return_string.push(character),
        }
    }
    return_string
}

/***** Menu options *****/
#[repr(u8)]
#[derive(Copy, Clone)]
enum MenuOption {
    Start = 0_u8,
    /// Connect to the R41Z middleman
    ConnectBluetooth = 1_u8,
    /// Establish a serial connection
    ConnectSerial = 2_u8,
    /// Input the distance to travel
    DistanceInput = 3_u8,
    /// Confirm ready to go
    Confirm = 4_u8,
    /// General status of the car
    Status = 5_u8,
    End = 6_u8,
}
impl TryFrom<u8> for MenuOption {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        // Ensure not out of bounds
        if value > MenuOption::End as u8 {
            Err(())?;
        }
        // Safety: not out of bounds
        Ok(unsafe { transmute((Self::Start as u8) + value) })
    }
}
impl MenuOption {
    fn at_boundary(option: &MenuOption) -> bool {
        use MenuOption::*;
        match *option {
            Start => true,
            End => true,
            _ => false,
        }
    }

    fn next(option: &MenuOption) -> Option<MenuOption> {
        let next_option = MenuOption::try_from((*option as u8) + 1_u8).ok()?;
        if Self::at_boundary(&next_option) {
            None?;
        }
        Some(next_option)
    }

    fn previous(option: &MenuOption) -> Option<MenuOption> {
        let option_u8 = *option as u8;
        // Ensure no underflow
        if option_u8 == 0 {
            None?;
        }
        let previous_option = MenuOption::try_from(option_u8 - 1_u8).ok()?;
        if Self::at_boundary(&previous_option) {
            None?;
        }
        Some(previous_option)
    }
}

trait ArduinoGUIHandlers {
    fn new(serial: Box<dyn SerialPort>) -> Self;
    fn update_status(&mut self) -> Result<(), ()>;
}
struct ArduinoGUI {
    serial: Box<dyn SerialPort>,
    status: Option<Status>,
    next_status_update: Instant,
    selected_option: MenuOption,
}
impl ArduinoGUIHandlers for ArduinoGUI {
    fn new(serial: Box<dyn SerialPort>) -> Self {
        Self {
            serial,
            status: None,
            next_status_update: Instant::now(),
            selected_option: MenuOption::Start,
        }
    }

    fn update_status(&mut self) -> Result<(), ()> {
        writeln!(&mut self.serial, "STATUS$").unwrap();
        let status_string = read_serial_until_newline(&mut self.serial);
        let status_parsed = Status::try_from(status_string.as_str())?;
        self.status = Some(status_parsed);

        Ok(())
    }
}
impl App for ArduinoGUI {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        let mut error_message = None;

        // Get the status if we need to
        let current_time = Instant::now();
        if current_time.duration_since(self.next_status_update) > STATUS_POLL_DURATION {
            self.next_status_update += STATUS_POLL_DURATION;
            if let Err(()) = self.update_status() {
                error_message = Some("Failed to get status from the Arduino!");
            };
        }

        if let Some(error_message) = error_message {
            CentralPanel::default().show(&ctx, |ui| ui.label(error_message));
        }

        ctx.request_repaint();
    }
}

fn main() -> Result<(), ()> {
    // Connect to the Arduino serial port
    let serial_port = args()
        .nth(1_usize)
        .expect("Please enter the serial port device (e.g. `./gui.x64 /dev/ttyACM0`");
    let serial = new_serialport(serial_port.clone(), BAUD_RATE)
        .timeout(Duration::from_millis(500_u64))
        .open()
        .expect(
            format!(
            "Failed to connect to the serial port. Please ensure it is connected on {serial_port}"
        )
            .as_str(),
        );

    // Make the window
    let options = NativeOptions {
        resizable: false,
        initial_window_size: Some(vec2(480_f32, 320_f32)),
        always_on_top: true,
        ..Default::default()
    };
    let app = ArduinoGUI::new(serial);
    run_native("Chemistry Car GUI", options, Box::new(|_cc| Box::new(app)))
        .map_err(|e| log_error(e))?;

    Ok(())
}
