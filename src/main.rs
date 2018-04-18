extern crate clap;
extern crate libc;

use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::ptr::write_volatile;
use clap::{App, Arg};

const MAX_SPEED: f64 = 1.0;
const MIN_SPEED: f64 = -1.0;

enum Direction {
    Forward,
    Backward,
}

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
    let time = time.parse::<u64>();

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
    let time: u64 = args.value_of("time").unwrap().parse().unwrap();

    println!("left speed:{}", left_speed);
    println!("right speed:{}", right_speed);
    println!("time:{}", time);

    let left_direction = if left_speed.is_sign_positive() {
        Direction::Forward
    } else {
        Direction::Backward
    };
    let right_direction = if right_speed.is_sign_positive() {
        Direction::Forward
    } else {
        Direction::Backward
    };

    let left_speed = left_speed.abs();
    let right_speed = right_speed.abs();

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

        let pin_setting_0 = gpio_page;
        let pin_setting_1 = gpio_page.offset(1);
        let pin_setting_2 = gpio_page.offset(2);

        let pin_output_set_0 = gpio_page.offset(7);
        let pin_output_clear_0 = gpio_page.offset(10);

        let pwm_settings = pwm_page;
        let pwm_range_0 = pwm_page.offset(4);
        let pwm_data_0 = pwm_page.offset(5);
        let pwm_range_1 = pwm_page.offset(8);
        let pwm_data_1 = pwm_page.offset(9);

        write_volatile(pin_setting_1, 0);

        write_volatile(pwm_settings, 0);
        write_volatile(pwm_clock_settings, 0);

        write_volatile(pwm_clock_divisor, 0x5a180000);
        write_volatile(pwm_clock_settings, 0x5a000011);

        //Pin motor mappings
        //
        //STBY: 25
        //
        //Left PWM: 12(PWM0)
        //Left Forward: 5
        //Left Backward: 6
        //
        //Right PWM: 13(PWM1)
        //Right Forward: 16
        //Right Backward: 26

        write_volatile(pin_setting_0, 1 << 15 | 1 << 18);
        write_volatile(pin_setting_1, 1 << 8 | 1 << 11 | 1 << 18);
        write_volatile(pin_setting_2, 1 << 15 | 1 << 18);

        write_volatile(pin_output_clear_0, 0xFFFF_FFFF);
        write_volatile(
            pin_output_set_0,
            1 << 25 | match left_direction {
                Direction::Forward => 1 << 5,
                Direction::Backward => 1 << 6,
            } | match right_direction {
                Direction::Forward => 1 << 16,
                Direction::Backward => 1 << 26,
            },
        );

        write_volatile(pwm_range_0, 1000);
        write_volatile(pwm_range_1, 1000);
        write_volatile(pwm_data_0, (1000.0 * left_speed).round() as u32);
        write_volatile(pwm_data_1, (1000.0 * right_speed).round() as u32);
        write_volatile(pwm_settings, 0b1_0000_0001);

        std::thread::sleep(std::time::Duration::from_millis(time));

        write_volatile(pwm_settings, 0);
        write_volatile(pwm_data_0, 0);
        write_volatile(pwm_data_1, 0);
        write_volatile(pin_output_clear_0, 0xFFFF_FFFF);
    }
}
