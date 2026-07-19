// HANDWRITE-BEGIN gap="missing-generator:unit-test:lumen-ec-claim-closure-consistency" tracker="1871" reason="The EC producer cannot independently prove that its authored claim document, generated inventory, wrapper dispatch, and README claim roots agree without a repository-level oracle."
// @spec apps/lumen/tech-design/semantic/lumen-ec-gates.md#schema
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

const CLAIM_DOCUMENT: &str = "apps/lumen/external-contracts/claim-closure/production-claims.md";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct ClaimCase {
    id: String,
    capability_id: String,
    claim_id: String,
    contract_id: String,
    category: String,
    command: String,
    assertions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ClaimDocument {
    e2e_tests: Vec<ClaimCase>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GeneratedCase {
    claim: ClaimCase,
    td_ref: String,
    test_path: String,
    required_for_production: bool,
}

#[test]
fn generated_inventory_matches_claim_commands_and_test_dispatch() {
    let root = workspace_root();
    let authored = authored_cases(&root);
    let generated = generated_cases(&root);

    let mut drift = Vec::new();
    if authored.len() != generated.len() {
        drift.push(format!(
            "authored/generated case count: {}/{}",
            authored.len(),
            generated.len()
        ));
    }

    for (id, authored_case) in &authored {
        let Some(generated_case) = generated.get(id) else {
            drift.push(format!("generated EC inventory is missing {id}"));
            continue;
        };
        if &generated_case.claim != authored_case {
            drift.push(format!(
                "generated EC inventory drifted for {id}: authored={authored_case:?}, generated={:?}",
                generated_case.claim
            ));
        }
        if generated_case.td_ref != format!("{CLAIM_DOCUMENT}#{id}") {
            drift.push(format!(
                "generated EC source reference drifted for {id}: {}",
                generated_case.td_ref
            ));
        }
        if !generated_case.required_for_production {
            drift.push(format!(
                "claim-closure case {id} is not production-required"
            ));
        }

        let wrapper_path = root.join(&generated_case.test_path);
        let wrapper = fs::read_to_string(&wrapper_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", wrapper_path.display()));
        for marker in [
            format!("// @ec {id}"),
            format!("// @capability {}", authored_case.capability_id),
            format!("// @claim {}", authored_case.claim_id),
            format!("// @contract {}", authored_case.contract_id),
            format!("// @category {}", authored_case.category),
            format!("// @command {}", authored_case.command),
        ] {
            if !wrapper.contains(&marker) {
                drift.push(format!(
                    "generated wrapper {} is missing marker `{marker}`",
                    wrapper_path.display()
                ));
            }
        }
    }

    assert!(
        drift.is_empty(),
        "claim-closure inventory drift:\n{}",
        drift.join("\n")
    );
}

#[test]
fn claim_closure_document_maps_to_readme_capability_claims() {
    let root = workspace_root();
    let authored = authored_cases(&root);
    let readme_path = root.join("apps/lumen/README.md");
    let readme = fs::read_to_string(&readme_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", readme_path.display()));

    for (id, case) in &authored {
        let capability_marker = format!("ID: {}", case.capability_id);
        assert!(
            readme.lines().any(|line| line.trim() == capability_marker),
            "claim-closure case {id} references missing README capability {}",
            case.capability_id
        );

        let claim_marker = format!("| {} |", case.claim_id);
        assert!(
            readme.contains(&claim_marker),
            "claim-closure case {id} references missing README work root {}",
            case.claim_id
        );
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical workspace root")
}

fn authored_cases(root: &Path) -> BTreeMap<String, ClaimCase> {
    let path = root.join(CLAIM_DOCUMENT);
    let markdown = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let yaml = markdown
        .split_once("```yaml\n")
        .and_then(|(_, rest)| rest.split_once("\n```").map(|(body, _)| body))
        .expect("claim-closure document must contain one YAML code fence");
    let document: ClaimDocument =
        serde_yaml::from_str(yaml).expect("parse claim-closure e2e-test YAML");

    let mut cases = BTreeMap::new();
    for case in document.e2e_tests {
        let id = case.id.clone();
        assert!(
            cases.insert(id.clone(), case).is_none(),
            "duplicate EC id {id}"
        );
    }
    assert!(
        !cases.is_empty(),
        "claim-closure inventory must not be empty"
    );
    cases
}

fn generated_cases(root: &Path) -> BTreeMap<String, GeneratedCase> {
    let path = root.join("apps/lumen/aw.toml");
    let value: toml::Value = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
        .parse()
        .expect("parse lumen aw.toml");
    let cases = value
        .get("aw")
        .and_then(|value| value.get("ec"))
        .and_then(|value| value.get("generated"))
        .and_then(|value| value.get("cases"))
        .and_then(toml::Value::as_array)
        .expect("aw.ec.generated.cases array");

    let mut generated = BTreeMap::new();
    let mut test_paths = BTreeSet::new();
    for value in cases {
        let table = value.as_table().expect("generated EC case table");
        let td_ref = table_string(table, "td_ref");
        if !td_ref.starts_with(&format!("{CLAIM_DOCUMENT}#")) {
            continue;
        }
        let claim = ClaimCase {
            id: table_string(table, "id"),
            capability_id: table_string(table, "capability_id"),
            claim_id: table_string(table, "claim_id"),
            contract_id: table_string(table, "contract_id"),
            category: table_string(table, "category"),
            command: table_string(table, "command"),
            assertions: table
                .get("assertions")
                .and_then(toml::Value::as_array)
                .expect("generated EC assertions array")
                .iter()
                .map(|value| value.as_str().expect("EC assertion string").to_owned())
                .collect(),
        };
        let id = claim.id.clone();
        let generated_case = GeneratedCase {
            claim,
            td_ref,
            test_path: table_string(table, "test_path"),
            required_for_production: table
                .get("required_for_production")
                .and_then(toml::Value::as_bool)
                .expect("generated EC required_for_production bool"),
        };
        assert!(
            test_paths.insert(generated_case.test_path.clone()),
            "duplicate generated EC test path {}",
            generated_case.test_path
        );
        assert!(
            generated.insert(id.clone(), generated_case).is_none(),
            "duplicate generated EC id {id}"
        );
    }
    generated
}

fn table_string(table: &toml::Table, key: &str) -> String {
    table
        .get(key)
        .and_then(toml::Value::as_str)
        .unwrap_or_else(|| panic!("generated EC case is missing string field {key}"))
        .to_owned()
}
// HANDWRITE-END
