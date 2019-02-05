use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_system_micros() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    u64::from(since_the_epoch.as_secs()) * 1_000_000 + u64::from(since_the_epoch.subsec_micros())
}

pub fn get_system_millis() -> u64 {
    get_system_micros() / 1000
}
