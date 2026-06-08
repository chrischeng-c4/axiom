//! Queue handoff contracts shared by mambalibs HTTP background tasks.
//!
//! This module intentionally accepts neutral JSON produced by `mambalibs.http`
//! instead of depending on httpkit types. It gives HTTP code a stable queue
//! handoff boundary without executing Python callables or reaching into stdlib
//! threading/async behavior.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::future::Future;
use std::sync::Arc;

use crate::{Broker, TaskError, TaskId, TaskMessage, TaskRegistry};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskHandoff {
    pub name: String,
    #[serde(default)]
    pub payload: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue: Option<String>,
}

impl TaskHandoff {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into().trim().to_string(),
            payload: Value::Null,
            queue: None,
        }
    }

    pub fn payload(mut self, payload: Value) -> Self {
        self.payload = payload;
        self
    }

    pub fn queue(mut self, queue: impl Into<String>) -> Self {
        self.queue = non_empty(queue.into());
        self
    }

    pub fn target_queue(&self, registry: &TaskRegistry) -> String {
        self.queue
            .clone()
            .unwrap_or_else(|| registry.route_task(&self.name, &self.payload))
    }

    pub fn into_task_message(self) -> TaskMessage {
        let Self { name, payload, .. } = self;
        match payload {
            Value::Null => TaskMessage::new(name, Value::Array(Vec::new())),
            Value::Array(_) => TaskMessage::new(name, payload),
            Value::Object(_) => {
                TaskMessage::new(name, Value::Array(Vec::new())).with_kwargs(payload)
            }
            scalar => TaskMessage::new(name, Value::Array(vec![scalar])),
        }
    }

    pub fn into_routed_message(self, registry: &TaskRegistry) -> RoutedTaskMessage {
        let queue = self.target_queue(registry);
        let message = self.into_task_message();
        RoutedTaskMessage { queue, message }
    }
}

