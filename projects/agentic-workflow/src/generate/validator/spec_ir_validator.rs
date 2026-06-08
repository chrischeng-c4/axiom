// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/validator/spec_ir_validator.md#source
// CODEGEN-BEGIN
//! SpecIR Validator — section-type validators for new spec kinds.
//!
//! Extends the generate validator subsystem with a pluggable registration
//! mechanism ([`SpecIRValidator`] trait) and concrete validators for the four
//! new section types introduced in the `sdd-codegen-and-fixes` change:
//!
//! | Validator | SpecIR variant | Section type |
//! |-----------|----------------|--------------|
//! | [`DeployValidator`] | `SpecIR::Deploy` | `deploy` |
//! | [`WireframeValidator`] | `SpecIR::Wireframe` | `wireframe` |
//! | [`ComponentValidator`] | `SpecIR::Component` | `component` |
//! | [`DesignTokenValidator`] | `SpecIR::DesignToken` | `design-token` |
//!
//! Cross-section reference validation (deploy ↔ db-model, deploy ↔ rest-api)
//! is deferred to a later iteration and emits soft **warnings** only, never
//! blocking errors.

use crate::generate::spec_ir::{ComponentSpec, DeploySpec, DesignTokenSpec, SpecIR, WireframeSpec};

use super::completeness::{ValidationIssue, ValidationResult};

// ---------------------------------------------------------------------------
// SpecIRValidator trait — shared registration mechanism (Q5)
// ---------------------------------------------------------------------------

/// Pluggable validator for a specific [`SpecIR`] variant.
///
/// Implement this trait to register a new validator for any SpecIR section
/// type.  The top-level [`validate_spec_ir`] function dispatches to all
/// registered validators in order, collecting every issue into a single
/// [`ValidationResult`].
///
/// ## Registration
///
/// Add your validator to the `VALIDATORS` static slice inside
/// [`validate_spec_ir`].
/// @spec projects/agentic-workflow/tech-design/core/generate/validator/spec_ir_validator.md#source
pub trait SpecIRValidator: Send + Sync {
    /// Return `true` if this validator handles the given [`SpecIR`] variant.
    fn can_validate(&self, spec: &SpecIR) -> bool;

    /// Validate `spec` and append any issues to `result`.
    ///
    /// Cross-section reference validation **must** emit only [`Severity::Warning`]
    /// (soft warning mode, Q2) — never [`Severity::Error`].
    fn validate(&self, spec: &SpecIR, result: &mut ValidationResult);
}

// ---------------------------------------------------------------------------
// Top-level dispatch function
// ---------------------------------------------------------------------------

