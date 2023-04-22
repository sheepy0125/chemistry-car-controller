/*
 * The Raspberry PI 3B GUI for car controller
 * Created by sheepy0125
 * 2023-02-16
 */

/***** Setup *****/
// Imports
use eframe::{
    egui::{Button, CentralPanel, Slider, TopBottomPanel, Ui},
    epaint::vec2,
    run_native, App, NativeOptions,
};
use serialport::{new as new_serialport, SerialPort};
use std::{
    env::args,
    fmt::Display,
    mem::transmute,
    time::{Duration, Instant},
};
mod bindings;
use bindings::*;
mod events;
use events::*;

// Constants
const STATUS_POLL_DURATION: Duration = Duration::from_millis(1000);
const SHOULD_REVERSE_BRAKE: bool = true;
const SHOULD_GO_FORWARD: bool = true;

/***** Helper functions *****/
fn log_error<E>(e: E)
where
    E: Display,
{
    println!("{e}");
}

/***** Pages *****/
#[repr(u8)]
#[derive(Copy, Clone)]
enum Pages {
    Start = 0_u8,
    /// Input the distance to travel
    DistanceInput = 1_u8,
    /// General status of the car (and e-stop)
    Status = 2_u8,
    /// Car has been stopped
    Stopped = 3_u8,
    End = 4_u8,
}
impl TryFrom<u8> for Pages {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        // Ensure not out of bounds
        if value > Self::End as u8 {
            Err(())?;
        }
        // Safety: not out of bounds
        Ok(unsafe { transmute((Self::Start as u8) + value) })
    }
}
impl Pages {
    fn at_boundary(option: &Pages) -> bool {
        use Pages::*;
        match *option {
            Start => true,
            End => true,
            _ => false,
        }
    }

    fn next(option: &Pages) -> Option<Pages> {
        let next_option = Pages::try_from((*option as u8) + 1_u8).ok()?;
        if Self::at_boundary(&next_option) {
            None?;
        }
        Some(next_option)
    }

    fn previous(option: &Pages) -> Option<Pages> {
        let option_u8 = *option as u8;
        // Ensure no underflow
        if option_u8 == 0 {
            None?;
        }
        let previous_option = Pages::try_from(option_u8 - 1_u8).ok()?;
        if Self::at_boundary(&previous_option) {
            None?;
        }
        Some(previous_option)
    }
}

/// What the button to go to the next page should show
enum NextPageButton {
    /// Show `Next`
    Regular,
    /// Show custom text
    Custom(&'static str),
    /// None
    NoNextPage,
}

/***** Client data *****/
struct ClientData {
    started: bool,
    start_response: Option<Event<StartResponse>>,
    /// Will be `None` every time no data is read from the serial
    rx: Option<String>,
    /// Distance is in centimeters
    distance: Option<f32>,
    static_status: Option<Event<StaticStatusResponse>>,
    status: Option<Event<StatusResponse>>,
    sent_static_status: bool,
    sent_stop_request: bool,
}
impl Default for ClientData {
    fn default() -> Self {
        Self {
            distance: None,
            static_status: None,
            status: None,
            rx: None,
            sent_static_status: false,
            started: false,
            start_response: None,
            sent_stop_request: false,
        }
    }
}

/***** Client GUI *****/
trait ClientGUIHandlers {
    fn new(serial_event_propagator: SerialEventPropagator) -> Self;
    fn distance_input_page(&mut self, ctx: &mut Ui) -> NextPageButton;
    fn stopped_page(&mut self, ctx: &mut Ui) -> NextPageButton;
    fn status_page(&mut self, ctx: &mut Ui) -> NextPageButton;
}
struct ClientGUI {
    serial_event_propagator: SerialEventPropagator,
    status: Option<StatusResponse>,
    next_status_update: Instant,
    selected_option: Pages,
    data: ClientData,
    error_message: Option<String>,
}
impl ClientGUIHandlers for ClientGUI {
    fn new(serial_event_propagator: SerialEventPropagator) -> Self {
        Self {
            serial_event_propagator,
            status: None,
            next_status_update: Instant::now(),
            selected_option: Pages::next(&Pages::Start).unwrap(),
            data: ClientData::default(),
            error_message: None,
        }
    }

