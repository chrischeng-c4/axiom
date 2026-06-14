//! cclab.meteor event handler generator
//!
//! Generates task handlers from AsyncAPI specs for the cclab.meteor task queue.

use crate::gen::python::type_to_python;
use crate::gen::traits::{CodeGenerator, GenContext, GenResult, GeneratedCode, Language};
use crate::spec::ir::{ChannelDef, DataModelSpec, EventApiSpec, OperationDef};

/// Swarm (event handlers) code generator
pub struct SwarmGenerator;

impl SwarmGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate event handlers from EventApiSpec
    pub fn generate_handlers(&self, spec: &EventApiSpec, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        let app_name = to_snake_case(&spec.title.replace(" ", "").replace("-", ""));

        // Create meteor app
        lines.push(format!("{}_app = Swarm(name=\"{}\")", app_name, spec.title));
        lines.push(String::new());

        // Generate handlers for each channel
        for channel in &spec.channels {
            if let Some(handler) = self.generate_channel_handlers(channel, &app_name, ctx) {
                lines.push(handler);
                lines.push(String::new());
            }
        }

        lines.join("\n")
    }

    /// Generate handlers for a channel
    fn generate_channel_handlers(
        &self,
        channel: &ChannelDef,
        app_name: &str,
        ctx: &GenContext,
    ) -> Option<String> {
        let mut lines = Vec::new();

        // Generate subscriber (consumer) handler
        if let Some(sub) = &channel.subscribe {
            lines.push(self.generate_subscriber(channel, sub, app_name, ctx));
            lines.push(String::new());
        }

        // Generate publisher (producer) helper
        if let Some(pub_op) = &channel.publish {
            lines.push(self.generate_publisher(channel, pub_op, app_name, ctx));
            lines.push(String::new());
        }

        if lines.is_empty() {
            None
        } else {
            Some(lines.join("\n"))
        }
    }

    /// Generate a subscriber handler
    fn generate_subscriber(
        &self,
        channel: &ChannelDef,
        op: &OperationDef,
        app_name: &str,
        ctx: &GenContext,
    ) -> String {
        let mut lines = Vec::new();

        // Handler name from operation_id or channel name
        let handler_name = op
            .operation_id
            .clone()
            .map(|s| to_snake_case(&s))
            .unwrap_or_else(|| self.channel_to_handler_name(&channel.name, "handle"));

        // Channel/queue name
        let queue_name = channel.name.replace("/", ".");

        // Decorator
        lines.push(format!("@{}_app.task(queue=\"{}\")", app_name, queue_name));

        // Message type
        let message_type = type_to_python(&op.message);

        // Function signature
        lines.push(format!(
            "async def {}(message: {}) -> None:",
            handler_name, message_type
        ));

        // Docstring
        if ctx.generate_docs {
            if let Some(summary) = &op.summary {
                lines.push(format!("{}\"\"\"{}", ctx.indent, summary));
                if let Some(desc) = &channel.description {
                    lines.push(format!("{}", ctx.indent));
                    lines.push(format!("{}Channel: {}", ctx.indent, channel.name));
                    lines.push(format!("{}Description: {}", ctx.indent, desc));
                }
                lines.push(format!("{}\"\"\"", ctx.indent));
            }
        }

        // Stub body — replace with actual handler logic
        lines.push(format!(
            "{}logger.info(f\"Processing message: {{message}}\")",
            ctx.indent
        ));
        lines.push(format!(
            "{}raise NotImplementedError(\"Implement handler: {}\")",
            ctx.indent, handler_name
        ));

        lines.join("\n")
    }

    /// Generate a publisher helper function
    fn generate_publisher(
        &self,
        channel: &ChannelDef,
        op: &OperationDef,
        app_name: &str,
        ctx: &GenContext,
    ) -> String {
        let mut lines = Vec::new();

        // Function name from operation_id or channel name
        let func_name = op
            .operation_id
            .clone()
            .map(|s| to_snake_case(&s))
            .unwrap_or_else(|| self.channel_to_handler_name(&channel.name, "publish"));

        // Queue name
        let queue_name = channel.name.replace("/", ".");

        // Message type
        let message_type = type_to_python(&op.message);

        // Function signature
        lines.push(format!(
            "async def {}(message: {}) -> None:",
            func_name, message_type
        ));

        // Docstring
        if ctx.generate_docs {
            if let Some(summary) = &op.summary {
                lines.push(format!("{}\"\"\"{}\"\"\"", ctx.indent, summary));
            } else {
                lines.push(format!(
                    "{}\"\"\"Publish message to {} channel.\"\"\"",
                    ctx.indent, channel.name
                ));
            }
        }

        // Publish message
        lines.push(format!(
            "{}await {}_app.publish(\"{}\", message)",
            ctx.indent, app_name, queue_name
        ));

        lines.join("\n")
    }

    /// Generate handler name from channel path
    fn channel_to_handler_name(&self, channel: &str, prefix: &str) -> String {
        let path_part = channel
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("_");

        format!("{}_{}", prefix, to_snake_case(&path_part))
    }

    /// Generate imports
    fn generate_imports(&self, spec: &EventApiSpec) -> Vec<String> {
        let mut imports = vec![
            "import logging".to_string(),
            "from typing import Optional, Any".to_string(),
            "from cclab.meteor import Swarm".to_string(),
        ];

        // Add model imports if schemas exist
        if !spec.messages.models.is_empty() {
            let model_names: Vec<_> = spec
                .messages
                .models
                .iter()
                .map(|m| m.name.clone())
                .collect();
            imports.push(format!("from .models import {}", model_names.join(", ")));
        }

        imports.push(String::new());
        imports.push("logger = logging.getLogger(__name__)".to_string());

        imports
    }
}

