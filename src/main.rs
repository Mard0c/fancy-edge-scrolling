use evdev_rs::{
    Device, DeviceWrapper, InputEvent, ReadFlag,
    enums::{EV_ABS, EV_KEY, EventCode},
};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    thread::sleep,
    time::Duration,
};

static RATE_LIMIT: i64 = 150000; //microseconds
static SCALING: f32 = 4000.0;
static EDGE_THICKNESS: f64 = 0.05;

#[derive(PartialEq, Debug, Clone, Copy)]
enum EdgeScroll {
    Left,
    Right,
    Top,
    Bottom,
}

fn adjust_volume(adjustment: i64) {
    let volume_arg = if adjustment < 0 {
        format!("{}%-", adjustment.abs())
    } else {
        format!("{}%+", adjustment)
    };
    let volume_arg = volume_arg.as_str();

    let _ = Command::new("wpctl")
        .args(["set-volume", "-l", "1", "@DEFAULT_AUDIO_SINK@", volume_arg])
        .spawn();
}

fn adjust_brightness(adjustment: i64) {
    let brightness_arg = if adjustment < 0 {
        format!("{}%-", adjustment.abs())
    } else {
        format!("{}%+", adjustment)
    };
    let brightness_arg = brightness_arg.as_str();

    let _ = Command::new("brightnessctl")
        .args(["set", brightness_arg])
        .stdout(Stdio::null())
        .spawn();
}

fn find_touchpad_device() -> Option<Device> {
    let device_list_dir = fs::read_dir(Path::new("/dev/input")).unwrap();
    for path in device_list_dir {
        let path = path.unwrap();
        if path.file_name().to_str().unwrap().starts_with("event") {
            let device = Device::new_from_path(path.path()).unwrap();

            if device.name().unwrap().to_lowercase().contains("touchpad") {
                println!("found touchpad");
                return Some(device);
            }
        }
    }
    None
}

fn vertical_edge_scroll(
    edge_scroll_target: &EdgeScroll,
    previous_event: &mut InputEvent,
    event: &InputEvent,
) {
    let time_difference = if event.time.tv_sec == previous_event.time.tv_sec {
        event.time.tv_usec - previous_event.time.tv_usec
    } else {
        let seconds_difference = event.time.tv_sec - previous_event.time.tv_sec;
        seconds_difference * 1000000 + (event.time.tv_usec - previous_event.time.tv_usec)
    };

    let position_difference = event.value - previous_event.value;

    if time_difference > RATE_LIMIT {
        let mut velocity =
            -1 * ((position_difference as f32) * SCALING / (time_difference as f32)) as i64;

        if previous_event.value > event.value {
            velocity += 1;
        } else {
            velocity -= 1;
        };

        match edge_scroll_target {
            EdgeScroll::Right => {
                adjust_brightness(velocity);
            }
            EdgeScroll::Left => {
                adjust_volume(velocity);
            }
            _ => println!("ERROR?!"),
        }
        // println!("velocity: {}", velocity);
        *previous_event = event.clone();
    }
}

fn main() {
    // let touchpad_device = Device::new_from_path("/dev/input/event7").unwrap();
    let touchpad_device = find_touchpad_device().unwrap();

    let mut touchpad_range: [Option<i32>; 2] = [None, None];

    let mut previous_event: Option<InputEvent> = None;

    let mut edge_scroll_target: Option<EdgeScroll> = None;

    let mut watch_for_edge_scroll = false;

    loop {
        if !touchpad_device.has_event_pending() {
            sleep(Duration::from_millis(1));
            continue;
        }
        let event_result = touchpad_device
            .next_event(ReadFlag::NORMAL)
            .map(|val| val.1);
        match event_result {
            Ok(input_event) => {
                // println!("{:#?}", input_event);
                match input_event.event_code {
                    EventCode::EV_ABS(abs_enum) => {
                        match abs_enum {
                            EV_ABS::ABS_X => {
                                match touchpad_range[0] {
                                    None => match touchpad_device.abs_info(&input_event.event_code)
                                    {
                                        Some(info) => touchpad_range[0] = Some(info.maximum),
                                        None => println!("Could not find touchpad range"),
                                    },
                                    Some(range_x) => {
                                        if watch_for_edge_scroll {
                                            // brightness
                                            if input_event.value
                                                > (range_x as f64 * (1.0 - EDGE_THICKNESS)) as i32
                                            {
                                                edge_scroll_target = Some(EdgeScroll::Right);
                                            } else {
                                                edge_scroll_target = None
                                            }

                                            // volume
                                            if input_event.value
                                                < (range_x as f64 * EDGE_THICKNESS) as i32
                                            {
                                                edge_scroll_target = Some(EdgeScroll::Left);
                                            }
                                        }
                                        watch_for_edge_scroll = false;
                                    }
                                }
                            }
                            EV_ABS::ABS_Y => match touchpad_range[1] {
                                None => match touchpad_device.abs_info(&input_event.event_code) {
                                    Some(info) => touchpad_range[1] = Some(info.maximum),
                                    None => println!("Could not find touchpad range"),
                                },
                                Some(range_y) => match edge_scroll_target {
                                    Some(EdgeScroll::Right) | Some(EdgeScroll::Left) => {
                                        if let Some(ref mut previous_event) = previous_event {
                                            vertical_edge_scroll(
                                                &edge_scroll_target.unwrap(),
                                                previous_event,
                                                &input_event,
                                            );
                                        } else {
                                            previous_event = Some(input_event);
                                        }
                                    }
                                    Some(_) | None => (),
                                },
                            },
                            _ => (),
                        }
                        // println!("ABS ENUM: {:#?}, ev val: {}", abs_enum, input_event.value)
                    }
                    EventCode::EV_KEY(key) => {
                        // println!("{:#?}", key);
                        if key == EV_KEY::BTN_TOUCH
                            && input_event.value == 0
                            && previous_event.is_some()
                            && edge_scroll_target.is_some()
                        {
                            previous_event = None;
                            edge_scroll_target = None;
                            watch_for_edge_scroll = false;
                            // println!("RESET");
                        }
                        if key == EV_KEY::BTN_TOUCH && input_event.value == 1 {
                            watch_for_edge_scroll = true;
                            // println!("STARTING");
                        }
                    }
                    _ => (),
                }
            }
            Err(_e) => (),
        }
        sleep(Duration::from_millis(1));
    }
}
