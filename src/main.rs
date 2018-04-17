extern crate clap;
extern crate libc;

use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::ptr::write_volatile;
use clap::{App, Arg};

const MAX_SPEED: f64 = 1.0;
const MIN_SPEED: f64 = -1.0;

unsafe fn get_page(mem_file: &File, page_location: i32) -> *mut u32 {
    libc::mmap(
        0 as *mut libc::c_void,
        4096,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
        mem_file.as_raw_fd(),
        page_location,
    ) as *mut u32
}

fn time_validator(time: String) -> Result<(), String> {
    let time = time.parse::<u32>();

    if time.is_err() {
        return Err("not an unsigned integer".to_string());
    }

    Ok(())
}

fn speed_check(speed: String) -> Result<(), String> {
    let speed = speed.parse::<f64>();

    if speed.is_err() {
        return Err("not a number".to_string());
    }

    let speed = speed
        .ok()
        .expect("no value for speed when there should be a value. perhaps clap is broken?");

    if speed > MAX_SPEED {
        return Err(format!("value above {:.1}", MAX_SPEED));
    }
    if speed < MIN_SPEED {
        return Err(format!("value below {:.1}", MIN_SPEED));
    }

    Ok(())
}

fn main() {
    let args = App::new("Raspberry Pi Robot Command Line Interface")
        .author("Loth'Tikar")
        .about("Sets the speed for motors and runs them for a given time")
        .arg(
            Arg::with_name("left speed")
                .short("l")
                .long("left")
                .takes_value(true)
                .default_value("0.0")
                .validator(speed_check)
                .required(true)
                .help("Sets the left motor speed"),
        )
        .arg(
            Arg::with_name("right speed")
                .short("r")
                .long("right")
                .takes_value(true)
                .default_value("0.0")
                .validator(speed_check)
                .required(true)
                .help("Sets the right motor speed"),
        )
        .arg(
            Arg::with_name("time")
                .short("t")
                .long("time")
                .takes_value(true)
                .default_value("0")
                .validator(time_validator)
                .required(true)
                .help("Sets the amount of time the motors will run in ms"),
        )
        .get_matches();

    let left_speed: f64 = args.value_of("left speed").unwrap().parse().unwrap();
    let right_speed: f64 = args.value_of("right speed").unwrap().parse().unwrap();
    let time: u32 = args.value_of("time").unwrap().parse().unwrap();

    println!("left speed:{}", left_speed);
    println!("right speed:{}", right_speed);
    println!("time:{}", time);

    let mem_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/mem")
        .expect("failed to open /dev/mem");

    unsafe {
        let clock_page = get_page(&mem_file, 0x2010_1000);
        let gpio_page = get_page(&mem_file, 0x2020_0000);
        let pwm_page = get_page(&mem_file, 0x2020_C000);

        let pwm_clock_settings = clock_page.offset(160);
        let pwm_clock_divisor = clock_page.offset(161);

        let pin_setting_1 = gpio_page.offset(1);

        let pwm_settings = pwm_page;
        let pwm_range_1 = pwm_page.offset(4);
        let pwm_data_1 = pwm_page.offset(5);

        write_volatile(pin_setting_1, 0);
        write_volatile(pwm_settings, 0);
        write_volatile(pwm_clock_settings, 0);

        write_volatile(pwm_clock_divisor, 0x5a180000);
        write_volatile(pwm_clock_settings, 0x5a000011);

        write_volatile(pin_setting_1, 1 << 8 | 1 << 11);

        write_volatile(pwm_range_1, 5);
        write_volatile(pwm_settings, 0b0000_0001);

        loop {
            for i in 0..6 {
                write_volatile(pwm_data_1, i);
                std::thread::sleep(std::time::Duration::from_millis(400));
            }
        }
    }
}
