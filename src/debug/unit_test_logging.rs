// SPDX-License-Identifier: Unlicense

extern crate std;
use std::sync::Once;
use log::{Level, LevelFilter, Metadata, Record};

static START: Once = Once::new();
static LOGGER: PrintLogger = PrintLogger;
struct PrintLogger;

impl log::Log for PrintLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        use std::{println, string::String};

        if self.enabled(record.metadata()) {
            println!(
                "{:5} [{:>50}:{:3}] {}",
                record.level(),
                record
                    .target()
                    .chars()
                    .into_iter()
                    .take(50)
                    .collect::<String>(),
                record.line().unwrap_or(0),
                record.args()
            );
        }
    }
    fn flush(&self) {}
}

pub fn setup() {
    START.call_once(|| {
        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(LevelFilter::Trace);
    });
}
