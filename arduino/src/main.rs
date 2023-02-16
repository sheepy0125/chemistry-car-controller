/*!
 * Chemistry Car Controller on an Arduino Uno
 * Created by sheepy0125
 * 2023-02-15
 */

/***** Setup *****/
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

// Imports
#[macro_use]
extern crate fixedvec;
use arduino_hal::{
    hal::port::{PD0, PD1},
    pac::USART0,
    port::{
        mode::{Input, Output},
        Pin,
    },
    prelude::*,
    Usart,
};
use core::cell::Cell;
use embedded_hal::serial::Read;
use fixedvec::FixedVec;
use nb::block;
// #[path = "../../shared/types.rs"]
#[path = "./types.rs"]
mod types;
use types::*;

// Statics
static MILLIS_COUNTER: avr_device::interrupt::Mutex<Cell<u32>> =
    avr_device::interrupt::Mutex::new(Cell::new(0));

// Types
type Serial = Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>;

/***** Panic handler *****/
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Let's steal our handlers
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, BAUD_RATE);

    // Print out panic location
    // For whatever reason, when not running in release mode then we get
    // garbage printed out for the file, line, and column
    match info.location() {
        #[cfg(not(debug_assertions))]
        Some(loc) => ufmt::uwriteln!(
            &mut serial,
            "PANICKED {}:{}:{}",
            loc.file(),
            loc.line(),
            loc.column()
        )
        .void_unwrap(),
        #[cfg(debug_assertions)]
        Some(loc) => ufmt::uwriteln!(
            &mut serial,
            "PANICKED: not release mode, garbage: {}",
            loc.file()
        )
        .void_unwrap(),
        None => ufmt::uwriteln!(&mut serial, "Panicked! No information.").void_unwrap(),
    }

    // Blink LED rapidly
    let mut led = pins.d13.into_output();
    loop {
        led.toggle();
        arduino_hal::delay_ms(500);
    }
}

/***** Structs *****/
struct ParsedCommand<'a> {
    command: Command,
    arguments: &'a [char; MAXIMUM_ARGUMENT_LENGTH],
}

/***** Helper functions *****/
/* Millis (from https://blog.rahix.de/005-avr-hal-millis/) */
fn millis_init(tc0: arduino_hal::pac::TC0) {
    // Configure the timer for the above interval (in CTC mode)
    // and enable its interrupt.
    tc0.tccr0a.write(|w| w.wgm0().ctc());
    tc0.ocr0a.write(|w| w.bits(TIMER_COUNTS as u8));
    tc0.tccr0b.write(|w| match PRESCALER {
        8 => w.cs0().prescale_8(),
        64 => w.cs0().prescale_64(),
        256 => w.cs0().prescale_256(),
        1024 => w.cs0().prescale_1024(),
        _ => panic!(),
    });
    tc0.timsk0.write(|w| w.ocie0a().set_bit());

    // Reset the global millisecond counter
    avr_device::interrupt::free(|cs| {
        MILLIS_COUNTER.borrow(cs).set(0);
    });
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    avr_device::interrupt::free(|cs| {
        let counter_cell = MILLIS_COUNTER.borrow(cs);
        let counter = counter_cell.get();
        counter_cell.set(counter + MILLIS_INCREMENT);
    })
}

fn millis() -> u32 {
    avr_device::interrupt::free(|cs| MILLIS_COUNTER.borrow(cs).get())
}

/***** Communicate with client *****/
fn write_status<'a>(serial: &'a mut Serial, status: &'a Status) {
    ufmt::uwriteln!(
        serial,
        "{}{}{}{}{}",
        OK_RESPONSE_PROMPT,
        COMMAND_SEPARATOR,
        status.running,
        PARSING_SEPARATOR,
        status.uptime
    )
    .void_unwrap();
}

fn get_command<'a, 'b>(
    serial: &'b mut Serial,
    argument_buffer: &'a mut [char; MAXIMUM_ARGUMENT_LENGTH],
) -> Result<ParsedCommand<'a>, ()> {
    ufmt::uWrite::write_str(serial, READY_PROMPT).void_unwrap();
    let mut preallocated_space = alloc_stack!([char; MAXIMUM_INPUT_LENGTH]);
    let buffer = {
        let mut new_line_received = false;
        let mut buffer = FixedVec::new(&mut preallocated_space);
        while !new_line_received {
            match block!(serial.read())
                .void_unwrap()
                .try_into()
                .unwrap_or('?')
            {
                // Newline signifies end of equation
                '\n' => new_line_received = true,
                // Ignore spaces
                ' ' => {}
                // Everything else
                byte => buffer.push(byte).unwrap_or_else(|_| {
                    // The only error that this can return is one of no space left
                    new_line_received = true;
                }),
            }
        }
        buffer
    };

    // Look for the index separating the command from the arguments
    let buffer_slice = buffer.as_slice();
    let separate_idx = buffer_slice
        .iter()
        .position(|character| character == &COMMAND_SEPARATOR)
        .ok_or(())?;

    // Parse these
    let command = Command::try_from(&buffer_slice[0..separate_idx])?;
    // No possible way to convert this to a `&str` (no allocation),
    // so instead we're using a fixed size array with null terminators!
    for (idx, argument_character) in buffer_slice[separate_idx + 1..].iter().enumerate() {
        // By the way, I hate doing this
        argument_buffer[idx] = *argument_character;
    }

    Ok(ParsedCommand {
        command,
        arguments: argument_buffer,
    })
}

fn run_command<'b>(serial: &'b mut Serial, command: ParsedCommand) {
    match command.command {
        Command::Status => {
            let status = Status {
                running: false,
                uptime: millis() as usize,
            };
            write_status(serial, &status);
        }
    };
}

/***** Main *****/
#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial: Serial = arduino_hal::default_serial!(dp, pins, BAUD_RATE);

    millis_init(dp.TC0);

    // Enable interrupts globally
    unsafe { avr_device::interrupt::enable() };

    loop {
        let arguments_buffer = &mut ['\0'; MAXIMUM_ARGUMENT_LENGTH];
        let command = match get_command(&mut serial, arguments_buffer) {
            Ok(command) => command,
            Err(()) => {
                ufmt::uwriteln!(&mut serial, "{}", ERR_RESPONSE_PROMPT).void_unwrap();
                continue;
            }
        };
        run_command(&mut serial, command);
    }
}
