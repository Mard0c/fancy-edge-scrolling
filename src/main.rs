use evdev_rs::{
    Device, DeviceWrapper, InputEvent, ReadFlag,
    enums::{EV_ABS, EV_KEY, EventCode},
};
use std::{
    fs,
    io::Error,
    path::Path,
    process::{Command, Stdio},
    thread,
    thread::sleep,
    time::Duration,
};

static RATE_LIMIT: i64 = 150000; //microseconds
static SCALING: f32 = 4000.0;
static EDGE_THICKNESS: f64 = 0.05;

#[derive(PartialEq, Debug, Clone, Copy)]
enum EdgeZone {
    Left,
    Right,
    Top,
    // Bottom,
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

fn scrub(adjustment: i64) {
    let scrub_arg = if adjustment > 0 { "105" } else { "106" };

    match Command::new("ydotool").args(["key", scrub_arg]).status() {
        Ok(s) => println!("tried something? {}, with status: {:#?}", scrub_arg, s),
        Err(e) => println!("Failed to run ydotool: {}", e),
    }
}

fn status_bar(summoned: bool) {
    thread::spawn(move || {
        if summoned {
            match Command::new("waybar").stdout(Stdio::null()).status() {
                Ok(s) => println!("{}", s),
                Err(e) => println!("Error summoning status bar: {}", e),
            }
        } else {
            match Command::new("pkill").arg("waybar").status() {
                Ok(s) => println!("Exit status: {}", s),
                Err(e) => println!("Error killing status bar: {}", e),
            }
        }
    });
}

fn find_touchpad_device() -> Result<Device, Error> {
    let device_list_dir = fs::read_dir(Path::new("/dev/input"))?;
    for path in device_list_dir {
        let path = path?;
        if path.file_name().to_str().unwrap().starts_with("event") {
            let device = Device::new_from_path(path.path())?;

            if device.name().unwrap().to_lowercase().contains("touchpad") {
                println!("found touchpad");
                return Ok(device);
            }
        }
    }
    Err(Error::new(
        std::io::ErrorKind::Other,
        "Couldn't find touchpad for some bloody reason...",
    ))
}

fn vertical_edge_scroll(
    edge_scroll_target: &EdgeZone,
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
            EdgeZone::Right => {
                adjust_brightness(velocity);
            }
            EdgeZone::Left => {
                adjust_volume(velocity);
            }
            _ => println!("ERROR?!"),
        }
        // println!("velocity: {}", velocity);
        *previous_event = event.clone();
    }
}

fn horizontal_edge_scroll(
    edge_scroll_target: &EdgeZone,
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
        match edge_scroll_target {
            EdgeZone::Top => {
                let adjustment = if previous_event.value > event.value {
                    1
                } else {
                    -1
                };
                // scrub(adjustment);
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

    let mut edge_scroll_target: Option<EdgeZone> = None;
    let mut edge_pull_target: Option<EdgeZone> = None;
    let mut pulled = false;

    let mut watch = false;
    let mut watch_for_pull_scroll = false;

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
                            EV_ABS::ABS_X => match touchpad_range[0] {
                                None => match touchpad_device.abs_info(&input_event.event_code) {
                                    Some(info) => touchpad_range[0] = Some(info.maximum),
                                    None => println!("Could not find touchpad range"),
                                },
                                Some(range_x) => {
                                    if watch {
                                        if input_event.value // brightness
                                                > (range_x as f64 * (1.0 - EDGE_THICKNESS)) as i32
                                        {
                                            edge_scroll_target = Some(EdgeZone::Right);
                                            watch = false;
                                            println!("no longer watching");
                                        } else if input_event.value // volume
                                                < (range_x as f64 * EDGE_THICKNESS) as i32
                                        {
                                            edge_scroll_target = Some(EdgeZone::Left);
                                            watch = false;
                                            println!("no longer watching");
                                        } else {
                                            edge_scroll_target = None; // TODO figure out a way to watch for edge scroll once on x and y axis before seizing to watch.
                                        }
                                    }
                                    // if watch_for_pull_scroll {
                                    //     if ((range_x as f64 * (1.0 - EDGE_THICKNESS)) as i32)
                                    //         > input_event.value
                                    //         && input_event.value
                                    //             > ((range_x as f64 * EDGE_THICKNESS) as i32)
                                    //     {
                                    //         edge_pull_target = Some(EdgeZone::Top);
                                    //         watch_for_pull_scroll = false;
                                    //         println!(
                                    //             "Assigned STATUS, no longer watching for pull"
                                    //         );
                                    //     }
                                    // }
                                    match edge_scroll_target {
                                        Some(EdgeZone::Top) => {
                                            if let Some(ref mut previous_event) = previous_event {
                                                horizontal_edge_scroll(
                                                    &edge_scroll_target.unwrap(),
                                                    previous_event,
                                                    &input_event,
                                                );
                                            } else {
                                                previous_event = Some(input_event);
                                            }
                                        }
                                        _ => (),
                                    }
                                }
                            },

                            EV_ABS::ABS_Y => match touchpad_range[1] {
                                None => match touchpad_device.abs_info(&input_event.event_code) {
                                    Some(info) => touchpad_range[1] = Some(info.maximum),
                                    None => println!("Could not find touchpad range"),
                                },
                                Some(range_y) => {
                                    // println!("watching? {}", watch_for_edge_scroll);
                                    if watch {
                                        if input_event.value
                                            < (range_y as f64 * EDGE_THICKNESS) as i32
                                        {
                                            println!(
                                                "Edge scroll target {:#?}",
                                                edge_scroll_target
                                            );
                                            edge_scroll_target = Some(EdgeZone::Top);
                                            edge_pull_target = Some(EdgeZone::Top);
                                            watch = false;
                                        } else {
                                            edge_scroll_target = None;
                                        }
                                    }

                                    match edge_pull_target {
                                        Some(EdgeZone::Top) => {
                                            if !pulled
                                                && input_event.value
                                                    > (range_y as f64 * EDGE_THICKNESS) as i32
                                            {
                                                status_bar(true);
                                                edge_pull_target = None;
                                                pulled = true;
                                            }
                                        }
                                        _ => (),
                                    }

                                    match edge_scroll_target {
                                        Some(EdgeZone::Right) | Some(EdgeZone::Left) => {
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
                                    }
                                }
                            },
                            _ => (),
                        }
                        // println!("ABS ENUM: {:#?}, ev val: {}", abs_enum, input_event.value)
                    }
                    EventCode::EV_KEY(key) => {
                        // println!("{:#?}", key);
                        if key == EV_KEY::BTN_TOUCH && input_event.value == 0 {
                            if previous_event.is_some() && edge_scroll_target.is_some() {
                                previous_event = None;
                                edge_scroll_target = None;
                                watch = false;
                                // println!("RESET");
                            }
                            if pulled {
                                println!("should kill waybar");
                                status_bar(false);
                                pulled = false;
                            }
                            if edge_pull_target.is_some() {
                                edge_pull_target = None;
                            }
                        }
                        if key == EV_KEY::BTN_TOUCH && input_event.value == 1 {
                            watch = true;
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
