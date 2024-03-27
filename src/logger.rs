use log::{Level, SetLoggerError};
use log::{LevelFilter, Metadata, Record};
use simplelog::*;
use std::env;
use std::fs::{self, File};
use std::path::Path;

// Custom logger struct implementing the Log trait from the log crate.
struct CustomLogger;

impl log::Log for CustomLogger {
    // Determines if a log record should be logged based on its metadata.
    // This logger logs all records with a level of Info or lower.
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    // Logs the record if it is enabled. Outputs to the console.
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    // Flush method required by the Log trait, but not used in this logger.
    fn flush(&self) {}
}

// Initializes the logging infrastructure for the application.
// Reads the desired log level from an environment variable and sets up file logging.
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok(); // Loads environment variables from a .env file if present.

    // Reads the LOG_LEVEL environment variable, defaulting to INFO if it's not set.
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "INFO".to_string());

    // Converts the log level string to a LevelFilter enum.
    let level_filter = match log_level.to_uppercase().as_str() {
        "TRACE" => LevelFilter::Trace,
        "DEBUG" => LevelFilter::Debug,
        "INFO" => LevelFilter::Info,
        "WARN" => LevelFilter::Warn,
        "ERROR" => LevelFilter::Error,
        _ => LevelFilter::Info, // Defaults to INFO for any unrecognized value.
    };

    // Sets the directory where log files will be saved.
    let log_directory = Path::new("logs");
    // Specifies the path for the log file.
    let log_file_path = log_directory.join("ot-ews.log");

    // Checks if the log directory exists; if not, it tries to create it.
    if !log_directory.exists() {
        fs::create_dir_all(log_directory)?;
    }

    // Verifies that the log directory is writable.
    if !is_writable(log_directory) {
        return Err("Log directory is not writable".into());
    }

    // Initializes the file logger with the specified level filter and log file path.
    WriteLogger::init(
        level_filter,
        Config::default(),
        File::create(log_file_path)?,
    )?;

    Ok(())
}

/// Checks if the specified directory is writable by attempting to write and then remove a temporary file.
fn is_writable<P: AsRef<Path>>(path: P) -> bool {
    fs::write(path.as_ref().join(".write_test"), b"").is_ok()
        && fs::remove_file(path.as_ref().join(".write_test")).is_ok()
}