/// Validate any [`SpecIR`] variant by dispatching to all registered
/// [`SpecIRValidator`] implementations.
///
/// Returns a [`ValidationResult`] aggregating issues from every matching
/// validator.  Callers can check [`ValidationResult::is_valid`] to determine
/// whether generation should proceed (errors block, warnings do not).
///
/// ## Cross-section reference validation
///
/// Cross-ref validation (e.g. deploy ↔ db-model, deploy ↔ rest-api) is
/// deferred to a later iteration.  Any such check **must** be emitted as a
/// soft [`Severity::Warning`] only.
/// @spec projects/agentic-workflow/tech-design/core/generate/validator/spec_ir_validator.md#source
pub fn validate_spec_ir(spec: &SpecIR) -> ValidationResult {
    static VALIDATORS: &[&dyn SpecIRValidator] = &[
        &DeployValidator,
        &WireframeValidator,
        &ComponentValidator,
        &DesignTokenValidator,
    ];

    let mut result = ValidationResult::new();
    for v in VALIDATORS {
        if v.can_validate(spec) {
            v.validate(spec, &mut result);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// DeployValidator
// ---------------------------------------------------------------------------

/// Internal-consistency validator for SpecIR Deploy.
/// @spec projects/agentic-workflow/tech-design/core/generate/validator/spec_ir_validator.md#schema
pub struct DeployValidator;

/// Internal-consistency validator for SpecIR Wireframe.
/// @spec projects/agentic-workflow/tech-design/core/generate/validator/spec_ir_validator.md#schema
pub struct WireframeValidator;

/// Internal-consistency validator for SpecIR Component.
/// @spec projects/agentic-workflow/tech-design/core/generate/validator/spec_ir_validator.md#schema
pub struct ComponentValidator;

/// Internal-consistency validator for SpecIR DesignToken.
/// @spec projects/agentic-workflow/tech-design/core/generate/validator/spec_ir_validator.md#schema
pub struct DesignTokenValidator;
/// @spec projects/agentic-workflow/tech-design/core/generate/validator/spec_ir_validator.md#source
impl SpecIRValidator for DeployValidator {
    fn can_validate(&self, spec: &SpecIR) -> bool {
        matches!(spec, SpecIR::Deploy { .. })
    }

    fn validate(&self, spec: &SpecIR, result: &mut ValidationResult) {
        let deploy = match spec {
            SpecIR::Deploy { spec, .. } => spec,
            _ => return,
        };
        validate_deploy(deploy, result);
    }
}

fn validate_deploy(spec: &DeploySpec, result: &mut ValidationResult) {
    // R1-equivalent: name must not be empty
    if spec.name.trim().is_empty() {
        result.add(ValidationIssue::error(
            "deploy.name",
            "Deploy spec must have a non-empty name",
        ));
    }

    // R1-equivalent: image must not be empty
    if spec.image.trim().is_empty() {
        result.add(ValidationIssue::error(
            "deploy.image",
            "Deploy spec must have a non-empty image reference",
        ));
    }

    // Replicas should be at least 1 — warn rather than error (soft)
    if spec.replicas == 0 {
        result.add(ValidationIssue::warning(
            "deploy.replicas",
            "Deploy spec has 0 replicas; the deployment will not run any pods",
        ));
    }

    // Port must be non-zero (0 is the unset default for some parsers)
    if spec.port == 0 {
        result.add(ValidationIssue::error(
            "deploy.port",
            "Deploy spec has port 0; specify a valid container port (1–65535)",
        ));
    }

    // Env-var consistency: each entry must have name and at least one value source
    for (i, env) in spec.env.iter().enumerate() {
        if env.name.trim().is_empty() {
            result.add(ValidationIssue::error(
                format!("deploy.env[{i}].name"),
                "Environment variable must have a non-empty name",
            ));
        }
        if env.value.is_none() && env.value_from.is_none() {
            result.add(ValidationIssue::warning(
                format!("deploy.env[{i}]"),
                format!(
                    "Environment variable '{}' has no value or valueFrom source",
                    env.name
                ),
            ));
        }
    }

    // Cross-section reference validation (db-model, rest-api) is deferred.
    // Any future cross-ref checks must only produce Severity::Warning (soft mode, Q2).
}

// ---------------------------------------------------------------------------
// WireframeValidator
// ---------------------------------------------------------------------------

/// @spec projects/agentic-workflow/tech-design/core/generate/validator/spec_ir_validator.md#source
impl SpecIRValidator for WireframeValidator {
    fn can_validate(&self, spec: &SpecIR) -> bool {
        matches!(spec, SpecIR::Wireframe { .. })
    }

    fn validate(&self, spec: &SpecIR, result: &mut ValidationResult) {
        let wireframe = match spec {
            SpecIR::Wireframe { spec, .. } => spec,
            _ => return,
        };
        validate_wireframe(wireframe, result);
    }
}

fn validate_wireframe(spec: &WireframeSpec, result: &mut ValidationResult) {
    // Component name must not be empty
    if spec.name.trim().is_empty() {
        result.add(ValidationIssue::error(
            "wireframe.name",
            "Wireframe spec must have a non-empty component name",
        ));
    }

    // component_type should be provided — warn if missing
    if spec.component_type.trim().is_empty() {
        result.add(ValidationIssue::warning(
            "wireframe.component_type",
            "Wireframe spec has no component_type; expected 'page', 'layout', 'card', 'form', etc.",
        ));
    }

    // Validate each prop definition
    for (i, prop) in spec.props.iter().enumerate() {
        if prop.name.trim().is_empty() {
            result.add(ValidationIssue::error(
                format!("wireframe.props[{i}].name"),
                "Prop definition must have a non-empty name",
            ));
        }
        if prop.prop_type.trim().is_empty() {
            result.add(ValidationIssue::warning(
                format!("wireframe.props[{i}].prop_type"),
                format!(
                    "Prop '{}' has no TypeScript type; code generation may produce 'any'",
                    prop.name
                ),
            ));
        }
    }

    // Validate top-level layout nodes
    for (i, node) in spec.layout.iter().enumerate() {
        validate_wireframe_node(node, &format!("wireframe.layout[{i}]"), result);
    }
}

fn validate_wireframe_node(
    node: &crate::generate::spec_ir::WireframeNode,
    path: &str,
    result: &mut ValidationResult,
) {
    if node.kind.trim().is_empty() {
        result.add(ValidationIssue::error(
            path,
            "Wireframe layout node must have a non-empty kind",
        ));
    }
    for (i, child) in node.children.iter().enumerate() {
        validate_wireframe_node(child, &format!("{path}.children[{i}]"), result);
    }
}

// ---------------------------------------------------------------------------
// ComponentValidator
// ---------------------------------------------------------------------------

/// @spec projects/agentic-workflow/tech-design/core/generate/validator/spec_ir_validator.md#source
impl SpecIRValidator for ComponentValidator {
    fn can_validate(&self, spec: &SpecIR) -> bool {
        matches!(spec, SpecIR::Component { .. })
    }

    fn validate(&self, spec: &SpecIR, result: &mut ValidationResult) {
        let component = match spec {
            SpecIR::Component { spec, .. } => spec,
            _ => return,
        };
        validate_component(component, result);
    }
}

fn validate_component(spec: &ComponentSpec, result: &mut ValidationResult) {
    // tag_name must be non-empty
    if spec.tag_name.trim().is_empty() {
        result.add(ValidationIssue::error(
            "component.tag_name",
            "Component spec must have a non-empty tag_name",
        ));
    } else if !is_valid_kebab_case(&spec.tag_name) {
        // Custom element tag names must be lower-kebab-case and contain a hyphen
        result.add(ValidationIssue::error(
            "component.tag_name",
            format!(
                "Component tag_name '{}' is not valid kebab-case with a hyphen \
                 (e.g. 'my-button')",
                spec.tag_name
            ),
        ));
    }

    // summary is encouraged — warn if absent
    if spec.summary.trim().is_empty() {
        result.add(ValidationIssue::warning(
            "component.summary",
            "Component spec has no summary; generated JSDoc will be empty",
        ));
    }

    // Validate attribute definitions
    for (i, attr) in spec.attributes.iter().enumerate() {
        if attr.name.trim().is_empty() {
            result.add(ValidationIssue::error(
                format!("component.attributes[{i}].name"),
                "Attribute definition must have a non-empty name",
            ));
        }
        if attr.attr_type.trim().is_empty() {
            result.add(ValidationIssue::warning(
                format!("component.attributes[{i}].type"),
                format!(
                    "Attribute '{}' has no TypeScript type; generated interface will use 'any'",
                    attr.name
                ),
            ));
        }
    }

    // Validate event definitions
    for (i, event) in spec.events.iter().enumerate() {
        if event.name.trim().is_empty() {
            result.add(ValidationIssue::error(
                format!("component.events[{i}].name"),
                "Event definition must have a non-empty name",
            ));
        }
    }
}

/// Returns `true` if `s` is non-empty, all lower-case ASCII + digits + hyphens,
/// and contains at least one hyphen (required for custom element names).
fn is_valid_kebab_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let all_valid_chars = s
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    let has_hyphen = s.contains('-');
    let no_leading_trailing_hyphen = !s.starts_with('-') && !s.ends_with('-');
    let no_consecutive_hyphens = !s.contains("--");
    all_valid_chars && has_hyphen && no_leading_trailing_hyphen && no_consecutive_hyphens
}

// ---------------------------------------------------------------------------
// DesignTokenValidator
// ---------------------------------------------------------------------------

/// Valid DTCG 2025.10 token types.
///
/// See <https://tr.designtokens.org/format/#types>
const VALID_DTCG_TYPES: &[&str] = &[
    "color",
    "dimension",
    "fontFamily",
    "fontWeight",
    "duration",
    "cubicBezier",
    "number",
    "strokeStyle",
    "border",
    "transition",
    "shadow",
    "gradient",
    "typography",
];

/// @spec projects/agentic-workflow/tech-design/core/generate/validator/spec_ir_validator.md#source
impl SpecIRValidator for DesignTokenValidator {
    fn can_validate(&self, spec: &SpecIR) -> bool {
        matches!(spec, SpecIR::DesignToken { .. })
    }

    fn validate(&self, spec: &SpecIR, result: &mut ValidationResult) {
        let design_token = match spec {
            SpecIR::DesignToken { spec, .. } => spec,
            _ => return,
        };
        validate_design_token(design_token, result);
    }
}

fn validate_design_token(spec: &DesignTokenSpec, result: &mut ValidationResult) {
    // name is a CSS prefix — warn if empty
    if spec.name.trim().is_empty() {
        result.add(ValidationIssue::warning(
            "design-token.name",
            "DesignToken spec has no name; generated CSS custom properties will have no prefix",
        ));
    }

    for (i, token) in spec.tokens.iter().enumerate() {
        // path must not be empty
        if token.path.trim().is_empty() {
            result.add(ValidationIssue::error(
                format!("design-token.tokens[{i}].path"),
                "Token entry must have a non-empty path",
            ));
        }

        // value must not be empty
        if token.value.trim().is_empty() {
            result.add(ValidationIssue::error(
                format!("design-token.tokens[{i}].value"),
                format!("Token '{}' must have a non-empty value", token.path),
            ));
        }

        // token_type must be a recognised DTCG type — warn if unknown
        if !token.token_type.is_empty() && !VALID_DTCG_TYPES.contains(&token.token_type.as_str()) {
            result.add(ValidationIssue::warning(
                format!("design-token.tokens[{i}].token_type"),
                format!(
                    "Token '{}' has unrecognised DTCG type '{}'; expected one of: {}",
                    token.path,
                    token.token_type,
                    VALID_DTCG_TYPES.join(", ")
                ),
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::spec_ir::{
        AttributeDef, ComponentSpec, DeploySpec, DesignTokenEntry, DesignTokenSpec, EnvVar,
        EventDef, PropDef, SpecIR, SpecMetadata, WireframeNode, WireframeSpec,
    };

    // --- helper builders ---

    fn deploy_ir(spec: DeploySpec) -> SpecIR {
        SpecIR::Deploy {
            spec,
            metadata: SpecMetadata::default(),
        }
    }

    fn wireframe_ir(spec: WireframeSpec) -> SpecIR {
        SpecIR::Wireframe {
            spec,
            metadata: SpecMetadata::default(),
        }
    }

    fn component_ir(spec: ComponentSpec) -> SpecIR {
        SpecIR::Component {
            spec,
            metadata: SpecMetadata::default(),
        }
    }

    fn design_token_ir(spec: DesignTokenSpec) -> SpecIR {
        SpecIR::DesignToken {
            spec,
            metadata: SpecMetadata::default(),
        }
    }

    // -----------------------------------------------------------------------
    // DeployValidator
    // -----------------------------------------------------------------------

    #[test]
    fn deploy_valid_spec_passes() {
        let ir = deploy_ir(DeploySpec {
            name: "api".into(),
            image: "api:latest".into(),
            port: 8080,
            replicas: 2,
            env: vec![],
            resources: None,
        });
        let result = validate_spec_ir(&ir);
        assert!(result.is_valid(), "errors: {:?}", result.issues);
    }

    #[test]
    fn deploy_empty_name_is_error() {
        let ir = deploy_ir(DeploySpec {
            name: "".into(),
            image: "api:latest".into(),
            port: 8080,
            replicas: 1,
            env: vec![],
            resources: None,
        });
        let result = validate_spec_ir(&ir);
        assert!(!result.is_valid());
        assert!(result.errors().any(|e| e.path.contains("name")));
    }

    #[test]
    fn deploy_empty_image_is_error() {
        let ir = deploy_ir(DeploySpec {
            name: "api".into(),
            image: "".into(),
            port: 8080,
            replicas: 1,
            env: vec![],
            resources: None,
        });
        let result = validate_spec_ir(&ir);
        assert!(!result.is_valid());
        assert!(result.errors().any(|e| e.path.contains("image")));
    }

    #[test]
    fn deploy_zero_replicas_is_warning_not_error() {
        let ir = deploy_ir(DeploySpec {
            name: "api".into(),
            image: "api:latest".into(),
            port: 8080,
            replicas: 0,
            env: vec![],
            resources: None,
        });
        let result = validate_spec_ir(&ir);
        // Still valid (warnings do not block)
        assert!(result.is_valid());
        assert_eq!(result.warning_count(), 1);
        assert!(result.warnings().any(|w| w.path.contains("replicas")));
    }

    #[test]
    fn deploy_zero_port_is_error() {
        let ir = deploy_ir(DeploySpec {
            name: "api".into(),
            image: "api:latest".into(),
            port: 0,
            replicas: 1,
            env: vec![],
            resources: None,
        });
        let result = validate_spec_ir(&ir);
        assert!(!result.is_valid());
        assert!(result.errors().any(|e| e.path.contains("port")));
    }

    #[test]
    fn deploy_env_no_value_is_warning() {
        let ir = deploy_ir(DeploySpec {
            name: "api".into(),
            image: "api:latest".into(),
            port: 8080,
            replicas: 1,
            env: vec![EnvVar {
                name: "SECRET".into(),
                value: None,
                value_from: None,
            }],
            resources: None,
        });
        let result = validate_spec_ir(&ir);
        assert!(result.is_valid()); // warning only
        assert_eq!(result.warning_count(), 1);
    }

    #[test]
    fn deploy_env_empty_name_is_error() {
        let ir = deploy_ir(DeploySpec {
            name: "api".into(),
            image: "api:latest".into(),
            port: 8080,
            replicas: 1,
            env: vec![EnvVar {
                name: "".into(),
                value: Some("x".into()),
                value_from: None,
            }],
            resources: None,
        });
        let result = validate_spec_ir(&ir);
        assert!(!result.is_valid());
        assert!(result.errors().any(|e| e.path.contains("env[0].name")));
    }

    // -----------------------------------------------------------------------
    // WireframeValidator
    // -----------------------------------------------------------------------

    #[test]
    fn wireframe_valid_spec_passes() {
        let ir = wireframe_ir(WireframeSpec {
            name: "UserCard".into(),
            component_type: "card".into(),
            props: vec![],
            layout: vec![WireframeNode {
                kind: "container".into(),
                label: None,
                children: vec![],
            }],
        });
        let result = validate_spec_ir(&ir);
        assert!(result.is_valid(), "errors: {:?}", result.issues);
    }

    #[test]
    fn wireframe_empty_name_is_error() {
        let ir = wireframe_ir(WireframeSpec {
            name: "".into(),
            component_type: "card".into(),
            props: vec![],
            layout: vec![],
        });
        let result = validate_spec_ir(&ir);
        assert!(!result.is_valid());
        assert!(result.errors().any(|e| e.path.contains("wireframe.name")));
    }

    #[test]
    fn wireframe_missing_component_type_is_warning() {
        let ir = wireframe_ir(WireframeSpec {
            name: "MyComp".into(),
            component_type: "".into(),
            props: vec![],
            layout: vec![],
        });
        let result = validate_spec_ir(&ir);
        assert!(result.is_valid());
        assert!(result.warnings().any(|w| w.path.contains("component_type")));
    }

    #[test]
    fn wireframe_prop_missing_type_is_warning() {
        let ir = wireframe_ir(WireframeSpec {
            name: "MyComp".into(),
            component_type: "card".into(),
            props: vec![PropDef {
                name: "title".into(),
                prop_type: "".into(),
                required: true,
                default_value: None,
                description: None,
            }],
            layout: vec![],
        });
        let result = validate_spec_ir(&ir);
        assert!(result.is_valid());
        assert!(result
            .warnings()
            .any(|w| w.path.contains("props[0].prop_type")));
    }

    #[test]
    fn wireframe_node_empty_kind_is_error() {
        let ir = wireframe_ir(WireframeSpec {
            name: "MyComp".into(),
            component_type: "card".into(),
            props: vec![],
            layout: vec![WireframeNode {
                kind: "".into(),
                label: None,
                children: vec![],
            }],
        });
        let result = validate_spec_ir(&ir);
        assert!(!result.is_valid());
    }

    // -----------------------------------------------------------------------
    // ComponentValidator
    // -----------------------------------------------------------------------

    #[test]
    fn component_valid_spec_passes() {
        let ir = component_ir(ComponentSpec {
            tag_name: "my-button".into(),
            summary: "A reusable button".into(),
            attributes: vec![],
            slots: vec![],
            events: vec![],
        });
        let result = validate_spec_ir(&ir);
        assert!(result.is_valid(), "errors: {:?}", result.issues);
    }

    #[test]
    fn component_empty_tag_name_is_error() {
        let ir = component_ir(ComponentSpec {
            tag_name: "".into(),
            summary: "desc".into(),
            ..Default::default()
        });
        let result = validate_spec_ir(&ir);
        assert!(!result.is_valid());
        assert!(result.errors().any(|e| e.path.contains("tag_name")));
    }

    #[test]
    fn component_invalid_kebab_case_is_error() {
        // Missing hyphen
        let ir = component_ir(ComponentSpec {
            tag_name: "mybutton".into(),
            summary: "desc".into(),
            ..Default::default()
        });
        let result = validate_spec_ir(&ir);
        assert!(!result.is_valid());
        assert!(result.errors().any(|e| e.message.contains("kebab-case")));
    }

    #[test]
    fn component_kebab_case_with_hyphen_passes() {
        let cases = ["my-button", "cc-input-field", "x-y"];
        for case in cases {
            assert!(is_valid_kebab_case(case), "should be valid: {case}");
        }
    }

    #[test]
    fn component_invalid_kebab_cases_fail() {
        let cases = [
            "",
            "MyButton",
            "-starts-with-hyphen",
            "ends-with-",
            "a--b",
            "nohyphen",
        ];
        for case in cases {
            assert!(!is_valid_kebab_case(case), "should be invalid: {case}");
        }
    }

    #[test]
    fn component_attribute_no_type_is_warning() {
        let ir = component_ir(ComponentSpec {
            tag_name: "my-input".into(),
            summary: "An input".into(),
            attributes: vec![AttributeDef {
                name: "value".into(),
                attr_type: "".into(),
                required: false,
                description: None,
            }],
            ..Default::default()
        });
        let result = validate_spec_ir(&ir);
        assert!(result.is_valid());
        assert!(result
            .warnings()
            .any(|w| w.path.contains("attributes[0].type")));
    }

    #[test]
    fn component_event_empty_name_is_error() {
        let ir = component_ir(ComponentSpec {
            tag_name: "my-input".into(),
            summary: "An input".into(),
            events: vec![EventDef {
                name: "".into(),
                detail_type: None,
                description: None,
            }],
            ..Default::default()
        });
        let result = validate_spec_ir(&ir);
        assert!(!result.is_valid());
        assert!(result.errors().any(|e| e.path.contains("events[0].name")));
    }

    #[test]
    fn component_missing_summary_is_warning() {
        let ir = component_ir(ComponentSpec {
            tag_name: "my-card".into(),
            summary: "".into(),
            ..Default::default()
        });
        let result = validate_spec_ir(&ir);
        assert!(result.is_valid());
        assert!(result.warnings().any(|w| w.path.contains("summary")));
    }

    // -----------------------------------------------------------------------
    // DesignTokenValidator
    // -----------------------------------------------------------------------

    #[test]
    fn design_token_valid_spec_passes() {
        let ir = design_token_ir(DesignTokenSpec {
            name: "theme".into(),
            tokens: vec![DesignTokenEntry {
                path: "color.primary.500".into(),
                value: "#3B82F6".into(),
                token_type: "color".into(),
                description: Some("Primary brand color".into()),
            }],
        });
        let result = validate_spec_ir(&ir);
        assert!(result.is_valid(), "errors: {:?}", result.issues);
    }

    #[test]
    fn design_token_empty_path_is_error() {
        let ir = design_token_ir(DesignTokenSpec {
            name: "theme".into(),
            tokens: vec![DesignTokenEntry {
                path: "".into(),
                value: "#fff".into(),
                token_type: "color".into(),
                description: None,
            }],
        });
        let result = validate_spec_ir(&ir);
        assert!(!result.is_valid());
        assert!(result.errors().any(|e| e.path.contains("tokens[0].path")));
    }

    #[test]
    fn design_token_empty_value_is_error() {
        let ir = design_token_ir(DesignTokenSpec {
            name: "theme".into(),
            tokens: vec![DesignTokenEntry {
                path: "color.primary".into(),
                value: "".into(),
                token_type: "color".into(),
                description: None,
            }],
        });
        let result = validate_spec_ir(&ir);
        assert!(!result.is_valid());
        assert!(result.errors().any(|e| e.path.contains("tokens[0].value")));
    }

    #[test]
    fn design_token_unknown_type_is_warning() {
        let ir = design_token_ir(DesignTokenSpec {
            name: "theme".into(),
            tokens: vec![DesignTokenEntry {
                path: "some.token".into(),
                value: "42".into(),
                token_type: "custom-unknown-type".into(),
                description: None,
            }],
        });
        let result = validate_spec_ir(&ir);
        assert!(result.is_valid()); // warning only
        assert!(result.warnings().any(|w| w.path.contains("token_type")));
    }

    #[test]
    fn design_token_empty_name_is_warning() {
        let ir = design_token_ir(DesignTokenSpec {
            name: "".into(),
            tokens: vec![],
        });
        let result = validate_spec_ir(&ir);
        assert!(result.is_valid());
        assert!(result
            .warnings()
            .any(|w| w.path.contains("design-token.name")));
    }

    #[test]
    fn design_token_all_valid_dtcg_types_pass() {
        for token_type in VALID_DTCG_TYPES {
            let ir = design_token_ir(DesignTokenSpec {
                name: "theme".into(),
                tokens: vec![DesignTokenEntry {
                    path: "a.b".into(),
                    value: "x".into(),
                    token_type: token_type.to_string(),
                    description: None,
                }],
            });
            let result = validate_spec_ir(&ir);
            assert!(
                result.is_valid() && result.warning_count() == 0,
                "type '{}' should produce no issues, got {:?}",
                token_type,
                result.issues
            );
        }
    }

    // -----------------------------------------------------------------------
    // can_validate routing
    // -----------------------------------------------------------------------

    #[test]
    fn deploy_validator_only_handles_deploy() {
        use crate::generate::schema::JsonSchema;
        let v = DeployValidator;
        let api_ir = SpecIR::Api {
            schema: JsonSchema::default(),
            metadata: SpecMetadata::default(),
        };
        assert!(!v.can_validate(&api_ir));
        let deploy_ir = deploy_ir(DeploySpec::default());
        assert!(v.can_validate(&deploy_ir));
    }

    #[test]
    fn wireframe_validator_only_handles_wireframe() {
        use crate::generate::schema::JsonSchema;
        let v = WireframeValidator;
        let api_ir = SpecIR::Api {
            schema: JsonSchema::default(),
            metadata: SpecMetadata::default(),
        };
        assert!(!v.can_validate(&api_ir));
        let wf_ir = wireframe_ir(WireframeSpec::default());
        assert!(v.can_validate(&wf_ir));
    }
}

// CODEGEN-END
