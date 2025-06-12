use std::{
    process::{Command, Stdio},
    thread,
};

pub fn adjust_volume(adjustment: i64) {
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

pub fn adjust_brightness(adjustment: i64) {
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

pub fn scrub(adjustment: i64) {
    let scrub_arg = if adjustment > 0 { "105" } else { "106" };

    match Command::new("ydotool").args(["key", scrub_arg]).status() {
        Ok(s) => println!("tried something? {}, with status: {:#?}", scrub_arg, s),
        Err(e) => println!("Failed to run ydotool: {}", e),
    }
}

pub fn status_bar(summoned: bool) {
    thread::spawn(move || {
        if summoned {
            let _ = Command::new("waybar").stdout(Stdio::null()).spawn();
        } else {
            let _ = Command::new("pkill").arg("waybar").spawn();
        }
    });
}
