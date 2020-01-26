use log::{Level, LevelFilter, Metadata, Record};

static LOGGER: PrintLogger = PrintLogger;

struct PrintLogger;

impl log::Log for PrintLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "{:5} [{:>15}:{:3}] {}",
                record.level(),
                record
                    .target()
                    .chars()
                    .into_iter()
                    .take(15)
                    .collect::<String>(),
                record.line().unwrap_or(0),
                record.args()
            );
        }
    }
    fn flush(&self) {}
}

pub fn init() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Trace);
}
