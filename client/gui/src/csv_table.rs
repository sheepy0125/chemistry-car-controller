/*!
 * Interface for saving and loading the status information
 * Created by sheepy0125 | MIT license | 2023-04-25
 */

use std::{error::Error, fs::File, path::Path};

/***** Setup *****/
// Imports
use bindings::{
    Command, DistanceInformation, Event, MetaData, StatusResponse, TransitMode, TransitType,
};
use csv::{Reader, Writer};

/***** CSV interface *****/
pub trait CSVInterface {
    fn write(file_path: &Path, data: &[Event<StatusResponse>]) -> Result<(), Box<dyn Error>>;
    fn read(file_path: &Path) -> Result<Vec<Event<StatusResponse>>, Box<dyn Error>>;
}

pub struct CSVDynamicStatus;
impl CSVInterface for CSVDynamicStatus {
    fn read(file_path: &Path) -> Result<Vec<Event<StatusResponse>>, Box<dyn Error>> {
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
            let stage = record[7]
                .parse::<u8>()?
                .try_into()
                .map_err(|_| "Failed to get status stage")?;

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
                    stage,
                },
                metadata: MetaData { time },
            });
        }

        Ok(ret_events)
    }

    fn write(file_path: &Path, data: &[Event<StatusResponse>]) -> Result<(), Box<dyn Error>> {
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
            "Stage",
        ])?;

        for record in data {
            let time = format!("{}", record.metadata.time);
            let running = format!("{}", record.value.running);
            let uptime = format!("{}", record.value.uptime);
            let runtime = format!("{}", record.value.runtime);
            let distance = format!("{}", record.value.distance.distance);
            let velocity = format!("{}", record.value.distance.velocity);
            let magnet_hit_counter = format!("{}", record.value.distance.magnet_hit_counter);
            let stage = format!("{}", record.value.stage as u8);
            csv_writer.write_record([
                time,
                running,
                uptime,
                runtime,
                distance,
                velocity,
                magnet_hit_counter,
                stage,
            ])?;
        }

        csv_writer.flush()?;

        Ok(())
    }
}
