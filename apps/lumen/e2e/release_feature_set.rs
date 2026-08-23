//! Release feature set consistency test and gate verification.

use std::path::Path;

const EXPECTED_DIRECT_FEATURES: &[&str] = &[
    "delegated-auth",
    "issue",
    "operator",
    "otel",
    "raft-wal",
    "self-update",
];

fn parse_release_features(content: &str) -> Result<Vec<String>, String> {
    let val: toml::Value = toml::from_str(content).map_err(|e| e.to_string())?;
    let release = val
        .get("features")
        .and_then(|f| f.get("release"))
        .and_then(|r| r.as_array())
        .ok_or_else(|| "missing [features].release array".to_string())?;
    let mut members = Vec::new();
    for item in release {
        let s = item
            .as_str()
            .ok_or_else(|| "feature member is not a string".to_string())?;
        members.push(s.to_string());
    }
    Ok(members)
}

fn validate_manifest_features(content: &str) -> Result<(), String> {
    let mut actual = parse_release_features(content)?;
    actual.sort();
    let mut expected: Vec<String> = EXPECTED_DIRECT_FEATURES
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    expected.sort();
    if actual != expected {
        return Err(format!("expected {expected:?}, got {actual:?}"));
    }
    Ok(())
}

fn logical_lines(content: &str) -> Vec<String> {
    let mut logical = Vec::new();
    let mut current = String::new();
    for raw_line in content.lines() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some(stripped) = trimmed.strip_suffix('\\') {
            current.push_str(stripped.trim_end());
            current.push(' ');
        } else {
            current.push_str(trimmed);
            if !current.trim().is_empty() {
                logical.push(current.trim().to_string());
            }
            current.clear();
        }
    }
    if !current.trim().is_empty() {
        logical.push(current.trim().to_string());
    }
    logical
}

fn validate_surface_invocation(content: &str) -> Result<(), String> {
    let lines = logical_lines(content);
    let mut invocations: Vec<Vec<String>> = Vec::new();

    for line in lines {
        for part in line
            .split("&&")
            .flat_map(|s| s.split("||"))
            .flat_map(|s| s.split(';'))
        {
            let tokens: Vec<String> = part
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            let has_cargo_build = tokens.windows(2).any(|w| w[0] == "cargo" && w[1] == "build");
            let has_release = tokens.iter().any(|t| t == "--release");
            let has_p_lumen = tokens
                .windows(2)
                .any(|w| (w[0] == "-p" || w[0] == "--package") && w[1] == "lumen")
                || tokens
                    .iter()
                    .any(|t| t == "-p=lumen" || t == "--package=lumen");
            let has_bin_lumen = tokens.windows(2).any(|w| w[0] == "--bin" && w[1] == "lumen")
                || tokens.iter().any(|t| t == "--bin=lumen");

            if has_cargo_build && has_release && has_p_lumen && has_bin_lumen {
                invocations.push(tokens);
            }
        }
    }

    if invocations.len() != 1 {
        return Err(format!(
            "expected exactly 1 lumen release build invocation, found {}",
            invocations.len()
        ));
    }

    let tokens = &invocations[0];
    let mut valid_feature_pair = false;

    for i in 0..tokens.len() {
        let t = tokens[i].trim_matches('"').trim_matches('\'');
        if t == "--features" {
            if i + 1 < tokens.len() {
                let val = tokens[i + 1].trim_matches('"').trim_matches('\'');
                if val == "release" {
                    if valid_feature_pair {
                        return Err("multiple --features flags found in invocation".to_string());
                    }
                    valid_feature_pair = true;
                } else {
                    return Err(format!("expected 'release' after --features, got '{val}'"));
                }
            } else {
                return Err("missing argument after --features".to_string());
            }
        } else if t.starts_with("--features=") {
            return Err(format!(
                "'--features=...' syntax is forbidden; use '--features release': {t}"
            ));
        }
    }

    if !valid_feature_pair {
        return Err("no valid '--features release' token pair found in invocation".to_string());
    }

    Ok(())
}

#[test]
fn test_manifest_release_features() {
    let manifest_path = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
    let content = std::fs::read_to_string(manifest_path).expect("read Cargo.toml");
    validate_manifest_features(&content).expect("live Cargo.toml must match canonical features");

    // Negative tests: remove each required direct feature in turn
    for missing in EXPECTED_DIRECT_FEATURES {
        let filtered: Vec<&str> = EXPECTED_DIRECT_FEATURES
            .iter()
            .copied()
            .filter(|&f| f != *missing)
            .collect();
        let toml_sample = format!("[features]\nrelease = {filtered:?}\n");
        assert!(
            validate_manifest_features(&toml_sample).is_err(),
            "manifest missing required feature '{missing}' must fail"
        );
    }

    // Negative test: add jieba
    let mut with_jieba: Vec<&str> = EXPECTED_DIRECT_FEATURES.to_vec();
    with_jieba.push("jieba");
    let toml_jieba = format!("[features]\nrelease = {with_jieba:?}\n");
    assert!(
        validate_manifest_features(&toml_jieba).is_err(),
        "manifest with 'jieba' must fail"
    );
}

#[test]
fn test_build_surfaces_release_invocation() {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = base_dir.parent().unwrap().parent().unwrap();
    let surfaces = [
        base_dir.join("build.sh"),
        base_dir.join("Dockerfile"),
        repo_root.join(".github/workflows/lumen-release.yml"),
        repo_root.join(".github/workflows/lumen-test-image.yml"),
    ];

    for path in &surfaces {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        validate_surface_invocation(&content)
            .unwrap_or_else(|e| panic!("surface {} failed validation: {e}", path.display()));
    }

    // Negative tests: table-driven invocation failure modes
    let negative_cases: &[(&str, &str)] = &[
        ("divergent_feature", "cargo build --release -p lumen --bin lumen --features \"otel operator\""),
        ("release_junk", "cargo build --release -p lumen --bin lumen --features release-junk"),
        ("features_equals_release", "cargo build --release -p lumen --bin lumen --features=release"),
        ("missing_features_flag", "cargo build --release -p lumen --bin lumen"),
        ("comment_only", "# cargo build --release -p lumen --bin lumen --features release"),
        ("two_divergent_invocations", "cargo build --release -p lumen --bin lumen --features release\ncargo build --release -p lumen --bin lumen --features other"),
        ("no_invocations", "echo 'no build invocation here'"),
    ];

    for (name, sample) in negative_cases {
        assert!(
            validate_surface_invocation(sample).is_err(),
            "negative invocation case '{name}' must fail"
        );
    }
}

#[test]
fn test_compiled_cfgs_when_release_feature_enabled() {
    #[cfg(feature = "release")]
    {
        assert!(cfg!(feature = "otel"), "otel must be active");
        assert!(cfg!(feature = "operator"), "operator must be active");
        assert!(cfg!(feature = "raft-wal"), "raft-wal must be active");
        assert!(cfg!(feature = "self-update"), "self-update must be active");
        assert!(cfg!(feature = "issue"), "issue must be active");
        assert!(cfg!(feature = "delegated-auth"), "delegated-auth must be active");
        assert!(cfg!(feature = "backup"), "transitive backup must be active");
        assert!(!cfg!(feature = "jieba"), "jieba must NOT be active");
    }
}
