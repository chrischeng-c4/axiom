// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/test_generator_preamble.md#source
// CODEGEN-BEGIN
//! Test scaffold generator for RequirementPlus diagrams
//!
//! Maps `RequirementDiagramDef` → pytest test files following the
//! Requirement+ → Test mapping contract in `code-generator-contract.md`.
//!
//! # Mapping rules
//!
//! | Source | Target |
//! |--------|--------|
//! | requirement with `verifymethod: Test` | `class TestR{id}_{name}` |
//! | `Scenario -verifies-> R` | `def test_{scenario_name}(self)` |
//! | `Module -satisfies-> R` | `import` statement at file top |
//! | `R2 -derives-> R1` | comment: "R2 depends on R1; run R1 tests first" |
//! | `risk: High` | `@pytest.mark.critical` marker |
//! | `verifymethod: Inspection` | `# TODO: Manual inspection required` (no function) |
//!
//! # Safe defaults (Q2)
//!
//! Test functions contain only `# Given/When/Then` comments + `pass  # TODO: implement`.
//! No NLP heuristics are applied.

use crate::generate::diagrams::{
    ReqRelationshipTypePlus, RequirementDefPlus, RequirementDiagramDef, RiskLevelPlus,
    VerificationMethodPlus,
};
use std::collections::{BTreeMap, HashMap};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/test_generator.md#schema
// CODEGEN-BEGIN
use serde::Serialize;

/// A coverage issue found during test generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/test_generator.md#schema
#[derive(Debug, Clone, Serialize)]
pub struct CoverageIssue {
    /// Requirement ID (e.g. R1).
    pub req_id: String,
    /// Requirement text.
    pub req_text: String,
    /// Issue description.
    pub message: String,
}

/// Errors returned by TestGenerator.generate.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/test_generator.md#schema
#[derive(Debug, thiserror::Error)]
pub enum TestGenError {
    /// All requirements have non-Test verifymethod.
    #[error("No testable requirements found (all have verifymethod != Test)")]
    NoTestableRequirements,
    /// Strict mode found uncovered requirements.
    #[error("Uncovered requirements in strict mode: {0}")]
    UncoveredRequirements(String),
}

/// Result of a successful test generation run.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/test_generator.md#schema
#[derive(Debug, Clone)]
pub struct TestGenResult {
    /// Relative output file path.
    pub file_path: String,
    /// Generated Python source.
    pub content: String,
    /// Coverage warnings.
    pub coverage_issues: Vec<CoverageIssue>,
}

/// Test scaffold generator from RequirementDiagramDef.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/test_generator.md#schema
pub struct TestGenerator {
    /// When true, uncovered requirements become hard errors instead of warnings.
    pub strict: bool,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/test_generator_runtime.md#source
// CODEGEN-BEGIN

// ---------------------------------------------------------------------------
// TestGenerator
// ---------------------------------------------------------------------------

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/test_generator_runtime.md#source
impl TestGenerator {
    pub fn new(strict: bool) -> Self {
        Self { strict }
    }

