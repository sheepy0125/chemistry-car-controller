/*!
 * Interface for saving and loading the status information
 * Created by sheepy0125 | MIT license | 2023-04-25
 */

use std::{error::Error, fs::File, path::PathBuf};

/***** Setup *****/
// Imports
use crate::{
    bindings::{Command, DistanceInformation, MetaData, StatusResponse, TransitMode, TransitType},
    events::Event,
};
use csv::{Reader, Writer};

/***** CSV interface *****/
pub trait CSVInterface {
    fn write(file_path: &PathBuf, data: &[Event<StatusResponse>]) -> Result<(), Box<dyn Error>>;
    fn read(file_path: &PathBuf) -> Result<Vec<Event<StatusResponse>>, Box<dyn Error>>;
}

pub struct CSVDynamicStatus;
impl CSVInterface for CSVDynamicStatus {
    fn read(file_path: &PathBuf) -> Result<Vec<Event<StatusResponse>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut csv_reader = Reader::from_reader(file);
        let mut ret_events = vec![];

        for record_result in csv_reader.records() {
            let record = record_result?;
            let time = record[0].parse()?;
            let running = record[1].parse()?;
            let uptime = record[2].parse()?;
            let runtime = record[3].parse()?;
            let distance = record[4].parse()?;
            let velocity = record[5].parse()?;
            let magnet_hit_counter = record[6].parse()?;

            ret_events.push(Event {
                command: Command::Status,
                transit_mode: TransitMode::ServerToClientResponse,
                transit_type: TransitType::Response,
                value: StatusResponse {
                    running,
                    uptime,
                    runtime,
                    distance: DistanceInformation {
                        distance,
                        velocity,
                        magnet_hit_counter,
                    },
                },
                metadata: MetaData { time },
            });
        }

        Ok(ret_events)
    }

    fn write(file_path: &PathBuf, data: &[Event<StatusResponse>]) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut csv_writer = Writer::from_writer(file);

        csv_writer.write_record([
            "Unix time",
            "Running",
            "Uptime",
            "Runtime",
            "Distance in centimeters",
            "Velocity in centimeters/second",
            "Magnet hit counter",
        ])?;

        for record in data {
            let time = format!("{}", record.metadata.time);
            let running = format!("{}", record.value.running);
            let uptime = format!("{}", record.value.uptime);
            let runtime = format!("{}", record.value.runtime);
            let distance = format!("{}", record.value.distance.distance);
            let velocity = format!("{}", record.value.distance.velocity);
            let magnet_hit_counter = format!("{}", record.value.distance.magnet_hit_counter);
            csv_writer.write_record([
                time,
                running,
                uptime,
                runtime,
                distance,
                velocity,
                magnet_hit_counter,
            ])?;
        }

        csv_writer.flush()?;

        Ok(())
    }
}
