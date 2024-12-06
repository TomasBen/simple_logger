use chrono::{DateTime, Local};
use std::env;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug)]
enum LogLevel {
    Default,
    Error,
    Debug,
    Info,
}

#[derive(Debug)]
pub struct LogEntry {
    timestap: DateTime<Local>,
    level: LogLevel,
    message: String,
}

pub struct Logger {
    entries: Vec<LogEntry>,
    file_path: Option<PathBuf>,
    flushed: Arc<AtomicBool>,
}

fn default_log_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        PathBuf::from(r"C:\PorgramData").join("Logs")
    } else if cfg!(target_os = "macos") {
        PathBuf::from("/Library/Logs")
    } else {
        PathBuf::from("/var/log")
    }
}

fn get_log_level() -> LogLevel {
    match env::var("LOG_LEVEL").unwrap().as_str() {
        "debug" => LogLevel::Debug,
        "info" => LogLevel::Info,
        _ => LogLevel::Default,
    }
}

impl Logger {
    pub fn new(file_path: PathBuf) -> Self {
        Logger {
            entries: Vec::new(),
            file_path: Some(file_path),
            flushed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn debug(&mut self, message: &str) {
        self.entries.push(LogEntry {
            timestap: Local::now(),
            level: LogLevel::Debug,
            message: message.to_string(),
        });
    }

    pub fn info(&mut self, message: &str) {
        self.entries.push(LogEntry {
            timestap: Local::now(),
            level: LogLevel::Info,
            message: message.to_string(),
        });
    }

    pub fn error(&mut self, message: &str) {
        self.entries.push(LogEntry {
            timestap: Local::now(),
            level: LogLevel::Error,
            message: message.to_string(),
        });
    }

    pub fn flush(self: Self) -> std::io::Result<()> {
        if self.flushed.load(Ordering::Relaxed) {
            return Ok(());
        }

        let log_level = get_log_level();
        let path = match self.file_path {
            Some(path) => path,
            None => default_log_path(),
        };

        let mut file = if path.exists() {
            File::open(&path)?
        } else {
            if let Some(parent) = path.parent() {
                create_dir_all(parent)?;
            }
            println!("Creating new log file at: {:?}", path);
            File::create(&path)?
        };

        match log_level {
            LogLevel::Debug => {
                for entry in self.entries {
                    writeln!(
                        file,
                        "[{}] {:?}: {}",
                        entry.timestap.format("%Y-%m-%d %H-%M-%S"),
                        entry.level,
                        entry.message
                    )?;
                }
            }
            LogLevel::Error => {
                for entry in self.entries {
                    match entry.level {
                        LogLevel::Error => {
                            writeln!(
                                file,
                                "[{}] {:?}: {}",
                                entry.timestap.format("%Y-%m-%d %H-%M-%S"),
                                entry.level,
                                entry.message
                            )?;
                        }
                        _ => continue,
                    }
                }
            }
            LogLevel::Info => {
                for entry in self.entries {
                    match entry.level {
                        LogLevel::Info => {
                            writeln!(
                                file,
                                "[{}] {:?}: {}",
                                entry.timestap.format("%Y-%m-%d %H-%M-%S"),
                                entry.level,
                                entry.message
                            )?;
                        }
                        _ => continue,
                    }
                }
            }
            LogLevel::Default => {
                for entry in self.entries {
                    writeln!(
                        file,
                        "[{}] {:?}: {}",
                        entry.timestap.format("%Y-%m-%d %H-%M-%S"),
                        entry.level,
                        entry.message
                    )?;
                }
            }
        }

        self.flushed.store(true, Ordering::Relaxed);
        Ok(())
    }
}