    /// Generate a pytest test scaffold from a `RequirementDiagramDef`.
    pub fn generate(&self, def: &RequirementDiagramDef) -> Result<TestGenResult, TestGenError> {
        // Build lookup maps from relationships.
        let verifies_map = build_verifies_map(&def.relationships); // req_id → [scenario_id, ...]
        let satisfies_map = build_satisfies_map(&def.relationships); // req_id → [module_id, ...]
        let derives_map = build_derives_map(&def.relationships); // req_id → [base_req_id, ...]

        // Coverage check: every testable requirement should have at least one verifies.
        let coverage_issues = check_coverage(def, &verifies_map);

        if self.strict && !coverage_issues.is_empty() {
            let ids: Vec<&str> = coverage_issues.iter().map(|i| i.req_id.as_str()).collect();
            return Err(TestGenError::UncoveredRequirements(ids.join(", ")));
        }

        // Check there is at least one testable requirement.
        let has_testable = def
            .requirements
            .values()
            .any(|r| r.verification == VerificationMethodPlus::Test);
        if !has_testable && coverage_issues.is_empty() {
            return Err(TestGenError::NoTestableRequirements);
        }

        let file_path = make_file_path(def);
        let content = render(
            def,
            &verifies_map,
            &satisfies_map,
            &derives_map,
            &coverage_issues,
        );

        Ok(TestGenResult {
            file_path,
            content,
            coverage_issues,
        })
    }
}

// ---------------------------------------------------------------------------
// Relationship helpers
// ---------------------------------------------------------------------------

/// Build `req_id → [scenario_id, ...]` from `Scenario -verifies-> Req` edges.
fn build_verifies_map(
    relationships: &[crate::generate::diagrams::ReqRelationshipDef],
) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for rel in relationships {
        if rel.rel_type == ReqRelationshipTypePlus::Verifies {
            // from: scenario, to: requirement
            map.entry(rel.to.clone())
                .or_default()
                .push(rel.from.clone());
        }
    }
    map
}

/// Build `req_id → [module_id, ...]` from `Module -satisfies-> Req` edges.
fn build_satisfies_map(
    relationships: &[crate::generate::diagrams::ReqRelationshipDef],
) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for rel in relationships {
        if rel.rel_type == ReqRelationshipTypePlus::Satisfies {
            // from: module, to: requirement
            map.entry(rel.to.clone())
                .or_default()
                .push(rel.from.clone());
        }
    }
    map
}

/// Build `req_id → [base_req_id, ...]` from `Req -derives-> BaseReq` edges.
fn build_derives_map(
    relationships: &[crate::generate::diagrams::ReqRelationshipDef],
) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for rel in relationships {
        if rel.rel_type == ReqRelationshipTypePlus::Derives {
            // from: derived req, to: base req
            map.entry(rel.from.clone())
                .or_default()
                .push(rel.to.clone());
        }
    }
    map
}

// ---------------------------------------------------------------------------
// Coverage check
// ---------------------------------------------------------------------------

fn check_coverage(
    def: &RequirementDiagramDef,
    verifies_map: &HashMap<String, Vec<String>>,
) -> Vec<CoverageIssue> {
    let mut issues = Vec::new();
    // Use BTreeMap for deterministic ordering.
    let sorted: BTreeMap<_, _> = def.requirements.iter().collect();
    for (req_id, req) in sorted {
        if req.verification != VerificationMethodPlus::Test {
            continue;
        }
        if !verifies_map.contains_key(req_id) {
            issues.push(CoverageIssue {
                req_id: req_id.clone(),
                req_text: req.text.clone(),
                message: format!(
                    "Requirement '{}' has verifymethod: Test but no Verifies relationship",
                    req_id
                ),
            });
        }
    }
    issues
}

// ---------------------------------------------------------------------------
// File path
// ---------------------------------------------------------------------------

