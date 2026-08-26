//! Release feature set consistency test and gate verification.

use std::{io::Write, path::Path, process::Command};

const EXPECTED_DIRECT_FEATURES: &[&str] = &[
    "delegated-auth",
    "issue",
    "operator",
    "otel",
    "raft-wal",
    "self-update",
];
const RELEASE_SKILL_SHA256: &str =
    "cf6fe67d1dddd6880faba54a7e9888cccc3e9c46a3cc2339b7e2b540b551369f";

fn validate_release_skill_order(content: &str) -> Result<(), String> {
    let markers = [
        "2. Run `git:land`",
        "3. Dispatch `lumen-release-candidate`",
        "Wait for the final v3 receipt.",
        "4. Independently run the candidate verifier in full mode.",
        "5. The controller creates one annotated `lumen@<version>` tag",
        "6. If the normal promotion has not run, dispatch `lumen-release`",
        "7. Run the public verifier.",
    ];
    let mut previous = 0;
    for marker in markers {
        let position = content
            .find(marker)
            .ok_or_else(|| format!("release skill is missing {marker:?}"))?;
        if position <= previous {
            return Err(format!("release skill order is invalid at {marker:?}"));
        }
        previous = position;
    }

    for retired in [
        ".claude/skills/lumen-build-release/scripts/release.sh",
        "scripts/project-build-monitor-release.sh",
    ] {
        if content.contains(retired) {
            return Err(format!("retired release route remains: {retired}"));
        }
    }
    validate_no_raw_git_tag_push(content)?;
    Ok(())
}

fn validate_release_skill_pair(agents: &str, claude: &str) -> Result<(), String> {
    if agents != claude {
        return Err("Claude and .agents release skills differ".to_string());
    }
    let digest = sha256_text(agents)?;
    if digest != RELEASE_SKILL_SHA256 {
        return Err(format!(
            "release skill digest changed: expected {RELEASE_SKILL_SHA256}, got {digest}"
        ));
    }
    validate_release_skill_order(agents)
}

fn sha256_text(content: &str) -> Result<String, String> {
    let mut file = tempfile::NamedTempFile::new().map_err(|error| error.to_string())?;
    file.write_all(content.as_bytes())
        .map_err(|error| error.to_string())?;
    let path = file.path();

    let shasum = Command::new("shasum")
        .args(["-a", "256"])
        .arg(path)
        .output();
    if let Ok(output) = shasum {
        if output.status.success() {
            if let Some(digest) = String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .next()
            {
                return Ok(digest.to_string());
            }
        }
    }

    let sha256sum = Command::new("sha256sum").arg(path).output();
    if let Ok(output) = sha256sum {
        if output.status.success() {
            if let Some(digest) = String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .next()
            {
                return Ok(digest.to_string());
            }
        }
    }
    Err("neither shasum nor sha256sum could hash the release skill".to_string())
}

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

