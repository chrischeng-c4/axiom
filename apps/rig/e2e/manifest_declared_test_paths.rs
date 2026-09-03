use std::fs;
use std::path::PathBuf;

#[test]
fn generated_contract_cases_reference_live_non_ignored_tests() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repository = manifest_dir
        .parent()
        .and_then(|apps| apps.parent())
        .expect("apps/rig must be inside the repository root");
    let manifest_text =
        fs::read_to_string(manifest_dir.join("aw.toml")).expect("read the Rig project manifest");
    let manifest: toml::Value = toml::from_str(&manifest_text).expect("parse the Rig manifest");
    let cases = manifest["aw"]["ec"]["generated"]["cases"]
        .as_array()
        .expect("generated contract cases");

    assert!(
        !cases.is_empty(),
        "Rig must declare its contract test coverage"
    );
    for case in cases {
        let id = case["id"].as_str().expect("case id");
        let relative = case["test_path"].as_str().expect("case test_path");
        let path = repository.join(relative);
        assert!(
            path.is_file(),
            "{id} references missing test path {relative}"
        );
        let source = fs::read_to_string(&path).expect("read declared test source");
        assert!(
            !source.contains("#[ignore"),
            "{id} references an ignored test source: {relative}"
        );
    }
}
