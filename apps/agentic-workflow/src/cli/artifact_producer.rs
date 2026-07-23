// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/interfaces/src/artifact_producer.md#source
// @spec apps/agentic-workflow/tech-design/surface/specs/aw-artifact-skeleton-fill-protocol.md#schema
// CODEGEN-BEGIN
//! Shared internal contract for CLI-owned WI, EC, and TD artifacts.
//!
//! Domain commands keep their public namespaces. This module only gives their
//! existing skeleton/fill flows one observable protocol and one shared
//! slot/schema preflight before lifecycle state may advance.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const ARTIFACT_PROTOCOL_VERSION: &str = "aw.artifact-producer.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ArtifactProducerKind {
    WorkItem,
    ExternalContract,
    TechDesign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ArtifactSlotFormat {
    MarkdownFragment,
    JsonSchema,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ArtifactCommand {
    pub command: String,
}

impl ArtifactCommand {
    pub(crate) fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ArtifactIdentity {
    pub producer: ArtifactProducerKind,
    pub id: String,
    pub artifact_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ArtifactSkeleton {
    pub path: String,
    pub initialized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ArtifactFillSlot {
    pub id: String,
    pub format: ArtifactSlotFormat,
    pub schema: String,
    pub payload_path: String,
    pub apply: ArtifactCommand,
}

impl ArtifactFillSlot {
    pub(crate) fn markdown(
        id: impl Into<String>,
        schema: impl Into<String>,
        payload_path: impl Into<String>,
        apply: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            format: ArtifactSlotFormat::MarkdownFragment,
            schema: schema.into(),
            payload_path: payload_path.into(),
            apply: ArtifactCommand::new(apply),
        }
    }

    pub(crate) fn json(
        id: impl Into<String>,
        schema: impl Into<String>,
        payload_path: impl Into<String>,
        apply: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            format: ArtifactSlotFormat::JsonSchema,
            schema: schema.into(),
            payload_path: payload_path.into(),
            apply: ArtifactCommand::new(apply),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ArtifactOwnershipOutput {
    pub marker: String,
    pub owner: ArtifactProducerKind,
    pub required_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ArtifactProducerContract {
    pub schema_version: String,
    pub identity: ArtifactIdentity,
    pub skeleton: ArtifactSkeleton,
    pub fill_slots: Vec<ArtifactFillSlot>,
    pub validation: ArtifactCommand,
    pub generation: Option<ArtifactCommand>,
    pub evidence: Vec<String>,
    pub next: ArtifactCommand,
    pub ownership_outputs: Vec<ArtifactOwnershipOutput>,
}

impl ArtifactProducerContract {
    fn new(
        producer: ArtifactProducerKind,
        id: impl Into<String>,
        artifact_path: impl Into<String>,
        initialized: bool,
        fill_slots: Vec<ArtifactFillSlot>,
        validation: impl Into<String>,
        generation: Option<String>,
        evidence: Vec<String>,
        ownership_outputs: Vec<ArtifactOwnershipOutput>,
    ) -> Result<Self> {
        let id = id.into();
        let artifact_path = artifact_path.into();
        if id.trim().is_empty() {
            bail!("artifact producer identity id must not be empty");
        }
        if artifact_path.trim().is_empty() {
            bail!("artifact producer skeleton path must not be empty");
        }
        let mut slot_ids = BTreeSet::new();
        for slot in &fill_slots {
            if slot.id.trim().is_empty()
                || slot.schema.trim().is_empty()
                || slot.payload_path.trim().is_empty()
                || slot.apply.command.trim().is_empty()
            {
                bail!("artifact producer `{id}` has an incomplete fill-slot declaration");
            }
            if !slot_ids.insert(slot.id.clone()) {
                bail!(
                    "artifact producer `{id}` declares duplicate fill slot `{}`",
                    slot.id
                );
            }
        }
        let validation = ArtifactCommand::new(validation);
        if validation.command.trim().is_empty() {
            bail!("artifact producer `{id}` validation command must not be empty");
        }
        let next = fill_slots
            .first()
            .map(|slot| slot.apply.clone())
            .unwrap_or_else(|| validation.clone());
        Ok(Self {
            schema_version: ARTIFACT_PROTOCOL_VERSION.to_string(),
            identity: ArtifactIdentity {
                producer,
                id,
                artifact_path: artifact_path.clone(),
            },
            skeleton: ArtifactSkeleton {
                path: artifact_path,
                initialized,
            },
            fill_slots,
            validation,
            generation: generation.map(ArtifactCommand::new),
            evidence,
            next,
            ownership_outputs,
        })
    }

    pub(crate) fn validate_slot_payload(&self, slot_id: &str, raw: &str) -> Result<()> {
        let Some(slot) = self.fill_slots.iter().find(|slot| slot.id == slot_id) else {
            let allowed = self
                .fill_slots
                .iter()
                .map(|slot| slot.id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(self.schema_violation(
                slot_id,
                format!("slot is not declared; allowed slots: {allowed}"),
            ));
        };
        if raw.trim().is_empty() {
            return Err(self.schema_violation(slot_id, "payload is empty"));
        }
        if slot.format == ArtifactSlotFormat::JsonSchema {
            let value: serde_json::Value = serde_json::from_str(raw).map_err(|error| {
                self.schema_violation(slot_id, format!("invalid JSON: {error}"))
            })?;
            if !value.is_object() {
                return Err(self.schema_violation(slot_id, "JSON payload root must be an object"));
            }
        }
        Ok(())
    }

    pub(crate) fn after_fill(mut self) -> Self {
        self.next = self.validation.clone();
        self
    }

    pub(crate) fn schema_violation(&self, slot_id: &str, detail: impl AsRef<str>) -> anyhow::Error {
        let slot = self.fill_slots.iter().find(|slot| slot.id == slot_id);
        let schema = slot
            .map(|slot| slot.schema.as_str())
            .unwrap_or("undeclared-slot");
        let remediation = slot
            .map(|slot| slot.apply.command.as_str())
            .unwrap_or(self.next.command.as_str());
        anyhow::anyhow!(
            "artifact slot/schema violation: producer={:?} artifact={} slot={} schema={}: {}; remediation: {}",
            self.identity.producer,
            self.identity.id,
            slot_id,
            schema,
            detail.as_ref(),
            remediation
        )
    }
}

pub(crate) fn wi_contract(
    slug: &str,
    skeleton_path: &str,
    payload_path: &str,
    section: &str,
    initialized: bool,
) -> Result<ArtifactProducerContract> {
    ArtifactProducerContract::new(
        ArtifactProducerKind::WorkItem,
        slug,
        skeleton_path,
        initialized,
        vec![ArtifactFillSlot::markdown(
            section,
            "aw.wi.structured-markdown.v1",
            payload_path,
            format!("aw wi fill-section --slug {slug} --section {section} --apply"),
        )],
        format!("aw wi validate {slug}"),
        None,
        vec![skeleton_path.to_string()],
        Vec::new(),
    )
}

pub(crate) fn ec_contract(
    project: &str,
    id: &str,
    skeleton_path: &str,
    slots: &[(String, String)],
    initialized: bool,
) -> Result<ArtifactProducerContract> {
    let slots = slots
        .iter()
        .map(|(section, payload_path)| {
            ArtifactFillSlot::json(
                section,
                format!("aw.ec.{section}.payload.v1"),
                payload_path,
                format!("aw ec fill --project {project} {skeleton_path} --section {section}"),
            )
        })
        .collect();
    ArtifactProducerContract::new(
        ArtifactProducerKind::ExternalContract,
        id,
        skeleton_path,
        initialized,
        slots,
        format!("aw ec review --project {project}"),
        Some(format!("aw ec gen --project {project} --verify")),
        vec![skeleton_path.to_string()],
        Vec::new(),
    )
}

pub(crate) fn td_contract(
    slug: &str,
    skeleton_path: &str,
    phase: &str,
    slots: &[(String, String)],
    initialized: bool,
) -> Result<ArtifactProducerContract> {
    let slots = slots
        .iter()
        .map(|(section, payload_path)| {
            ArtifactFillSlot::json(
                section,
                format!("aw.td.{section}.payload.v1"),
                payload_path,
                format!(
                    "aw td create {slug} --apply --phase {phase} --section {section} --spec-path {skeleton_path}"
                ),
            )
        })
        .collect();
    ArtifactProducerContract::new(
        ArtifactProducerKind::TechDesign,
        slug,
        skeleton_path,
        initialized,
        slots,
        format!("aw td check {skeleton_path}"),
        Some(format!("aw cb gen {slug}")),
        vec![skeleton_path.to_string()],
        vec![
            ArtifactOwnershipOutput {
                marker: "CODEGEN-BEGIN/END".to_string(),
                owner: ArtifactProducerKind::TechDesign,
                required_fields: Vec::new(),
            },
            ArtifactOwnershipOutput {
                marker: "HANDWRITE-BEGIN/END".to_string(),
                owner: ArtifactProducerKind::TechDesign,
                required_fields: vec![
                    "gap".to_string(),
                    "tracker".to_string(),
                    "reason".to_string(),
                ],
            },
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_contracts() -> Vec<ArtifactProducerContract> {
        vec![
            wi_contract(
                "1499",
                "/tmp/wi/1499.md",
                "/tmp/payloads/wi/1499/body.md",
                "all",
                true,
            )
            .unwrap(),
            ec_contract(
                "agentic-workflow",
                "artifact-contract",
                "external-contracts/behavior/artifact-contract.md",
                &[(
                    "e2e-test".to_string(),
                    "/tmp/payloads/ec/artifact-contract/e2e-test.json".to_string(),
                )],
                true,
            )
            .unwrap(),
            td_contract(
                "1499",
                "apps/agentic-workflow/tech-design/specs/1499.md",
                "contract",
                &[(
                    "logic".to_string(),
                    "/tmp/payloads/td/1499/contract/logic.json".to_string(),
                )],
                true,
            )
            .unwrap(),
        ]
    }

    #[test]
    fn wi_ec_td_share_one_observable_contract_shape() {
        for contract in fixture_contracts() {
            let value = serde_json::to_value(contract).unwrap();
            for field in [
                "schema_version",
                "identity",
                "skeleton",
                "fill_slots",
                "validation",
                "generation",
                "evidence",
                "next",
                "ownership_outputs",
            ] {
                assert!(value.get(field).is_some(), "missing {field}: {value}");
            }
            assert_eq!(value["schema_version"], ARTIFACT_PROTOCOL_VERSION);
            assert!(value["next"]["command"].as_str().is_some());
        }
    }

    #[test]
    fn contracts_round_trip_deterministically() {
        for contract in fixture_contracts() {
            let encoded = serde_json::to_string(&contract).unwrap();
            let decoded: ArtifactProducerContract = serde_json::from_str(&encoded).unwrap();
            assert_eq!(decoded, contract);
            assert_eq!(serde_json::to_string(&decoded).unwrap(), encoded);
        }
    }

    #[test]
    fn applied_contract_advances_to_domain_validation() {
        for contract in fixture_contracts() {
            let applied = contract.after_fill();
            assert_eq!(applied.next, applied.validation);
        }
    }

    #[test]
    fn wi_ec_td_commands_round_trip_through_the_live_cli_parser() {
        for contract in fixture_contracts() {
            let commands = contract
                .fill_slots
                .iter()
                .map(|slot| slot.apply.command.as_str())
                .chain(std::iter::once(contract.validation.command.as_str()))
                .chain(
                    contract
                        .generation
                        .iter()
                        .map(|command| command.command.as_str()),
                )
                .chain(std::iter::once(contract.next.command.as_str()));
            for command in commands {
                crate::cli::chain::validate_aw_command_string(command).unwrap_or_else(|error| {
                    panic!("invalid artifact command `{command}`: {error}")
                });
            }
        }
    }

    #[test]
    fn invalid_slot_and_schema_name_one_runnable_remediation() {
        let contract = ec_contract(
            "agentic-workflow",
            "artifact-contract",
            "external-contracts/behavior/artifact-contract.md",
            &[(
                "e2e-test".to_string(),
                "/tmp/payloads/ec/artifact-contract/e2e-test.json".to_string(),
            )],
            true,
        )
        .unwrap();
        for error in [
            contract.validate_slot_payload("unknown", "{}").unwrap_err(),
            contract
                .validate_slot_payload("e2e-test", "not-json")
                .unwrap_err(),
        ] {
            let message = error.to_string();
            assert!(message.contains("artifact slot/schema violation"));
            assert_eq!(message.matches("remediation:").count(), 1);
            assert!(message.contains("aw ec fill --project agentic-workflow"));
        }
    }

    #[test]
    fn td_contract_owns_codegen_and_handwrite_outputs() {
        let td = fixture_contracts().pop().unwrap();
        assert_eq!(td.ownership_outputs.len(), 2);
        assert_eq!(td.ownership_outputs[0].marker, "CODEGEN-BEGIN/END");
        assert_eq!(td.ownership_outputs[1].marker, "HANDWRITE-BEGIN/END");
        assert_eq!(
            td.ownership_outputs[1].required_fields,
            ["gap", "tracker", "reason"]
        );
    }
}
// CODEGEN-END