fn validate_no_raw_git_tag_push(content: &str) -> Result<(), String> {
    for line in logical_lines(content) {
        let tokens: Vec<&str> = line
            .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            .filter(|token| !token.is_empty())
            .collect();
        if let Some(git) = tokens.iter().position(|token| *token == "git") {
            if tokens[git + 1..]
                .iter()
                .any(|token| *token == "tag" || *token == "push")
            {
                return Err(format!("raw Git tag/push command remains: {line}"));
            }
        }
    }
    Ok(())
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
            let tokens: Vec<String> = part.split_whitespace().map(|s| s.to_string()).collect();

            let has_cargo_build = tokens
                .windows(2)
                .any(|w| w[0] == "cargo" && w[1] == "build");
            let has_release = tokens.iter().any(|t| t == "--release");
            let has_p_lumen = tokens
                .windows(2)
                .any(|w| (w[0] == "-p" || w[0] == "--package") && w[1] == "lumen")
                || tokens
                    .iter()
                    .any(|t| t == "-p=lumen" || t == "--package=lumen");
            let has_bin_lumen = tokens
                .windows(2)
                .any(|w| w[0] == "--bin" && w[1] == "lumen")
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

fn validate_local_release_invocation(content: &str) -> Result<(), String> {
    let mut invocations: Vec<Vec<String>> = Vec::new();
    for line in logical_lines(content) {
        for part in line
            .split("&&")
            .flat_map(|s| s.split("||"))
            .flat_map(|s| s.split(';'))
        {
            let tokens: Vec<String> = part
                .split_whitespace()
                .take_while(|token| !token.starts_with('#'))
                .map(|s| s.trim_matches('"').trim_matches('\'').to_string())
                .collect();
            let is_lumen_release_build = tokens.first().is_some_and(|t| t == "cargo")
                && tokens
                    .windows(2)
                    .any(|w| w[0] == "build" && w[1] == "--release")
                && tokens.windows(2).any(|w| w[0] == "-p" && w[1] == "lumen")
                && tokens
                    .windows(2)
                    .any(|w| w[0] == "--bin" && w[1] == "lumen")
                && tokens
                    .windows(2)
                    .any(|w| w[0] == "--features" && w[1] == "release");
            if is_lumen_release_build {
                invocations.push(tokens);
            }
        }
    }

    if invocations.len() != 1 {
        return Err(format!(
            "expected exactly one local Lumen release build, found {}",
            invocations.len()
        ));
    }
    if !invocations[0].iter().any(|token| token == "--locked") {
        return Err("local release build must use a distinct --locked token".to_string());
    }
    Ok(())
}

fn validate_release_image_pin_contract(content: &str) -> Result<(), String> {
    let version_selection = "CURRENT_VERSION=\"$(project_build_read_version apps/lumen/Cargo.toml)\"\nsync_lumen_release_image_pins \"$CURRENT_VERSION\"";
    content
        .find(version_selection)
        .ok_or_else(|| "local release version preparation is missing".to_string())?;

    for retired in [
        "project_build_prepare_release_version",
        "PROJECT_BUILD_REQUIRE_REMOTE_TAG_CHECK",
        "cargo update",
        "git add ",
        "git commit",
        "git tag",
        "git push",
        "project_build_print_release_next_steps",
    ] {
        if content.contains(retired) {
            return Err(format!("retired release route remains: {retired}"));
        }
    }
    if !content.contains("git land main") || !content.contains("lumen-release-candidate from main")
    {
        return Err("local preparation must direct the next candidate workflow step".to_string());
    }

    for path in [
        "apps/lumen/k8s/base/deployment.yaml",
        "apps/lumen/k8s/operator/deployment.yaml",
    ] {
        if content.matches(path).count() != 1 {
            return Err(format!(
                "release image synchronization must name {path} exactly once"
            ));
        }
    }
    Ok(())
}

fn validate_llms_release_guidance(content: &str) -> Result<(), String> {
    if !content.contains("lumen llm --topic verify-release") {
        return Err("llms release guidance must direct users to verify-release".to_string());
    }
    if !content.contains("does not publish releases") {
        return Err("llms release guidance must not describe build.sh as publication".to_string());
    }
    for stale in [
        "Build release: `./build.sh release`",
        "build.sh release publishes",
        "git tag",
        "git push",
        "gh release create",
        "tag push",
    ] {
        if content.contains(stale) {
            return Err(format!("stale llms release guidance remains: {stale}"));
        }
    }
    Ok(())
}

fn release_image_pin_function(content: &str) -> Result<&str, String> {
    let start = content
        .find("sync_lumen_release_image_pins() {")
        .ok_or_else(|| "release image pin function is missing".to_string())?;
    let tail = &content[start..];
    let end = tail
        .find("\n}\n\nif [[ \"$MODE\"")
        .ok_or_else(|| "release image pin function boundary is missing".to_string())?;
    Ok(&tail[..end + 3])
}

fn run_release_image_pin_function(
    function: &str,
    version: &str,
    first: &Path,
    second: &Path,
) -> std::process::Output {
    let script = format!(
        "set -euo pipefail\n{function}\nsync_lumen_release_image_pins \"$1\" \"$2\" \"$3\"\n"
    );
    Command::new("bash")
        .arg("-c")
        .arg(script)
        .arg("release-image-pin-test")
        .arg(version)
        .arg(first)
        .arg(second)
        .output()
        .expect("run release image pin function")
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
        repo_root.join(".github/workflows/lumen-release-candidate.yml"),
        repo_root.join(".github/workflows/lumen-test-image.yml"),
    ];

    for path in &surfaces {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        validate_surface_invocation(&content)
            .unwrap_or_else(|e| panic!("surface {} failed validation: {e}", path.display()));
    }

    let local_build = std::fs::read_to_string(base_dir.join("build.sh"))
        .expect("read local release build script");
    validate_local_release_invocation(&local_build).expect("local release build must use --locked");

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

    let live_local_build = std::fs::read_to_string(base_dir.join("build.sh"))
        .expect("read local release build script");
    let expected_locked = "cargo build --release --locked -p lumen --bin lumen --features release";
    let unlocked_local_build = live_local_build.replace(
        expected_locked,
        "cargo build --release -p lumen --bin lumen --features release",
    );
    let unlocked_with_comment = format!("{unlocked_local_build}\n# {expected_locked}\n");
    assert_ne!(
        unlocked_with_comment, live_local_build,
        "the unlocked/comment mutation must change the live local release build"
    );
    assert!(
        validate_local_release_invocation(&unlocked_with_comment).is_err(),
        "an unlocked real command plus a commented locked command must fail validation"
    );

    let inline_comment_mutation = live_local_build.replace(
        expected_locked,
        "cargo build --release -p lumen --bin lumen --features release # --locked",
    );
    assert_ne!(
        inline_comment_mutation, live_local_build,
        "the inline-comment mutation must change the live local release build"
    );
    assert!(
        validate_local_release_invocation(&inline_comment_mutation).is_err(),
        "an inline comment must not satisfy the --locked requirement"
    );

    let hash_comment_mutation = live_local_build.replace(
        expected_locked,
        "cargo build --release -p lumen --bin lumen --features release #comment --locked",
    );
    assert_ne!(
        hash_comment_mutation, live_local_build,
        "the hash-comment mutation must change the live local release build"
    );
    assert!(
        validate_local_release_invocation(&hash_comment_mutation).is_err(),
        "a #comment token must not satisfy the --locked requirement"
    );
}