    fn distance_input_page(&mut self, ui: &mut Ui) -> NextPageButton {
        let mut distance = self.data.distance.unwrap_or_default();

        ui.heading("Input distance");
        ui.horizontal(|ui| {
            ui.label("Centimeters");
            if ui.button("/\\").clicked() {
                distance += 10.0_f32;
                if distance > 2_000.0_f32 {
                    distance = 2_000.0_f32;
                }
            }
            if ui.button("\\/").clicked() {
                distance -= 10.0_f32;
                if distance < 0.0_f32 {
                    distance = 0.0_f32;
                }
            }
        });
        ui.add(Slider::new(&mut distance, 0.0f32..=2_000.0_f32));

        if distance != 0.0_f32 {
            self.data.distance = Some(distance);
            NextPageButton::Custom("Start car")
        } else {
            NextPageButton::NoNextPage
        }
    }

    fn stopped_page(&mut self, ui: &mut Ui) -> NextPageButton {
        // Send stop request if we haven't already
        if !self.data.sent_stop_request {
            self.serial_event_propagator
                .write_to_serial(Command::Stop, StopArguments {})
                .unwrap();
            self.data.sent_stop_request = true;
        }

        ui.label("A stop signal has been sent.");

        NextPageButton::Custom("Restart")
    }

    fn status_page(&mut self, ui: &mut Ui) -> NextPageButton {
        // Get static status if we haven't already
        if !self.data.sent_static_status {
            self.serial_event_propagator
                .write_to_serial(Command::StaticStatus, StaticStatusArguments {})
                .unwrap();
            self.data.sent_static_status = true;
            self.data.rx = None;
        }
        // We've sent the request but not received it
        else if let None = self.data.static_status {
            if let Some(data) = &self.data.rx {
                let parsed =
                    SerialEventPropagator::parse_response::<StaticStatusResponse>(data).unwrap();
                self.data.static_status = match parsed {
                    Ok(parsed) => Some(parsed),
                    Err(e) => {
                        self.error_message = Some(format!(
                            "Error getting static status: {:?}: {}",
                            ServerError::try_from(e.value.error_variant)
                                .unwrap_or(ServerError::AnyOtherError),
                            e.value.message
                        ));
                        None
                    }
                };
            }
            self.data.rx = None;
        }

        // Start the car after we have received static status
        if self.data.static_status.is_some() && !self.data.started {
            self.serial_event_propagator
                .write_to_serial(
                    Command::Start,
                    StartArguments {
                        distance: self.data.distance.unwrap(),
                        reverse_brake: SHOULD_REVERSE_BRAKE,
                        forward: SHOULD_GO_FORWARD,
                    },
                )
                .unwrap();
            self.data.started = true;
            self.data.rx = None;
        }
        // We've sent the request but not received it
        else if let None = self.data.start_response {
            if let Some(data) = &self.data.rx {
                let parsed = SerialEventPropagator::parse_response::<StartResponse>(data).unwrap();
                self.data.start_response = match parsed {
                    Ok(parsed) => Some(parsed),
                    Err(e) => {
                        self.error_message = Some(format!(
                            "Error getting start response: {:?}: {}",
                            ServerError::try_from(e.value.error_variant)
                                .unwrap_or(ServerError::AnyOtherError),
                            e.value.message
                        ));
                        None
                    }
                };
            }
            self.data.rx = None;
        }

        // Handle dynamic status after we have started
        if self.data.started && self.data.start_response.is_some() {
            // let current_time = Instant::now();
            // if current_time.duration_since(self.next_status_update) > STATUS_POLL_DURATION {
            // self.next_status_update += STATUS_POLL_DURATION;
            // self.serial_event_propagator
            // .write_to_serial(Command::Status, StatusArguments {})
            // .unwrap();
            // }

            // Parse status
            if let Some(data) = &self.data.rx {
                let parsed = SerialEventPropagator::parse_response::<StatusResponse>(data).unwrap();
                let status = match parsed {
                    Ok(parsed) => Some(parsed),
                    Err(e) => {
                        self.error_message = Some(format!(
                            "Error getting dynamic status: {:?}: {}",
                            ServerError::try_from(e.value.error_variant)
                                .unwrap_or(ServerError::AnyOtherError),
                            e.value.message
                        ));
                        None
                    }
                };
                if let Some(status) = status {
                    self.data.status = Some(status);
                }
            }
        }

        // Display status
        if let Some(event) = &self.data.static_status {
            let status = &event.value;
            ui.label(format!("Number of magnets: {}", status.number_of_magnets));
            ui.label(format!(
                "Wheel diameter: {} inches / {} centimeters",
                status.wheel_diameter / 2.54,
                status.wheel_diameter
            ));
        } else {
            ui.label("No static status");
        }
        if let Some(event) = &self.data.status {
            let status = &event.value;
            ui.label(format!(
                "Running: {}",
                match status.running {
                    true => "YES",
                    false => "NO",
                }
            ));
            ui.label(format!("Server uptime: {}", status.uptime));
            ui.label(format!("Runtime: {}", status.runtime));
            ui.label(format!(
                "Distance traveled in centimeters: {}",
                status.distance.distance
            ));
            ui.label(format!(
                "Velocity in centimeters/second: {}",
                status.distance.velocity
            ));
            ui.label(format!(
                "Magnet odometer hits: {}",
                status.distance.magnet_hit_counter
            ));
        } else {
            ui.label("No dynamic status");
        }

        NextPageButton::Custom("Stop")
    }
}
impl App for ClientGUI {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        // Read from serial
        self.data.rx = match self.serial_event_propagator.read_from_serial() {
            Some(read) => Some(read.unwrap()),
            None => None,
        };

