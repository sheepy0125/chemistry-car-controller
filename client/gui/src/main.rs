/*!
 * The Raspberry PI 3B GUI for car controller
 * Created by sheepy0125 | MIT license | 2023-02-16
 */

/***** Setup *****/
// Imports
use bindings::*;
use chrono::{DateTime, Local};
use eframe::{epaint::vec2, run_native, App, NativeOptions};
use egui::{
    Align, Button, Context, Label, Layout, SidePanel, Slider, TextEdit, TopBottomPanel, Ui,
    Visuals, Window,
};
use egui_extras::{Column, TableBuilder};
use egui_file::FileDialog;
use serialport::new as new_serialport;
use smart_default::SmartDefault;
use std::{
    env::args,
    f64::consts::PI,
    path::PathBuf,
    time::{Duration, Instant},
};
pub mod events;
use events::*;
pub mod shared;
use shared::*;
pub mod csv_table;
use csv_table::*;

/***** Client *****/

/// Error message data
pub struct ErrorData {
    pub error: ClientError,
    pub time: DateTime<Local>,
}
impl ErrorData {
    pub fn new(error: ClientError) -> Self {
        Self {
            error,
            time: Local::now(),
        }
    }
}
impl From<ClientError> for ErrorData {
    fn from(value: ClientError) -> Self {
        Self::new(value)
    }
}

/// GUI data
#[derive(SmartDefault)]
pub struct GUIData {
    /// Distance in centimeters
    #[default = 0.0]
    pub distance: f64,
    #[default = false]
    pub reverse_braking: bool,
    #[default = false]
    pub expanded_status_table: bool,
    #[default = true]
    pub show_bluetooth_connect_screen: bool,
    pub current_job: ClientStatus,
    #[default = "status.csv"]
    pub display_file_path: String,
    pub file_path: Option<PathBuf>,
    pub file_dialog: Option<FileDialog>,
}

/// Possible values for the large button
pub enum LargeButton {
    Start,
    Reset,
    Stop,
}
impl ToString for LargeButton {
    fn to_string(&self) -> String {
        match *self {
            Self::Start => "START",
            Self::Reset => "RESET",
            Self::Stop => "STOP",
        }
        .to_owned()
    }
}

pub trait ClientGUIHandlers {
    fn new(serial_event_propagator: SerialEventPropagator) -> Self;
    fn get_serial_responses(&mut self) -> Result<(), ClientError>;
    fn show_error_messages(&mut self, ctx: &Context);
    fn show_bluetooth_connect_screen(&mut self, ctx: &Context);
    fn show_status_table(&self, ui: &mut Ui);
    fn logic(&mut self);
    fn start(&mut self);
    fn stop(&mut self);
    fn reset(&mut self);
}
pub struct ClientGUI {
    pub serial_event_propagator: SerialEventPropagator,
    pub run_data: RunData,
    pub gui_data: GUIData,
    pub errors: Vec<ErrorData>,
}
impl ClientGUIHandlers for ClientGUI {
    fn new(serial_event_propagator: SerialEventPropagator) -> Self {
        Self {
            serial_event_propagator,
            run_data: Default::default(),
            gui_data: Default::default(),
            errors: Default::default(),
        }
    }

