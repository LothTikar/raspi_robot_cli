enum NextArgType {
    LeftMotor,
    RightMotor,
    Time,
}

fn main() {
    let mut left_speed: f64 = 0.0;
    let mut right_speed: f64 = 0.0;
    let mut time: i32 = 0;

    let mut next_arg: Option<NextArgType> = None;
    for arg in std::env::args() {
        match next_arg {
            Some(NextArgType::LeftMotor) => {
                left_speed = arg.parse().unwrap();
                next_arg = None;
            }
            Some(NextArgType::RightMotor) => {
                right_speed = arg.parse().unwrap();
                next_arg = None;
            }
            Some(NextArgType::Time) => {
                time = arg.parse().unwrap();
                next_arg = None;
            }
            None => {
                next_arg = match arg.as_ref() {
                    "left" => Some(NextArgType::LeftMotor),
                    "right" => Some(NextArgType::RightMotor),
                    "time" => Some(NextArgType::Time),
                    &_ => None,
                }
            }
        }
    }
}
