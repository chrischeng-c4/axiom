// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#source
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    FrontendPage,
    RedesignStandardization,
    Documentation,
    CliSurface,
    ApiSurface,
    CodeArtifact,
    TestArtifact,
    Other,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#logic
pub fn infer_artifact_kind_from_hint(hint: &str) -> ArtifactKind {
    let lower = hint.to_ascii_lowercase();
    if contains_any(
        &lower,
        &[
            "standardize audit",
            "audit-first",
            "preservation audit",
            "redesign standardization",
        ],
    ) {
        return ArtifactKind::RedesignStandardization;
    }
    if contains_any(
        &lower,
        &[
            "frontend",
            "ui/ux",
            "src/ui",
            "/ui/",
            "ui/",
            "web/",
            "pages/",
            "components/",
            ".tsx",
            ".jsx",
            ".vue",
            ".svelte",
            ".css",
            "wireframe",
            "design-token",
            "viewport",
            "screenshot",
        ],
    ) {
        return ArtifactKind::FrontendPage;
    }
    if contains_any(
        &lower,
        &[
            "aw health",
            "--verify-tests",
            "e2e-test",
            "unit-test",
            "test artifact",
            "tests/",
            "/tests/",
            "fixture",
        ],
    ) {
        return ArtifactKind::TestArtifact;
    }
    if contains_any(
        &lower,
        &[
            "src/cli",
            "/cli/",
            "cli-surface",
            "cli surface",
            "cli section",
            "help/output",
            "command contract",
            "stdout",
            "json envelope",
        ],
    ) {
        return ArtifactKind::CliSurface;
    }
    if contains_any(
        &lower,
        &[
            "api surface",
            "rest-api",
            "rpc-api",
            "async-api",
            "openapi",
            "routes/",
            "/routes/",
            "server/",
            "schema contract",
        ],
    ) {
        return ArtifactKind::ApiSurface;
    }
    if contains_any(
        &lower,
        &[
            "readme",
            "capability",
            "documentation",
            "docs/",
            ".md",
            "markdown",
            "epicize",
            "atomize",
            "prioritize",
            "wi plan",
        ],
    ) {
        return ArtifactKind::Documentation;
    }
    if contains_any(
        &lower,
        &[
            "standardize managed",
            "standardize semantic",
            "standardize traceability",
            "generator",
        ],
    ) {
        return ArtifactKind::RedesignStandardization;
    }
    ArtifactKind::CodeArtifact
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactQualityProfile {
    pub artifact_kind: ArtifactKind,
    pub intent_read: String,
    pub audience: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<String>,
    pub quality_dials: Vec<QualityDial>,
    pub source_policy: ArtifactSourcePolicy,
    pub preflight_gate_set: PreflightGateSet,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityDial {
    pub key: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSourcePolicy {
    pub mode: ArtifactSourceMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freshness: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactSourceMode {
    Spec,
    ScreenshotReference,
    CliTranscript,
    ApiContract,
    ValidationInventory,
    CodeOwnershipMap,
    Mixed,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightGateSet {
    pub id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gates: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#logic
impl ArtifactQualityProfile {
    /// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#logic
    pub fn default_for_kind(artifact_kind: ArtifactKind) -> Self {
        match artifact_kind {
            ArtifactKind::FrontendPage => profile(
                artifact_kind,
                "Render a usable page that matches the brief, audience, and visible constraints.",
                "end users and product reviewers",
                &[
                    "respect existing UX conventions",
                    "avoid layout overlap at supported viewport sizes",
                    "prove desktop and mobile user paths before completion",
                ],
                &[
                    dial("visual_fit", "brief-matched", "page choices must serve the stated artifact goal"),
                    dial("accessibility", "keyboard-and-contrast-aware", "frontend output needs basic usability proof"),
                    dial("responsive_layout", "required", "viewport behavior is part of the artifact contract"),
                    dial("interaction_path", "smoke-tested", "human product journeys need runnable evidence"),
                ],
                ArtifactSourceMode::ScreenshotReference,
                Some("design brief, screenshots, or live UI reference"),
                "frontend-page-preflight",
                &["desktop+mobile viewport proof", "interaction smoke", "accessibility/readability smoke", "placeholder-free primary state"],
            ),
            ArtifactKind::RedesignStandardization => profile(
                artifact_kind,
                "Standardize an existing artifact without changing unrelated behavior.",
                "maintainers reviewing ownership and regeneration safety",
                &["preserve observed behavior", "separate generator gaps from handwrite ownership"],
                &[
                    dial("behavior_preservation", "strict", "standardization must not become a redesign"),
                    dial("ownership_clarity", "required", "CODEGEN/HANDWRITE boundaries must stay inspectable"),
                    dial("regeneration_readiness", "evidence-backed", "claims need cb/cold verification evidence"),
                ],
                ArtifactSourceMode::CodeOwnershipMap,
                Some("managed coverage, semantic TDs, and code ownership map"),
                "standardization-preflight",
                &["managed coverage check", "semantic coverage check", "cb verify"],
            ),
            ArtifactKind::Documentation => profile(
                artifact_kind,
                "Produce documentation that explains the contract without adding product-only filler.",
                "operators, implementers, and reviewers",
                &["keep docs aligned with the canonical spec", "avoid unverifiable claims"],
                &[
                    dial("contract_precision", "high", "docs must be actionable by implementation agents"),
                    dial("reader_path", "scannable", "operators need fast command and state lookup"),
                    dial("evidence_links", "required", "workflow docs need source references"),
                ],
                ArtifactSourceMode::Spec,
                Some("canonical spec and README capability map"),
                "documentation-preflight",
                &["spec alignment check", "link/path check"],
            ),
            ArtifactKind::CliSurface => profile(
                artifact_kind,
                "Expose CLI behavior that is deterministic, scriptable, and backward compatible.",
                "agents and operators invoking AW commands",
                &["stdout is the live protocol", "do not require new flags for existing flows"],
                &[
                    dial("machine_readability", "strict", "JSON envelopes are consumed by agents"),
                    dial("compatibility", "backward-compatible", "existing CLI consumers must continue to parse output"),
                    dial("error_actionability", "high", "CLI failures should emit next commands or concrete causes"),
                ],
                ArtifactSourceMode::CliTranscript,
                Some("CLI transcript or command contract"),
                "cli-surface-preflight",
                &["json roundtrip", "help/output smoke"],
            ),
            ArtifactKind::ApiSurface => profile(
                artifact_kind,
                "Preserve API contracts while making intent, compatibility, and validation explicit.",
                "client implementers and service maintainers",
                &["schema changes need compatibility notes", "typed contracts beat prose"],
                &[
                    dial("schema_stability", "high", "API clients depend on field-level compatibility"),
                    dial("validation_clarity", "required", "invalid states need structured rejection paths"),
                    dial("migration_scope", "bounded", "API work must state old/new behavior"),
                ],
                ArtifactSourceMode::ApiContract,
                Some("OpenAPI, RPC schema, or typed interface spec"),
                "api-surface-preflight",
                &["schema validation", "compatibility review"],
            ),
            ArtifactKind::CodeArtifact => profile(
                artifact_kind,
                "Implement the spec with bounded source changes and reviewable ownership markers.",
                "maintainers and lifecycle reviewers",
                &["keep edits scoped to the TD changes list", "include deterministic tests for behavior"],
                &[
                    dial("scope_control", "strict", "code artifacts should not absorb unrelated refactors"),
                    dial("testability", "required", "behavior needs local verification"),
                    dial("ownership_markers", "required", "source ownership is part of AW readiness"),
                ],
                ArtifactSourceMode::Spec,
                Some("TD contract and changes section"),
                "code-artifact-preflight",
                &["cargo fmt", "targeted unit tests", "diff ownership check"],
            ),
            ArtifactKind::TestArtifact => profile(
                artifact_kind,
                "Create tests that prove the declared behavior and fail on meaningful regressions.",
                "maintainers and release gates",
                &["tests must avoid fixture-only assertions", "skip only unavailable external services"],
                &[
                    dial("regression_signal", "high", "tests should fail on behavior drift"),
                    dial("fixture_realism", "representative", "fixtures must encode the profile contract"),
                    dial("gate_cost", "bounded", "tests should stay usable in lifecycle gates"),
                ],
                ArtifactSourceMode::ValidationInventory,
                Some("unit-test/e2e-test TD sections and fixture inventory"),
                "test-artifact-preflight",
                &["targeted test run", "fixture roundtrip"],
            ),
            ArtifactKind::Other => Self::neutral_default(),
        }
    }

    /// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#logic
    pub fn neutral_default() -> Self {
        profile(
            ArtifactKind::Other,
            "Produce the artifact according to its governing workflow brief.",
            "workflow reviewers",
            &["preserve backward compatibility when the artifact kind is not known"],
            &[dial(
                "intent_alignment",
                "required",
                "unknown artifact kinds still need an explicit review target",
            )],
            ArtifactSourceMode::Mixed,
            None,
            "artifact-quality-default",
            &["profile present or neutral default"],
        )
    }

    /// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#schema
    pub fn validate(&self) -> Result<(), String> {
        if self.intent_read.trim().is_empty() {
            return Err("intent_read must not be empty".to_string());
        }
        if self.audience.trim().is_empty() {
            return Err("audience must not be empty".to_string());
        }
        if self.quality_dials.is_empty() {
            return Err("quality_dials must not be empty".to_string());
        }
        if self.preflight_gate_set.id.trim().is_empty() {
            return Err("preflight_gate_set.id must not be empty".to_string());
        }
        Ok(())
    }

    /// @spec projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#logic
    pub fn to_review_prompt_context(&self) -> String {
        let mut out = String::new();
        out.push_str("Artifact Quality Profile\n");
        out.push_str(&format!("kind: {:?}\n", self.artifact_kind));
        out.push_str(&format!("intent_read: {}\n", self.intent_read));
        out.push_str(&format!("audience: {}\n", self.audience));
        if !self.constraints.is_empty() {
            out.push_str("constraints:\n");
            for constraint in &self.constraints {
                out.push_str(&format!("- {}\n", constraint));
            }
        }
        out.push_str("quality_dials:\n");
        for dial in &self.quality_dials {
            out.push_str(&format!("- {}={}", dial.key, dial.value));
            if let Some(rationale) = &dial.rationale {
                out.push_str(&format!(" ({})", rationale));
            }
            out.push('\n');
        }
        out.push_str(&format!("source_policy: {:?}", self.source_policy.mode));
        if let Some(evidence_ref) = &self.source_policy.evidence_ref {
            out.push_str(&format!(" evidence_ref={}", evidence_ref));
        }
        out.push('\n');
        out.push_str(&format!(
            "preflight_gate_set: {} [{}]\n",
            self.preflight_gate_set.id,
            self.preflight_gate_set.gates.join(", ")
        ));
        out
    }
}

fn profile(
    artifact_kind: ArtifactKind,
    intent_read: &str,
    audience: &str,
    constraints: &[&str],
    quality_dials: &[QualityDial],
    source_mode: ArtifactSourceMode,
    evidence_ref: Option<&str>,
    gate_id: &str,
    gates: &[&str],
) -> ArtifactQualityProfile {
    ArtifactQualityProfile {
        artifact_kind,
        intent_read: intent_read.to_string(),
        audience: audience.to_string(),
        constraints: constraints.iter().map(|value| value.to_string()).collect(),
        quality_dials: quality_dials.to_vec(),
        source_policy: ArtifactSourcePolicy {
            mode: source_mode,
            evidence_ref: evidence_ref.map(str::to_string),
            freshness: None,
        },
        preflight_gate_set: PreflightGateSet {
            id: gate_id.to_string(),
            gates: gates.iter().map(|value| value.to_string()).collect(),
        },
    }
}

fn dial(key: &str, value: &str, rationale: &str) -> QualityDial {
    QualityDial {
        key: key.to_string(),
        value: value.to_string(),
        rationale: Some(rationale.to_string()),
    }
}
// CODEGEN-END
