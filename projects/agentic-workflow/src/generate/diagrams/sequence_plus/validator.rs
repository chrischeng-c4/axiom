// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/validator.md#source
// CODEGEN-BEGIN
//! Sequence+ semantic validator
//!
//! Validates sequence diagram definitions for:
//! - Participant existence (message from/to must exist)
//! - Loop/Alt block validity (indices within bounds)
//! - Activation balance

use super::schema::{AltDef, SequenceDef};
use std::collections::HashSet;

use serde::Serialize;

/// Error severity.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/validator.md#schema
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SequenceSeverity {
    Error,
    Warning,
}

/// Validation result.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/validator.md#schema
#[derive(Debug, Clone, Serialize, Default)]
pub struct SequenceValidationResult {
    /// Whether validation passed (no errors).
    pub valid: bool,
    /// Validation errors.
    pub errors: Vec<SequenceValidationError>,
    /// Validation warnings.
    pub warnings: Vec<SequenceValidationError>,
}

/// Validation error/warning.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/validator.md#schema
#[derive(Debug, Clone, Serialize)]
pub struct SequenceValidationError {
    /// Error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// JSON pointer path.
    pub path: String,
    /// Severity level.
    pub severity: SequenceSeverity,
}

/// Sequence diagram validator.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/validator.md#schema
pub struct SequenceValidator {
    /// Strict mode flag.
    strict: bool,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/validator.md#source
impl SequenceValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn with_error(mut self, error: SequenceValidationError) -> Self {
        self.valid = false;
        self.errors.push(error);
        self
    }

    pub fn with_warning(mut self, warning: SequenceValidationError) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/validator.md#source
impl SequenceValidator {
    pub fn new() -> Self {
        Self { strict: false }
    }

    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Validate a sequence diagram definition
    pub fn validate(&self, sequence: &SequenceDef) -> SequenceValidationResult {
        let mut result = SequenceValidationResult::ok();

        // Collect participant IDs
        let participant_ids: HashSet<String> = sequence.participants.keys().cloned().collect();

        // 1. Check minimum participants
        if sequence.participants.len() < 2 {
            result = result.with_error(SequenceValidationError {
                code: "TOO_FEW_PARTICIPANTS".to_string(),
                message: "Sequence diagram requires at least 2 participants".to_string(),
                path: "participants".to_string(),
                severity: SequenceSeverity::Error,
            });
        }

        // 2. Validate message participants
        for (idx, msg) in sequence.messages.iter().enumerate() {
            if !participant_ids.contains(&msg.from) {
                result = result.with_error(SequenceValidationError {
                    code: "INVALID_MESSAGE_FROM".to_string(),
                    message: format!("Message source '{}' not found in participants", msg.from),
                    path: format!("messages[{}].from", idx),
                    severity: SequenceSeverity::Error,
                });
            }
            if !participant_ids.contains(&msg.to) {
                result = result.with_error(SequenceValidationError {
                    code: "INVALID_MESSAGE_TO".to_string(),
                    message: format!("Message target '{}' not found in participants", msg.to),
                    path: format!("messages[{}].to", idx),
                    severity: SequenceSeverity::Error,
                });
            }
        }

        // 3. Validate loop indices
        let msg_count = sequence.messages.len();
        for (idx, loop_def) in sequence.loops.iter().enumerate() {
            if loop_def.start >= msg_count {
                result = result.with_error(SequenceValidationError {
                    code: "LOOP_START_OUT_OF_BOUNDS".to_string(),
                    message: format!(
                        "Loop start index {} exceeds message count {}",
                        loop_def.start, msg_count
                    ),
                    path: format!("loops[{}].start", idx),
                    severity: SequenceSeverity::Error,
                });
            }
            if loop_def.end >= msg_count {
                result = result.with_error(SequenceValidationError {
                    code: "LOOP_END_OUT_OF_BOUNDS".to_string(),
                    message: format!(
                        "Loop end index {} exceeds message count {}",
                        loop_def.end, msg_count
                    ),
                    path: format!("loops[{}].end", idx),
                    severity: SequenceSeverity::Error,
                });
            }
            if loop_def.start > loop_def.end {
                result = result.with_error(SequenceValidationError {
                    code: "LOOP_INVALID_RANGE".to_string(),
                    message: format!(
                        "Loop start {} is greater than end {}",
                        loop_def.start, loop_def.end
                    ),
                    path: format!("loops[{}]", idx),
                    severity: SequenceSeverity::Error,
                });
            }
        }

        // 4. Validate alt block indices
        for (idx, alt) in sequence.alts.iter().enumerate() {
            self.validate_alt_block(alt, idx, msg_count, &mut result);
        }

        // 5. Validate note participants
        for (idx, note) in sequence.notes.iter().enumerate() {
            for participant in &note.participants {
                if !participant_ids.contains(participant) {
                    result = result.with_error(SequenceValidationError {
                        code: "INVALID_NOTE_PARTICIPANT".to_string(),
                        message: format!(
                            "Note references non-existent participant '{}'",
                            participant
                        ),
                        path: format!("notes[{}].participants", idx),
                        severity: SequenceSeverity::Error,
                    });
                }
            }
            if let Some(after) = note.after_message {
                if after >= msg_count {
                    result = result.with_error(SequenceValidationError {
                        code: "NOTE_AFTER_OUT_OF_BOUNDS".to_string(),
                        message: format!(
                            "Note after_message {} exceeds message count {}",
                            after, msg_count
                        ),
                        path: format!("notes[{}].after_message", idx),
                        severity: SequenceSeverity::Error,
                    });
                }
            }
        }

        // 6. Check activation balance (warning)
        self.check_activation_balance(sequence, &mut result);

        // 7. In strict mode, promote certain warnings to errors
        if self.strict {
            let strict_codes = ["UNBALANCED_ACTIVATION"];
            let (promoted, remaining): (Vec<_>, Vec<_>) = result
                .warnings
                .into_iter()
                .partition(|w| strict_codes.contains(&w.code.as_str()));

            result.warnings = remaining;
            for mut warning in promoted {
                warning.severity = SequenceSeverity::Error;
                result.errors.push(warning);
            }
            if !result.errors.is_empty() {
                result.valid = false;
            }
        }

        result
    }