    /// Show the bluetooth connection screen
    fn show_bluetooth_connect_screen(&mut self, ctx: &Context) {
        Window::new("Microwave-Blaster 9000 Utility")
            .resizable(false)
            .show(ctx, |ui| {
                let close_button_size = [60., 40.];
                if ui
                    .add_sized(close_button_size, Button::new("Close"))
                    .clicked()
                {
                    self.gui_data.show_bluetooth_connect_screen = false;
                };

                ui.heading(format!(
                    "We *think* we are {}",
                    match self.run_data.bluetooth_bridge_connected {
                        true => "CONNECTED",
                        false => "DISCONNECTED",
                    },
                ));

                ui.horizontal(|ui| {
                    let bluetooth_control_button_size = [80., 40.];
                    if ui
                        .add_sized(bluetooth_control_button_size, Button::new("Connect"))
                        .clicked()
                    {
                        self.serial_event_propagator
                            .write_to_serial(Command::Connect, BluetoothConnectRequest {})
                            .unwrap_or_else(|e| {
                                self.errors
                                    .push(ErrorData::new(ClientError::Serial(e.to_string())))
                            });
                    }
                    if ui
                        .add_sized(bluetooth_control_button_size, Button::new("Disconnect"))
                        .clicked()
                    {
                        self.serial_event_propagator
                            .write_to_serial(Command::Disconnect, BluetoothDisconnectRequest {})
                            .unwrap_or_else(|e| {
                                self.errors
                                    .push(ErrorData::new(ClientError::Serial(e.to_string())))
                            });
                    }
                    if ui
                        .add_sized(bluetooth_control_button_size, Button::new("Update status"))
                        .clicked()
                    {
                        self.serial_event_propagator
                            .write_to_serial(Command::BluetoothStatus, BluetoothStatusRequest {})
                            .unwrap_or_else(|e| {
                                self.errors
                                    .push(ErrorData::new(ClientError::Serial(e.to_string())))
                            });
                    }
                });
            });
    }

