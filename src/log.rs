use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use std::{fs::File, sync::OnceLock};

struct Logger {
    file: Mutex<File>,
}

static INSTANCE: OnceLock<Logger> = OnceLock::new();

impl Logger {
    pub fn log(&self, str: impl Into<String>) {
        self.file
            .lock()
            .unwrap()
            .write((str.into() + "\n").as_bytes())
            .unwrap();
    }
}
impl Default for Logger {
    fn default() -> Self {
        Logger {
            file: Mutex::new(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open("log.txt")
                    .unwrap(),
            ),
        }
    }
}

pub fn log(str: impl Into<String>) {
    INSTANCE.get_or_init(|| Default::default()).log(str);
}