    /// Validate alt block indices
    fn validate_alt_block(
        &self,
        alt: &AltDef,
        idx: usize,
        msg_count: usize,
        result: &mut SequenceValidationResult,
    ) {
        if alt.start >= msg_count {
            *result = std::mem::take(result).with_error(SequenceValidationError {
                code: "ALT_START_OUT_OF_BOUNDS".to_string(),
                message: format!(
                    "Alt start index {} exceeds message count {}",
                    alt.start, msg_count
                ),
                path: format!("alts[{}].start", idx),
                severity: SequenceSeverity::Error,
            });
        }
        if alt.end >= msg_count {
            *result = std::mem::take(result).with_error(SequenceValidationError {
                code: "ALT_END_OUT_OF_BOUNDS".to_string(),
                message: format!(
                    "Alt end index {} exceeds message count {}",
                    alt.end, msg_count
                ),
                path: format!("alts[{}].end", idx),
                severity: SequenceSeverity::Error,
            });
        }

        for (branch_idx, branch) in alt.else_branches.iter().enumerate() {
            if branch.start >= msg_count {
                *result = std::mem::take(result).with_error(SequenceValidationError {
                    code: "ELSE_START_OUT_OF_BOUNDS".to_string(),
                    message: format!(
                        "Else branch start {} exceeds message count {}",
                        branch.start, msg_count
                    ),
                    path: format!("alts[{}].else_branches[{}].start", idx, branch_idx),
                    severity: SequenceSeverity::Error,
                });
            }
            if branch.end >= msg_count {
                *result = std::mem::take(result).with_error(SequenceValidationError {
                    code: "ELSE_END_OUT_OF_BOUNDS".to_string(),
                    message: format!(
                        "Else branch end {} exceeds message count {}",
                        branch.end, msg_count
                    ),
                    path: format!("alts[{}].else_branches[{}].end", idx, branch_idx),
                    severity: SequenceSeverity::Error,
                });
            }
        }
    }

    /// Check activation balance
    fn check_activation_balance(
        &self,
        sequence: &SequenceDef,
        result: &mut SequenceValidationResult,
    ) {
        // Track activation count per participant
        let mut activation_count: std::collections::HashMap<String, i32> =
            std::collections::HashMap::new();

        for msg in &sequence.messages {
            if msg.activate {
                *activation_count.entry(msg.to.clone()).or_default() += 1;
            }
            if msg.deactivate {
                *activation_count.entry(msg.from.clone()).or_default() -= 1;
            }
        }

        for (participant, count) in &activation_count {
            if *count != 0 {
                *result = std::mem::take(result).with_warning(SequenceValidationError {
                    code: "UNBALANCED_ACTIVATION".to_string(),
                    message: format!(
                        "Participant '{}' has unbalanced activation (net: {})",
                        participant, count
                    ),
                    path: format!("participants.{}", participant),
                    severity: SequenceSeverity::Warning,
                });
            }
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/validator.md#source
impl Default for SequenceValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse_sequence(json: serde_json::Value) -> SequenceDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_valid_sequence() {
        let sequence = parse_sequence(json!({
            "id": "test",
            "participants": {
                "a": { "label": "A" },
                "b": { "label": "B" }
            },
            "messages": [
                { "from": "a", "to": "b", "text": "Hello" },
                { "from": "b", "to": "a", "text": "Hi", "type": "dotted" }
            ]
        }));

        let result = SequenceValidator::new().validate(&sequence);
        assert!(result.valid);
    }

    #[test]
    fn test_invalid_message_participant() {
        let sequence = parse_sequence(json!({
            "id": "test",
            "participants": {
                "a": { "label": "A" },
                "b": { "label": "B" }
            },
            "messages": [
                { "from": "a", "to": "nonexistent", "text": "Hello" }
            ]
        }));

        let result = SequenceValidator::new().validate(&sequence);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.code == "INVALID_MESSAGE_TO"));
    }

    #[test]
    fn test_loop_out_of_bounds() {
        let sequence = parse_sequence(json!({
            "id": "test",
            "participants": {
                "a": { "label": "A" },
                "b": { "label": "B" }
            },
            "messages": [
                { "from": "a", "to": "b", "text": "Hello" }
            ],
            "loops": [
                { "label": "Loop", "start": 0, "end": 5 }
            ]
        }));

        let result = SequenceValidator::new().validate(&sequence);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == "LOOP_END_OUT_OF_BOUNDS"));
    }

    #[test]
    fn test_unbalanced_activation() {
        let sequence = parse_sequence(json!({
            "id": "test",
            "participants": {
                "a": { "label": "A" },
                "b": { "label": "B" }
            },
            "messages": [
                { "from": "a", "to": "b", "text": "Hello", "activate": true }
            ]
        }));

        let result = SequenceValidator::new().validate(&sequence);
        assert!(result.valid); // Warnings don't invalidate
        assert!(result
            .warnings
            .iter()
            .any(|w| w.code == "UNBALANCED_ACTIVATION"));
    }
}

// CODEGEN-END