fn make_file_path(def: &RequirementDiagramDef) -> String {
    let slug = def
        .id
        .to_lowercase()
        .replace(' ', "_")
        .replace('-', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>();
    format!("tests/test_{}.py", slug)
}

// ---------------------------------------------------------------------------
// Renderer
// ---------------------------------------------------------------------------

fn render(
    def: &RequirementDiagramDef,
    verifies_map: &HashMap<String, Vec<String>>,
    satisfies_map: &HashMap<String, Vec<String>>,
    derives_map: &HashMap<String, Vec<String>>,
    coverage_issues: &[CoverageIssue],
) -> String {
    let mut out = String::new();

    // --- Module docstring ---
    let title = def.title.as_deref().unwrap_or(&def.id);
    out.push_str(&format!(
        "\"\"\"Test scaffold for: {}\n\nGenerated by sdd from RequirementPlus diagram '{}'.\nDo not edit the class/function structure; fill in test logic.\n\"\"\"\n",
        title, def.id
    ));
    out.push('\n');

    // --- Standard imports ---
    out.push_str("import pytest\n");
    out.push('\n');

    // --- Module imports from satisfies relationships ---
    // Collect all modules and the requirements they satisfy.
    let mut module_to_reqs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (req_id, modules) in satisfies_map {
        for module_id in modules {
            module_to_reqs
                .entry(module_id.clone())
                .or_default()
                .push(req_id.clone());
        }
    }

    if !module_to_reqs.is_empty() {
        out.push_str("# Modules under test (from satisfies relationships)\n");
        for (module_id, req_ids) in &module_to_reqs {
            let docref = def
                .elements
                .get(module_id)
                .and_then(|e| e.docref.as_deref())
                .unwrap_or("");
            let mut req_list = req_ids.clone();
            req_list.sort();
            let comment = req_list.join(", ");
            if docref.is_empty() {
                out.push_str(&format!(
                    "# from {} import ...  # satisfies {}\n",
                    module_id, comment
                ));
            } else {
                // Convert docref path to importable module path
                let import_path = docref_to_import(docref);
                out.push_str(&format!(
                    "# from {} import ...  # satisfies {} ({})\n",
                    import_path, comment, module_id
                ));
            }
        }
        out.push('\n');
    }

    // --- Coverage warnings as comments ---
    if !coverage_issues.is_empty() {
        out.push_str("# Coverage warnings:\n");
        for issue in coverage_issues {
            out.push_str(&format!("# WARNING: {}\n", issue.message));
        }
        out.push('\n');
    }

    // --- Test classes (sorted by req_id for determinism) ---
    let sorted_reqs: BTreeMap<_, _> = def.requirements.iter().collect();

    for (req_id, req) in &sorted_reqs {
        out.push_str(&render_test_class(
            req_id,
            req,
            def,
            verifies_map,
            derives_map,
        ));
    }

    out
}

/// Render a single test class for one requirement.
fn render_test_class(
    req_id: &str,
    req: &RequirementDefPlus,
    def: &RequirementDiagramDef,
    verifies_map: &HashMap<String, Vec<String>>,
    derives_map: &HashMap<String, Vec<String>>,
) -> String {
    let mut out = String::new();

    let class_name = make_class_name(req_id, &req.text);
    let risk_label = risk_label(&req.risk);

    // Class-level comment for non-Test requirements
    if req.verification != VerificationMethodPlus::Test {
        let method_str = verification_method_str(&req.verification);
        out.push_str(&format!(
            "# {}: {} (risk: {}, verifymethod: {})\n",
            req_id, req.text, risk_label, method_str,
        ));
        out.push_str(&format!(
            "# TODO: Manual {} required for {}\n\n",
            method_str.to_lowercase(),
            req_id
        ));
        return out;
    }

    // Derives comment: test ordering
    if let Some(bases) = derives_map.get(req_id) {
        let mut sorted_bases = bases.clone();
        sorted_bases.sort();
        out.push_str(&format!(
            "# {}: depends on {} — run {} tests first (derives relationship)\n",
            req_id,
            sorted_bases.join(", "),
            sorted_bases.join("/"),
        ));
    }

    // Class decorator for high-risk
    if req.risk == RiskLevelPlus::High {
        out.push_str("@pytest.mark.critical\n");
    }

    out.push_str(&format!("class {}:\n", class_name));
    out.push_str(&format!(
        "    \"\"\"{}: {} (risk: {})\"\"\"\n",
        req_id, req.text, risk_label
    ));
    out.push('\n');

    // Test methods from verifies scenarios
    let scenarios: Vec<&str> = verifies_map
        .get(req_id)
        .map(|v| {
            let mut s: Vec<&str> = v.iter().map(|s| s.as_str()).collect();
            s.sort();
            s
        })
        .unwrap_or_default();

    if scenarios.is_empty() {
        // No verifying scenarios: generate a placeholder
        out.push_str("    def test_placeholder(self):\n");
        out.push_str(&format!(
            "        # TODO: Add test scenarios that verify {}\n",
            req_id
        ));
        out.push_str("        pass  # TODO: implement\n");
    } else {
        for scenario_id in scenarios {
            out.push_str(&render_test_method(scenario_id, req_id, def));
        }
    }

    out.push('\n');
    out
}

/// Render a single test method from a scenario element.
fn render_test_method(scenario_id: &str, req_id: &str, def: &RequirementDiagramDef) -> String {
    let mut out = String::new();

    let func_name = make_function_name(scenario_id);
    let elem = def.elements.get(scenario_id);

    out.push_str(&format!("    def {}(self):\n", func_name));

    // Scenario docstring / description
    if let Some(e) = elem {
        if !e.text.is_empty() {
            out.push_str(&format!(
                "        # Scenario: {} → verifies {}\n",
                e.text, req_id
            ));
        } else {
            out.push_str(&format!(
                "        # Scenario: {} → verifies {}\n",
                scenario_id, req_id
            ));
        }

        // BDD comments from ElementDef fields
        if let Some(given) = &e.given {
            out.push_str(&format!("        # Given: {}\n", given));
        }
        if let Some(when) = &e.when {
            out.push_str(&format!("        # When: {}\n", when));
        }
        if let Some(then) = &e.then {
            out.push_str(&format!("        # Then: {}\n", then));
        } else {
            out.push_str("        # Then: <expected outcome>\n");
        }
    } else {
        // Element not defined in diagram — use scenario ID as description
        out.push_str(&format!(
            "        # Scenario: {} → verifies {}\n",
            scenario_id, req_id
        ));
        out.push_str("        # Given: <precondition>\n");
        out.push_str("        # When: <action>\n");
        out.push_str("        # Then: <expected outcome>\n");
    }

    out.push_str("        pass  # TODO: implement\n");
    out.push('\n');
    out
}

// ---------------------------------------------------------------------------
// Naming helpers
// ---------------------------------------------------------------------------

fn make_class_name(req_id: &str, text: &str) -> String {
    // TestR1_TokenRefresh from "R1" + "Users can refresh expired tokens"
    let safe_id = req_id
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>();

    let suffix = text
        .split_whitespace()
        .take(4)
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    let rest: String = chars.filter(|c| c.is_alphanumeric()).collect();
                    upper + &rest
                }
            }
        })
        .collect::<String>();

    format!("Test{}_{}", safe_id, suffix)
}

