//! Logger implementation with multiple sinks and context binding.

use crate::sink::{LogLevel, LogRecord, Sink};
use parking_lot::RwLock;
use std::collections::HashMap;

/// The main logger.
pub struct Logger {
    sinks: RwLock<Vec<Box<dyn Sink>>>,
    context: RwLock<HashMap<String, String>>,
    min_level: RwLock<LogLevel>,
}

impl Logger {
    /// Create a new logger with no sinks.
    pub fn new() -> Self {
        Self {
            sinks: RwLock::new(vec![]),
            context: RwLock::new(HashMap::new()),
            min_level: RwLock::new(LogLevel::Debug),
        }
    }

    /// Add a sink.
    pub fn add_sink(&self, sink: Box<dyn Sink>) {
        self.sinks.write().push(sink);
    }

    /// Set global minimum log level.
    pub fn set_level(&self, level: LogLevel) {
        *self.min_level.write() = level;
    }

    /// Bind context key-value pairs.
    pub fn bind(&self, key: &str, value: &str) {
        self.context
            .write()
            .insert(key.to_string(), value.to_string());
    }

    /// Create a child logger with additional context.
    pub fn with_context<'a>(&'a self, extra: HashMap<String, String>) -> BoundLogger<'a> {
        let mut ctx = self.context.read().clone();
        ctx.extend(extra);
        BoundLogger {
            parent: self,
            context: ctx,
        }
    }

    /// Log a message at the given level.
    pub fn log(&self, level: LogLevel, message: &str) {
        let min_level = *self.min_level.read();
        if level < min_level {
            return;
        }
        let record = LogRecord {
            level,
            message: message.to_string(),
            timestamp: chrono::Utc::now(),
            context: self.context.read().clone(),
            module: None,
            file: None,
            line: None,
        };
        let sinks = self.sinks.read();
        for sink in sinks.iter() {
            let _ = sink.write(&record);
        }
    }

    pub fn trace(&self, msg: &str) {
        self.log(LogLevel::Trace, msg);
    }
    pub fn debug(&self, msg: &str) {
        self.log(LogLevel::Debug, msg);
    }
    pub fn info(&self, msg: &str) {
        self.log(LogLevel::Info, msg);
    }
    pub fn warning(&self, msg: &str) {
        self.log(LogLevel::Warning, msg);
    }
    pub fn error(&self, msg: &str) {
        self.log(LogLevel::Error, msg);
    }
    pub fn critical(&self, msg: &str) {
        self.log(LogLevel::Critical, msg);
    }

    /// Get the number of registered sinks.
    pub fn sink_count(&self) -> usize {
        self.sinks.read().len()
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

/// A logger with bound context (returned by logger.bind()).
pub struct BoundLogger<'a> {
    parent: &'a Logger,
    context: HashMap<String, String>,
}

impl<'a> BoundLogger<'a> {
    pub fn log(&self, level: LogLevel, message: &str) {
        let min_level = *self.parent.min_level.read();
        if level < min_level {
            return;
        }
        let record = LogRecord {
            level,
            message: message.to_string(),
            timestamp: chrono::Utc::now(),
            context: self.context.clone(),
            module: None,
            file: None,
            line: None,
        };
        let sinks = self.parent.sinks.read();
        for sink in sinks.iter() {
            let _ = sink.write(&record);
        }
    }

    pub fn info(&self, msg: &str) {
        self.log(LogLevel::Info, msg);
    }
    pub fn debug(&self, msg: &str) {
        self.log(LogLevel::Debug, msg);
    }
    pub fn warning(&self, msg: &str) {
        self.log(LogLevel::Warning, msg);
    }
    pub fn error(&self, msg: &str) {
        self.log(LogLevel::Error, msg);
    }
    pub fn critical(&self, msg: &str) {
        self.log(LogLevel::Critical, msg);
    }
}
