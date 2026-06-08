// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#source
// CODEGEN-BEGIN
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::cli::capability::{CapabilityReportItem, CapabilitySection};

// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#source
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProductionStatus {
    Ready,
    Blocked,
    NotEvaluated,
}

// @spec projects/agentic-workflow/tech-design/surface/specs/project-health-governance-report.md#source
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductionCapabilityReadiness {
    pub id: String,
    pub release_scope: bool,
    pub dependencies: Vec<String>,
    pub dependency_closure: Vec<String>,
    pub production_ready: bool,
    pub production_blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ProductionReadinessReport {
    pub(crate) production_ready: bool,
    pub(crate) production_status: ProductionStatus,
    pub(crate) production_scope: Vec<String>,
    pub(crate) production_blockers: Vec<String>,
    pub(crate) global_blockers: Vec<String>,
    pub(crate) capabilities: Vec<ProductionCapabilityReadiness>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProductionCapabilityInput {
    pub(crate) id: String,
    pub(crate) release_scope: bool,
    pub(crate) dependencies: Vec<String>,
    pub(crate) catalog_verified: bool,
    pub(crate) full_regenerability_required: bool,
}

pub(crate) fn inputs_from_sections(
    sections: &[CapabilitySection],
    verified_by_id: &BTreeMap<String, bool>,
) -> Vec<ProductionCapabilityInput> {
    sections
        .iter()
        .map(|section| ProductionCapabilityInput {
            id: section.id.clone(),
            release_scope: section.release_scope,
            dependencies: section.dependencies.clone(),
            catalog_verified: verified_by_id.get(&section.id).copied().unwrap_or(false),
            full_regenerability_required: section
                .verification_contract
                .as_ref()
                .is_some_and(|contract| contract.full_regenerability_required),
        })
        .collect()
}

pub(crate) fn inputs_from_report_items(
    items: &[CapabilityReportItem],
) -> Vec<ProductionCapabilityInput> {
    items
        .iter()
        .map(|item| ProductionCapabilityInput {
            id: item.id.clone(),
            release_scope: item.release_scope,
            dependencies: item.dependencies.clone(),
            catalog_verified: item.verified,
            full_regenerability_required: item.full_regenerability_required,
        })
        .collect()
}

pub(crate) fn evaluate_release_scope(
    inputs: Vec<ProductionCapabilityInput>,
    global_blockers: Vec<String>,
    production_gates_evaluated: bool,
) -> ProductionReadinessReport {
    evaluate_scope(inputs, None, global_blockers, production_gates_evaluated, 0)
}

pub(crate) fn evaluate_capability_scope(
    inputs: Vec<ProductionCapabilityInput>,
    capability_id: &str,
    global_blockers: Vec<String>,
    production_gates_evaluated: bool,
) -> ProductionReadinessReport {
    evaluate_scope(
        inputs,
        Some(capability_id),
        global_blockers,
        production_gates_evaluated,
        0,
    )
}

pub(crate) fn evaluate_release_scope_with_regenerability(
    inputs: Vec<ProductionCapabilityInput>,
    global_blockers: Vec<String>,
    production_gates_evaluated: bool,
    regenerability_gap_count: usize,
) -> ProductionReadinessReport {
    evaluate_scope(
        inputs,
        None,
        global_blockers,
        production_gates_evaluated,
        regenerability_gap_count,
    )
}

pub(crate) fn evaluate_capability_scope_with_regenerability(
    inputs: Vec<ProductionCapabilityInput>,
    capability_id: &str,
    global_blockers: Vec<String>,
    production_gates_evaluated: bool,
    regenerability_gap_count: usize,
) -> ProductionReadinessReport {
    evaluate_scope(
        inputs,
        Some(capability_id),
        global_blockers,
        production_gates_evaluated,
        regenerability_gap_count,
    )
}

fn evaluate_scope(
    inputs: Vec<ProductionCapabilityInput>,
    requested_capability: Option<&str>,
    mut global_blockers: Vec<String>,
    production_gates_evaluated: bool,
    regenerability_gap_count: usize,
) -> ProductionReadinessReport {
    global_blockers.sort();
    global_blockers.dedup();

    let input_by_id = inputs
        .iter()
        .map(|input| (input.id.clone(), input.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut selected = BTreeSet::new();
    match requested_capability {
        Some(id) => {
            selected.insert(id.to_string());
            if !input_by_id.contains_key(id) {
                global_blockers.push(format!("capability `{id}` is not declared"));
            }
        }
        None => {
            selected.extend(
                inputs
                    .iter()
                    .filter(|input| input.release_scope)
                    .map(|input| input.id.clone()),
            );
            if selected.is_empty() {
                global_blockers
                    .push("no capability has Production=ready in the Capability Index".to_string());
            }
        }
    }

    let mut dependency_errors = Vec::new();
    let mut scope = BTreeSet::new();
    let mut visiting = Vec::new();
    for id in selected.clone() {
        visit_scope(
            &id,
            &input_by_id,
            &mut visiting,
            &mut scope,
            &mut dependency_errors,
        );
    }
    dependency_errors.sort();
    dependency_errors.dedup();
    global_blockers.extend(dependency_errors);
    global_blockers.sort();
    global_blockers.dedup();

    let direct_blockers = inputs
        .iter()
        .map(|input| {
            let mut blockers = Vec::new();
            if scope.contains(&input.id) && !input.catalog_verified {
                blockers.push("catalog/claim verification is not complete".to_string());
            }
            if scope.contains(&input.id)
                && input.full_regenerability_required
                && regenerability_gap_count > 0
            {
                blockers.push(format!(
                    "full regenerability is required but {regenerability_gap_count} gap(s) remain"
                ));
            }
            for dependency in &input.dependencies {
                if !input_by_id.contains_key(dependency) {
                    blockers.push(format!("dependency `{dependency}` is not declared"));
                }
            }
            (input.id.clone(), blockers)
        })
        .collect::<BTreeMap<_, _>>();

    let mut memo = BTreeMap::new();
    let mut readiness = Vec::new();
    for input in &inputs {
        let dependency_closure = dependency_closure_for(&input.id, &input_by_id);
        let mut blockers = direct_blockers.get(&input.id).cloned().unwrap_or_default();
        if scope.contains(&input.id) {
            for dependency in &input.dependencies {
                if scope.contains(dependency)
                    && !capability_ready(
                        dependency,
                        &input_by_id,
                        &direct_blockers,
                        &global_blockers,
                        production_gates_evaluated,
                        &mut memo,
                    )
                {
                    blockers.push(format!("dependency `{dependency}` is not production ready"));
                }
            }
        }
        blockers.sort();
        blockers.dedup();
        let production_ready = scope.contains(&input.id)
            && blockers.is_empty()
            && global_blockers.is_empty()
            && production_gates_evaluated
            && capability_ready(
                &input.id,
                &input_by_id,
                &direct_blockers,
                &global_blockers,
                production_gates_evaluated,
                &mut memo,
            );
        readiness.push(ProductionCapabilityReadiness {
            id: input.id.clone(),
            release_scope: input.release_scope,
            dependencies: input.dependencies.clone(),
            dependency_closure,
            production_ready,
            production_blockers: blockers,
        });
    }

    let production_scope = scope.into_iter().collect::<Vec<_>>();
    let mut production_blockers = global_blockers.clone();
    for capability in &readiness {
        if production_scope.contains(&capability.id) {
            production_blockers.extend(
                capability
                    .production_blockers
                    .iter()
                    .map(|blocker| format!("{}: {blocker}", capability.id)),
            );
        }
    }
    production_blockers.sort();
    production_blockers.dedup();

    let production_status = if !production_gates_evaluated {
        ProductionStatus::NotEvaluated
    } else if production_blockers.is_empty()
        && production_scope.iter().all(|id| {
            readiness
                .iter()
                .any(|cap| cap.id == *id && cap.production_ready)
        })
    {
        ProductionStatus::Ready
    } else {
        ProductionStatus::Blocked
    };

    ProductionReadinessReport {
        production_ready: production_status == ProductionStatus::Ready,
        production_status,
        production_scope,
        production_blockers,
        global_blockers,
        capabilities: readiness,
    }
}
// CODEGEN-END

fn visit_scope(
    id: &str,
    inputs: &BTreeMap<String, ProductionCapabilityInput>,
    visiting: &mut Vec<String>,
    scope: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    if !inputs.contains_key(id) {
        errors.push(format!("dependency `{id}` is not declared"));
        return;
    }
    if let Some(pos) = visiting.iter().position(|visited| visited == id) {
        let mut cycle = visiting[pos..].to_vec();
        cycle.push(id.to_string());
        errors.push(format!(
            "capability dependency cycle detected: {}",
            cycle.join(" -> ")
        ));
        return;
    }
    if scope.contains(id) {
        return;
    }
    visiting.push(id.to_string());
    scope.insert(id.to_string());
    if let Some(input) = inputs.get(id) {
        for dependency in &input.dependencies {
            visit_scope(dependency, inputs, visiting, scope, errors);
        }
    }
    visiting.pop();
}

fn dependency_closure_for(
    id: &str,
    inputs: &BTreeMap<String, ProductionCapabilityInput>,
) -> Vec<String> {
    let mut closure = BTreeSet::new();
    let mut visiting = Vec::new();
    let mut errors = Vec::new();
    if let Some(input) = inputs.get(id) {
        for dependency in &input.dependencies {
            visit_scope(dependency, inputs, &mut visiting, &mut closure, &mut errors);
        }
    }
    closure.into_iter().filter(|dep| dep != id).collect()
}

fn capability_ready(
    id: &str,
    inputs: &BTreeMap<String, ProductionCapabilityInput>,
    direct_blockers: &BTreeMap<String, Vec<String>>,
    global_blockers: &[String],
    production_gates_evaluated: bool,
    memo: &mut BTreeMap<String, bool>,
) -> bool {
    if let Some(ready) = memo.get(id) {
        return *ready;
    }
    let Some(input) = inputs.get(id) else {
        memo.insert(id.to_string(), false);
        return false;
    };
    let ready = production_gates_evaluated
        && global_blockers.is_empty()
        && direct_blockers
            .get(id)
            .map(|blockers| blockers.is_empty())
            .unwrap_or(true)
        && input.dependencies.iter().all(|dependency| {
            capability_ready(
                dependency,
                inputs,
                direct_blockers,
                global_blockers,
                production_gates_evaluated,
                memo,
            )
        });
    memo.insert(id.to_string(), ready);
    ready
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        id: &str,
        release_scope: bool,
        dependencies: &[&str],
        verified: bool,
    ) -> ProductionCapabilityInput {
        ProductionCapabilityInput {
            id: id.to_string(),
            release_scope,
            dependencies: dependencies.iter().map(|dep| dep.to_string()).collect(),
            catalog_verified: verified,
            full_regenerability_required: false,
        }
    }

    fn input_full_regenerability_required(
        id: &str,
        release_scope: bool,
        dependencies: &[&str],
        verified: bool,
    ) -> ProductionCapabilityInput {
        ProductionCapabilityInput {
            full_regenerability_required: true,
            ..input(id, release_scope, dependencies, verified)
        }
    }

    fn section(id: &str, release_scope: bool) -> CapabilitySection {
        CapabilitySection {
            title: id.to_string(),
            id: id.to_string(),
            status: crate::cli::capability::CapabilityStatus::Verified,
            promise: "promise".to_string(),
            current_state: "state".to_string(),
            gaps: Vec::new(),
            verification_contract: None,
            evidence: crate::cli::capability::CapabilityEvidence::default(),
            done_when: Vec::new(),
            out_of_scope: Vec::new(),
            release_scope,
            dependencies: Vec::new(),
            line: 1,
        }
    }

    #[test]
    fn section_status_verified_does_not_count_as_runtime_proof() {
        let inputs = inputs_from_sections(&[section("ship", true)], &BTreeMap::new());
        let report = evaluate_release_scope(inputs, Vec::new(), true);

        assert_eq!(report.production_status, ProductionStatus::Blocked);
        assert!(!report.production_ready);
        assert!(report
            .production_blockers
            .iter()
            .any(|blocker| blocker.contains("catalog/claim verification")));
    }

    #[test]
    fn release_scope_includes_dependencies_and_ignores_unrelated_capabilities() {
        let report = evaluate_release_scope(
            vec![
                input("ship", true, &["shared"], true),
                input("shared", false, &[], true),
                input("future", false, &[], false),
            ],
            Vec::new(),
            true,
        );

        assert_eq!(report.production_status, ProductionStatus::Ready);
        assert_eq!(report.production_scope, vec!["shared", "ship"]);
        assert!(report.production_ready);
    }

    #[test]
    fn dependency_failure_blocks_dependent_capability() {
        let report = evaluate_release_scope(
            vec![
                input("ship", true, &["shared"], true),
                input("shared", false, &[], false),
            ],
            Vec::new(),
            true,
        );

        assert_eq!(report.production_status, ProductionStatus::Blocked);
        assert!(!report.production_ready);
        let ship = report
            .capabilities
            .iter()
            .find(|capability| capability.id == "ship")
            .unwrap();
        assert!(ship
            .production_blockers
            .iter()
            .any(|blocker| blocker.contains("dependency `shared`")));
    }

    #[test]
    fn unknown_dependency_is_global_blocker() {
        let report = evaluate_release_scope(
            vec![input("ship", true, &["missing"], true)],
            Vec::new(),
            true,
        );

        assert_eq!(report.production_status, ProductionStatus::Blocked);
        assert!(report
            .production_blockers
            .iter()
            .any(|blocker| blocker.contains("missing")));
    }

    #[test]
    fn dependency_cycle_is_global_blocker() {
        let report = evaluate_release_scope(
            vec![
                input("ship", true, &["shared"], true),
                input("shared", false, &["ship"], true),
            ],
            Vec::new(),
            true,
        );

        assert_eq!(report.production_status, ProductionStatus::Blocked);
        assert!(report
            .production_blockers
            .iter()
            .any(|blocker| blocker.contains("cycle")));
    }

    #[test]
    fn verified_capability_is_not_production_ready_when_global_gate_blocks() {
        let report = evaluate_release_scope(
            vec![input("ship", true, &[], true)],
            vec!["cb verify has 1 finding(s)".to_string()],
            true,
        );

        assert_eq!(report.production_status, ProductionStatus::Blocked);
        assert!(!report.production_ready);
        assert_eq!(report.production_scope, vec!["ship"]);
        assert!(report
            .production_blockers
            .iter()
            .any(|blocker| blocker.contains("cb verify")));
    }

    #[test]
    fn non_verified_unrelated_capability_does_not_block_release_scope() {
        let report = evaluate_release_scope(
            vec![
                input("ship", true, &[], true),
                input("future", false, &[], false),
            ],
            Vec::new(),
            true,
        );

        assert_eq!(report.production_status, ProductionStatus::Ready);
        assert!(report.production_ready);
        assert_eq!(report.production_scope, vec!["ship"]);
    }

    #[test]
    fn scoped_full_regenerability_requirement_blocks_selected_capability() {
        let report = evaluate_release_scope_with_regenerability(
            vec![
                input_full_regenerability_required("ship", true, &[], true),
                input("future", false, &[], true),
            ],
            Vec::new(),
            true,
            2,
        );

        assert_eq!(report.production_status, ProductionStatus::Blocked);
        assert!(report
            .production_blockers
            .iter()
            .any(|blocker| { blocker.contains("ship: full regenerability is required") }));
        assert!(!report.capabilities[0].production_ready);
        assert!(!report.capabilities[1].production_ready);
        assert!(report.capabilities[1].production_blockers.is_empty());
    }
}
