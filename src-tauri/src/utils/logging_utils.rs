
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use crate::utils::path_utils;

use super::fs_utils;

pub fn init_logging() -> WorkerGuard {
    let logs_dir = match fs_utils::get_logs_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Warning: Could not get logs dirs ({}). Falling back to Temp.", e);
            path_utils::join(fs_utils::get_env_temp_dir().unwrap(), "logs")
        }
    };
    fs_utils::ensure_directory_exists(&logs_dir).ok();

    let now = chrono::Local::now();
    let filename = format!("Log_{}.log", now.format("%Y-%m-%d-%H-%M-%S"));
    
    // Setup file appender
    // Note: Standard tracing-appender rotates by *time* (Daily/Hourly).
    // We simply create a new file appender pointing to our unique timestamped filename so that "New file on every startup".
    let file_appender = tracing_appender::rolling::never(&logs_dir, &filename);
    
    // Make it non-blocking (so logging doesn't slow down your UI)
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::new("info,M3U8Downloader=trace")
        .add_directive("hyper=off".parse().unwrap())   // Optional: Completely silence hyper
        .add_directive("reqwest=off".parse().unwrap()); // Optional: Completely silence reqwest

    // Initialize the subscriber
    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_writer(non_blocking) // Write to file
                .with_ansi(false)          // Disable colors in file
                .with_target(false)        // Clean format
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
        )
        .with(
            fmt::layer()
                .with_writer(std::io::stdout) // Also write to Console
                .with_ansi(true)              // Keep colors for console
        )
        .init();

    // Return the guard. IMPORTANT: You must keep this variable alive in main()!
    guard
}

pub trait LogErr<T, E> {
    fn log_err(self) -> Result<T, E>;
}

impl<T, E: std::fmt::Display> LogErr<T, E> for Result<T, E> {
    fn log_err(self) -> Result<T, E> {
        if let Err(ref e) = self {
            tracing::error!("Operation failed: {}", e);
        }
        self
    }
}