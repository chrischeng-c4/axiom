---
id: sdd-generate-sequence-plus-generator
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Sequence Plus Generator Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/sequence_plus/generator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SequencePlusGenerator` | projects/agentic-workflow/src/generate/diagrams/sequence_plus/generator.rs | struct | pub | 28 |  |
| `SequencePlusOutput` | projects/agentic-workflow/src/generate/diagrams/sequence_plus/generator.rs | struct | pub | 15 |  |
| `generate` | projects/agentic-workflow/src/generate/diagrams/sequence_plus/generator.rs | function | pub | 36 | generate(         &self,         sequence: &SequenceDef,         validation: SequenceValidationResult,     ) -> Result<SequencePlusOutput, String> |
| `generate_mermaid` | projects/agentic-workflow/src/generate/diagrams/sequence_plus/generator.rs | function | pub | 77 | generate_mermaid(&self, sequence: &SequenceDef) -> Result<String, String> |
| `new` | projects/agentic-workflow/src/generate/diagrams/sequence_plus/generator.rs | function | pub | 31 | new() -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  SequencePlusOutput:
    type: object
    required: [frontmatter, diagram, validation, combined]
    description: Sequence+ generator output bundle.
    properties:
      frontmatter:
        type: string
        description: "YAML frontmatter."
      diagram:
        type: string
        description: "Mermaid sequence diagram."
      validation:
        type: object
        x-rust-type: "SequenceValidationResult"
        description: "Validation result."
      combined:
        type: string
        description: "Combined frontmatter + diagram output."
    x-rust-struct:
      derive: [Debug, Clone, Serialize]

  SequencePlusGenerator:
    type: object
    required: []
    properties: {}
    description: Sequence+ generator (unit struct).
    x-rust-struct:
      derive: []
      unit: true
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/sequence_plus/generator.rs -->
````rust
//! Sequence+ generator
//!
//! Generates Mermaid+ output from validated sequence diagram definitions.

use super::schema::{AltBlockType, ArrowType, NotePosition, ParticipantType, SequenceDef};
use super::validator::SequenceValidationResult;

use serde::Serialize;

/// Sequence+ generator output bundle.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/generator.md#schema
#[derive(Debug, Clone, Serialize)]
pub struct SequencePlusOutput {
    /// YAML frontmatter.
    pub frontmatter: String,
    /// Mermaid sequence diagram.
    pub diagram: String,
    /// Validation result.
    pub validation: SequenceValidationResult,
    /// Combined frontmatter + diagram output.
    pub combined: String,
}

/// Sequence+ generator (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/generator.md#schema
pub struct SequencePlusGenerator;
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/generator.md#source
impl SequencePlusGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate Mermaid+ output from a sequence definition
    pub fn generate(
        &self,
        sequence: &SequenceDef,
        validation: SequenceValidationResult,
    ) -> Result<SequencePlusOutput, String> {
        let frontmatter = self.generate_frontmatter(sequence)?;
        let diagram = self.generate_mermaid(sequence)?;

        // Combine into Mermaid+ format (frontmatter inside code block per Mermaid spec)
        let mut combined = String::new();
        combined.push_str("```mermaid\n");
        combined.push_str("---\n");
        combined.push_str(&frontmatter);
        combined.push_str("---\n");
        combined.push_str(&diagram);
        combined.push_str("```\n");

        if !validation.warnings.is_empty() {
            combined.push_str("\n<!-- Validation Warnings:\n");
            for w in &validation.warnings {
                combined.push_str(&format!("  - {}: {} (at {})\n", w.code, w.message, w.path));
            }
            combined.push_str("-->\n");
        }

        Ok(SequencePlusOutput {
            frontmatter,
            diagram,
            validation,
            combined,
        })
    }

    fn generate_frontmatter(&self, sequence: &SequenceDef) -> Result<String, String> {
        let yaml = serde_yaml::to_string(sequence)
            .map_err(|e| format!("YAML serialization error: {}", e))?;
        let yaml = yaml.strip_prefix("---\n").unwrap_or(&yaml);
        Ok(yaml.to_string())
    }

    /// Generate Mermaid sequence diagram
    pub fn generate_mermaid(&self, sequence: &SequenceDef) -> Result<String, String> {
        let mut mermaid = String::new();
        mermaid.push_str("sequenceDiagram\n");

        // Title
        if let Some(ref title) = sequence.title {
            mermaid.push_str(&format!("    title {}\n", title));
        }

        // Participants (sorted for consistent output)
        let mut participants: Vec<_> = sequence.participants.iter().collect();
        participants.sort_by(|a, b| a.0.cmp(b.0));

        for (id, participant) in participants {
            let type_str = match participant.participant_type {
                ParticipantType::Actor => "actor",
                ParticipantType::Participant => "participant",
            };
            mermaid.push_str(&format!(
                "    {} {} as {}\n",
                type_str, id, participant.label
            ));
        }

        // Messages with loops/alts/notes
        for (i, msg) in sequence.messages.iter().enumerate() {
            // Check for loop starts
            for loop_def in &sequence.loops {
                if loop_def.start == i {
                    mermaid.push_str(&format!("    loop {}\n", loop_def.label));
                }
            }

            // Check for alt starts
            for alt in &sequence.alts {
                if alt.start == i {
                    let block_type = match alt.block_type {
                        AltBlockType::Alt => "alt",
                        AltBlockType::Opt => "opt",
                        AltBlockType::Par => "par",
                        AltBlockType::Critical => "critical",
                        AltBlockType::Break => "break",
                    };
                    mermaid.push_str(&format!("    {} {}\n", block_type, alt.condition));
                }
            }

            // Generate message
            let arrow = match msg.arrow_type {
                ArrowType::Solid => "->>",
                ArrowType::Dotted => "-->>",
                ArrowType::SolidOpen => "->",
                ArrowType::DottedOpen => "-->",
            };

            if msg.activate {
                mermaid.push_str(&format!(
                    "    {}{}+{}: {}\n",
                    msg.from, arrow, msg.to, msg.text
                ));
            } else if msg.deactivate {
                mermaid.push_str(&format!(
                    "    {}{}-{}: {}\n",
                    msg.from, arrow, msg.to, msg.text
                ));
            } else {
                mermaid.push_str(&format!(
                    "    {}{}{}: {}\n",
                    msg.from, arrow, msg.to, msg.text
                ));
            }

            // Notes after this message
            for note in &sequence.notes {
                if note.after_message == Some(i) {
                    self.generate_note(&mut mermaid, note);
                }
            }

            // Check for alt else branches
            for alt in &sequence.alts {
                if alt.end == i {
                    for branch in &alt.else_branches {
                        if branch.start == i + 1 {
                            let condition = branch.condition.as_deref().unwrap_or("");
                            if condition.is_empty() {
                                mermaid.push_str("    else\n");
                            } else {
                                mermaid.push_str(&format!("    else {}\n", condition));
                            }
                        }
                    }
                }
            }

            // Check for loop ends
            for loop_def in &sequence.loops {
                if loop_def.end == i {
                    mermaid.push_str("    end\n");
                }
            }

            // Check for alt ends
            for alt in &sequence.alts {
                let final_end = alt.else_branches.last().map(|b| b.end).unwrap_or(alt.end);
                if final_end == i {
                    mermaid.push_str("    end\n");
                }
            }
        }

        // Notes without after_message (at the end)
        for note in &sequence.notes {
            if note.after_message.is_none() {
                self.generate_note(&mut mermaid, note);
            }
        }

        Ok(mermaid)
    }

    fn generate_note(&self, mermaid: &mut String, note: &super::schema::NoteDef) {
        let pos = match note.position {
            NotePosition::RightOf => "right of",
            NotePosition::LeftOf => "left of",
            NotePosition::Over => "over",
        };

        if note.participants.is_empty() {
            // Free-floating note (not valid in Mermaid, skip)
            return;
        }

        if note.participants.len() == 1 {
            mermaid.push_str(&format!(
                "    Note {} {}: {}\n",
                pos, note.participants[0], note.text
            ));
        } else {
            // Over multiple participants
            let participants = note.participants.join(",");
            mermaid.push_str(&format!("    Note over {}: {}\n", participants, note.text));
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/generator.md#source
impl Default for SequencePlusGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::validator::SequenceValidator;
    use super::*;
    use serde_json::json;

    fn parse_sequence(json: serde_json::Value) -> SequenceDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_generate_simple_sequence() {
        let sequence = parse_sequence(json!({
            "id": "test",
            "participants": {
                "alice": { "label": "Alice", "type": "actor" },
                "bob": { "label": "Bob" }
            },
            "messages": [
                { "from": "alice", "to": "bob", "text": "Hello" },
                { "from": "bob", "to": "alice", "text": "Hi", "type": "dotted" }
            ]
        }));

        let validation = SequenceValidator::new().validate(&sequence);
        let output = SequencePlusGenerator::new()
            .generate(&sequence, validation)
            .unwrap();

        assert!(output.diagram.contains("sequenceDiagram"));
        assert!(output.diagram.contains("actor alice as Alice"));
        assert!(output.diagram.contains("participant bob as Bob"));
        assert!(output.diagram.contains("alice->>bob: Hello"));
        assert!(output.diagram.contains("bob-->>alice: Hi"));
    }

    #[test]
    fn test_generate_with_activation() {
        let sequence = parse_sequence(json!({
            "id": "test",
            "participants": {
                "a": { "label": "A" },
                "b": { "label": "B" }
            },
            "messages": [
                { "from": "a", "to": "b", "text": "Request", "activate": true },
                { "from": "b", "to": "a", "text": "Response", "type": "dotted", "deactivate": true }
            ]
        }));

        let validation = SequenceValidator::new().validate(&sequence);
        let output = SequencePlusGenerator::new()
            .generate(&sequence, validation)
            .unwrap();

        assert!(output.diagram.contains("a->>+b: Request"));
        assert!(output.diagram.contains("b-->>-a: Response"));
    }

    #[test]
    fn test_generate_with_loop() {
        let sequence = parse_sequence(json!({
            "id": "test",
            "participants": {
                "a": { "label": "A" },
                "b": { "label": "B" }
            },
            "messages": [
                { "from": "a", "to": "b", "text": "Retry" },
                { "from": "b", "to": "a", "text": "Ack", "type": "dotted" }
            ],
            "loops": [
                { "label": "3 times", "start": 0, "end": 1 }
            ]
        }));

        let validation = SequenceValidator::new().validate(&sequence);
        let output = SequencePlusGenerator::new()
            .generate(&sequence, validation)
            .unwrap();

        assert!(output.diagram.contains("loop 3 times"));
        assert!(output.diagram.contains("end"));
    }

    #[test]
    fn test_mermaid_plus_format() {
        let sequence = parse_sequence(json!({
            "id": "simple",
            "participants": {
                "a": { "label": "A" },
                "b": { "label": "B" }
            },
            "messages": [
                { "from": "a", "to": "b", "text": "Test" }
            ]
        }));

        let validation = SequenceValidator::new().validate(&sequence);
        let output = SequencePlusGenerator::new()
            .generate(&sequence, validation)
            .unwrap();

        // Check combined format (frontmatter inside code block)
        assert!(output.combined.starts_with("```mermaid\n---\n"));
        assert!(output.combined.contains("id: simple"));
        assert!(output.combined.contains("sequenceDiagram"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/sequence_plus/generator.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete Sequence+ generator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Output struct + unit-struct generator; standard pattern.
- [schema] Both well-formed; foreign type SequenceValidationResult via x-rust-type.
- [changes] Standard split with both in `replaces`; impls preserved hand-written.