#[test]
fn test_release_preparation_updates_all_checked_in_image_pins() {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let build_script =
        std::fs::read_to_string(base_dir.join("build.sh")).expect("read apps/lumen/build.sh");
    validate_release_image_pin_contract(&build_script)
        .expect("release preparation must update every checked-in image pin");

    for path in [
        base_dir.join("k8s/base/deployment.yaml"),
        base_dir.join("k8s/operator/deployment.yaml"),
    ] {
        let manifest = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        assert!(
            manifest.contains(&format!(
                "image: ghcr.io/chrischeng-c4/lumen:{}",
                env!("CARGO_PKG_VERSION")
            )),
            "{} must pin the workspace Lumen version {}",
            path.display(),
            env!("CARGO_PKG_VERSION")
        );
    }

    let missing_base = build_script.replace("  apps/lumen/k8s/base/deployment.yaml \\\n", "");
    assert!(validate_release_image_pin_contract(&missing_base).is_err());

    let stale_version = build_script.replace(
        "sync_lumen_release_image_pins \"$CURRENT_VERSION\"",
        "sync_lumen_release_image_pins \"$PROJECT_BUILD_RELEASE_VERSION\"",
    );
    assert!(validate_release_image_pin_contract(&stale_version).is_err());

    let before_selection = build_script.replace(
        "CURRENT_VERSION=\"$(project_build_read_version apps/lumen/Cargo.toml)\"\nsync_lumen_release_image_pins",
        "sync_lumen_release_image_pins",
    );
    assert!(validate_release_image_pin_contract(&before_selection).is_err());

    let function = release_image_pin_function(&build_script).expect("extract image pin function");
    let temp = tempfile::tempdir().expect("create image pin fixture directory");
    let standalone = temp.path().join("standalone.yaml");
    let operator = temp.path().join("operator.yaml");
    std::fs::write(
        &standalone,
        "image: ghcr.io/chrischeng-c4/lumen:0.4.26\nimage: ghcr.io/example/sidecar:9\n",
    )
    .expect("write standalone fixture");
    std::fs::write(&operator, "  image: ghcr.io/chrischeng-c4/lumen:0.4.25\n")
        .expect("write operator fixture");

    let updated = run_release_image_pin_function(function, "0.4.28", &standalone, &operator);
    assert!(
        updated.status.success(),
        "image pin function failed: {}",
        String::from_utf8_lossy(&updated.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(&standalone).expect("read updated standalone fixture"),
        "image: ghcr.io/chrischeng-c4/lumen:0.4.28\nimage: ghcr.io/example/sidecar:9\n"
    );
    assert_eq!(
        std::fs::read_to_string(&operator).expect("read updated operator fixture"),
        "  image: ghcr.io/chrischeng-c4/lumen:0.4.28\n"
    );

    let missing = temp.path().join("missing.yaml");
    std::fs::write(&missing, "image: ghcr.io/example/sidecar:9\n")
        .expect("write missing-pin fixture");
    let missing_result = run_release_image_pin_function(function, "0.4.28", &missing, &operator);
    assert!(
        !missing_result.status.success(),
        "zero Lumen pins must fail"
    );

    let duplicate = temp.path().join("duplicate.yaml");
    std::fs::write(
        &duplicate,
        "image: ghcr.io/chrischeng-c4/lumen:0.4.25\nimage: ghcr.io/chrischeng-c4/lumen:0.4.26\n",
    )
    .expect("write duplicate-pin fixture");
    let duplicate_result =
        run_release_image_pin_function(function, "0.4.28", &duplicate, &operator);
    assert!(
        !duplicate_result.status.success(),
        "duplicate Lumen pins must fail"
    );
}

#[test]
fn test_release_preparation_rejects_retired_routes_and_stale_llms_guidance() {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let build_script = std::fs::read_to_string(base_dir.join("build.sh")).expect("read build.sh");
    let llms = std::fs::read_to_string(base_dir.join("llms.txt")).expect("read llms.txt");
    validate_release_image_pin_contract(&build_script).expect("live build route is current");
    validate_llms_release_guidance(&llms).expect("live llms guidance is current");

    for retired in [
        "git add Cargo.lock apps/lumen",
        "git commit --allow-empty -m \"release(lumen): ${TAG}\"",
        "git push origin lumen@0.4.28",
        "project_build_prepare_release_version lumen",
    ] {
        let mutated = format!("{build_script}\n{retired}\n");
        assert!(
            validate_release_image_pin_contract(&mutated).is_err(),
            "retired build route must fail: {retired}"
        );
    }

    let canonical_prefix = "- Prepare and verify a release:";
    assert_eq!(
        llms.lines()
            .filter(|line| line.starts_with(canonical_prefix))
            .count(),
        1,
        "live llms guidance must contain exactly one canonical release line"
    );
    let canonical_line = llms
        .lines()
        .find(|line| line.starts_with(canonical_prefix))
        .expect("canonical llms release line is present");

    for stale in [
        "- Build release: `./build.sh release`.",
        "- Prepare release: `./build.sh release` publishes the release.",
        "- [build.sh](build.sh): release publication entrypoint.",
    ] {
        let mutated = llms.replacen(canonical_line, stale, 1);
        assert_ne!(
            mutated, llms,
            "stale llms release route mutation must change the fixture"
        );
        assert!(
            validate_llms_release_guidance(&mutated).is_err(),
            "stale llms release route must fail: {stale}"
        );
    }

    for retired in [
        "git tag lumen@0.4.28",
        "git push origin main",
        "gh release create",
        "tag push",
    ] {
        let mutated = format!("{llms}\n{retired}\n");
        assert_ne!(
            mutated, llms,
            "retired LLM route mutation must change the fixture"
        );
        assert!(
            validate_llms_release_guidance(&mutated).is_err(),
            "retired LLM publication route must fail: {retired}"
        );
    }
}

#[test]
fn test_release_skill_entrypoints_are_identical_and_candidate_first() {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = base_dir.parent().unwrap().parent().unwrap();
    let agents =
        std::fs::read_to_string(repo_root.join(".agents/skills/lumen-build-release/SKILL.md"))
            .expect("read .agents release skill");
    let claude =
        std::fs::read_to_string(repo_root.join(".claude/skills/lumen-build-release/SKILL.md"))
            .expect("read .claude release skill");

    validate_release_skill_pair(&agents, &claude)
        .expect("release skill entrypoints must be identical and candidate-first");

    let escaped_agents = agents.replace(
        "3. Dispatch `lumen-release-candidate`",
        "g\\it tag -m release lumen@<version>; g\\it push --tags\n3. Dispatch `lumen-release-candidate`",
    );
    let escaped_claude = claude.replace(
        "3. Dispatch `lumen-release-candidate`",
        "g\\it tag -m release lumen@<version>; g\\it push --tags\n3. Dispatch `lumen-release-candidate`",
    );
    assert_ne!(
        escaped_agents, agents,
        "shell-escape fixture must change the skill bytes"
    );
    assert_eq!(
        escaped_agents, escaped_claude,
        "parity fixture must keep both entrypoints equal"
    );
    assert!(
        validate_release_skill_pair(&escaped_agents, &escaped_claude).is_err(),
        "same-byte shell-escape fixture must fail the fixed digest oracle"
    );

    let drift = agents.replace(
        "Publish one verified Lumen release",
        "Publish a different verified Lumen release",
    );
    assert_ne!(drift, agents, "skill drift fixture must change bytes");
    assert!(
        validate_release_skill_pair(&agents, &drift).is_err(),
        "skill drift must fail the parity oracle"
    );

    let candidate_verifier_step =
        "4. Independently run the candidate verifier in full mode. Stop on any mismatch.\n";
    let tag_step = "5. The controller creates one annotated `lumen@<version>` tag at the exact\n";
    let tag_first = agents
        .replace(candidate_verifier_step, "")
        .replace(tag_step, &format!("{tag_step}{candidate_verifier_step}"));
    assert_ne!(
        tag_first, agents,
        "tag-first fixture must move the complete candidate-verifier step"
    );
    assert!(
        validate_release_skill_order(&tag_first).is_err(),
        "tag-first fixture must fail the release-order oracle"
    );

    let receipt = "Wait for the final v3 receipt.";
    let receipt_after_tag = agents.replace(receipt, "").replace(
        "5. The controller creates one annotated `lumen@<version>` tag",
        &format!("5. The controller creates one annotated `lumen@<version>` tag\n{receipt}"),
    );
    assert_ne!(
        receipt_after_tag, agents,
        "receipt fixture must move the receipt after the annotated tag"
    );
    assert!(
        validate_release_skill_order(&receipt_after_tag).is_err(),
        "receipt-after-tag fixture must fail the release-order oracle"
    );

    let raw_git_before_candidate = agents.replace(
        "3. Dispatch `lumen-release-candidate`",
        "git tag -m release lumen@<version>; git push --tags\n3. Dispatch `lumen-release-candidate`",
    );
    assert_ne!(
        raw_git_before_candidate, agents,
        "raw Git tag/push fixture must change the skill bytes"
    );
    assert!(
        validate_release_skill_order(&raw_git_before_candidate).is_err(),
        "any raw Git tag/push command before the candidate must fail"
    );

    let raw_git_with_global_options = agents.replace(
        "3. Dispatch `lumen-release-candidate`",
        "git -C . tag -m release lumen@<version>; git -C . push --tags\n3. Dispatch `lumen-release-candidate`",
    );
    assert_ne!(
        raw_git_with_global_options, agents,
        "global-option raw Git fixture must change the skill bytes"
    );
    assert!(
        validate_release_skill_order(&raw_git_with_global_options).is_err(),
        "global-option raw Git tag/push must fail"
    );

    let raw_git_with_c_and_continuation = agents.replace(
        "3. Dispatch `lumen-release-candidate`",
        concat!(
            "git \\\n",
            "-C . \\\n",
            "tag -m release lumen@<version>\n3. Dispatch `lumen-release-candidate`"
        ),
    );
    assert_ne!(
        raw_git_with_c_and_continuation, agents,
        "git -C backslash fixture must change the skill bytes"
    );
    assert!(
        validate_release_skill_order(&raw_git_with_c_and_continuation).is_err(),
        "git -C backslash tag must fail"
    );

    let raw_git_with_git_dir_and_continuation = agents.replace(
        "3. Dispatch `lumen-release-candidate`",
        concat!(
            "git \\\n",
            "--git-dir=.git \\\n",
            "push --tags\n3. Dispatch `lumen-release-candidate`"
        ),
    );
    assert_ne!(
        raw_git_with_git_dir_and_continuation, agents,
        "git --git-dir backslash fixture must change the skill bytes"
    );
    assert!(
        validate_release_skill_order(&raw_git_with_git_dir_and_continuation).is_err(),
        "git --git-dir backslash push must fail"
    );
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
        assert!(
            cfg!(feature = "delegated-auth"),
            "delegated-auth must be active"
        );
        assert!(cfg!(feature = "backup"), "transitive backup must be active");
        assert!(!cfg!(feature = "jieba"), "jieba must NOT be active");
    }
}
