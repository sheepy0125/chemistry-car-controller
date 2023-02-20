/*!
 * TO BE OVERHAULED
 * DO NOT WORRY ABOUT IT
 * sheepy0125|mit|some time in 2022
 */

pub const PARSING_SEPARATOR: char = '|';
pub const COMMAND_SEPARATOR: char = '$';
pub const READY_PROMPT: &'static str = "READY>";
pub const OK_RESPONSE_PROMPT: &'static str = "RESP>";
pub const ERR_RESPONSE_PROMPT: &'static str = "ERR!";

pub const MAXIMUM_INPUT_LENGTH: usize = 64_usize;
pub const MAXIMUM_ARGUMENT_LENGTH: usize = 48_usize;
pub const BAUD_RATE: u32 = 57600_u32;
pub const PRESCALER: u32 = 1024;
pub const TIMER_COUNTS: u32 = 125;
pub const MILLIS_INCREMENT: u32 = PRESCALER * TIMER_COUNTS / 16000;

/***** Commands *****/
pub enum Command {
    Status,
}
impl Into<&str> for Command {
    fn into(self) -> &'static str {
        use Command::*;
        match self {
            Status => "STATUS",
        }
    }
}
impl TryFrom<&[char]> for Command {
    type Error = ();

    fn try_from(value: &[char]) -> Result<Self, Self::Error> {
        use Command::*;
        match value {
            ['S', 'T', 'A', 'T', 'U', 'S'] => Ok(Status),
            _ => Err(()),
        }
    }
}

pub struct Status {
    pub running: bool,
    pub uptime: usize,
}
impl TryFrom<&str> for Status {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Get sections to parse
        let parse_idxs = {
            const NUMBER_PARSE_IDXS: usize = 2_usize;
            let mut parse_idxs = [(0_usize, 0_usize); NUMBER_PARSE_IDXS];
            let mut current_parse_idxs_idx = 0_usize;
            let mut idx = 0_usize;
            for character in value.chars() {
                if character == PARSING_SEPARATOR {
                    parse_idxs[current_parse_idxs_idx - 1].1 = idx - 1;
                    current_parse_idxs_idx += 1;
                    parse_idxs[current_parse_idxs_idx].0 = idx + 1;
                }
                idx += 1;
            }
            parse_idxs[NUMBER_PARSE_IDXS - 1].1 = idx;
            parse_idxs
        };

        // Parse the sections
        let running = (&value[parse_idxs[0].0..=parse_idxs[0].1])
            .parse()
            .map_err(|_| ())?;
        let uptime = (&value[parse_idxs[1].0..=parse_idxs[1].1])
            .parse()
            .map_err(|_| ())?;

        Ok(Self { running, uptime })
    }
}
