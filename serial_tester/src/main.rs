extern crate ansi_term;
use ansi_term::Colour::{Green, Red};

#[macro_use]
extern crate clap;
use clap::{App, ArgMatches};

extern crate rand;

extern crate serial;
use serial::*;
use serial::core::SerialDevice;

use std::io::{Read, Write};
use std::result::Result;
use std::{thread, time};

/// The parameters passed into the program.
struct Params {
    /// The baud rate to communicate with the device.
    pub baud: BaudRate,

    /// Only show the test failures.
    pub fail_only: bool,

    /// The port to connect to the device.
    pub port: String,

    /// The number of repetitions for each test size.
    pub reps: usize,

    /// The size of each test.
    pub sizes: Vec<usize>
}

impl Params {
    /// Get the program arguments.
    ///
    /// # Returns
    ///
    /// The parameters if successful, otherwise an error message.
    fn get() -> Result<Params, String> {
        let yml = load_yaml!("app.yml");
        let matches = App::from_yaml(yml).get_matches();
        return Ok(Params {
            baud: Params::get_baud(&matches)?,
            fail_only: Params::get_fail_only(&matches)?,
            port: Params::get_port(&matches)?,
            reps: Params::get_reps(&matches)?,
            sizes: Params::get_sizes(&matches)?
        })
    }

    /// Get the baud rate from the program arguments.
    ///
    /// # Returns
    ///
    /// The baud rate if successful, otherwise an error message.
    fn get_baud(matches: &ArgMatches) -> Result<BaudRate, String> {
        let v = matches.value_of("baud").unwrap();
        match v.parse::<usize>() {
            Ok(speed) => Ok(BaudRate::from_speed(speed)),
            _ => Err(format!("Bad vaud value: {}", v))
        }
    }

    /// Get the fail only flag from the program arguments.
    ///
    /// # Returns
    ///
    /// The fail-only indicator, otherwise an error message.
    fn get_fail_only(matches: &ArgMatches) -> Result<bool, String> {
        Ok(matches.occurrences_of("fail-only") > 0)
    }

    /// Get the port name from the program arguments.
    ///
    /// # Returns
    ///
    /// The port name if successful, otherwise an error message.
    fn get_port(matches: &ArgMatches) -> Result<String, String> {
        Ok(matches.value_of("port").unwrap().to_string())
    }

    /// Get the test repetitions from the program arguments.
    ///
    /// # Returns
    ///
    /// The test repetitions if successful, otherwise an error message.
    fn get_reps(matches: &ArgMatches) -> Result<usize, String>{
        let v = matches.value_of("reps").unwrap();
        match v.parse::<usize>() {
            Ok(r) => Ok(r),
            _ => Err(format!("Bad reps value. {}", v))
        }
    }

    /// Get the test sizes from the program arguments.
    ///
    /// # Returns
    ///
    /// The test sizes if successful, otherwise an error message.
    fn get_sizes(matches: &ArgMatches) -> Result<Vec<usize>, String> {
        let mut sizes = vec![];
        for size in matches.values_of("size").unwrap() {
            match size.parse::<usize>() {
                Ok(s) => sizes.push(s),
                _ => return Err(format!("Bad size value: {}", size))
            }
        }
        return Ok(sizes);
    }
}

/// Create random byte array.
///
/// # Arguments
///
/// * `size` - The size of the array to be created.
///
/// # Returns
///
/// An array of the specified size.
fn random_data(size: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(size);
    for _ in 0..size {
        v.push(rand::random());
    }
    return v;
}

/// Create a zeroed byte array.
///
/// # Arguments
///
/// * `size` - The size of the array to be created.
///
/// # Returns
///
/// An array of the specified size.
fn zero_data(size: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(size);
    for _ in 0..size {
        v.push(0);
    }
    return v;
}

/// Perform a basic test using the serial device as an echo server. A set of data is written to the
/// device, and the function checks if the the returned message is the same as what was sent. An
/// approriate delay is added to make sure there is enough time to read the data. The delay is
/// based on the baudrate of the serial port and the size of the transmission.
///
/// # Arguments
///
/// * `port` - An initialized serial port to communicate over.
/// * `wdata` - The data to use in the test.
///
/// # Returns
///
/// `true` if the test passed, `false` on failure.
fn echo_test(port: &mut SystemPort, wdata: &[u8]) -> bool {
    // Flush before doing anything to make sure that any old data isn't still in the buffer.
    let _ = port.flush();

    // Write some data on the port and flush it to make sure it was written
    match port.write(&wdata) {
        Ok(_size) => {},
        Err(msg) => println!("Error writing: {}", msg)
    }
    let _ = port.flush();

    // Delay for a small amount of time that is proportional to the data size. This makes sure that
    // the write is finished before reading
    let baudrate = match port.read_settings() {
        Ok(settings) => settings.baud_rate().unwrap(),
        Err(_) => Baud9600
    };
    // Determine the amount of time a round trip of the bits will take
    let num_bits = 8.0 * (wdata.len() as f64);
    let bit_rate = (8.0 * (baudrate.speed() as f64)) / 10.0;
    // This is for one trip only, and we need to account for the bits on the way back by
    // multiplying by 2 to get the full path
    let seconds = num_bits / bit_rate;
    let millis = 1000.0 * seconds;
    // The time only needs to be multiplied by a factor of 2 to get the whole round trip, though 3
    // is used just to guaranteed that enough time passes.
    thread::sleep(time::Duration::from_millis((3.0 * millis) as u64));

    // Read back the data
    let mut rdata = zero_data(wdata.len());
    match port.read(&mut rdata) {
        Ok(_size) => {},
        Err(msg) => println!("Error reading: {}", msg)
    }

    return wdata == rdata.as_slice();
}

fn main() {
    // Get the command line parameters.
    let params = match Params::get() {
        Ok(p) => p,
        Err(message) => {
            println!("{}", message);
            std::process::exit(1)
        }
    };

    // Open a new port
    let mut port = if let Ok(port) = serial::open(&params.port) {
        port
    } else {
        println!("Could not open port: {}", params.port);
        std::process::exit(1)
    };

    // Configure the port settings
    match port.reconfigure(&|settings| {
        settings.set_baud_rate(params.baud)?;
        settings.set_char_size(Bits8);
        settings.set_parity(ParityNone);
        settings.set_stop_bits(Stop1);
        settings.set_flow_control(FlowNone);
        Ok(())
    }) {
        Ok(_) => {},
        Err(msg) => println!("Error configuring: {}", msg)
    }

    for size in params.sizes {
        let title = format!("Size={}, Reps={}", size, params.reps);
        println!();
        println!("{}", title);
        println!("{}", std::iter::repeat("-").take(title.len()).collect::<String>()); // Underline
        let mut all_passed = true;
        for i in 0..params.reps {
            let wdata = random_data(size);
            let passed = echo_test(&mut port, &wdata);
            let status = match passed {
                true => Green.paint("Passed"),
                false => Red.paint("Failed")
            };
            if !passed {
                all_passed = false;
            }
            // Only print the result when fail only is disabled or fail-only is enabled and there
            // is a failure.
            if !(passed && params.fail_only) {
                println!("{} ({} bytes): {}", i + 1, wdata.len(), status);
            }
        }
        // In fail-only mode, print a message if everything passed
        if all_passed && params.fail_only {
            println!("{}", Green.paint("All passed"));
        }
    }
}