#[derive(Debug, Clone)]
pub struct RoutedTaskMessage {
    pub queue: String,
    pub message: TaskMessage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedTaskMessage {
    pub queue: String,
    pub task_id: TaskId,
    pub task_name: String,
}

pub fn parse_http_background_handoffs(raw_json: &str) -> Result<Vec<TaskHandoff>, TaskError> {
    let mut handoffs: Vec<TaskHandoff> = serde_json::from_str(raw_json)
        .map_err(|err| TaskError::Deserialization(err.to_string()))?;
    for handoff in &mut handoffs {
        handoff.name = handoff.name.trim().to_string();
        handoff.queue = handoff.queue.take().and_then(non_empty);
        if handoff.name.is_empty() {
            return Err(TaskError::Configuration(
                "background task handoff requires a non-empty task name".to_string(),
            ));
        }
    }
    Ok(handoffs)
}

pub fn route_http_background_handoffs(
    raw_json: &str,
    registry: &TaskRegistry,
) -> Result<Vec<RoutedTaskMessage>, TaskError> {
    parse_http_background_handoffs(raw_json).map(|handoffs| {
        handoffs
            .into_iter()
            .map(|handoff| handoff.into_routed_message(registry))
            .collect()
    })
}

pub async fn publish_routed_handoffs<B>(
    broker: &B,
    routed: Vec<RoutedTaskMessage>,
) -> Result<Vec<PublishedTaskMessage>, TaskError>
where
    B: Broker + ?Sized,
{
    let mut published = Vec::with_capacity(routed.len());
    for RoutedTaskMessage { queue, message } in routed {
        let task_id = message.id.clone();
        let task_name = message.task_name.clone();
        broker.publish(&queue, message).await?;
        published.push(PublishedTaskMessage {
            queue,
            task_id,
            task_name,
        });
    }
    Ok(published)
}

pub async fn publish_http_background_handoffs<B>(
    raw_json: &str,
    registry: &TaskRegistry,
    broker: &B,
) -> Result<Vec<PublishedTaskMessage>, TaskError>
where
    B: Broker + ?Sized,
{
    let routed = route_http_background_handoffs(raw_json, registry)?;
    publish_routed_handoffs(broker, routed).await
}

pub fn publish_routed_handoffs_blocking(
    broker: Arc<dyn Broker>,
    routed: Vec<RoutedTaskMessage>,
) -> Result<Vec<PublishedTaskMessage>, TaskError> {
    run_handoff_blocking(async move { publish_routed_handoffs(broker.as_ref(), routed).await })
}

pub fn publish_http_background_handoffs_blocking(
    raw_json: &str,
    registry: &TaskRegistry,
    broker: Arc<dyn Broker>,
) -> Result<Vec<PublishedTaskMessage>, TaskError> {
    let routed = route_http_background_handoffs(raw_json, registry)?;
    publish_routed_handoffs_blocking(broker, routed)
}

fn run_handoff_blocking<Fut, T>(future: Fut) -> Result<T, TaskError>
where
    Fut: Future<Output = Result<T, TaskError>> + Send + 'static,
    T: Send + 'static,
{
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| {
                TaskError::Internal(format!("failed to create handoff runtime: {err}"))
            })?;
        runtime.block_on(future)
    })
    .join()
    .map_err(|_| {
        TaskError::Internal("background handoff blocking publisher panicked".to_string())
    })?
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::RouterConfig;
    use crate::{BrokerCapabilities, DeliveryModel};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct RecordingBroker {
        published: Arc<Mutex<Vec<(String, TaskMessage)>>>,
    }

    #[async_trait]
    impl Broker for RecordingBroker {
        async fn connect(&self) -> Result<(), TaskError> {
            Ok(())
        }

        async fn disconnect(&self) -> Result<(), TaskError> {
            Ok(())
        }

        async fn publish(&self, queue: &str, message: TaskMessage) -> Result<(), TaskError> {
            self.published
                .lock()
                .expect("published lock")
                .push((queue.to_string(), message));
            Ok(())
        }

        async fn health_check(&self) -> Result<(), TaskError> {
            Ok(())
        }

        fn delivery_model(&self) -> DeliveryModel {
            DeliveryModel::Pull
        }

        fn capabilities(&self) -> BrokerCapabilities {
            BrokerCapabilities::default()
        }
    }

    #[test]
    fn parses_http_background_tasks_json_and_preserves_explicit_queue() {
        let raw = r#"[{"name":"send_email","payload":{"user_id":42},"queue":"email"}]"#;
        let handoffs = parse_http_background_handoffs(raw).unwrap();

        assert_eq!(handoffs.len(), 1);
        assert_eq!(handoffs[0].name, "send_email");
        assert_eq!(handoffs[0].payload["user_id"].as_i64(), Some(42));
        assert_eq!(handoffs[0].queue.as_deref(), Some("email"));
    }

    #[test]
    fn routes_to_explicit_queue_and_maps_object_payload_to_kwargs() {
        let registry = TaskRegistry::new().with_router(
            RouterConfig::new()
                .route("send_email", "routed-email")
                .default_queue("default")
                .build(),
        );
        let routed = route_http_background_handoffs(
            r#"[{"name":"send_email","payload":{"user_id":42},"queue":"email"}]"#,
            &registry,
        )
        .unwrap();

        assert_eq!(routed.len(), 1);
        assert_eq!(routed[0].queue, "email");
        assert_eq!(routed[0].message.task_name, "send_email");
        assert_eq!(routed[0].message.args, Value::Array(Vec::new()));
        assert_eq!(routed[0].message.kwargs["user_id"].as_i64(), Some(42));
    }

    #[test]
    fn falls_back_to_registry_route_when_queue_is_missing() {
        let registry = TaskRegistry::new().with_router(
            RouterConfig::new()
                .route("audit.write", "audit")
                .default_queue("default")
                .build(),
        );
        let routed = route_http_background_handoffs(
            r#"[{"name":"audit.write","payload":{"event":"created"},"queue":null}]"#,
            &registry,
        )
        .unwrap();

        assert_eq!(routed[0].queue, "audit");
        assert_eq!(routed[0].message.kwargs["event"].as_str(), Some("created"));
    }

    #[test]
    fn maps_array_payload_to_args_and_scalar_payload_to_single_arg() {
        let registry = TaskRegistry::new();
        let routed = route_http_background_handoffs(
            r#"[
                {"name":"sum","payload":[1,2],"queue":"math"},
                {"name":"notify","payload":"done","queue":"events"}
            ]"#,
            &registry,
        )
        .unwrap();

        assert_eq!(routed[0].message.args, serde_json::json!([1, 2]));
        assert_eq!(routed[0].message.kwargs, Value::Null);
        assert_eq!(routed[1].message.args, serde_json::json!(["done"]));
    }

    #[test]
    fn rejects_empty_task_name() {
        let err = parse_http_background_handoffs(r#"[{"name":" ","payload":null}]"#)
            .expect_err("empty task name should fail");

        assert!(matches!(err, TaskError::Configuration(message) if message.contains("task name")));
    }

    #[tokio::test]
    async fn publishes_http_background_handoffs_to_broker() {
        let registry = TaskRegistry::new().with_router(
            RouterConfig::new()
                .route("audit.write", "audit")
                .default_queue("default")
                .build(),
        );
        let broker = RecordingBroker::default();

        let published = publish_http_background_handoffs(
            r#"[
                {"name":"send_email","payload":{"user_id":42},"queue":"email"},
                {"name":"audit.write","payload":{"event":"created"},"queue":null}
            ]"#,
            &registry,
            &broker,
        )
        .await
        .unwrap();

        assert_eq!(published.len(), 2);
        assert_eq!(published[0].queue, "email");
        assert_eq!(published[0].task_name, "send_email");
        assert_eq!(published[1].queue, "audit");
        assert_eq!(published[1].task_name, "audit.write");

        let stored = broker.published.lock().expect("published lock");
        assert_eq!(stored.len(), 2);
        assert_eq!(stored[0].0, "email");
        assert_eq!(stored[0].1.kwargs["user_id"].as_i64(), Some(42));
        assert_eq!(stored[1].0, "audit");
        assert_eq!(stored[1].1.kwargs["event"].as_str(), Some("created"));
    }

    #[test]
    fn publishes_http_background_handoffs_blocking_to_broker() {
        let registry = TaskRegistry::new().with_router(
            RouterConfig::new()
                .route("audit.write", "audit")
                .default_queue("default")
                .build(),
        );
        let broker = Arc::new(RecordingBroker::default());

        let published = publish_http_background_handoffs_blocking(
            r#"[
                {"name":"send_email","payload":{"user_id":42},"queue":"email"},
                {"name":"audit.write","payload":{"event":"created"},"queue":null}
            ]"#,
            &registry,
            broker.clone(),
        )
        .unwrap();

        assert_eq!(published.len(), 2);
        assert_eq!(published[0].queue, "email");
        assert_eq!(published[0].task_name, "send_email");
        assert_eq!(published[1].queue, "audit");
        assert_eq!(published[1].task_name, "audit.write");

        let stored = broker.published.lock().expect("published lock");
        assert_eq!(stored.len(), 2);
        assert_eq!(stored[0].0, "email");
        assert_eq!(stored[0].1.kwargs["user_id"].as_i64(), Some(42));
        assert_eq!(stored[1].0, "audit");
        assert_eq!(stored[1].1.kwargs["event"].as_str(), Some("created"));
    }
}
