#![allow(dead_code)]

pub fn keyboard_backlight_paths() -> Vec<String> {
    // TODO: Probe non-dangerous sysfs paths if Lenovo exposes keyboard LEDs through a kernel driver.
    // Do not hardcode write targets and do not require the GUI to run as root.
    Vec::new()
}
