use crate::error::NovaResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Events emitted during agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEvent {
    Started,
    Thinking {
        model: String,
    },
    TextChunk {
        content: String,
    },
    ToolCallRequested {
        tool_name: String,
        arguments: serde_json::Value,
    },
    ToolExecutionStarted {
        tool_name: String,
    },
    ToolExecutionCompleted {
        tool_name: String,
        result: serde_json::Value,
        duration_ms: u64,
    },
    ToolExecutionFailed {
        tool_name: String,
        error: String,
    },
    ApprovalRequested {
        tool_name: String,
        description: String,
    },
    ApprovalReceived {
        approved: bool,
    },
    TurnCompleted {
        turn_number: u32,
    },
    Completed {
        content: String,
    },
    Error {
        message: String,
    },
}

/// Handler for stream events
#[async_trait]
pub trait StreamHandler: Send + Sync {
    async fn on_event(&self, event: StreamEvent) -> NovaResult<()>;

    fn should_cancel(&self) -> bool {
        false
    }
}

/// Default stream handler that does nothing
pub struct NoOpHandler;

#[async_trait]
impl StreamHandler for NoOpHandler {
    async fn on_event(&self, _event: StreamEvent) -> NovaResult<()> {
        Ok(())
    }
}

/// Stream handler that collects all events
pub struct CollectingHandler {
    events: std::sync::Mutex<Vec<StreamEvent>>,
}

impl CollectingHandler {
    pub fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn events(&self) -> Vec<StreamEvent> {
        self.events.lock().unwrap().clone()
    }
}

impl Default for CollectingHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StreamHandler for CollectingHandler {
    async fn on_event(&self, event: StreamEvent) -> NovaResult<()> {
        self.events.lock().unwrap().push(event);
        Ok(())
    }
}

/// Stream handler that prints events to stdout
pub struct PrintHandler {
    verbose: bool,
}

impl PrintHandler {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl Default for PrintHandler {
    fn default() -> Self {
        Self::new(false)
    }
}

#[async_trait]
impl StreamHandler for PrintHandler {
    async fn on_event(&self, event: StreamEvent) -> NovaResult<()> {
        match event {
            StreamEvent::TextChunk { content } => {
                print!("{}", content);
            }
            StreamEvent::ToolCallRequested { tool_name, .. } => {
                if self.verbose {
                    println!("\n[Calling tool: {}]", tool_name);
                }
            }
            StreamEvent::ToolExecutionCompleted {
                tool_name,
                duration_ms,
                ..
            } => {
                if self.verbose {
                    println!("[Tool {} completed in {}ms]", tool_name, duration_ms);
                }
            }
            StreamEvent::ToolExecutionFailed { tool_name, error } => {
                eprintln!("\n[Tool {} failed: {}]", tool_name, error);
            }
            StreamEvent::Completed { .. } => {
                println!();
            }
            StreamEvent::Error { message } => {
                eprintln!("\n[Error: {}]", message);
            }
            _ if self.verbose => {
                println!("[Event: {:?}]", event);
            }
            _ => {}
        }
        Ok(())
    }
}

/// Callback-based stream handler
pub struct CallbackHandler<F>
where
    F: Fn(StreamEvent) + Send + Sync,
{
    callback: F,
}

impl<F> CallbackHandler<F>
where
    F: Fn(StreamEvent) + Send + Sync,
{
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

#[async_trait]
impl<F> StreamHandler for CallbackHandler<F>
where
    F: Fn(StreamEvent) + Send + Sync,
{
    async fn on_event(&self, event: StreamEvent) -> NovaResult<()> {
        (self.callback)(event);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collecting_handler() {
        let handler = CollectingHandler::new();

        handler.on_event(StreamEvent::Started).await.unwrap();
        handler
            .on_event(StreamEvent::TextChunk {
                content: "Hello".to_string(),
            })
            .await
            .unwrap();
        handler
            .on_event(StreamEvent::Completed {
                content: "Hello".to_string(),
            })
            .await
            .unwrap();

        let events = handler.events();
        assert_eq!(events.len(), 3);
    }

    #[tokio::test]
    async fn test_noop_handler() {
        let handler = NoOpHandler;
        handler.on_event(StreamEvent::Started).await.unwrap();
    }
}