impl Default for SwarmGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for SwarmGenerator {
    fn name(&self) -> &str {
        "meteor"
    }

    fn generate_data_models(
        &self,
        _spec: &DataModelSpec,
        _ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        // SwarmGenerator primarily generates from EventApiSpec
        Ok(vec![])
    }

    fn generate_event_api(
        &self,
        spec: &EventApiSpec,
        ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        let content = self.generate_handlers(spec, ctx);
        let imports = self.generate_imports(spec);

        let name = ctx
            .module_name
            .clone()
            .unwrap_or_else(|| "handlers".to_string());

        Ok(vec![
            GeneratedCode::new(name, content, Language::Python).with_imports(imports)
        ])
    }
}

/// Convert to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else if c == '-' || c == ' ' {
            result.push('_');
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_inference::Type;

    #[test]
    fn test_generate_handlers() {
        let spec = EventApiSpec {
            title: "User Service".to_string(),
            version: "1.0.0".to_string(),
            description: Some("User events".to_string()),
            channels: vec![
                ChannelDef {
                    name: "user/created".to_string(),
                    description: Some("User creation events".to_string()),
                    subscribe: Some(OperationDef {
                        operation_id: Some("onUserCreated".to_string()),
                        summary: Some("Handle user created event".to_string()),
                        description: None,
                        message: Type::Instance {
                            name: "UserCreatedEvent".to_string(),
                            module: None,
                            type_args: vec![],
                        },
                    }),
                    publish: None,
                },
                ChannelDef {
                    name: "user/updated".to_string(),
                    description: None,
                    subscribe: None,
                    publish: Some(OperationDef {
                        operation_id: Some("publishUserUpdated".to_string()),
                        summary: Some("Publish user updated event".to_string()),
                        description: None,
                        message: Type::Instance {
                            name: "UserUpdatedEvent".to_string(),
                            module: None,
                            type_args: vec![],
                        },
                    }),
                },
            ],
            messages: DataModelSpec::default(),
        };

        let gen = SwarmGenerator::new();
        let ctx = GenContext::default();
        let result = gen.generate_event_api(&spec, &ctx).unwrap();

        assert_eq!(result.len(), 1);
        let code = &result[0].content;

        assert!(code.contains("user_service_app = Swarm"));
        assert!(code.contains("@user_service_app.task"));
        assert!(code.contains("async def on_user_created"));
        assert!(code.contains("async def publish_user_updated"));
        assert!(code.contains("UserCreatedEvent"));
    }

    #[test]
    fn test_channel_to_handler_name() {
        let gen = SwarmGenerator::new();

        assert_eq!(
            gen.channel_to_handler_name("user/created", "handle"),
            "handle_user_created"
        );
        assert_eq!(
            gen.channel_to_handler_name("orders/placed", "on"),
            "on_orders_placed"
        );
        assert_eq!(
            gen.channel_to_handler_name("notifications", "process"),
            "process_notifications"
        );
    }
}
