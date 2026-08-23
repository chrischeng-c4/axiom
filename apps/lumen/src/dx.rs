// HANDWRITE-BEGIN gap="missing-generator:dx-contract:lumen-runtime-renderer" tracker="1683" reason="The runtime renderer binds TD task decisions to Rust FieldType capabilities; AW has the dx-contract parser/skeleton but not yet this cross-source Rust emitter."
//! Generated Developer & Agent Experience surface for Lumen.
//!
//! Runtime field operations come from [`FieldType::capabilities`]. Task
//! navigation decisions are compiled from `src/dx-contract.yaml`, which this
//! module `include_str!`s so the contract is in the binary rather than read at
//! run time — runbook prose, typed inputs, and command templates cannot become
//! a second hand-maintained CLI catalogue.
//!
//! The split is deliberate and it is an authority split, not a layout one. The
//! yaml owns task classification, narrative, preconditions, typed inputs,
//! templates and artifact selection; the Rust owns structural behaviour —
//! [`FieldType`] and its capabilities, runtime validation, CLI registration.
//! Neither may restate the other's half: a capability written into the yaml is
//! a claim nothing checks, and a runbook step written into Rust is a catalogue
//! that drifts from the one `lumen llm` serves.

use cli_std::llm::v2::{Input, ProtocolDocument, Risk, Runbook, Step, Task, Topic};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::types::FieldType;

const DX_CONTRACT_REF: &str = "apps/lumen/src/dx-contract.yaml";
const DX_CONTRACT_SOURCE: &str = include_str!("dx-contract.yaml");

/// Field declarations and query operations emitted by `lumen spec --fields`.
pub fn field_catalog() -> Value {
    json!({
        "schema_endpoint": "PUT /collections/{collection}",
        "field_types": FieldType::ALL.into_iter().map(field_entry).collect::<Vec<_>>(),
        "analyzers": [
            { "name": "whitespace_lower", "purpose": "split on whitespace, lowercase (default lexical)" },
            { "name": "ngram", "purpose": "character n-grams — substring and CJK matching" },
            { "name": "jieba", "purpose": "Chinese word segmentation (requires the `jieba` build feature)" }
        ]
    })
}

fn field_entry(field_type: FieldType) -> Value {
    let capabilities = field_type.capabilities();
    let (name, purpose) = match field_type {
        FieldType::Text => (
            "text",
            "long text for BM25 lexical ranking; tokenized at index time",
        ),
        FieldType::Keyword => (
            "keyword",
            "varchar-like exact value; byte/lexicographic range and sort",
        ),
        FieldType::Number => ("number", "numeric value; numeric range and sort"),
        FieldType::Set => ("set", "multi-valued keyword membership"),
        FieldType::Vector => ("vector", "semantic kNN over a caller-supplied embedding"),
        FieldType::Hash => ("hash", "perceptual/structural near-duplicate search"),
    };
    let mut entry = json!({
        "type": name,
        "purpose": purpose,
        "operations": {
            "bm25": capabilities.bm25,
            "exact": capabilities.exact,
            "prefix": capabilities.prefix,
            "range": capabilities.range,
            "sort": capabilities.sort,
            "set_membership": capabilities.set_membership,
            "vector_search": capabilities.vector_search,
            "hamming": capabilities.hamming,
        }
    });
    let object = entry
        .as_object_mut()
        .expect("field catalog entry is an object");
    match field_type {
        FieldType::Text => {
            object.insert(
                "analyzers".into(),
                json!(["whitespace_lower", "ngram", "jieba"]),
            );
            object.insert("queries".into(), json!(["match"]));
        }
        FieldType::Keyword => {
            object.insert(
                "queries".into(),
                json!(["term", "terms", "prefix", "range", "sort"]),
            );
        }
        FieldType::Number => {
            object.insert("queries".into(), json!(["term", "range", "sort"]));
        }
        FieldType::Set => {
            object.insert("queries".into(), json!(["term", "terms"]));
        }
        FieldType::Vector => {
            object.insert("metrics".into(), json!(["cosine", "dot", "l2"]));
            object.insert("queries".into(), json!(["knn"]));
        }
        FieldType::Hash => {
            object.insert(
                "value".into(),
                json!("16-hex-character string; optional 0x prefix accepted"),
            );
            object.insert("queries".into(), json!(["hamming"]));
            object.insert("schema".into(), json!({ "type": "hash" }));
        }
    }
    entry
}

/// The Lumen `cclab.llm.v2` task-navigation protocol, rendered from the TD.
pub fn llm_protocol() -> ProtocolDocument {
    let contract = dx_contract();
    assert_eq!(
        contract.llm_protocol.protocol,
        cli_std::llm::v2::PROTOCOL,
        "DX contract must declare the supported LLM protocol"
    );
    ProtocolDocument::new(
        "lumen",
        contract
            .llm_protocol
            .tasks
            .into_iter()
            .map(task_from_contract)
            .collect(),
    )
    .expect("Lumen DX contract is internally valid")
}

pub fn render_llm(topic: &str, format: cli_std::llm::Format) -> anyhow::Result<String> {
    llm_protocol().render(topic, format)
}

#[derive(Debug, Deserialize)]
struct DxContract {
    llm_protocol: LlmProtocolContract,
}

#[derive(Debug, Deserialize)]
struct LlmProtocolContract {
    protocol: String,
    tasks: Vec<TaskContract>,
}

#[derive(Debug, Deserialize)]
struct TaskContract {
    id: String,
    use_when: String,
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    reads: Vec<String>,
    #[serde(default)]
    produces: Vec<String>,
    risk: String,
    purpose: String,
    #[serde(default)]
    preconditions: Vec<String>,
    #[serde(default)]
    inputs: Vec<InputContract>,
    #[serde(default)]
    constraints: Vec<String>,
    instruction: String,
    command: Option<String>,
    command_template: Option<String>,
    #[serde(default)]
    verification: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct InputContract {
    name: String,
    #[serde(rename = "type")]
    value_type: String,
    description: String,
    required: bool,
}

fn dx_contract() -> DxContract {
    serde_yaml::from_str(DX_CONTRACT_SOURCE).expect("DX contract YAML is valid")
}

fn task_from_contract(contract: TaskContract) -> Topic {
    let inputs = contract
        .inputs
        .into_iter()
        .map(|input| Input {
            name: input.name,
            value_type: input.value_type,
            description: input.description,
            required: input.required,
        })
        .collect::<Vec<_>>();
    let step_inputs = inputs.clone();
    Topic {
        task: Task {
            id: contract.id.clone(),
            use_when: contract.use_when,
            requires: contract.requires,
            reads: contract.reads,
            produces: contract.produces,
            risk: match contract.risk.as_str() {
                "inspect" => Risk::Inspect,
                "local_write" => Risk::LocalWrite,
                "remote_write" => Risk::RemoteWrite,
                other => panic!("DX contract has unknown risk `{other}`"),
            },
            topic: contract.id,
            contract_refs: vec![DX_CONTRACT_REF.into()],
        },
        runbook: Runbook {
            purpose: contract.purpose,
            preconditions: contract.preconditions,
            inputs,
            constraints: contract.constraints,
            steps: vec![Step {
                id: "primary".into(),
                instruction: contract.instruction,
                command: contract.command,
                command_template: contract.command_template,
                inputs: step_inputs,
            }],
            verification: contract.verification,
            references: vec![DX_CONTRACT_REF.into()],
        },
    }
}
// HANDWRITE-END
