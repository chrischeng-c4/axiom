// HANDWRITE-BEGIN gap="sift-pre-journal-governance" tracker="1657" reason="Load default/project policies and apply denied-key, truncation, and default-off GenAI content redaction idempotently."
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::Path,
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{AttributeValue, OperationalEventV2};

const POLICY_FILE_ENV: &str = "SIFT_GOVERNANCE_POLICY_FILE";
const DEFAULT_MAX_STRING_BYTES: usize = 4_096;
const DEFAULT_REDACTION: &str = "[REDACTED]";

/// Privacy and cardinality policy applied before an event can enter Raft or
/// the raw journal. Attribute keys are matched case-insensitively.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct GovernancePolicy {
    pub capture_genai_content: bool,
    pub max_string_bytes: usize,
    pub allowed_attribute_keys: Option<BTreeSet<String>>,
    pub denied_attribute_keys: BTreeSet<String>,
    pub redaction_text: String,
}

impl Default for GovernancePolicy {
    fn default() -> Self {
        Self {
            capture_genai_content: false,
            max_string_bytes: DEFAULT_MAX_STRING_BYTES,
            allowed_attribute_keys: None,
            denied_attribute_keys: BTreeSet::new(),
            redaction_text: DEFAULT_REDACTION.to_string(),
        }
    }
}

impl GovernancePolicy {
    fn validate(&self) -> Result<()> {
        if self.max_string_bytes == 0 {
            bail!("governance max_string_bytes must be greater than zero");
        }
        if self.redaction_text.is_empty() {
            bail!("governance redaction_text must not be empty");
        }
        Ok(())
    }

    fn allows_key(&self, key: &str) -> bool {
        let allowed = self.allowed_attribute_keys.as_ref().is_none_or(|keys| {
            keys.iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(key))
        });
        let denied = self
            .denied_attribute_keys
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(key));
        allowed && !denied
    }
}

/// Default policy plus project-specific overrides. The exact project key is
/// selected from the canonical event before any governed bytes are serialized.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct GovernancePolicySet {
    pub default: GovernancePolicy,
    pub projects: BTreeMap<String, GovernancePolicy>,
}

impl GovernancePolicySet {
    pub fn from_env() -> Result<Self> {
        let Ok(path) = env::var(POLICY_FILE_ENV) else {
            return Ok(Self::default());
        };
        Self::from_path(path)
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let policies: Self = serde_json::from_slice(
            &fs::read(path)
                .with_context(|| format!("read Sift governance policy {}", path.display()))?,
        )
        .with_context(|| format!("decode Sift governance policy {}", path.display()))?;
        policies.validate()?;
        Ok(policies)
    }

    pub fn validate(&self) -> Result<()> {
        self.default.validate().context("validate default policy")?;
        for (project, policy) in &self.projects {
            if project.trim().is_empty() {
                bail!("governance project key must not be empty");
            }
            policy
                .validate()
                .with_context(|| format!("validate policy for project {project}"))?;
        }
        Ok(())
    }

    pub fn govern(&self, mut event: OperationalEventV2) -> Result<OperationalEventV2> {
        event
            .validate()
            .context("validate event before governance")?;
        let policy = self.projects.get(&event.project).unwrap_or(&self.default);
        policy.validate()?;

        let is_genai = event.attributes.keys().any(|key| {
            let key = key.to_ascii_lowercase();
            key.starts_with("gen_ai.") || key.starts_with("llm.")
        });
        govern_attributes(&mut event.attributes, policy, is_genai);
        if let Some(scope) = &mut event.instrumentation_scope {
            govern_attributes(&mut scope.attributes, policy, is_genai);
        }
        for value in event.resource.values_mut() {
            truncate_utf8(value, policy.max_string_bytes);
        }
        govern_json(&mut event.payload, policy, is_genai, None);

        event
            .validate()
            .context("validate event after governance")?;
        Ok(event)
    }
}

fn govern_attributes(
    attributes: &mut BTreeMap<String, AttributeValue>,
    policy: &GovernancePolicy,
    is_genai: bool,
) {
    for (key, value) in attributes {
        let redact = !policy.allows_key(key)
            || (is_genai && !policy.capture_genai_content && is_content_key(key));
        if redact {
            *value = AttributeValue::String(policy.redaction_text.clone());
        } else {
            govern_attribute_value(value, policy.max_string_bytes);
        }
    }
}

fn govern_attribute_value(value: &mut AttributeValue, max_string_bytes: usize) {
    match value {
        AttributeValue::String(value) => truncate_utf8(value, max_string_bytes),
        AttributeValue::Array(values) => {
            for value in values {
                govern_attribute_value(value, max_string_bytes);
            }
        }
        AttributeValue::Map(values) => {
            for value in values.values_mut() {
                govern_attribute_value(value, max_string_bytes);
            }
        }
        AttributeValue::Bool(_)
        | AttributeValue::Int(_)
        | AttributeValue::Double(_)
        | AttributeValue::Bytes(_) => {}
    }
}

fn govern_json(value: &mut Value, policy: &GovernancePolicy, is_genai: bool, key: Option<&str>) {
    if is_genai && !policy.capture_genai_content && key.is_some_and(is_content_key) {
        *value = Value::String(policy.redaction_text.clone());
        return;
    }
    match value {
        Value::String(value) => {
            // Binary/profile payloads are externalized by the raw-storage
            // boundary immediately after governance. Preserve valid base64
            // long enough to hash it, while GenAI content keys above remain
            // default-off and are redacted before any blob is written.
            if !key.is_some_and(is_base64_key) {
                truncate_utf8(value, policy.max_string_bytes);
            }
        }
        Value::Array(values) => {
            for value in values {
                govern_json(value, policy, is_genai, key);
            }
        }
        Value::Object(values) => {
            for (key, value) in values {
                govern_json(value, policy, is_genai, Some(key));
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn is_content_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    let without_base64 = key.strip_suffix("base64").unwrap_or(&key);
    let without_bytes = without_base64
        .trim_end_matches(['_', '.'])
        .strip_suffix("bytes")
        .unwrap_or(without_base64)
        .trim_end_matches(['_', '.']);
    matches!(
        without_bytes,
        "prompt"
            | "prompts"
            | "completion"
            | "completions"
            | "response"
            | "responses"
            | "message"
            | "messages"
            | "input"
            | "output"
            | "system_instruction"
            | "system_instructions"
            | "gen_ai.prompt"
            | "gen_ai.completion"
            | "gen_ai.input.messages"
            | "gen_ai.output.messages"
            | "gen_ai.system_instructions"
            | "llm.input_messages"
            | "llm.output_messages"
    ) || without_bytes.ends_with(".prompt")
        || without_bytes.ends_with(".completion")
        || without_bytes.ends_with(".response")
        || without_bytes.ends_with(".messages")
}

fn is_base64_key(key: &str) -> bool {
    key.to_ascii_lowercase().ends_with("base64")
}

fn truncate_utf8(value: &mut String, max_bytes: usize) {
    if value.len() <= max_bytes {
        return;
    }
    let mut boundary = max_bytes;
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    value.truncate(boundary);
}

// HANDWRITE-END
