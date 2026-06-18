//! Log sink implementations.

use crate::error::Result;
use std::path::PathBuf;

/// Configuration for a log sink.
#[derive(Debug, Clone)]
pub enum SinkConfig {
    Console(ConsoleConfig),
    File(FileConfig),
    Network(NetworkConfig),
}

/// Console sink configuration.
#[derive(Debug, Clone)]
pub struct ConsoleConfig {
    pub colorize: bool,
    pub stderr: bool,
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self {
            colorize: true,
            stderr: true,
        }
    }
}

/// File sink configuration.
#[derive(Debug, Clone)]
pub struct FileConfig {
    pub path: PathBuf,
    pub rotation: Option<Rotation>,
    pub retention: Option<usize>,
}

/// File rotation policy.
#[derive(Debug, Clone)]
pub enum Rotation {
    /// Rotate when file exceeds this size in bytes.
    Size(u64),
    /// Rotate daily.
    Daily,
    /// Rotate hourly.
    Hourly,
}

/// Network sink configuration (syslog).
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub host: String,
    pub port: u16,
    pub protocol: NetworkProtocol,
}

/// Network transport protocol.
#[derive(Debug, Clone)]
pub enum NetworkProtocol {
    Udp,
    Tcp,
}

/// Trait for log sinks.
pub trait Sink: Send + Sync {
    fn write(&self, record: &LogRecord) -> Result<()>;
    fn flush(&self) -> Result<()>;
}

/// A structured log record.
#[derive(Debug, Clone)]
pub struct LogRecord {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub context: std::collections::HashMap<String, String>,
    pub module: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
}

/// Log levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TRACE" => Some(Self::Trace),
            "DEBUG" => Some(Self::Debug),
            "INFO" => Some(Self::Info),
            "WARNING" | "WARN" => Some(Self::Warning),
            "ERROR" => Some(Self::Error),
            "CRITICAL" | "FATAL" => Some(Self::Critical),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
            Self::Critical => "CRITICAL",
        }
    }

    /// ANSI color code for this level.
    pub fn color_code(&self) -> &'static str {
        match self {
            Self::Trace => "\x1b[37m",
            Self::Debug => "\x1b[36m",
            Self::Info => "\x1b[32m",
            Self::Warning => "\x1b[33m",
            Self::Error => "\x1b[31m",
            Self::Critical => "\x1b[1;31m",
        }
    }
}

/// Console sink: writes to stderr/stdout with optional ANSI colors.
pub struct ConsoleSink {
    config: ConsoleConfig,
    min_level: LogLevel,
}

impl ConsoleSink {
    pub fn new(config: ConsoleConfig, min_level: LogLevel) -> Self {
        Self { config, min_level }
    }
}

impl Sink for ConsoleSink {
    fn write(&self, record: &LogRecord) -> Result<()> {
        if record.level < self.min_level {
            return Ok(());
        }
        let reset = "\x1b[0m";
        let msg = if self.config.colorize {
            format!(
                "{}{} | {}{} | {}",
                record.level.color_code(),
                record.timestamp.format("%Y-%m-%d %H:%M:%S"),
                record.level.as_str(),
                reset,
                record.message
            )
        } else {
            format!(
                "{} | {} | {}",
                record.timestamp.format("%Y-%m-%d %H:%M:%S"),
                record.level.as_str(),
                record.message
            )
        };

        if self.config.stderr {
            eprintln!("{}", msg);
        } else {
            println!("{}", msg);
        }
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

/// File sink: writes to file with rotation and retention.
pub struct FileSink {
    config: FileConfig,
    min_level: LogLevel,
}

impl FileSink {
    pub fn new(config: FileConfig, min_level: LogLevel) -> Self {
        Self { config, min_level }
    }
}

impl Sink for FileSink {
    fn write(&self, record: &LogRecord) -> Result<()> {
        if record.level < self.min_level {
            return Ok(());
        }
        // TODO: Implement async file writing via tokio
        // TODO: Implement rotation logic
        let _ = &self.config;
        let _ = record;
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

/// Network sink: sends to syslog via UDP/TCP.
pub struct NetworkSink {
    config: NetworkConfig,
    min_level: LogLevel,
}

impl NetworkSink {
    pub fn new(config: NetworkConfig, min_level: LogLevel) -> Self {
        Self { config, min_level }
    }
}

impl Sink for NetworkSink {
    fn write(&self, record: &LogRecord) -> Result<()> {
        if record.level < self.min_level {
            return Ok(());
        }
        // TODO: Implement syslog UDP/TCP sending
        let _ = &self.config;
        let _ = record;
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn record(level: LogLevel) -> LogRecord {
        LogRecord {
            level,
            message: "message".to_string(),
            timestamp: chrono::Utc::now(),
            context: HashMap::from([("request_id".to_string(), "req-1".to_string())]),
            module: Some("cclab_log::tests".to_string()),
            file: Some("sink.rs".to_string()),
            line: Some(1),
        }
    }

    #[test]
    fn log_level_parses_aliases_and_orders_by_severity() {
        assert_eq!(LogLevel::from_str("trace"), Some(LogLevel::Trace));
        assert_eq!(LogLevel::from_str("WARN"), Some(LogLevel::Warning));
        assert_eq!(LogLevel::from_str("fatal"), Some(LogLevel::Critical));
        assert_eq!(LogLevel::from_str("verbose"), None);

        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Info < LogLevel::Warning);
        assert!(LogLevel::Error < LogLevel::Critical);
        assert_eq!(LogLevel::Warning.as_str(), "WARNING");
    }

    #[test]
    fn sink_configs_and_stub_sinks_accept_records() {
        let file_config = FileConfig {
            path: PathBuf::from("/tmp/cclab-log-smoke.log"),
            rotation: Some(Rotation::Size(1024)),
            retention: Some(3),
        };
        let network_config = NetworkConfig {
            host: "127.0.0.1".to_string(),
            port: 514,
            protocol: NetworkProtocol::Udp,
        };

        let file_sink = FileSink::new(file_config, LogLevel::Info);
        let network_sink = NetworkSink::new(network_config, LogLevel::Warning);
        let console_sink = ConsoleSink::new(
            ConsoleConfig {
                colorize: false,
                stderr: false,
            },
            LogLevel::Error,
        );

        file_sink.write(&record(LogLevel::Debug)).unwrap();
        file_sink.write(&record(LogLevel::Error)).unwrap();
        file_sink.flush().unwrap();

        network_sink.write(&record(LogLevel::Info)).unwrap();
        network_sink.write(&record(LogLevel::Critical)).unwrap();
        network_sink.flush().unwrap();

        console_sink.write(&record(LogLevel::Info)).unwrap();
        console_sink.flush().unwrap();
    }
}