fn make_function_name(scenario_id: &str) -> String {
    // Convert "Scenario_happy" → "test_scenario_happy"
    let snake = scenario_id
        .to_lowercase()
        .replace('-', "_")
        .replace(' ', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>();
    if snake.starts_with("test_") {
        snake
    } else {
        format!("test_{}", snake)
    }
}

fn docref_to_import(docref: &str) -> String {
    // "src/handlers/auth.py" → "src.handlers.auth"
    docref
        .trim_end_matches(".py")
        .replace('/', ".")
        .replace('-', "_")
}

fn risk_label(risk: &RiskLevelPlus) -> &'static str {
    match risk {
        RiskLevelPlus::Low => "Low",
        RiskLevelPlus::Medium => "Medium",
        RiskLevelPlus::High => "High",
    }
}

fn verification_method_str(method: &VerificationMethodPlus) -> &'static str {
    match method {
        VerificationMethodPlus::Test => "Test",
        VerificationMethodPlus::Inspection => "Inspection",
        VerificationMethodPlus::Analysis => "Analysis",
        VerificationMethodPlus::Demonstration => "Demonstration",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::diagrams::{
        ElementDef, ReqRelationshipDef, ReqRelationshipTypePlus, RequirementDefPlus,
        RequirementDiagramDef, RequirementTypePlus, RiskLevelPlus, VerificationMethodPlus,
    };
    use std::collections::HashMap;

    fn make_auth_diagram() -> RequirementDiagramDef {
        let mut requirements = HashMap::new();
        requirements.insert(
            "R1".to_string(),
            RequirementDefPlus {
                text: "Users can refresh expired tokens".to_string(),
                req_type: RequirementTypePlus::FunctionalRequirement,
                risk: RiskLevelPlus::Medium,
                verification: VerificationMethodPlus::Test,
                description: None,
            },
        );
        requirements.insert(
            "R2".to_string(),
            RequirementDefPlus {
                text: "Invalid tokens return 401".to_string(),
                req_type: RequirementTypePlus::FunctionalRequirement,
                risk: RiskLevelPlus::High,
                verification: VerificationMethodPlus::Test,
                description: None,
            },
        );

        let mut elements = HashMap::new();
        elements.insert(
            "Scenario_happy".to_string(),
            ElementDef {
                text: "valid refresh token".to_string(),
                elem_type: "Scenario".to_string(),
                docref: None,
                description: None,
                test_type: None,
                given: Some("a user with a valid refresh token".to_string()),
                when: Some("POST /refresh is called".to_string()),
                then: Some("a new access token is returned".to_string()),
            },
        );
        elements.insert(
            "Scenario_expired".to_string(),
            ElementDef {
                text: "expired token returns 401".to_string(),
                elem_type: "Scenario".to_string(),
                docref: None,
                description: None,
                test_type: None,
                given: Some("an expired refresh token".to_string()),
                when: Some("POST /refresh is called".to_string()),
                then: Some("401 is returned".to_string()),
            },
        );
        elements.insert(
            "Scenario_revoked".to_string(),
            ElementDef {
                text: "revoked token returns 401".to_string(),
                elem_type: "Scenario".to_string(),
                docref: None,
                description: None,
                test_type: None,
                given: Some("a revoked refresh token".to_string()),
                when: Some("POST /refresh is called".to_string()),
                then: Some("401 is returned".to_string()),
            },
        );
        elements.insert(
            "auth_handler".to_string(),
            ElementDef {
                text: "Authentication Handler".to_string(),
                elem_type: "Module".to_string(),
                docref: Some("src/handlers/auth.py".to_string()),
                description: None,
                test_type: None,
                given: None,
                when: None,
                then: None,
            },
        );

        RequirementDiagramDef {
            id: "auth-token-refresh".to_string(),
            title: Some("Auth Token Refresh Requirements".to_string()),
            direction: None,
            requirements,
            elements,
            relationships: vec![
                ReqRelationshipDef {
                    from: "Scenario_happy".into(),
                    to: "R1".into(),
                    rel_type: ReqRelationshipTypePlus::Verifies,
                },
                ReqRelationshipDef {
                    from: "Scenario_expired".into(),
                    to: "R2".into(),
                    rel_type: ReqRelationshipTypePlus::Verifies,
                },
                ReqRelationshipDef {
                    from: "Scenario_revoked".into(),
                    to: "R2".into(),
                    rel_type: ReqRelationshipTypePlus::Verifies,
                },
                ReqRelationshipDef {
                    from: "auth_handler".into(),
                    to: "R1".into(),
                    rel_type: ReqRelationshipTypePlus::Satisfies,
                },
                ReqRelationshipDef {
                    from: "auth_handler".into(),
                    to: "R2".into(),
                    rel_type: ReqRelationshipTypePlus::Satisfies,
                },
                ReqRelationshipDef {
                    from: "R2".into(),
                    to: "R1".into(),
                    rel_type: ReqRelationshipTypePlus::Derives,
                },
            ],
            description: None,
        }
    }

    #[test]
    fn test_generate_auth_diagram() {
        let def = make_auth_diagram();
        let gen = TestGenerator::new(false);
        let result = gen.generate(&def).unwrap();

        assert_eq!(result.coverage_issues.len(), 0);
        assert!(result.file_path.ends_with(".py"));
        assert!(result.content.contains("import pytest"));
        assert!(result.content.contains("class TestR1_"));
        assert!(result.content.contains("class TestR2_"));
        assert!(result.content.contains("@pytest.mark.critical")); // R2 is High risk
        assert!(result.content.contains("test_scenario_happy"));
        assert!(result.content.contains("pass  # TODO: implement"));
    }

    #[test]
    fn test_coverage_warning_no_verifies() {
        let mut def = make_auth_diagram();
        // Remove all verifies-R1 relationships
        def.relationships
            .retain(|r| !(r.rel_type == ReqRelationshipTypePlus::Verifies && r.to == "R1"));

        let gen = TestGenerator::new(false);
        let result = gen.generate(&def).unwrap();
        assert_eq!(result.coverage_issues.len(), 1);
        assert_eq!(result.coverage_issues[0].req_id, "R1");
    }

    #[test]
    fn test_strict_mode_fails_on_uncovered() {
        let mut def = make_auth_diagram();
        def.relationships
            .retain(|r| r.rel_type != ReqRelationshipTypePlus::Verifies);

        let gen = TestGenerator::new(true);
        let err = gen.generate(&def).unwrap_err();
        assert!(matches!(err, TestGenError::UncoveredRequirements(_)));
    }

    #[test]
    fn test_file_path_from_diagram_id() {
        let mut def = make_auth_diagram();
        def.id = "my-diagram-id".to_string();
        let path = make_file_path(&def);
        assert_eq!(path, "tests/test_my_diagram_id.py");
    }

    #[test]
    fn test_derives_comment_in_output() {
        let def = make_auth_diagram();
        let gen = TestGenerator::new(false);
        let result = gen.generate(&def).unwrap();
        // R2 derives R1, should have ordering comment
        assert!(result.content.contains("R2") && result.content.contains("R1"));
        assert!(result.content.contains("depends on"));
    }

    #[test]
    fn test_bdd_comments_in_output() {
        let def = make_auth_diagram();
        let gen = TestGenerator::new(false);
        let result = gen.generate(&def).unwrap();
        assert!(result.content.contains("# Given:"));
        assert!(result.content.contains("# When:"));
        assert!(result.content.contains("# Then:"));
    }

    #[test]
    fn test_inspection_method_generates_comment_not_class() {
        let mut def = make_auth_diagram();
        def.requirements.get_mut("R1").unwrap().verification = VerificationMethodPlus::Inspection;
        // Remove verifies for R1 to avoid no-testable-requirements error
        let gen = TestGenerator::new(false);
        let result = gen.generate(&def).unwrap();
        // R1 should not produce a class; should produce a TODO comment
        assert!(result.content.contains("TODO: Manual inspection"));
        // R2 should still produce a class
        assert!(result.content.contains("class TestR2_"));
    }

    #[test]
    fn test_no_testable_requirements_error() {
        let mut def = make_auth_diagram();
        for req in def.requirements.values_mut() {
            req.verification = VerificationMethodPlus::Inspection;
        }
        let gen = TestGenerator::new(false);
        let err = gen.generate(&def).unwrap_err();
        assert!(matches!(err, TestGenError::NoTestableRequirements));
    }

    #[test]
    fn test_make_class_name() {
        assert_eq!(
            make_class_name("R1", "Users can refresh tokens"),
            "TestR1_UsersCanRefreshTokens"
        );
        assert_eq!(
            make_class_name("REQ-001", "Auth check"),
            "TestREQ001_AuthCheck"
        );
    }

    #[test]
    fn test_make_function_name() {
        assert_eq!(make_function_name("Scenario_happy"), "test_scenario_happy");
        assert_eq!(
            make_function_name("test_already_prefixed"),
            "test_already_prefixed"
        );
        assert_eq!(make_function_name("Some Scenario"), "test_some_scenario");
    }
}
// CODEGEN-END
