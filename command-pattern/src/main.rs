use std::io;

mod speed_calculator_impl;
mod command;
mod invoker;

use speed_calculator_impl::SpeedCalculatorImpl;
use command::CalculateSpeedCommand;
use invoker::SpeedCalculatorInvoker;

fn main() {
    let speed_calculator = Box::new(SpeedCalculatorImpl);
    let mut invoker = SpeedCalculatorInvoker { command: None };

    println!("Enter time per kilometer in minutes (format: mm:ss):");
    let mut input = String::new();
    let time_per_kilometer: f64;

    loop {
        input.clear();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        let parts: Vec<&str> = input.trim().split(':').collect();
        if parts.len() != 2 {
            println!("Invalid input format. Please enter time in mm:ss format:");
            continue;
        }

        let minutes: u32 = match parts[0].parse() {
            Ok(val) => val,
            Err(_) => {
                println!("Invalid input. Please enter a valid number for minutes:");
                continue;
            }
        };

        let seconds: u32 = match parts[1].parse() {
            Ok(val) => val,
            Err(_) => {
                println!("Invalid input. Please enter a valid number for seconds:");
                continue;
            }
        };

        if minutes == 0 && seconds == 0 {
            println!("Invalid input. Time per kilometer cannot be zero.");
            continue;
        }

        time_per_kilometer = minutes as f64 + seconds as f64 / 60.0;
        break;
    }

    let command = Box::new(CalculateSpeedCommand {
        receiver: speed_calculator,
        time_per_kilometer,
    });

    invoker.set_command(command);
    invoker.execute_command();

    // Wait for user input before exiting
    let _ = io::stdin().read_line(&mut input);
}
