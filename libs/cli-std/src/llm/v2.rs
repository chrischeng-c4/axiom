//! `cclab.llm.v2` — offline task navigation for agent-facing CLIs.
//!
//! Version 1 remains the lightweight `Topic` registry. Version 2 adds typed
//! task selection and runbooks while preserving the `topic` and `markdown`
//! JSON compatibility fields consumed by existing clients.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Result};
use serde::Serialize;

use super::Format;

pub const PROTOCOL: &str = "cclab.llm.v2";

/// JSON Schema for the additive `cclab.llm.v2` JSON envelopes.
///
/// It describes the public wire shape, so clients can validate an outline or
/// detail runbook without linking this Rust crate.
pub fn json_schema() -> serde_json::Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": PROTOCOL,
        "oneOf": [
            {
                "title": "Task manifest",
                "type": "object",
                "required": ["topic", "markdown", "protocol", "tasks"],
                "properties": {
                    "topic": { "const": "outline" },
                    "markdown": { "type": "string" },
                    "protocol": { "const": PROTOCOL },
                    "tasks": { "type": "array", "items": { "$ref": "#/$defs/task" } }
                }
            },
            {
                "title": "Typed runbook",
                "type": "object",
                "required": ["topic", "markdown", "protocol", "task", "runbook"],
                "properties": {
                    "topic": { "type": "string", "minLength": 1 },
                    "markdown": { "type": "string" },
                    "protocol": { "const": PROTOCOL },
                    "task": { "$ref": "#/$defs/task" },
                    "runbook": { "$ref": "#/$defs/runbook" },
                    "providers": {
                        "type": "array",
                        "items": { "$ref": "#/$defs/provider" }
                    }
                }
            }
        ],
        "$defs": {
            "provider": {
                "type": "object",
                "required": ["id", "summary", "markdown"],
                "properties": {
                    "id": { "type": "string", "minLength": 1 },
                    "summary": { "type": "string" },
                    "markdown": { "type": "string", "minLength": 1 }
                }
            },
            "task": {
                "type": "object",
                "required": ["id", "use_when", "requires", "reads", "produces", "risk", "topic", "contract_refs"],
                "properties": {
                    "id": { "type": "string", "minLength": 1 },
                    "use_when": { "type": "string" },
                    "requires": { "type": "array", "items": { "type": "string" } },
                    "reads": { "type": "array", "items": { "type": "string" } },
                    "produces": { "type": "array", "items": { "type": "string" } },
                    "risk": { "enum": ["inspect", "local_write", "remote_write"] },
                    "topic": { "type": "string", "minLength": 1 },
                    "contract_refs": { "type": "array", "items": { "type": "string" } }
                }
            },
            "input": {
                "type": "object",
                "required": ["name", "type", "description", "required"],
                "properties": {
                    "name": { "type": "string", "minLength": 1 },
                    "type": { "type": "string", "minLength": 1 },
                    "description": { "type": "string" },
                    "required": { "type": "boolean" }
                }
            },
            "runbook": {
                "type": "object",
                "required": ["purpose", "preconditions", "inputs", "constraints", "steps", "verification", "references"],
                "properties": {
                    "purpose": { "type": "string" },
                    "preconditions": { "type": "array", "items": { "type": "string" } },
                    "inputs": { "type": "array", "items": { "$ref": "#/$defs/input" } },
                    "constraints": { "type": "array", "items": { "type": "string" } },
                    "steps": { "type": "array", "items": { "type": "object" } },
                    "verification": { "type": "array", "items": { "type": "string" } },
                    "references": { "type": "array", "items": { "type": "string" } }
                }
            }
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    Inspect,
    LocalWrite,
    RemoteWrite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Task {
    pub id: String,
    pub use_when: String,
    pub requires: Vec<String>,
    pub reads: Vec<String>,
    pub produces: Vec<String>,
    pub risk: Risk,
    pub topic: String,
    pub contract_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Input {
    pub name: String,
    #[serde(rename = "type")]
    pub value_type: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Step {
    pub id: String,
    pub instruction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_template: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<Input>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Runbook {
    pub purpose: String,
    pub preconditions: Vec<String>,
    pub inputs: Vec<Input>,
    pub constraints: Vec<String>,
    pub steps: Vec<Step>,
    pub verification: Vec<String>,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topic {
    pub task: Task,
    pub runbook: Runbook,
}

/// Content owned by a shared library and composed into one app task topic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProviderContent {
    pub id: String,
    pub summary: String,
    pub markdown: String,
}

#[derive(Debug, Clone)]
pub struct ProtocolDocument {
    project: String,
    topics: Vec<Topic>,
    providers: BTreeMap<String, Vec<ProviderContent>>,
}

impl ProtocolDocument {
    pub fn new(project: impl Into<String>, topics: Vec<Topic>) -> Result<Self> {
        let document = Self {
            project: project.into(),
            topics,
            providers: BTreeMap::new(),
        };
        document.validate()?;
        Ok(document)
    }

    /// Compose a legacy shared-library topic into one typed app topic.
    ///
    /// Provider order follows call order. The provider remains library-owned;
    /// this document only selects the task topic that includes it.
    pub fn with_topic_provider(mut self, topic: &str, provider: &super::Topic) -> Result<Self> {
        self.topic(topic)?;
        if provider.id.trim().is_empty() {
            bail!("LLM provider id cannot be empty");
        }
        if provider.body.trim().is_empty() {
            bail!("LLM provider `{}` markdown cannot be empty", provider.id);
        }

        let providers = self.providers.entry(topic.to_string()).or_default();
        if providers.iter().any(|entry| entry.id == provider.id) {
            bail!(
                "duplicate LLM provider `{}` for topic `{topic}`",
                provider.id
            );
        }
        providers.push(ProviderContent {
            id: provider.id.to_string(),
            summary: provider.summary.to_string(),
            markdown: provider.body.to_string(),
        });
        Ok(self)
    }

    pub fn topics(&self) -> &[Topic] {
        &self.topics
    }

    pub fn render(&self, topic: &str, format: Format) -> Result<String> {
        let markdown = if topic == "outline" {
            self.outline_markdown()
        } else {
            self.topic_markdown(self.topic(topic)?)
        };
        match format {
            Format::Md => Ok(markdown),
            Format::Json => {
                if topic == "outline" {
                    serde_json::to_string_pretty(&serde_json::json!({
                        "topic": "outline",
                        "markdown": markdown,
                        "protocol": PROTOCOL,
                        "tasks": self.topics.iter().map(|entry| &entry.task).collect::<Vec<_>>(),
                    }))
                    .map_err(Into::into)
                } else {
                    let entry = self.topic(topic)?;
                    let mut envelope = serde_json::json!({
                        "topic": topic,
                        "markdown": markdown,
                        "protocol": PROTOCOL,
                        "task": &entry.task,
                        "runbook": &entry.runbook,
                    });
                    if let Some(providers) = self.providers.get(topic) {
                        envelope
                            .as_object_mut()
                            .expect("LLM detail envelope is an object")
                            .insert("providers".into(), serde_json::json!(providers));
                    }
                    serde_json::to_string_pretty(&envelope).map_err(Into::into)
                }
            }
        }
    }

    fn topic(&self, id: &str) -> Result<&Topic> {
        self.topics
            .iter()
            .find(|entry| entry.task.topic == id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "unknown llm topic `{id}`; run `{} llm --topic outline`",
                    self.project
                )
            })
    }

    fn validate(&self) -> Result<()> {
        if self.project.trim().is_empty() {
            bail!("LLM protocol project cannot be empty");
        }
        let mut ids = BTreeSet::new();
        let mut topics = BTreeSet::new();
        for entry in &self.topics {
            if entry.task.id.trim().is_empty() || entry.task.topic.trim().is_empty() {
                bail!("LLM task id and topic cannot be empty");
            }
            if !ids.insert(entry.task.id.as_str()) {
                bail!("duplicate LLM task id `{}`", entry.task.id);
            }
            if !topics.insert(entry.task.topic.as_str()) {
                bail!("duplicate LLM topic `{}`", entry.task.topic);
            }
            for step in &entry.runbook.steps {
                if step.command.is_some() && step.command_template.is_some() {
                    bail!(
                        "step `{}` cannot contain both command and command_template",
                        step.id
                    );
                }
                if let Some(command) = &step.command {
                    if !step.inputs.is_empty() || command.contains('<') || command.contains('{') {
                        bail!("step `{}` command must be fully bound; use command_template with typed inputs", step.id);
                    }
                }
                if step.command_template.is_some() && step.inputs.is_empty() {
                    bail!("step `{}` command_template requires typed inputs", step.id);
                }
                if let Some(template) = &step.command_template {
                    let declared = step
                        .inputs
                        .iter()
                        .map(|input| input.name.as_str())
                        .collect::<BTreeSet<_>>();
                    let referenced = template_placeholders(template)?;
                    if declared != referenced {
                        bail!(
                            "step `{}` command_template placeholders must exactly match its typed inputs",
                            step.id
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn outline_markdown(&self) -> String {
        let mut out = format!("# {} task navigation\n\n", self.project);
        out.push_str("Select the smallest task, then read its typed runbook:\n\n");
        for entry in &self.topics {
            out.push_str(&format!(
                "- `{}` — {} (`{} llm --topic {}`)\n",
                entry.task.id, entry.task.use_when, self.project, entry.task.topic
            ));
        }
        out.push_str(&format!(
            "\nUse `{} llm --topic <topic> --format json` for {} data.\n",
            self.project, PROTOCOL
        ));
        out
    }

    fn topic_markdown(&self, entry: &Topic) -> String {
        let runbook = &entry.runbook;
        let mut out = format!(
            "# {} — {}\n\n{}\n",
            self.project, entry.task.id, runbook.purpose
        );
        markdown_list(&mut out, "Preconditions", &runbook.preconditions);
        if !runbook.inputs.is_empty() {
            out.push_str("\n## Inputs\n\n");
            for input in &runbook.inputs {
                out.push_str(&format!(
                    "- `{}` ({}, {}) — {}\n",
                    input.name,
                    input.value_type,
                    if input.required {
                        "required"
                    } else {
                        "optional"
                    },
                    input.description
                ));
            }
        }
        markdown_list(&mut out, "Constraints", &runbook.constraints);
        if !runbook.steps.is_empty() {
            out.push_str("\n## Steps\n\n");
            for step in &runbook.steps {
                out.push_str(&format!("1. {}\n", step.instruction));
                if let Some(command) = &step.command {
                    out.push_str(&format!("   `{command}`\n"));
                }
                if let Some(template) = &step.command_template {
                    out.push_str(&format!("   Template: `{template}`\n"));
                }
            }
        }
        markdown_list(&mut out, "Verification", &runbook.verification);
        markdown_list(&mut out, "References", &runbook.references);
        if let Some(providers) = self.providers.get(&entry.task.topic) {
            out.push_str("\n## Shared providers\n");
            for provider in providers {
                out.push_str(&format!(
                    "\n### `{}`\n\n{}\n\n{}\n",
                    provider.id,
                    provider.summary,
                    provider.markdown.trim()
                ));
            }
        }
        out
    }
}

fn template_placeholders(template: &str) -> Result<BTreeSet<&str>> {
    let mut placeholders = BTreeSet::new();
    let mut remaining = template;
    while let Some(open) = remaining.find('{') {
        let after_open = &remaining[open + 1..];
        let Some(close) = after_open.find('}') else {
            bail!("command_template has an unclosed placeholder");
        };
        let placeholder = &after_open[..close];
        if placeholder.trim().is_empty()
            || !placeholder
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        {
            bail!("command_template has an invalid placeholder `{{{placeholder}}}`");
        }
        placeholders.insert(placeholder);
        remaining = &after_open[close + 1..];
    }
    if remaining.contains('}') {
        bail!("command_template has a closing brace without an opening brace");
    }
    Ok(placeholders)
}

fn markdown_list(out: &mut String, heading: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    out.push_str(&format!("\n## {heading}\n\n"));
    for value in values {
        out.push_str(&format!("- {value}\n"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROVIDER: crate::llm::Topic = crate::llm::Topic {
        id: "openapi-codegen",
        summary: "Shared generated-client rules.",
        body: "# Shared generator\n\nThe library owns these bytes.",
    };
    const SECOND_PROVIDER: crate::llm::Topic = crate::llm::Topic {
        id: "transport-policy",
        summary: "Shared transport rules.",
        body: "# Shared transport\n\nThe transport library owns these bytes.",
    };

    fn sample() -> ProtocolDocument {
        ProtocolDocument::new(
            "lumen",
            vec![Topic {
                task: Task {
                    id: "local-search".into(),
                    use_when: "inspect a local search contract".into(),
                    requires: vec![],
                    reads: vec!["lumen spec --fields".into()],
                    produces: vec!["validated request body".into()],
                    risk: Risk::Inspect,
                    topic: "search".into(),
                    contract_refs: vec!["spec.fields".into()],
                },
                runbook: Runbook {
                    purpose: "Read the offline contract before querying.".into(),
                    preconditions: vec![],
                    inputs: vec![],
                    constraints: vec!["No network is required.".into()],
                    steps: vec![Step {
                        id: "read-fields".into(),
                        instruction: "Read field capabilities.".into(),
                        command: Some("lumen spec --fields".into()),
                        command_template: None,
                        inputs: vec![],
                    }],
                    verification: vec!["Choose a supported operator.".into()],
                    references: vec!["spec.fields".into()],
                },
            }],
        )
        .unwrap()
    }

    #[test]
    fn json_keeps_compatibility_fields_and_adds_protocol() {
        let json = sample().render("outline", Format::Json).unwrap();
        assert!(json.contains("\"topic\": \"outline\""));
        assert!(json.contains("\"markdown\""));
        assert!(json.contains(PROTOCOL));
        assert!(json.contains("\"tasks\""));
    }

    #[test]
    fn schema_covers_manifest_and_typed_runbook_envelopes() {
        let schema = json_schema();
        assert_eq!(schema["$id"], PROTOCOL);
        assert_eq!(schema["oneOf"].as_array().unwrap().len(), 2);
        assert!(schema["$defs"]["task"]["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field == "contract_refs"));
        assert!(schema["$defs"]["runbook"]["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field == "verification"));
        assert_eq!(
            schema["oneOf"][1]["properties"]["providers"]["items"]["$ref"],
            "#/$defs/provider"
        );
        assert!(!schema["oneOf"][1]["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field == "providers"));
    }

    #[test]
    fn provider_is_ordered_and_does_not_change_the_outline() {
        let mut base = sample();
        let mut other = base.topics[0].clone();
        other.task.id = "other-task".into();
        other.task.topic = "other".into();
        base.topics.push(other);
        let outline_before = base.render("outline", Format::Json).unwrap();
        let outline_markdown_before = base.render("outline", Format::Md).unwrap();
        let other_before = base.render("other", Format::Json).unwrap();
        let other_markdown_before = base.render("other", Format::Md).unwrap();
        let document = base
            .with_topic_provider("search", &PROVIDER)
            .unwrap()
            .with_topic_provider("search", &SECOND_PROVIDER)
            .unwrap();

        assert_eq!(
            document.render("outline", Format::Json).unwrap(),
            outline_before,
            "providers must not change the task manifest"
        );
        assert_eq!(
            document.render("outline", Format::Md).unwrap(),
            outline_markdown_before,
            "providers must not change the outline Markdown"
        );
        assert_eq!(
            document.render("other", Format::Json).unwrap(),
            other_before,
            "a provider must not change another topic's detail envelope"
        );
        assert_eq!(
            document.render("other", Format::Md).unwrap(),
            other_markdown_before,
            "a provider must not change another topic's Markdown"
        );
        let detail: serde_json::Value =
            serde_json::from_str(&document.render("search", Format::Json).unwrap()).unwrap();
        assert_eq!(detail["providers"][0]["id"], "openapi-codegen");
        assert_eq!(detail["providers"][1]["id"], "transport-policy");
        assert_eq!(
            detail["providers"][0]["markdown"],
            "# Shared generator\n\nThe library owns these bytes."
        );
        let markdown = document.render("search", Format::Md).unwrap();
        assert!(markdown.contains("## Shared providers"));
        assert!(markdown.contains("The library owns these bytes."));
        assert!(
            markdown.find(PROVIDER.body).unwrap() < markdown.find(SECOND_PROVIDER.body).unwrap(),
            "provider Markdown must follow registration order"
        );
    }

    #[test]
    fn provider_registration_rejects_unknown_empty_and_duplicate_inputs() {
        assert!(sample().with_topic_provider("missing", &PROVIDER).is_err());

        const EMPTY_ID: crate::llm::Topic = crate::llm::Topic {
            id: " ",
            summary: "invalid",
            body: "body",
        };
        const EMPTY_BODY: crate::llm::Topic = crate::llm::Topic {
            id: "empty-body",
            summary: "invalid",
            body: " \n",
        };
        assert!(sample().with_topic_provider("search", &EMPTY_ID).is_err());
        assert!(sample().with_topic_provider("search", &EMPTY_BODY).is_err());
        assert!(sample()
            .with_topic_provider("search", &PROVIDER)
            .unwrap()
            .with_topic_provider("search", &PROVIDER)
            .is_err());
        let mut two_topics = sample();
        let mut other = two_topics.topics[0].clone();
        other.task.id = "other-task".into();
        other.task.topic = "other".into();
        two_topics.topics.push(other);
        assert!(two_topics
            .with_topic_provider("search", &PROVIDER)
            .unwrap()
            .with_topic_provider("other", &PROVIDER)
            .is_ok());
    }

    #[test]
    fn unbound_commands_require_a_template() {
        let mut document = sample();
        let mut topic = document.topics.remove(0);
        topic.runbook.steps[0].command = Some("lumen query --url <url>".into());
        assert!(ProtocolDocument::new("lumen", vec![topic]).is_err());
    }

    #[test]
    fn templates_must_name_each_typed_input_exactly_once_or_more() {
        let mut document = sample();
        let mut topic = document.topics.remove(0);
        topic.runbook.steps[0].command = None;
        topic.runbook.steps[0].command_template = Some("lumen query --url {url}".into());
        topic.runbook.steps[0].inputs = vec![Input {
            name: "other".into(),
            value_type: "string".into(),
            description: "wrong input".into(),
            required: true,
        }];
        assert!(ProtocolDocument::new("lumen", vec![topic]).is_err());
    }
}