    /// Show error messages
    ///
    /// Assumes there are error messages, otherwise the window it shows would be
    /// pretty useless
    fn show_error_messages(&mut self, ctx: &Context) {
        Window::new("Errors!").resizable(false).show(ctx, |ui| {
            ui.heading(match self.errors.len() {
                0 => unreachable!(),
                1 => "An error has occurred!",
                2..=5 => "Some errors have occurred!",
                _ => "Something has *definitely* gone wrong!",
            });

            let clear_errors_button_size = [60., 40.];
            if ui
                .add_sized(clear_errors_button_size, Button::new("Clear"))
                .clicked()
            {
                self.errors.clear();
            };

            let errors_table = TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::auto())
                .column(Column::remainder())
                .min_scrolled_height(0.0);

            errors_table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Time");
                    });
                    header.col(|ui| {
                        ui.strong("Error");
                    });
                })
                .body(|mut body| {
                    for error in self.errors.iter() {
                        let error_text = error.error.to_string();
                        body.row(18.0, |mut row| {
                            row.col(|ui| {
                                ui.label(error.time.format("%H:%M:%S").to_string());
                            });
                            row.col(|ui| {
                                ui.add(
                                    Label::new(error_text)
                                        .wrap(false /* FIXME: fix wrapping */),
                                );
                            });
                        });
                    }
                })
        });
    }

    /// Read the serial port for any response and parse it, placing it in `self.run_data`
    fn get_serial_responses(&mut self) -> Result<(), ClientError> {
        // Get down if available
        let data = match self.serial_event_propagator.read_from_serial()? {
            Some(data) => data,
            None => return Ok(()),
        };

        // Parse into a response
        let parsed_response = SerialEventPropagator::parse_response(&data[..])?;

        // Add to corresponding run data
        use Response::*;
        match parsed_response {
            Ping(resp) => {
                self.run_data.ping_status_response = Some((
                    Box::new(resp),
                    (Local::now().timestamp_millis() as f64) / 1000.0,
                ))
            }
            StaticStatus(resp) => self.run_data.static_status_response = Some(Box::new(resp)),
            Status(resp) => self.run_data.status_responses.push(resp),
            Error(resp) => self.errors.push(ErrorData::new(ClientError::Server(format!(
                "{}: {}",
                ServerError::try_from(resp.value.error_variant)
                    .unwrap_or(ServerError::AnyOtherError)
                    .to_string(),
                resp.value.message
            )))),
            BluetoothStatus(resp) => {
                self.run_data.bluetooth_bridge_connected = resp.value.connected;
            }
            _ => self.run_data.other_responses.push(parsed_response),
        };

        Ok(())
    }

    /// All logic that is run every time the window is updated (i.e. every frame)
    fn logic(&mut self) {
        // Receive new serial information if needed
        {
            // `Instant::elapsed()` *does* exist, but if we are going to update
            // the last_get_time with the current time instead of just adding the
            // delay to it, then it's practical to just get the current time here
            // and use `Instant::duration_since(...)`
            let current_time = Instant::now();
            if current_time.duration_since(self.serial_event_propagator.last_get_time)
                > Duration::from_secs_f64(SERIAL_DELAY_TIME)
            {
                self.get_serial_responses()
                    .unwrap_or_else(|e| self.errors.push(e.into()));
                self.serial_event_propagator.last_get_time = current_time
            }
        }

        // Handle current job / status
        use ClientStatus::*;
        match self.gui_data.current_job {
            GatheringData => Ok(()),
            SendingPing => {
                self.gui_data.current_job = self.gui_data.current_job.next();
                self.serial_event_propagator.write_to_serial(
                    Command::Ping,
                    PingArguments {
                        time: (Local::now().timestamp_millis() as f64) / 1000.0,
                    },
                )
            }
            ReceivingPing => {
                if self.run_data.ping_status_response.is_some() {
                    self.gui_data.current_job = self.gui_data.current_job.next();
                }
                Ok(())
            }
            RequestingStaticStatus => {
                self.gui_data.current_job = self.gui_data.current_job.next();
                self.serial_event_propagator
                    .write_to_serial(Command::StaticStatus, StaticStatusArguments {})
            }
            ReceivingStaticStatus => {
                if self.run_data.static_status_response.is_some() {
                    self.gui_data.current_job = self.gui_data.current_job.next();
                }
                Ok(())
            }
            RequestingStart => {
                self.gui_data.current_job = self.gui_data.current_job.next();
                self.serial_event_propagator.write_to_serial(
                    Command::Start,
                    StartArguments {
                        distance: self.gui_data.distance,
                        reverse_brake: self.gui_data.reverse_braking,
                    },
                )
            }
            ReceivingStatus => Ok(()),
            RequestingStop => {
                self.gui_data.current_job = self.gui_data.current_job.next();
                self.serial_event_propagator
                    .write_to_serial(Command::Stop, StopArguments {})
            }
            Finished => Ok(()),
            #[allow(unreachable_patterns)]
            unhandled => {
                self.gui_data.current_job = self.gui_data.current_job.next();
                Err(ClientError::Unknown(format!(
                    "Not sure how to handle current job of '{}', skipping it!",
                    unhandled.to_string()
                )))
            }
        }
        .unwrap_or_else(|e| self.errors.push(e.into()));
    }

    fn start(&mut self) {
        if self.run_data.running {
            return;
        }

        // Ensure we have all the user input
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(self.gui_data.distance > 0.0) {
            return self.errors.push(ErrorData::new(ClientError::Run(
                "Distance is not over 0 centimeters".to_owned(),
            )));
        }

        self.run_data.running = true;
        self.gui_data.current_job = ClientStatus::SendingPing;
    }

    fn stop(&mut self) {
        self.run_data.running = false;
        self.gui_data.current_job = ClientStatus::RequestingStop;
    }

    fn reset(&mut self) {
        self.run_data.running = false;
        self.run_data.other_responses.clear();
        self.run_data.ping_status_response = None;
        self.run_data.static_status_response = None;
        self.run_data.status_responses.clear();
    }

    fn show_status_table(&self, ui: &mut Ui) {
        let status_table = TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::auto()) // Runtime
            .column(Column::auto()) // Distance
            .column(Column::auto()) // Velocity
            .column(Column::auto()) // Magnet odometer hits
            .column(Column::auto()) // Stage
            .min_scrolled_height(0.0);

        status_table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Runtime");
                });
                header.col(|ui| {
                    ui.strong("Distance");
                });
                header.col(|ui| {
                    ui.strong("Speed");
                });
                header.col(|ui| {
                    ui.strong("Spins");
                });
                header.col(|ui| {
                    ui.strong("Status");
                });
            })
            .body(|mut body| {
                for status in self.run_data.status_responses.iter().rev() {
                    body.row(18.0, |mut row| {
                        row.col(|ui| {
                            ui.label(format!("{}", status.value.runtime));
                        });
                        row.col(|ui| {
                            ui.label(format!("{:.3}cm", status.value.distance.distance));
                        });
                        row.col(|ui| {
                            ui.label(format!("{:.3}cm/s", status.value.distance.velocity));
                        });
                        row.col(|ui| {
                            ui.label(format!("{}", status.value.distance.magnet_hit_counter));
                        });
                        row.col(|ui| {
                            ui.label(format!("{}", status.value.stage));
                        });
                    });
                }
            });
    }
}
impl App for ClientGUI {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.logic();

