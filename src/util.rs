use crate::error::Error;
use chrono::prelude::*;
use slog::Drain;
use std::ops::Deref;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Logger {
    pub instance: slog::Logger,
}

impl Logger {
    pub fn new(app_name: &'static str) -> Logger {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();

        Logger {
            instance: slog::Logger::root(drain, o!("app" => app_name)),
        }
    }
}

impl Deref for Logger {
    type Target = slog::Logger;
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

pub fn get_system_micros() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_secs() * 1_000_000 + u64::from(since_the_epoch.subsec_micros())
}

pub fn get_system_millis() -> u64 {
    get_system_micros() / 1000
}

pub fn ts_to_ticks(input: &str) -> Result<u64, Error> {
    Ok(Utc
        .datetime_from_str(input, "%Y-%m-%dT%H:%M:%S+00:00")?
        .timestamp() as u64
        * 1000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ts_to_ticks_should_return_ticks() {
        let ts = "2018-11-25T22:46:00+00:00";
        let result = ts_to_ticks(&ts);
        assert!(result.unwrap() == 1543185960000);
    }
}
