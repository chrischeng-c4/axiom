// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-test-case-rs.md#source
// CODEGEN-BEGIN
//! Agent test case definitions and expected outputs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An agent test case with input, expected outputs, and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-test-case-rs.md#source
pub struct AgentTestCase {
    /// Unique test case identifier
    pub id: String,

    /// Human-readable test name
    pub name: String,

    /// Input prompt for the agent
    pub input: String,

    /// Expected output (exact match or contains)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_output: Option<String>,

    /// Expected output regex pattern
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_output_regex: Option<String>,

    /// Expected tools to be called
    #[serde(default)]
    pub expected_tools: Vec<ExpectedToolCall>,

    /// Maximum allowed latency in milliseconds (SLA budget)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_latency_ms: Option<f64>,

    /// Maximum allowed cost in USD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cost_usd: Option<f64>,

    /// Quality criteria for LLM-as-judge evaluation
    #[serde(default)]
    pub quality_criteria: Vec<QualityCriterion>,

    /// Test category for grouping (e.g., "customer_support", "code_generation")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-test-case-rs.md#source
impl Default for AgentTestCase {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            input: String::new(),
            expected_output: None,
            expected_output_regex: None,
            expected_tools: Vec::new(),
            max_latency_ms: None,
            max_cost_usd: None,
            quality_criteria: Vec::new(),
            category: None,
            metadata: HashMap::new(),
        }
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-test-case-rs.md#source
impl AgentTestCase {
    /// Create a new test case with id, name, and input
    pub fn new(id: impl Into<String>, name: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            input: input.into(),
            ..Default::default()
        }
    }

    /// Set expected output (exact match or contains)
    pub fn with_expected_output(mut self, output: impl Into<String>) -> Self {
        self.expected_output = Some(output.into());
        self
    }

    /// Set expected output regex pattern
    pub fn with_expected_output_regex(mut self, regex: impl Into<String>) -> Self {
        self.expected_output_regex = Some(regex.into());
        self
    }

    /// Add an expected tool call
    pub fn with_expected_tool(mut self, tool: ExpectedToolCall) -> Self {
        self.expected_tools.push(tool);
        self
    }

    /// Set maximum latency constraint
    pub fn with_max_latency_ms(mut self, max_ms: f64) -> Self {
        self.max_latency_ms = Some(max_ms);
        self
    }

    /// Set maximum cost constraint
    pub fn with_max_cost_usd(mut self, max_usd: f64) -> Self {
        self.max_cost_usd = Some(max_usd);
        self
    }

    /// Add a quality criterion for LLM-as-judge
    pub fn with_quality_criterion(mut self, criterion: QualityCriterion) -> Self {
        self.quality_criteria.push(criterion);
        self
    }

    /// Set category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
}

/// Expected tool call specification
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-test-case-rs.md#source
pub struct ExpectedToolCall {
    /// Tool name (e.g., "calculate", "search_web")
    pub name: String,

    /// Expected arguments (optional, for strict matching)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<String, serde_json::Value>>,

    /// Whether this tool call is optional (affects recall calculation)
    #[serde(default)]
    pub optional: bool,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-test-case-rs.md#source
impl ExpectedToolCall {
    /// Create a new expected tool call
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arguments: None,
            optional: false,
        }
    }

    /// Set expected arguments
    pub fn with_arguments(mut self, args: HashMap<String, serde_json::Value>) -> Self {
        self.arguments = Some(args);
        self
    }

    /// Mark as optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

/// Quality criterion for LLM-as-judge evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-test-case-rs.md#source
pub struct QualityCriterion {
    /// Criterion name (e.g., "relevance", "clarity", "accuracy")
    pub name: String,

    /// Description/guidance for the judge
    pub description: String,

    /// Weight for weighted scoring (default: 1.0)
    #[serde(default = "default_weight")]
    pub weight: f64,
}

fn default_weight() -> f64 {
    1.0
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-test-case-rs.md#source
impl QualityCriterion {
    /// Create a new quality criterion
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            weight: 1.0,
        }
    }

    /// Set weight
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_test_case_builder() {
        let test_case = AgentTestCase::new(
            "test-001",
            "Capital question",
            "What is the capital of France?",
        )
        .with_expected_output_regex(r"Paris")
        .with_max_latency_ms(2000.0)
        .with_max_cost_usd(0.01)
        .with_category("geography");

        assert_eq!(test_case.id, "test-001");
        assert_eq!(test_case.name, "Capital question");
        assert_eq!(test_case.input, "What is the capital of France?");
        assert_eq!(test_case.expected_output_regex.unwrap(), "Paris");
        assert_eq!(test_case.max_latency_ms.unwrap(), 2000.0);
        assert_eq!(test_case.max_cost_usd.unwrap(), 0.01);
        assert_eq!(test_case.category.unwrap(), "geography");
    }

    #[test]
    fn test_expected_tool_call() {
        let tool = ExpectedToolCall::new("calculate").optional();

        assert_eq!(tool.name, "calculate");
        assert!(tool.optional);
        assert!(tool.arguments.is_none());
    }

    #[test]
    fn test_quality_criterion() {
        let criterion =
            QualityCriterion::new("relevance", "Is the response relevant?").with_weight(2.0);

        assert_eq!(criterion.name, "relevance");
        assert_eq!(criterion.description, "Is the response relevant?");
        assert_eq!(criterion.weight, 2.0);
    }

    #[test]
    fn test_serialization() {
        let test_case = AgentTestCase::new("test-001", "Test", "Input")
            .with_expected_output("Output")
            .with_expected_tool(ExpectedToolCall::new("tool1"));

        let json = serde_json::to_string(&test_case).unwrap();
        let deserialized: AgentTestCase = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "test-001");
        assert_eq!(deserialized.expected_tools.len(), 1);
        assert_eq!(deserialized.expected_tools[0].name, "tool1");
    }
}
// CODEGEN-END
