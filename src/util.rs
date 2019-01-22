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
    u64::from(since_the_epoch.as_secs()) * 1_000_000 + u64::from(since_the_epoch.subsec_micros())
}

pub fn get_system_millis() -> u64 {
    get_system_micros() / 1000
}
