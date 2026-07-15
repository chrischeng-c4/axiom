//! `dx-contract` TD section generator.
//!
//! This is deliberately not an EC `tool-contract` generator. It emits the
//! decisions needed to project an offline task manifest and runbooks; runtime
//! facts and EC evidence stay in their respective sources.

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

pub struct DxContractGenerator;

impl Generator for DxContractGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::DxContract
    }

    fn generate(&self, _args: &GeneratorArgs) -> String {
        r#"```yaml
version: 1
authority:
  runtime: "Runtime types, validation, CLI registration, and project traits establish structural behavior."
  decisions: "This dx-contract owns task selection, narrative, typed inputs, templates, and artifact selection."
  verification: "EC proves runtime behavior and generated public artifacts remain aligned."
field_catalog:
  source: "runtime field capability mapping"
artifacts:
  task_manifest: "<cli> llm --topic outline --format json"
  runbooks: "<cli> llm --topic <id> [--format md|json]"
llm_protocol:
  protocol: cclab.llm.v2
  tasks:
    - id: inspect-contract
      use_when: "select the smallest supported task before changing a live surface"
      requires: []
      reads: ["offline contract"]
      produces: ["validated next task selection"]
      risk: inspect
      purpose: "Read the generated contract before issuing a command."
      preconditions: ["The project CLI is installed."]
      inputs: []
      constraints: ["Only fully-bound commands are runnable; templates require typed inputs."]
      instruction: "Read the offline contract."
      command: "<cli> llm --topic outline"
      verification: ["Confirm the selected task is advertised by the manifest."]
```
"#
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_a_distinct_dx_contract_skeleton() {
        let output = DxContractGenerator.generate(&GeneratorArgs::new(SectionType::DxContract));
        assert!(output.contains("cclab.llm.v2"));
        assert!(output.contains("typed inputs"));
        assert!(!output.contains("tool_contracts:"));
    }
}