        // Handle file dialog if needed
        if let Some(dialog) = &mut self.gui_data.file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    self.gui_data.display_file_path =
                        file.file_name().unwrap().to_string_lossy().to_string();
                    self.gui_data.file_path = Some(file);
                }
            }
        }

        // Show error messages
        if !self.errors.is_empty() {
            self.show_error_messages(ctx);
        }

        // Show expanded status table
        if self.gui_data.expanded_status_table {
            Window::new("Status table")
                .resizable(false)
                .show(ctx, |ui| {
                    let retract_button_size = [60., 20.];
                    if ui
                        .add_sized(retract_button_size, Button::new("Retract"))
                        .clicked()
                    {
                        self.gui_data.expanded_status_table = false;
                    }
                    self.show_status_table(ui);
                });
        }

        // Show connection window
        if self.gui_data.show_bluetooth_connect_screen {
            self.show_bluetooth_connect_screen(ctx);
        }

        ctx.set_visuals(Visuals::light());
        TopBottomPanel::top("banner")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.heading("CHARGE Dynamics' EC1B-Horme Route Planner");
                });
            });
        ctx.set_visuals(Visuals::dark());
        SidePanel::left("route-planner")
            .resizable(false)
            .exact_width(150.0)
            .show(ctx, |ui| {
                ui.heading("Plan your route");

                /* Distance input */

                ui.push_id("distance input", |ui| {
                    if self.run_data.running {
                        ui.set_enabled(false);
                    }

                    let distance: f64 = self.gui_data.distance;
                    ui.separator();
                    ui.label("Distance in centimeters");
                    ui.add(Slider::new(
                        &mut self.gui_data.distance,
                        0.0..=match distance > MAX_DISTANCE_RANGE_CENTIMETERS {
                            true => distance,
                            false => MAX_DISTANCE_RANGE_CENTIMETERS,
                        },
                    ));
                    // Increment buttons
                    let increment_button_size = [70., 60.];
                    // This is a slightly strange way of layout out items *vertically*
                    // by using two horizontals... but whatever!
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized(increment_button_size, Button::new("-10"))
                            .clicked()
                        {
                            if self.gui_data.distance < 10.0 {
                                self.gui_data.distance = 0.0;
                            } else {
                                self.gui_data.distance -= 10.0;
                            }
                        }
                        if ui
                            .add_sized(increment_button_size, Button::new("+10"))
                            .clicked()
                        {
                            self.gui_data.distance += 10.0;
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized(increment_button_size, Button::new("-100"))
                            .clicked()
                        {
                            if self.gui_data.distance < 100.0 {
                                self.gui_data.distance = 0.0;
                            } else {
                                self.gui_data.distance -= 100.0;
                            }
                        }
                        if ui
                            .add_sized(increment_button_size, Button::new("+100"))
                            .clicked()
                        {
                            self.gui_data.distance += 100.0;
                        }
                    });
                });

                /* Reverse motor braking */

                // FIXME: The server doesn't do anything with this information; it's literally useless
                // ui.add(Checkbox::new(
                //     &mut self.gui_data.reverse_braking,
                //     "Reverse motor braking",
                // ));

                /* Large control button */

                ui.separator();
                use LargeButton::*;
                let large_button_size = [150.0, 50.0];
                let large_button = match self.run_data.running {
                    false => match self.run_data.ping_status_response.is_none()
                        && self.run_data.status_responses.is_empty()
                    {
                        false => Reset,
                        true => Start,
                    },
                    true => Stop,
                };
                if ui
                    .add_sized(large_button_size, Button::new(large_button.to_string()))
                    .clicked()
                {
                    match large_button {
                        Start => self.start(),
                        Reset => self.reset(),
                        Stop => self.stop(),
                    }
                };

                /* Bluetooth control panel */

                ui.separator();
                let bluetooth_control_button_size = [150., 20.];
                if ui
                    .add_sized(bluetooth_control_button_size, Button::new("Connection"))
                    .clicked()
                {
                    self.gui_data.show_bluetooth_connect_screen = true;
                }
            });
        SidePanel::right("status")
            .exact_width(WIDTH - 150.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Information");

                /* Current job */

                ui.label(format!(
                    "Current job: {}",
                    self.gui_data.current_job.to_string()
                ));

                /* Ping */

                ui.separator();
                if let Some((ping_response, got_time)) = &self.run_data.ping_status_response {
                    ui.label(format!(
                        "Round-trip latency: {}ms",
                        (got_time - ping_response.value.sent_time) * 1000.0
                    ));
                } else {
                    ui.label("No ping information available");
                }

                /* Static status */

                ui.separator();
                if let Some(static_status) = &self.run_data.static_status_response {
                    ui.push_id("static status table", |ui| {
                        let static_status_table = TableBuilder::new(ui)
                            .striped(true)
                            .resizable(false)
                            .cell_layout(Layout::left_to_right(Align::Center))
                            .column(Column::auto()) // Number of magnets
                            .column(Column::auto()) // Wheel diameter inches
                            .column(Column::auto()) // Wheel diameter centimeters
                            .min_scrolled_height(0.0);

                        static_status_table
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.strong("Magnets");
                                });
                                header.col(|ui| {
                                    ui.strong("Wheel Diameter");
                                });
                                header.col(|ui| {
                                    ui.strong("Circumference");
                                });
                            })
                            .body(|mut body| {
                                body.row(18.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{}",
                                            static_status.value.number_of_magnets
                                        ));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{:.3}in / {:.3}cm",
                                            static_status.value.wheel_diameter / 2.54,
                                            static_status.value.wheel_diameter
                                        ));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{:.3}in / {:.3}cm",
                                            (static_status.value.wheel_diameter * PI) / 2.54,
                                            static_status.value.wheel_diameter * PI,
                                        ));
                                    });
                                });
                            });
                    });
                } else {
                    ui.label("No static status available");
                }

                /* Dynamic status */

                if let Some(latest_and_greatest_status) = self.run_data.status_responses.last() {
                    ui.push_id("dynamic status latest and greatest status table", |ui| {
                        let latest_and_greatest_status_table = TableBuilder::new(ui)
                            .striped(true)
                            .resizable(false)
                            .cell_layout(Layout::left_to_right(Align::Center))
                            .column(Column::auto()) // Running
                            .column(Column::auto()) // Uptime
                            .column(Column::auto()) // Runtime
                            .column(Column::remainder()) // Last received
                            .min_scrolled_height(0.0);

                        latest_and_greatest_status_table
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.strong("Running");
                                });
                                header.col(|ui| {
                                    ui.strong("Uptime");
                                });
                                header.col(|ui| {
                                    ui.strong("Runtime");
                                });
                                header.col(|ui| {
                                    ui.strong("Last received");
                                });
                            })
                            .body(|mut body| {
                                body.row(18.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(match latest_and_greatest_status.value.running {
                                            true => "YES",
                                            false => "NO",
                                        });
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{}",
                                            latest_and_greatest_status.value.uptime
                                        ));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{}",
                                            latest_and_greatest_status.value.runtime
                                        ));
                                    });
                                    row.col(|ui| {
                                        ui.label(format!(
                                            " {:.1} seconds ago",
                                            ((Local::now().timestamp_millis() as f64) / 1000.0)
                                                - latest_and_greatest_status.metadata.time
                                        ));
                                    });
                                });
                            });
                    });
                } else {
                    ui.label("No dynamic status available");
                }

                ui.separator();
                ui.horizontal(|ui| {
                    /* Expand */
                    let expand_button_size = [60., 20.];
                    if ui
                        .add_sized(expand_button_size, Button::new("Expand"))
                        .clicked()
                    {
                        self.gui_data.expanded_status_table = true;
                    }
                    /* Load / save */
                    ui.push_id("load and save", |ui| {
                        if self.run_data.running {
                            ui.set_enabled(false);
                        }

                        let text_input_size = [85., 20.];
                        ui.push_id("text input disabled", |ui| {
                            ui.set_enabled(false);
                            ui.add_sized(
                                text_input_size,
                                TextEdit::singleline(&mut self.gui_data.display_file_path),
                            )
                        });
                        let csv_handling_button_size = [60., 20.];
                        if ui
                            .add_sized(csv_handling_button_size, Button::new("Browse"))
                            .clicked()
                        {
                            // `FileDialog::new` is private for some reason...
                            let mut dialog = match self.run_data.status_responses.is_empty() {
                                true => FileDialog::open_file(None),
                                false => FileDialog::save_file(None),
                            }
                            .default_size(vec2(WIDTH, HEIGHT - 120.));
                            dialog.open();
                            self.gui_data.file_dialog = Some(dialog);
                        }
                        if self.run_data.status_responses.is_empty() {
                            if ui
                                .add_sized(csv_handling_button_size, Button::new("Load status"))
                                .clicked()
                            {
                                match CSVDynamicStatus::read(
                                    &(self.gui_data.file_path.clone().unwrap_or_else(|| {
                                        PathBuf::from(&self.gui_data.display_file_path)
                                    })),
                                ) {
                                    Ok(mut new_table) => {
                                        self.run_data.status_responses.clear();
                                        self.run_data.status_responses.append(&mut new_table);
                                    }
                                    Err(e) => {
                                        self.errors
                                            .push(ErrorData::new(ClientError::CSV(e.to_string())));
                                    }
                                };
                            }
                        } else if ui
                            .add_sized(csv_handling_button_size, Button::new("Save status"))
                            .clicked()
                        {
                            CSVDynamicStatus::write(
                                &(self.gui_data.file_path.clone().unwrap_or_else(|| {
                                    PathBuf::from(&self.gui_data.display_file_path)
                                })),
                                &self.run_data.status_responses,
                            )
                            .unwrap_or_else(|e| {
                                self.errors
                                    .push(ErrorData::new(ClientError::CSV(e.to_string())));
                            });
                        }
                    });
                });

                if self.gui_data.expanded_status_table {
                    ui.label("Table rendered elsewhere");
                    // The table is rendered outside of this current UI in
                    // the right panel for it to freely move around
                } else {
                    ui.push_id("dynamic status history table", |ui| {
                        self.show_status_table(ui);
                    });
                }
            });

        ctx.request_repaint();
    }
}

fn main() -> Result<(), ()> {
    // Connect to the server serial port
    let serial_port = args()
        .nth(1_usize)
        .expect("Please enter the serial port device (e.g. `cargo run /dev/pts/3`");
    let mut serial = new_serialport(serial_port.clone(), BAUD_RATE)
        .timeout(Duration::from_millis(500_u64))
        .open()
        .unwrap_or_else(|_| panic!("Failed to connect to the serial port. Please ensure it is connected on {serial_port}"));
    serial
        .set_timeout(Duration::from_secs_f64(SERIAL_DELAY_TIME))
        .map_err(|e| println!("{e}"))?;

    // Create the serial event propagator
    let serial_event_propagator = SerialEventPropagator::new(serial);

    // Create app
    let app = ClientGUI::new(serial_event_propagator);

    // Make the window
    let options = NativeOptions {
        resizable: false,
        initial_window_size: Some(vec2(WIDTH, HEIGHT)),
        always_on_top: true,
        ..Default::default()
    };
    run_native(
        "CHARGE Dynamics' EC1B-Horme Route Planner",
        options,
        Box::new(|_cc| Box::new(app)),
    )
    .map_err(|e| println!("{e}"))?;

    Ok(())
}