        CentralPanel::default().show(&ctx, |ui| {
            ui.heading("CHARGE Dynamics' Cool Chemistry Car Controller (C4)");
            if let Some(e) = &self.error_message {
                ui.heading(e);
            }
            ui.separator();

            use Pages::*;
            let next_page_button = match self.selected_option {
                Stopped => self.stopped_page(ui),
                DistanceInput => self.distance_input_page(ui),
                Status => self.status_page(ui),
                _ => NextPageButton::NoNextPage,
            };

            if !matches!(next_page_button, NextPageButton::NoNextPage) {
                let next_page_option = Pages::next(&self.selected_option).unwrap_or(Pages::End);
                let next_button = ui.add_sized(
                    [120., 40.],
                    Button::new(match next_page_button {
                        NextPageButton::Custom(text) => text,
                        NextPageButton::Regular => "Next",
                        _ => unreachable!(),
                    }),
                );
                if next_button.clicked() {
                    if !Pages::at_boundary(&next_page_option) {
                        self.selected_option = next_page_option;
                    } else {
                        self.selected_option = Pages::next(&Pages::Start).unwrap();
                        // Reset everything
                        self.data.sent_static_status = false;
                        self.data.sent_stop_request = false;
                        self.data.started = false;
                        self.data.distance = None;
                        self.data.start_response = None;
                        self.data.static_status = None;
                        self.data.status = None;
                        self.data.rx = None;
                        self.status = None;
                    }
                }
            }
        });

        ctx.request_repaint();
    }
}

fn main() -> Result<(), ()> {
    // Connect to the server serial port
    let serial_port = args()
        .nth(1_usize)
        .expect("Please enter the serial port device (e.g. `cargo run -- /dev/ttyACM0`");
    let serial = new_serialport(serial_port.clone(), BAUD_RATE)
        .timeout(Duration::from_millis(500_u64))
        .open()
        .expect(
            format!(
            "Failed to connect to the serial port. Please ensure it is connected on {serial_port}"
        )
            .as_str(),
        );

    // Create the serial event propagator
    let serial_event_propagator = SerialEventPropagator::new(serial);

    // Create app
    let app = ClientGUI::new(serial_event_propagator);

    // Make the window
    let options = NativeOptions {
        resizable: false,
        initial_window_size: Some(vec2(480_f32, 320_f32)),
        always_on_top: true,
        ..Default::default()
    };
    run_native("Chemistry Car GUI", options, Box::new(|_cc| Box::new(app)))
        .map_err(|e| log_error(e))?;

    Ok(())
}
