extern crate libc;

use std::os::unix::io::AsRawFd;

const MAX_SPEED: f64 = 1.0;
const MIN_SPEED: f64 = -1.0;

enum NextArgType {
    LeftMotor,
    RightMotor,
    Time,
}

fn print_help() {
    println!("Runs left and right motors for a given amount of time");
    println!(
        "Speed for the motors must be within the range of {:.2} to {:.2}",
        MIN_SPEED, MAX_SPEED
    );
    println!("\nFlags:");
    println!("    -h, --help    Prints this message");
    println!("\nCommands:");
    println!("    left <value>     Sets the left motor speed");
    println!("    right <value>    Sets the right motor speed");
    println!("    time             Sets the amount of time to run in milliseconds");
}

fn speed_check(speed: f64, speed_name: &str) {
    if speed > MAX_SPEED {
        panic!(
            "Given value for {} speed above {:.2}",
            speed_name, MAX_SPEED
        );
    }
    if speed < MIN_SPEED {
        panic!(
            "Given value for {} speed below {:.2}",
            speed_name, MIN_SPEED
        );
    }
}

fn main() {
    let mut left_speed: std::option::Option<f64> = None;
    let mut right_speed: std::option::Option<f64> = None;
    let mut time: std::option::Option<u32> = None;

    let mut next_arg: Option<NextArgType> = None;

    if std::env::args().len() < 2 {
        print_help();
        return;
    }

    for arg in std::env::args() {
        match next_arg {
            Some(NextArgType::LeftMotor) => {
                left_speed = Some(arg.parse().unwrap());
                speed_check(left_speed.unwrap(), "left");
                next_arg = None;
            }
            Some(NextArgType::RightMotor) => {
                right_speed = Some(arg.parse().unwrap());
                speed_check(right_speed.unwrap(), "right");
                next_arg = None;
            }
            Some(NextArgType::Time) => {
                time = Some(arg.parse().unwrap());
                next_arg = None;
            }
            None => {
                next_arg = match arg.as_ref() {
                    "left" => Some(NextArgType::LeftMotor),
                    "right" => Some(NextArgType::RightMotor),
                    "time" => Some(NextArgType::Time),
                    "-h" | "--help" => {
                        print_help();
                        None
                    }
                    &_ => None,
                }
            }
        }
    }
    left_speed.expect("No value provided for left speed");
    right_speed.expect("No value provided for right speed");
    time.expect("No value provided for time");

    let mem_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/gpiomem")
        .expect("failed to open /dev/gpiomem");

    unsafe {
        let mem = libc::mmap(
            0 as *mut libc::c_void,
            0x00B0,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            mem_file.as_raw_fd(),
            0,
        ) as *mut u32;

        let pin_mode_1 = mem.offset(1);
        *pin_mode_1 = 1 << 8 | 1 << 11;

        let pin_set_1 = mem.offset(7);
        let pin_clear_1 = mem.offset(10);

        loop {
            *pin_set_1 = 1 << 26;
            std::thread::sleep(std::time::Duration::from_millis(500));

            *pin_clear_1 = 1 << 26;
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
}
