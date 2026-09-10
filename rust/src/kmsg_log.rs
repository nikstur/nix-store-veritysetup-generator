use std::{
    fs::{File, OpenOptions},
    io::Write,
    process,
    sync::Mutex,
};

use log::{Level, LevelFilter, Log, Metadata, Record};

pub struct KmsgLog {
    kmsg: Mutex<File>,
    max_level: LevelFilter,
}

impl KmsgLog {
    const DEVICE: &'static str = "/dev/kmsg";

    pub fn new(filter: LevelFilter) -> Self {
        Self {
            kmsg: Mutex::new(
                OpenOptions::new()
                    .write(true)
                    .open(Self::DEVICE)
                    .expect("Failed to open /dev/kmsg"),
            ),
            max_level: filter,
        }
    }
}

impl Log for KmsgLog {
    fn enabled(&self, meta: &Metadata) -> bool {
        meta.level() <= self.max_level
    }

    fn log(&self, record: &Record) {
        if record.level() > self.max_level {
            return;
        }

        let level: u8 = match record.level() {
            Level::Error => 3,
            Level::Warn => 4,
            Level::Info => 5,
            Level::Debug => 6,
            Level::Trace => 7,
        };

        let mut buf = Vec::new();
        writeln!(
            buf,
            "<{}>{}[{}]: {}",
            level,
            record.target(),
            process::id(),
            record.args()
        )
        .unwrap();

        if let Ok(mut kmsg) = self.kmsg.lock() {
            let _ = kmsg.write(&buf);
            let _ = kmsg.flush();
        }
    }

    fn flush(&self) {}
}

pub fn init() {
    let klog = KmsgLog::new(LevelFilter::Trace);
    let max_level = klog.max_level;
    log::set_boxed_logger(Box::new(klog)).expect("Failed to set boxed logger");
    log::set_max_level(max_level);
}
