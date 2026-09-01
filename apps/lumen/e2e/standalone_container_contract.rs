//! Deterministic contract test for Lumen standalone container and binary defaults.

use std::{fs, io::Write, path::PathBuf, process::Command};

const EXPECTED_DOCKERFILES: &[&str] = &[
    "apps/lumen/Dockerfile",
    "apps/lumen/Dockerfile.release",
    "apps/lumen/Dockerfile.test",
];

fn repo_root() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
        .unwrap_or(dir)
}

fn discover_dockerfiles() -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(repo_root().join("apps/lumen")) {
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("Dockerfile") {
                files.push(format!("apps/lumen/{name}"));
            }
        }
    }
    files.sort();
    files
}

fn replace_exact(source: &str, target: &str, replacement: &str) -> String {
    let count = source.matches(target).count();
    assert_eq!(
        count, 1,
        "expected target {target:?} to occur exactly once, found {count}"
    );
    source.replacen(target, replacement, 1)
}

fn insert_after_first_from(source: &str, instruction: &str) -> String {
    let mut output = String::with_capacity(source.len() + instruction.len() + 1);
    let mut inserted = false;
    for line in source.split_inclusive('\n') {
        output.push_str(line);
        if !inserted && line.trim_start().to_ascii_uppercase().starts_with("FROM ") {
            output.push_str(instruction);
            output.push('\n');
            inserted = true;
        }
    }
    assert!(inserted, "fixture has no FROM instruction");
    output
}

const DURABLE_BEGIN: &str = "  # DURABLE-CONTRACT-BEGIN\n";
const DURABLE_END: &str = "  # DURABLE-CONTRACT-END\n";
const DURABLE_BLOCK_SHA256: &str =
    "d269cc1b8bd8174d5ef97a158daca836f355a9d8ffc842492c297884c5783b4b";
const CANDIDATE_ROOT_REGEX: &str = "^ghcr\\.io/chrischeng-c4/lumen@sha256:[0-9a-f]{64}$";
const OLD_IMAGE: &str =
    "ghcr.io/chrischeng-c4/lumen@sha256:59a85c96d807428c424ec8889ac830b14e02869da49c4b44ae12dcce3786d03d";
const DATA_MOUNT: &str = "--mount \"type=volume,src=$VOLUME,dst=/var/lib/lumen/data\"";
const REJECT_DATA_MOUNT: &str =
    "--mount \"type=volume,src=$REJECT_VOLUME,dst=/var/lib/lumen/data\"";
const OLD_RUN: &str = concat!(
    "docker run -d --name \"$OLD_CONTAINER\" ",
    "--mount \"type=volume,src=$VOLUME,dst=/var/lib/lumen/data\" ",
    "-e LUMEN_AUTH=off -e LUMEN_DATA_DIR=/var/lib/lumen/data -e LUMEN_PERSISTENCE=segment -e LUMEN_SNAPSHOT_SECS=1 -e LUMEN_GRACE_SECS=1 ",
    "-p 127.0.0.1::7373 \"$OLD_IMAGE\" >/dev/null"
);
const CANDIDATE_RUN: &str = concat!(
    "docker run -d --name \"$CANDIDATE_CONTAINER\" ",
    "--mount \"type=volume,src=$VOLUME,dst=/var/lib/lumen/data\" ",
    "-e LUMEN_AUTH=off -p 127.0.0.1::7373 ",
    "\"$LUMEN_STANDALONE_DURABLE_IMAGE\" >/dev/null"
);
const REPLACEMENT_RUN: &str = concat!(
    "docker run -d --name \"$REPLACEMENT_CONTAINER\" ",
    "--mount \"type=volume,src=$VOLUME,dst=/var/lib/lumen/data\" ",
    "-e LUMEN_AUTH=off -p 127.0.0.1::7373 ",
    "\"$LUMEN_STANDALONE_DURABLE_IMAGE\" >/dev/null"
);
const REJECTED_RUN: &str = concat!(
    "docker run -d --name \"$REJECTED_CONTAINER\" ",
    "--mount \"type=volume,src=$REJECT_VOLUME,dst=/var/lib/lumen/data\" ",
    "-e LUMEN_AUTH=off -p 127.0.0.1::7373 ",
    "\"$LUMEN_STANDALONE_DURABLE_IMAGE\" >/dev/null"
);
const CANDIDATE_RUN_SOURCE: &str = concat!(
    "  docker run -d --name \"$CANDIDATE_CONTAINER\" ",
    "--mount \"type=volume,src=$VOLUME,dst=/var/lib/lumen/data\" -e LUMEN_AUTH=off \\\n",
    "    -p 127.0.0.1::7373 \"$LUMEN_STANDALONE_DURABLE_IMAGE\" >/dev/null\n"
);
const REPLACEMENT_RUN_SOURCE: &str = concat!(
    "  docker run -d --name \"$REPLACEMENT_CONTAINER\" ",
    "--mount \"type=volume,src=$VOLUME,dst=/var/lib/lumen/data\" -e LUMEN_AUTH=off \\\n",
    "    -p 127.0.0.1::7373 \"$LUMEN_STANDALONE_DURABLE_IMAGE\" >/dev/null\n"
);
const CANDIDATE_AUTH_LINE: &str = concat!(
    "  docker run -d --name \"$CANDIDATE_CONTAINER\" ",
    "--mount \"type=volume,src=$VOLUME,dst=/var/lib/lumen/data\" -e LUMEN_AUTH=off \\\n"
);
const CURRENT_FIRST_ASSERT: &str = r#"python3 -c 'import pathlib,re,sys; b=pathlib.Path(sys.argv[1]).read_bytes(); sys.exit(0 if re.fullmatch(rb"generation:gen-[0-9]+\n",b) else 1)' "$TEMP_DIR/current-first""#;
const CURRENT_REV_ASSERT: &str = r#"python3 -c 'import pathlib,re,sys; b=pathlib.Path(sys.argv[1]).read_bytes(); sys.exit(0 if re.fullmatch(rb"generation:gen-[0-9]+-rev-[1-9][0-9]*\n",b) else 1)' "$TEMP_DIR/current-rev""#;
const CURRENT_REPLACEMENT_ASSERT: &str = r#"python3 -c 'import pathlib,re,sys; b=pathlib.Path(sys.argv[1]).read_bytes(); sys.exit(0 if re.fullmatch(rb"generation:gen-[0-9]+-rev-[1-9][0-9]*\n",b) else 1)' "$TEMP_DIR/current-replacement""#;

fn durable_script_block(source: &str) -> Result<&str, String> {
    let starts = source.matches(DURABLE_BEGIN).count();
    if starts != 1 {
        return Err(format!(
            "durable block must have one start marker, found {starts}"
        ));
    }
    let ends = source.matches(DURABLE_END).count();
    if ends != 1 {
        return Err(format!(
            "durable block must have one end marker, found {ends}"
        ));
    }
    let start = source
        .find(DURABLE_BEGIN)
        .ok_or_else(|| "durable block start marker is missing".to_string())?
        + DURABLE_BEGIN.len();
    let end = source[start..]
        .find(DURABLE_END)
        .map(|offset| start + offset)
        .ok_or_else(|| "durable block end marker is before its start".to_string())?;
    let block = &source[start..end];
    if block.trim().is_empty() {
        return Err("durable block is empty".to_string());
    }
    Ok(block)
}

fn strip_shell_comment(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let mut quote = None;
    let mut escaped = false;
    let mut token_start = true;
    for ch in raw.chars() {
        if escaped {
            output.push(ch);
            escaped = false;
            token_start = ch.is_whitespace();
            continue;
        }
        match quote {
            Some('\'') => {
                output.push(ch);
                if ch == '\'' {
                    quote = None;
                }
                token_start = false;
            }
            Some('"') => {
                output.push(ch);
                if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    quote = None;
                }
                token_start = false;
            }
            None if ch == '\\' => {
                output.push(ch);
                escaped = true;
                token_start = false;
            }
            None if ch == '\'' || ch == '"' => {
                output.push(ch);
                quote = Some(ch);
                token_start = false;
            }
            None if ch == '#' && token_start => break,
            None => {
                output.push(ch);
                token_start = ch.is_whitespace();
            }
            Some(other) => unreachable!("unsupported shell quote delimiter: {other}"),
        }
    }
    output
}

fn has_unescaped_continuation(line: &str) -> bool {
    let slash_count = line.chars().rev().take_while(|ch| *ch == '\\').count();
    slash_count % 2 == 1
}

fn durable_logical_lines(block: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut pending = String::new();
    for raw in block.lines() {
        let active = strip_shell_comment(raw);
        let trimmed = active.trim();
        if trimmed.is_empty() {
            continue;
        }
        let continued = has_unescaped_continuation(trimmed);
        let part = if continued {
            trimmed.strip_suffix('\\').expect("continuation suffix")
        } else {
            trimmed
        };
        if !pending.is_empty() {
            pending.push(' ');
        }
        pending.push_str(part.trim_end());
        if continued {
            continue;
        }
        lines.push(std::mem::take(&mut pending));
    }
    if !pending.is_empty() {
        lines.push(pending);
    }
    lines
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
    Err("neither shasum nor sha256sum could hash the durable block".to_string())
}

fn require_exact_once(lines: &[String], expected: &str, detail: &str) -> Result<usize, String> {
    let matches: Vec<_> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| (line == expected).then_some(index))
        .collect();
    if matches.len() != 1 {
        return Err(format!(
            "{detail}: expected one active command, found {}",
            matches.len()
        ));
    }
    Ok(matches[0])
}

fn require_active_contains(lines: &[String], expected: &str, detail: &str) -> Result<(), String> {
    if lines.iter().any(|line| line.contains(expected)) {
        Ok(())
    } else {
        Err(format!("{detail}: missing active text {expected:?}"))
    }
}

fn require_before(
    lines: &[String],
    assignment: &str,
    command: &str,
    detail: &str,
) -> Result<(), String> {
    let assignment = require_exact_once(lines, assignment, detail)?;
    let command = require_exact_once(lines, command, detail)?;
    if assignment < command {
        Ok(())
    } else {
        Err(format!(
            "{detail}: container is not tracked before docker run"
        ))
    }
}

fn validate_durable_script_semantics(source: &str) -> Result<(), String> {
    let block = durable_script_block(source)?;
    let lines = durable_logical_lines(block);

    require_active_contains(&lines, CANDIDATE_ROOT_REGEX, "candidate root digest")?;
    require_exact_once(
        &lines,
        &format!("OLD_IMAGE=\"{OLD_IMAGE}\""),
        "old root digest",
    )?;
    require_exact_once(
        &lines,
        "VOLUME=\"lumen-smoke-durable-${ID_SUFFIX}\"",
        "run volume",
    )?;
    require_exact_once(
        &lines,
        "REJECT_VOLUME=\"lumen-smoke-durable-reject-${ID_SUFFIX}\"",
        "reject volume",
    )?;
    require_exact_once(
        &lines,
        "if docker volume inspect \"$VOLUME\" >/dev/null 2>&1; then",
        "volume ownership check",
    )?;
    require_exact_once(
        &lines,
        "docker volume create \"$VOLUME\" >/dev/null",
        "volume creation",
    )?;
    require_exact_once(
        &lines,
        "if docker volume inspect \"$REJECT_VOLUME\" >/dev/null 2>&1; then",
        "reject volume ownership check",
    )?;
    require_exact_once(
        &lines,
        "docker volume create \"$REJECT_VOLUME\" >/dev/null",
        "reject volume creation",
    )?;
    require_exact_once(&lines, "CREATED_VOLUME=1", "owned volume tracking")?;
    require_exact_once(
        &lines,
        "CREATED_REJECT_VOLUME=1",
        "owned reject volume tracking",
    )?;
    require_exact_once(
        &lines,
        "if [[ \"$CREATED_VOLUME\" == 1 ]]; then",
        "owned volume cleanup",
    )?;
    require_exact_once(
        &lines,
        "if [[ \"$CREATED_REJECT_VOLUME\" == 1 ]]; then",
        "owned reject volume cleanup",
    )?;
    require_exact_once(
        &lines,
        "if ! docker volume rm \"$VOLUME\" >/dev/null 2>&1; then",
        "volume cleanup",
    )?;
    require_exact_once(
        &lines,
        "if ! docker volume rm \"$REJECT_VOLUME\" >/dev/null 2>&1; then",
        "reject volume cleanup",
    )?;
    require_exact_once(
        &lines,
        "if ! docker rm -f \"$container\" >/dev/null 2>&1; then",
        "container cleanup",
    )?;
    require_exact_once(
        &lines,
        "if [[ -n \"$TEMP_DIR\" ]] && ! rm -rf -- \"$TEMP_DIR\"; then",
        "temporary cleanup",
    )?;
    require_active_contains(
        &lines,
        "if [[ \"$exit_code\" -ne 0 ]]; then",
        "main failure preservation",
    )?;
    require_active_contains(
        &lines,
        "if [[ \"$cleanup_failed\" -ne 0 ]]; then",
        "cleanup failure",
    )?;
    require_exact_once(&lines, "trap cleanup_durable EXIT", "cleanup exit trap")?;
    require_exact_once(&lines, "trap 'exit 130' INT", "INT signal trap")?;
    require_exact_once(&lines, "trap 'exit 143' TERM", "TERM signal trap")?;

    require_before(&lines, "CREATED_OLD=\"$OLD_CONTAINER\"", OLD_RUN, "old run")?;
    require_before(
        &lines,
        "CREATED_CANDIDATE=\"$CANDIDATE_CONTAINER\"",
        CANDIDATE_RUN,
        "candidate run",
    )?;
    require_before(
        &lines,
        "CREATED_REPLACEMENT=\"$REPLACEMENT_CONTAINER\"",
        REPLACEMENT_RUN,
        "replacement run",
    )?;
    require_before(
        &lines,
        "CREATED_REJECTED=\"$REJECTED_CONTAINER\"",
        REJECTED_RUN,
        "rejected run",
    )?;
    require_exact_once(&lines, OLD_RUN, "old image command")?;
    require_exact_once(&lines, CANDIDATE_RUN, "candidate image command")?;
    require_exact_once(&lines, REPLACEMENT_RUN, "replacement image command")?;
    require_exact_once(&lines, REJECTED_RUN, "rejected image command")?;
    if lines
        .iter()
        .filter(|line| line.contains(DATA_MOUNT))
        .count()
        != 3
    {
        return Err("each run must mount only the named volume at the data path".to_string());
    }
    if lines
        .iter()
        .filter(|line| line.contains(REJECT_DATA_MOUNT))
        .count()
        != 2
    {
        return Err(
            "reject seed and run must mount only the reject volume at the data path".to_string(),
        );
    }
    require_exact_once(
        &lines,
        "docker kill \"$CANDIDATE_CONTAINER\" >/dev/null",
        "hard-kill proof",
    )?;
    require_exact_once(
        &lines,
        "docker rm \"$CANDIDATE_CONTAINER\" >/dev/null",
        "candidate removal",
    )?;
    require_exact_once(
        &lines,
        "docker create --name \"$SEED_CONTAINER\" --mount \"type=volume,src=$REJECT_VOLUME,dst=/var/lib/lumen/data\" \"$LUMEN_STANDALONE_DURABLE_IMAGE\" >/dev/null",
        "reject volume seed container",
    )?;
    require_exact_once(
        &lines,
        "docker cp \"$TEMP_DIR/foreign-layout\" \"$SEED_CONTAINER:/var/lib/lumen/data/foreign-layout\"",
        "reject volume unknown-root seed",
    )?;
    require_exact_once(
        &lines,
        "docker rm \"$SEED_CONTAINER\" >/dev/null",
        "reject seed removal",
    )?;
    require_exact_once(
        &lines,
        "wait_for_exit \"$REJECTED_CONTAINER\"",
        "rejected startup exit",
    )?;
    require_exact_once(
        &lines,
        "[[ \"$exit_status\" -ne 0 ]] && return 0",
        "rejected startup nonzero exit",
    )?;
    require_exact_once(
        &lines,
        "if curl -fsS --noproxy '*' --connect-timeout 2 --max-time 5 \"http://127.0.0.1:${REJECTED_PORT}/readyz\" -o \"$TEMP_DIR/rejected-ready\" >/dev/null 2>&1; then",
        "rejected startup never ready",
    )?;
    require_exact_once(
        &lines,
        "grep -Eq 'segment checkpoint root entry.*refusing to initialize CURRENT' \"$TEMP_DIR/rejected.log\"",
        "rejected startup reason",
    )?;
    require_exact_once(
        &lines,
        "[[ ! -e \"$TEMP_DIR/rejected-data/CURRENT\" ]]",
        "rejected CURRENT absence",
    )?;
    require_exact_once(
        &lines,
        "cmp -s \"$TEMP_DIR/foreign-layout/sentinel\" \"$TEMP_DIR/rejected-data/foreign-layout/sentinel\"",
        "rejected unknown-root preservation",
    )?;
    require_exact_once(
        &lines,
        "grep -Eq 'segment checkpoint startup decision.*decision=\\\"?adopted_legacy_0428\\\"?' \"$TEMP_DIR/candidate.log\"",
        "legacy adoption startup decision",
    )?;
    require_exact_once(
        &lines,
        "grep -Eq 'segment checkpoint startup decision.*decision=\\\"?restored_current_generation\\\"?' \"$TEMP_DIR/replacement.log\"",
        "current generation startup decision",
    )?;

    require_exact_once(
        &lines,
        "[[ -d \"$1\" ]] || return 1",
        "legacy checkpoint directory",
    )?;
    require_exact_once(
        &lines,
        "[[ ! -e \"$1/CURRENT\" ]] || return 1",
        "legacy CURRENT absence",
    )?;
    require_exact_once(
        &lines,
        "[[ \"$base\" =~ ^gen-[0-9]+$ ]] && return 0",
        "legacy generation name",
    )?;
    require_exact_once(
        &lines,
        "has_legacy_generation \"$TEMP_DIR/old-data\" && break",
        "legacy checkpoint adoption",
    )?;
    require_active_contains(&lines, "/readyz", "readiness proof")?;
    require_exact_once(
        &lines,
        "request -X POST \"$url/collections/durable/search\" -H 'Content-Type: application/json' -d \"{\\\"query\\\":{\\\"term\\\":{\\\"field\\\":\\\"tag\\\",\\\"value\\\":\\\"$value\\\"}},\\\"limit\\\":10}\" -o \"$response\"",
        "search request",
    )?;
    require_exact_once(
        &lines,
        "jq -e --arg id \"durable-${value}\" '.total == 1 and (.hits | length) == 1 and .hits[0].external_id == $id' \"$response\" >/dev/null",
        "search response assertion",
    )?;
    require_exact_once(
        &lines,
        "request -X PUT \"$OLD_URL/collections/durable\" -H 'Content-Type: application/json' -d '{\"fields\":{\"tag\":{\"type\":\"keyword\"}}}' -o \"$TEMP_DIR/create\"",
        "collection creation",
    )?;
    require_exact_once(
        &lines,
        "request -X POST \"$OLD_URL/collections/durable/index\" -H 'Content-Type: application/json' -d '{\"items\":[{\"external_id\":\"durable-first\",\"field\":\"tag\",\"value\":\"first\"}]}' -o \"$TEMP_DIR/index-first\"",
        "first record",
    )?;
    require_exact_once(
        &lines,
        "request -X POST \"$CANDIDATE_URL/collections/durable/index\" -H 'Content-Type: application/json' -d '{\"items\":[{\"external_id\":\"durable-second\",\"field\":\"tag\",\"value\":\"second\"}]}' -o \"$TEMP_DIR/index-second\"",
        "second record",
    )?;
    for search in [
        "assert_search \"$OLD_URL\" first",
        "assert_search \"$CANDIDATE_URL\" first",
        "assert_search \"$CANDIDATE_URL\" second",
        "assert_search \"$REPLACEMENT_URL\" first",
        "assert_search \"$REPLACEMENT_URL\" second",
    ] {
        require_exact_once(&lines, search, "search proof")?;
    }
    require_exact_once(
        &lines,
        "docker cp \"$CANDIDATE_CONTAINER:/var/lib/lumen/data/CURRENT\" \"$TEMP_DIR/current-first\"",
        "CURRENT adoption",
    )?;
    require_exact_once(&lines, CURRENT_FIRST_ASSERT, "CURRENT adoption bytes")?;
    require_exact_once(
        &lines,
        "request -X POST \"$CANDIDATE_URL/admin/checkpoint\" -H 'Content-Type: application/json' -d '{}' -o \"$TEMP_DIR/checkpoint\"",
        "checkpoint request",
    )?;
    require_exact_once(
        &lines,
        "jq -e '.persisted == true' \"$TEMP_DIR/checkpoint\" >/dev/null",
        "persisted checkpoint",
    )?;
    require_exact_once(
        &lines,
        "docker cp \"$CANDIDATE_CONTAINER:/var/lib/lumen/data/CURRENT\" \"$TEMP_DIR/current-rev\"",
        "candidate revision copy",
    )?;
    require_exact_once(
        &lines,
        "docker cp \"$REPLACEMENT_CONTAINER:/var/lib/lumen/data/CURRENT\" \"$TEMP_DIR/current-replacement\"",
        "replacement revision copy",
    )?;
    require_exact_once(&lines, CURRENT_REV_ASSERT, "candidate revision bytes")?;
    require_exact_once(
        &lines,
        CURRENT_REPLACEMENT_ASSERT,
        "replacement revision bytes",
    )?;

    for line in &lines {
        if line.contains("if false") || line.contains("false &&") || line.contains("|| true") {
            return Err(format!("dead or bypassed durable proof: {line}"));
        }
        if [
            "docker build",
            "docker rmi",
            "docker system prune",
            "docker container prune",
            "docker volume prune",
        ]
        .iter()
        .any(|forbidden| line.contains(forbidden))
        {
            return Err(format!(
                "durable block has forbidden cleanup or image action: {line}"
            ));
        }
        if line.starts_with("docker rm -f ")
            && line != "docker rm -f \"$OLD_CONTAINER\" >/dev/null"
            && line != "if ! docker rm -f \"$container\" >/dev/null 2>&1; then"
        {
            return Err(format!(
                "durable block has unscoped container cleanup: {line}"
            ));
        }
        if line.starts_with("docker volume rm ")
            && line != "if ! docker volume rm \"$VOLUME\" >/dev/null 2>&1; then"
            && line != "if ! docker volume rm \"$REJECT_VOLUME\" >/dev/null 2>&1; then"
        {
            return Err(format!("durable block has unscoped volume cleanup: {line}"));
        }
        if (line.starts_with("echo ") || line.starts_with("cat "))
            && (line.contains("$response")
                || line.contains("$TEMP_DIR/search")
                || line.contains("/CURRENT")
                || line.contains("$TEMP_DIR/current"))
        {
            return Err(format!(
                "durable block exposes private response bytes: {line}"
            ));
        }
    }

    Ok(())
}

fn validate_durable_script_bytes(source: &str) -> Result<(), String> {
    let block = durable_script_block(source)?;
    let digest = sha256_text(block)?;
    if digest != DURABLE_BLOCK_SHA256 {
        return Err(format!(
            "durable block digest changed: expected {DURABLE_BLOCK_SHA256}, got {digest}"
        ));
    }
    Ok(())
}

fn validate_durable_script(source: &str) -> Result<(), String> {
    validate_durable_script_semantics(source)?;
    validate_durable_script_bytes(source)
}

fn insert_after_exact(source: &str, target: &str, addition: &str) -> String {
    replace_exact(source, target, &format!("{target}{addition}"))
}

fn candidate_with_override(source: &str, variable: &str) -> String {
    replace_exact(
        source,
        CANDIDATE_AUTH_LINE,
        &format!(
            "  docker run -d --name \"$CANDIDATE_CONTAINER\" {DATA_MOUNT} -e LUMEN_AUTH=off -e {variable}=forbidden \\\n"
        ),
    )
}

fn assert_durable_rejected(label: &str, source: String) {
    assert!(
        validate_durable_script_semantics(&source).is_err(),
        "mutation must fail: {label}"
    );
}

#[test]
fn test_durable_script_contract_and_negative_mutations() {
    let path = repo_root().join("apps/lumen/scripts/standalone-container-smoke.sh");
    let source = fs::read_to_string(path).expect("read durable smoke script");
    validate_durable_script(&source).expect("durable script contract");

    let byte_drift = insert_after_exact(&source, DURABLE_BEGIN, "  # reviewed byte drift\n");
    validate_durable_script_semantics(&byte_drift)
        .expect("a comment-only byte drift keeps the durable semantics");
    assert!(
        validate_durable_script_bytes(&byte_drift).is_err(),
        "durable block digest must reject a generic byte drift"
    );

    assert_durable_rejected(
        "full-line comment",
        replace_exact(
            &source,
            "  assert_search \"$OLD_URL\" first\n",
            "  # assert_search \"$OLD_URL\" first\n",
        ),
    );
    assert_durable_rejected(
        "inline comment truncation",
        replace_exact(
            &source,
            "  assert_search \"$CANDIDATE_URL\" first\n",
            "  assert_search # \"$CANDIDATE_URL\" first\n",
        ),
    );
    assert_durable_rejected(
        "quoted prose",
        replace_exact(
            &source,
            "  assert_search \"$CANDIDATE_URL\" second\n",
            "  : 'assert_search \"$CANDIDATE_URL\" second'\n",
        ),
    );
    assert_durable_rejected(
        "if-false dead branch",
        replace_exact(
            &source,
            "  assert_search \"$REPLACEMENT_URL\" first\n",
            "  if false; then assert_search \"$REPLACEMENT_URL\" first; fi\n",
        ),
    );
    assert_durable_rejected(
        "false-and dead proof",
        replace_exact(
            &source,
            "  assert_search \"$REPLACEMENT_URL\" second\n",
            "  false && assert_search \"$REPLACEMENT_URL\" second\n",
        ),
    );
    assert_durable_rejected(
        "true bypass",
        replace_exact(
            &source,
            "  assert_search \"$CANDIDATE_URL\" first\n",
            "  assert_search \"$CANDIDATE_URL\" first || true\n",
        ),
    );
    for (label, target, replacement) in [
        (
            "cleanup exit trap removed",
            "  trap cleanup_durable EXIT\n",
            "",
        ),
        (
            "cleanup exit trap quoted",
            "  trap cleanup_durable EXIT\n",
            "  : 'trap cleanup_durable EXIT'\n",
        ),
        (
            "cleanup exit trap dead branch",
            "  trap cleanup_durable EXIT\n",
            "  if false; then trap cleanup_durable EXIT; fi\n",
        ),
        ("INT signal trap removed", "  trap 'exit 130' INT\n", ""),
        (
            "TERM signal trap quoted",
            "  trap 'exit 143' TERM\n",
            "  : 'trap exit 143 TERM'\n",
        ),
    ] {
        assert_durable_rejected(label, replace_exact(&source, target, replacement));
    }

    for (label, replacement) in [
        ("candidate tag", "\"ghcr.io/chrischeng-c4/lumen:latest\""),
        ("candidate local image", "\"./lumen\""),
        (
            "candidate child image",
            "\"${LUMEN_STANDALONE_DURABLE_IMAGE}-child\"",
        ),
    ] {
        assert_durable_rejected(
            label,
            replace_exact(
                &source,
                CANDIDATE_RUN_SOURCE,
                &CANDIDATE_RUN_SOURCE.replace("\"$LUMEN_STANDALONE_DURABLE_IMAGE\"", replacement),
            ),
        );
    }
    assert_durable_rejected(
        "old digest drift",
        replace_exact(
            &source,
            OLD_IMAGE,
            "ghcr.io/chrischeng-c4/lumen@sha256:0000000000000000000000000000000000000000000000000000000000000000",
        ),
    );
    for (label, replacement) in [
        (
            "old image data directory flag removed",
            "-e LUMEN_AUTH=off -e LUMEN_PERSISTENCE=segment -e LUMEN_SNAPSHOT_SECS=1 -e LUMEN_GRACE_SECS=1",
        ),
        (
            "old image persistence flag removed",
            "-e LUMEN_AUTH=off -e LUMEN_DATA_DIR=/var/lib/lumen/data -e LUMEN_SNAPSHOT_SECS=1 -e LUMEN_GRACE_SECS=1",
        ),
    ] {
        assert_durable_rejected(
            label,
            replace_exact(
                &source,
                "-e LUMEN_AUTH=off -e LUMEN_DATA_DIR=/var/lib/lumen/data -e LUMEN_PERSISTENCE=segment -e LUMEN_SNAPSHOT_SECS=1 -e LUMEN_GRACE_SECS=1",
                replacement,
            ),
        );
    }
    assert_durable_rejected(
        "replacement image differs",
        replace_exact(
            &source,
            REPLACEMENT_RUN_SOURCE,
            &REPLACEMENT_RUN_SOURCE.replace(
                "\"$LUMEN_STANDALONE_DURABLE_IMAGE\"",
                "\"ghcr.io/chrischeng-c4/lumen:other\"",
            ),
        ),
    );
    for (label, replacement) in [
        (
            "anonymous mount",
            "--mount \"type=volume,dst=/var/lib/lumen/data\"",
        ),
        (
            "bind mount",
            "--mount \"type=bind,src=$TEMP_DIR,dst=/var/lib/lumen/data\"",
        ),
        (
            "wrong data path",
            "--mount \"type=volume,src=$VOLUME,dst=/data\"",
        ),
    ] {
        assert_durable_rejected(
            label,
            replace_exact(
                &source,
                CANDIDATE_RUN_SOURCE,
                &CANDIDATE_RUN_SOURCE.replace(DATA_MOUNT, replacement),
            ),
        );
    }
    for variable in [
        "LUMEN_DATA_DIR",
        "LUMEN_PERSISTENCE",
        "LUMEN_WAL",
        "LUMEN_FSYNC",
        "LUMEN_SNAPSHOT_SECS",
        "LUMEN_GRACE_SECS",
    ] {
        assert_durable_rejected(variable, candidate_with_override(&source, variable));
    }

    for (label, target) in [
        (
            "first record",
            "  request -X POST \"$OLD_URL/collections/durable/index\" -H 'Content-Type: application/json' -d '{\"items\":[{\"external_id\":\"durable-first\",\"field\":\"tag\",\"value\":\"first\"}]}' -o \"$TEMP_DIR/index-first\"\n",
        ),
        (
            "second record",
            "  request -X POST \"$CANDIDATE_URL/collections/durable/index\" -H 'Content-Type: application/json' -d '{\"items\":[{\"external_id\":\"durable-second\",\"field\":\"tag\",\"value\":\"second\"}]}' -o \"$TEMP_DIR/index-second\"\n",
        ),
        (
            "search request",
            "    request -X POST \"$url/collections/durable/search\" -H 'Content-Type: application/json' \\\n      -d \"{\\\"query\\\":{\\\"term\\\":{\\\"field\\\":\\\"tag\\\",\\\"value\\\":\\\"$value\\\"}},\\\"limit\\\":10}\" -o \"$response\"\n",
        ),
        ("old first search", "  assert_search \"$OLD_URL\" first\n"),
        ("candidate first search", "  assert_search \"$CANDIDATE_URL\" first\n"),
        ("candidate second search", "  assert_search \"$CANDIDATE_URL\" second\n"),
        ("replacement first search", "  assert_search \"$REPLACEMENT_URL\" first\n"),
        ("replacement second search", "  assert_search \"$REPLACEMENT_URL\" second\n"),
        (
            "CURRENT adoption check",
            "  docker cp \"$CANDIDATE_CONTAINER:/var/lib/lumen/data/CURRENT\" \"$TEMP_DIR/current-first\"\n",
        ),
        (
            "revision check",
            "  python3 -c 'import pathlib,re,sys; b=pathlib.Path(sys.argv[1]).read_bytes(); sys.exit(0 if re.fullmatch(rb\"generation:gen-[0-9]+-rev-[1-9][0-9]*\\n\",b) else 1)' \"$TEMP_DIR/current-rev\"\n",
        ),
        (
            "checkpoint request",
            "  request -X POST \"$CANDIDATE_URL/admin/checkpoint\" -H 'Content-Type: application/json' -d '{}' -o \"$TEMP_DIR/checkpoint\"\n",
        ),
        (
            "persisted checkpoint",
            "  jq -e '.persisted == true' \"$TEMP_DIR/checkpoint\" >/dev/null\n",
        ),
    ] {
        assert_durable_rejected(label, replace_exact(&source, target, ""));
    }
    for (label, assertion) in [
        ("CURRENT adoption quoted prose", CURRENT_FIRST_ASSERT),
        ("candidate revision quoted prose", CURRENT_REV_ASSERT),
        (
            "replacement revision quoted prose",
            CURRENT_REPLACEMENT_ASSERT,
        ),
    ] {
        assert_durable_rejected(
            label,
            replace_exact(
                &source,
                &format!("  {assertion}\n"),
                "  : 'CURRENT assertion is prose, not a proof'\n",
            ),
        );
    }
    for (label, copy) in [
        (
            "CURRENT adoption copy",
            "  docker cp \"$CANDIDATE_CONTAINER:/var/lib/lumen/data/CURRENT\" \"$TEMP_DIR/current-first\"\n",
        ),
        (
            "candidate revision copy",
            "  docker cp \"$CANDIDATE_CONTAINER:/var/lib/lumen/data/CURRENT\" \"$TEMP_DIR/current-rev\"\n",
        ),
        (
            "replacement revision copy",
            "  docker cp \"$REPLACEMENT_CONTAINER:/var/lib/lumen/data/CURRENT\" \"$TEMP_DIR/current-replacement\"\n",
        ),
    ] {
        assert_durable_rejected(label, replace_exact(&source, copy, ""));
    }

    for (label, target) in [
        (
            "reject volume seed",
            "  docker cp \"$TEMP_DIR/foreign-layout\" \"$SEED_CONTAINER:/var/lib/lumen/data/foreign-layout\"\n",
        ),
        (
            "reject run",
            "  docker run -d --name \"$REJECTED_CONTAINER\" --mount \"type=volume,src=$REJECT_VOLUME,dst=/var/lib/lumen/data\" -e LUMEN_AUTH=off \\\n    -p 127.0.0.1::7373 \"$LUMEN_STANDALONE_DURABLE_IMAGE\" >/dev/null\n",
        ),
        ("reject exit wait", "  wait_for_exit \"$REJECTED_CONTAINER\"\n"),
        (
            "reject error assertion",
            "  grep -Eq 'segment checkpoint root entry.*refusing to initialize CURRENT' \"$TEMP_DIR/rejected.log\"\n",
        ),
        (
            "reject CURRENT absence",
            "  [[ ! -e \"$TEMP_DIR/rejected-data/CURRENT\" ]]\n",
        ),
        (
            "reject unknown preservation",
            "  cmp -s \"$TEMP_DIR/foreign-layout/sentinel\" \"$TEMP_DIR/rejected-data/foreign-layout/sentinel\"\n",
        ),
        (
            "legacy adoption decision log",
            "  grep -Eq 'segment checkpoint startup decision.*decision=\\\"?adopted_legacy_0428\\\"?' \"$TEMP_DIR/candidate.log\"\n",
        ),
        (
            "current generation decision log",
            "  grep -Eq 'segment checkpoint startup decision.*decision=\\\"?restored_current_generation\\\"?' \"$TEMP_DIR/replacement.log\"\n",
        ),
    ] {
        assert_durable_rejected(label, replace_exact(&source, target, ""));
    }

    for action in [
        "docker container prune -f\n",
        "docker volume prune -f\n",
        "docker system prune -af\n",
        "docker build -t lumen:bad .\n",
        "docker rmi lumen:bad\n",
        "docker rm -f $(docker ps -aq)\n",
    ] {
        assert_durable_rejected(
            action.trim(),
            insert_after_exact(&source, DURABLE_BEGIN, &format!("  {action}")),
        );
    }
    assert_durable_rejected(
        "response output",
        insert_after_exact(
            &source,
            "  assert_search \"$CANDIDATE_URL\" first\n",
            "  cat \"$TEMP_DIR/search-first\"\n",
        ),
    );
    assert_durable_rejected(
        "CURRENT output",
        insert_after_exact(
            &source,
            "  docker cp \"$CANDIDATE_CONTAINER:/var/lib/lumen/data/CURRENT\" \"$TEMP_DIR/current-first\"\n",
            "  echo \"$(cat \"$TEMP_DIR/current-first\")\"\n",
        ),
    );
}

fn validate_compose_contract(source: &str) -> Result<(), String> {
    let document: serde_yaml::Value = serde_yaml::from_str(source).map_err(|e| e.to_string())?;
    let services = document
        .get("services")
        .and_then(serde_yaml::Value::as_mapping)
        .ok_or("services must be a mapping")?;
    let expected_services = ["otel-collector", "lumen", "prometheus", "jaeger", "grafana"];
    if services.len() != expected_services.len()
        || expected_services
            .iter()
            .any(|name| !services.contains_key(serde_yaml::Value::String((*name).into())))
    {
        return Err("Compose service inventory changed".into());
    }
    for (name, image) in [
        (
            "otel-collector",
            "otel/opentelemetry-collector-contrib:0.119.0",
        ),
        ("prometheus", "prom/prometheus:v3.1.0"),
        ("jaeger", "jaegertracing/all-in-one:1.65.0"),
        ("grafana", "grafana/grafana:11.5.1"),
    ] {
        if services
            .get(serde_yaml::Value::String(name.into()))
            .and_then(|service| service.get("image"))
            .and_then(serde_yaml::Value::as_str)
            != Some(image)
        {
            return Err(format!("{name} image changed"));
        }
    }
    let lumen = document
        .get("services")
        .and_then(|v| v.get("lumen"))
        .ok_or("services.lumen is missing")?;
    let mounts = lumen
        .get("volumes")
        .and_then(serde_yaml::Value::as_sequence)
        .ok_or("services.lumen.volumes must be a sequence")?;
    if mounts.len() != 1 || mounts[0].as_str() != Some("lumen-data:/var/lib/lumen/data") {
        return Err("services.lumen must mount exactly lumen-data:/var/lib/lumen/data".into());
    }
    let ports = lumen
        .get("ports")
        .and_then(serde_yaml::Value::as_sequence)
        .ok_or("services.lumen.ports must be a sequence")?;
    if ports.len() != 1 || ports[0].as_str() != Some("7373:7373") {
        return Err("services.lumen must expose exactly 7373:7373".into());
    }
    let named_volumes = document
        .get("volumes")
        .and_then(serde_yaml::Value::as_mapping)
        .ok_or("top-level volumes must be a mapping")?;
    if !named_volumes
        .get(serde_yaml::Value::String("lumen-data".into()))
        .is_some_and(serde_yaml::Value::is_mapping)
    {
        return Err("top-level lumen-data volume is missing".into());
    }
    if document
        .get("networks")
        .and_then(|v| v.get("default"))
        .and_then(|v| v.get("name"))
        .and_then(serde_yaml::Value::as_str)
        != Some("lumen-otlp")
    {
        return Err("default network name changed".into());
    }
    let environment = lumen
        .get("environment")
        .and_then(serde_yaml::Value::as_mapping)
        .ok_or("services.lumen.environment must be a mapping")?;
    if environment.get(serde_yaml::Value::String("LUMEN_AUTH".into()))
        != Some(&serde_yaml::Value::String("off".into()))
    {
        return Err("LUMEN_AUTH must remain off".into());
    }
    for forbidden in [
        "LUMEN_DATA_DIR",
        "LUMEN_PERSISTENCE",
        "LUMEN_WAL",
        "LUMEN_FSYNC",
    ] {
        if environment.contains_key(serde_yaml::Value::String(forbidden.into())) {
            return Err(format!("forbidden persistence override: {forbidden}"));
        }
    }
    Ok(())
}

#[test]
fn test_checked_in_compose_satisfies_durability_contract() {
    let source = std::fs::read_to_string(repo_root().join("apps/lumen/compose.yaml"))
        .expect("read compose file");
    validate_compose_contract(&source).expect("compose durability contract must hold");
}

#[test]
fn test_negative_compose_durability_mutations() {
    let source = std::fs::read_to_string(repo_root().join("apps/lumen/compose.yaml"))
        .expect("read compose file");
    let mount = "      - lumen-data:/var/lib/lumen/data\n";
    for replacement in [
        "      - /var/lib/lumen/data\n",
        "      - ./lumen-data:/var/lib/lumen/data\n",
        "      - lumen-data:/data\n",
        "      - other-data:/var/lib/lumen/data\n",
    ] {
        assert!(validate_compose_contract(&replace_exact(&source, mount, replacement)).is_err());
    }
    for replacement in [
        "volumes:\n",
        "volumes:\n  lumen-data: scalar\n",
        "volumes:\n  lumen-data: []\n",
    ] {
        assert!(validate_compose_contract(&replace_exact(
            &source,
            "volumes:\n  lumen-data: {}\n",
            replacement,
        ))
        .is_err());
    }
    assert!(validate_compose_contract(&replace_exact(
        &source,
        "LUMEN_AUTH: \"off\"",
        "LUMEN_AUTH: \"on\""
    ))
    .is_err());
    for forbidden in [
        "LUMEN_DATA_DIR",
        "LUMEN_PERSISTENCE",
        "LUMEN_WAL",
        "LUMEN_FSYNC",
    ] {
        let mutated = source.replace(
            "LUMEN_OTLP_ENDPOINT: http://otel-collector:4317",
            &format!(
                "LUMEN_OTLP_ENDPOINT: http://otel-collector:4317\n      {forbidden}: forbidden"
            ),
        );
        assert!(
            validate_compose_contract(&mutated).is_err(),
            "{forbidden} must be rejected"
        );
    }
    assert!(validate_compose_contract(&replace_exact(
        &source,
        "    ports: [\"7373:7373\"]",
        "    ports: [\"7374:7373\"]",
    ))
    .is_err());
    assert!(validate_compose_contract(&source.replace(
        "\nnetworks:\n",
        "\n  extra:\n    image: example/extra:1\n\nnetworks:\n",
    ))
    .is_err());
    assert!(validate_compose_contract(&replace_exact(
        &source,
        "  jaeger:\n    image: jaegertracing/all-in-one:1.65.0\n    environment:\n      COLLECTOR_OTLP_ENABLED: \"true\"\n    ports: [\"16686:16686\"] # Jaeger UI\n",
        "",
    ))
    .is_err());
    assert!(validate_compose_contract(&replace_exact(
        &source,
        "otel/opentelemetry-collector-contrib:0.119.0",
        "otel/opentelemetry-collector-contrib:changed",
    ))
    .is_err());
    assert!(validate_compose_contract(&replace_exact(
        &source,
        "name: lumen-otlp",
        "name: changed-network",
    ))
    .is_err());
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockerfileValidationError {
    InventoryMismatch(Vec<String>, Vec<String>),
    NoStagesFound(String),
    MissingFinalStageInstruction(String, String),
    BuilderStageOnly(String, String),
    DuplicateInstruction(String, String, usize),
    MissingEntrypoint(String),
    PlacedAfterEntrypoint(String, String),
    ForbiddenHostFlagInEntrypoint(String),
    ForbiddenHostFlagInCmd(String),
    InvalidInstruction(String, String, String),
    ForbiddenFsyncKnob(String),
}

pub fn validate_inventory(discovered: &[String]) -> Result<(), DockerfileValidationError> {
    let expected: Vec<String> = EXPECTED_DOCKERFILES.iter().map(|s| s.to_string()).collect();
    if discovered != expected.as_slice() {
        return Err(DockerfileValidationError::InventoryMismatch(
            expected,
            discovered.to_vec(),
        ));
    }
    Ok(())
}

pub fn validate_dockerfile_content(
    path: &str,
    content: &str,
) -> Result<(), DockerfileValidationError> {
    let error_path = path.to_owned();
    let mut stages: Vec<Vec<(usize, &str)>> = Vec::new();
    let mut cur: Vec<(usize, &str)> = Vec::new();
    let mut in_stage = false;

    for (idx, line) in content.lines().enumerate() {
        let (num, trimmed) = (idx + 1, line.trim());
        let is_from =
            trimmed.starts_with("FROM ") || trimmed.to_ascii_uppercase().starts_with("FROM ");
        if is_from {
            if in_stage {
                stages.push(std::mem::take(&mut cur));
            }
            in_stage = true;
        }
        if in_stage {
            cur.push((num, line));
        }
    }
    if in_stage && !cur.is_empty() {
        stages.push(cur);
    }
    if stages.is_empty() {
        return Err(DockerfileValidationError::NoStagesFound(error_path));
    }

    let last_s = stages.len() - 1;
    let entrypoints: Vec<(usize, &str)> = stages[last_s]
        .iter()
        .filter(|(_, l)| l.trim().starts_with("ENTRYPOINT"))
        .copied()
        .collect();

    if entrypoints.is_empty() {
        return Err(DockerfileValidationError::MissingEntrypoint(error_path));
    }

    for (_, ep_line) in &entrypoints {
        if ep_line.contains("--host") {
            return Err(DockerfileValidationError::ForbiddenHostFlagInEntrypoint(
                error_path.clone(),
            ));
        }
    }

    let (last_ep_num, _) = entrypoints.last().unwrap();
    if stages.iter().flatten().any(|(_, line)| {
        line.trim()
            .strip_prefix("ENV ")
            .is_some_and(|value| value.starts_with("LUMEN_FSYNC"))
    }) {
        return Err(DockerfileValidationError::ForbiddenFsyncKnob(
            error_path.clone(),
        ));
    }

    const REQUIRED: [(&str, &str); 5] = [
        ("ENV LUMEN_HOST", "ENV LUMEN_HOST=0.0.0.0"),
        (
            "ENV LUMEN_DATA_DIR",
            "ENV LUMEN_DATA_DIR=/var/lib/lumen/data",
        ),
        ("ENV LUMEN_PERSISTENCE", "ENV LUMEN_PERSISTENCE=segment"),
        ("ENV LUMEN_WAL", "ENV LUMEN_WAL=embedded"),
        ("VOLUME", "VOLUME [\"/var/lib/lumen/data\"]"),
    ];
    for (prefix, expected) in REQUIRED {
        let matches: Vec<_> = stages
            .iter()
            .enumerate()
            .flat_map(|(stage, lines)| {
                lines.iter().filter_map(move |(line_number, line)| {
                    let line = line.trim();
                    (line == prefix
                        || line.starts_with(&format!("{prefix} "))
                        || line.starts_with(&format!("{prefix}=")))
                    .then_some((stage, *line_number, line))
                })
            })
            .collect();
        if matches.is_empty() {
            return Err(DockerfileValidationError::MissingFinalStageInstruction(
                error_path.clone(),
                expected.into(),
            ));
        }
        if matches.len() > 1 {
            return Err(DockerfileValidationError::DuplicateInstruction(
                error_path.clone(),
                expected.into(),
                matches.len(),
            ));
        }
        let (stage, line_number, actual) = matches[0];
        if stage != last_s {
            return Err(DockerfileValidationError::BuilderStageOnly(
                error_path.clone(),
                expected.into(),
            ));
        }
        if actual != expected {
            return Err(DockerfileValidationError::InvalidInstruction(
                error_path.clone(),
                prefix.into(),
                actual.into(),
            ));
        }
        if line_number >= *last_ep_num {
            return Err(DockerfileValidationError::PlacedAfterEntrypoint(
                error_path.clone(),
                expected.into(),
            ));
        }
    }

    if stages[last_s]
        .iter()
        .any(|(_, l)| l.trim().starts_with("CMD") && l.contains("--host"))
    {
        return Err(DockerfileValidationError::ForbiddenHostFlagInCmd(
            error_path,
        ));
    }
    Ok(())
}

fn for_each_dockerfile(f: impl Fn(&str, &str)) {
    let root = repo_root();
    for &rel_path in EXPECTED_DOCKERFILES {
        let content = std::fs::read_to_string(root.join(rel_path))
            .unwrap_or_else(|err| panic!("read {rel_path}: {err}"));
        f(rel_path, &content);
    }
}

#[test]
fn test_checked_in_dockerfiles_satisfy_contract() {
    let discovered = discover_dockerfiles();
    validate_inventory(&discovered).expect("inventory must match expected");
    for_each_dockerfile(|path, content| {
        validate_dockerfile_content(path, content)
            .unwrap_or_else(|err| panic!("validation failed for {path}: {err:?}"));
    });
}

#[test]
fn test_negative_fixture_inventory_mismatch() {
    let mut missing: Vec<String> = EXPECTED_DOCKERFILES.iter().map(|s| s.to_string()).collect();
    missing.pop();
    assert!(matches!(
        validate_inventory(&missing),
        Err(DockerfileValidationError::InventoryMismatch(_, _))
    ));

    let mut extra = missing;
    extra.extend([
        "apps/lumen/Dockerfile.test".into(),
        "apps/lumen/Dockerfile.extra".into(),
    ]);
    assert!(matches!(
        validate_inventory(&extra),
        Err(DockerfileValidationError::InventoryMismatch(_, _))
    ));
}

#[test]
fn test_negative_fixture_no_stages_found() {
    let no_from = "# syntax=docker/dockerfile:1\nENV LUMEN_HOST=0.0.0.0\n\
        ENTRYPOINT [\"/usr/local/bin/lumen\"]\nCMD [\"serve\"]";
    assert_eq!(
        validate_dockerfile_content("apps/lumen/Dockerfile", no_from),
        Err(DockerfileValidationError::NoStagesFound(
            "apps/lumen/Dockerfile".to_string()
        ))
    );
}

#[test]
fn test_negative_fixtures_per_file() {
    for_each_dockerfile(|path, content| {
        let p = path.to_string();
        let check = |mutated: String, err: DockerfileValidationError| {
            assert_eq!(validate_dockerfile_content(path, &mutated), Err(err));
        };

        check(
            replace_exact(content, "ENTRYPOINT [\"/usr/local/bin/lumen\"]\n", ""),
            DockerfileValidationError::MissingEntrypoint(p.clone()),
        );
        check(
            replace_exact(
                content,
                "ENTRYPOINT [\"/usr/local/bin/lumen\"]\n",
                "ENTRYPOINT [\"/usr/local/bin/lumen\"]\nENTRYPOINT [\"/usr/local/bin/lumen\", \"--host\", \"0.0.0.0\"]\n",
            ),
            DockerfileValidationError::ForbiddenHostFlagInEntrypoint(p.clone()),
        );
        check(
            replace_exact(
                content,
                "CMD [\"serve\"]",
                "CMD [\"serve\", \"--host\", \"0.0.0.0\"]",
            ),
            DockerfileValidationError::ForbiddenHostFlagInCmd(p.clone()),
        );
        check(
            replace_exact(
                content,
                "ENTRYPOINT [\"/usr/local/bin/lumen\"]\n",
                "ENV LUMEN_FSYNC=always\nENTRYPOINT [\"/usr/local/bin/lumen\"]\n",
            ),
            DockerfileValidationError::ForbiddenFsyncKnob(p.clone()),
        );

        let required = [
            (
                "ENV LUMEN_HOST",
                "ENV LUMEN_HOST=0.0.0.0",
                "ENV LUMEN_HOST=127.0.0.1",
            ),
            (
                "ENV LUMEN_DATA_DIR",
                "ENV LUMEN_DATA_DIR=/var/lib/lumen/data",
                "ENV LUMEN_DATA_DIR=/data",
            ),
            (
                "ENV LUMEN_PERSISTENCE",
                "ENV LUMEN_PERSISTENCE=segment",
                "ENV LUMEN_PERSISTENCE=cbor",
            ),
            (
                "ENV LUMEN_WAL",
                "ENV LUMEN_WAL=embedded",
                "ENV LUMEN_WAL=auto",
            ),
            (
                "VOLUME",
                "VOLUME [\"/var/lib/lumen/data\"]",
                "VOLUME [\"/data\"]",
            ),
        ];
        for (prefix, exact, wrong) in required {
            let exact_line = format!("{exact}\n");
            let missing = replace_exact(content, &exact_line, "");
            check(
                missing.clone(),
                DockerfileValidationError::MissingFinalStageInstruction(p.clone(), exact.into()),
            );
            check(
                replace_exact(content, &exact_line, &format!("{exact}\n{exact}\n")),
                DockerfileValidationError::DuplicateInstruction(p.clone(), exact.into(), 2),
            );
            check(
                replace_exact(content, exact, wrong),
                DockerfileValidationError::InvalidInstruction(
                    p.clone(),
                    prefix.into(),
                    wrong.into(),
                ),
            );
            check(
                replace_exact(
                    &missing,
                    "ENTRYPOINT [\"/usr/local/bin/lumen\"]\n",
                    &format!("ENTRYPOINT [\"/usr/local/bin/lumen\"]\n{exact}\n"),
                ),
                DockerfileValidationError::PlacedAfterEntrypoint(p.clone(), exact.into()),
            );
            check(
                insert_after_first_from(&missing, exact),
                DockerfileValidationError::BuilderStageOnly(p.clone(), exact.into()),
            );
        }
    });
}

#[tokio::test]
async fn test_bare_process_defaults_to_localhost() {
    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind free port");
        listener.local_addr().expect("local addr").port()
    };

    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_lumen"));
    cmd.args(["serve", "--port", &port.to_string()])
        .env_remove("LUMEN_HOST")
        .env_remove("RUST_LOG")
        .env("LUMEN_AUTH", "off")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    struct ChildGuard(Option<std::process::Child>);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            if let Some(mut child) = self.0.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    let child = cmd.spawn().expect("failed to spawn lumen serve");
    let mut guard = ChildGuard(Some(child));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(500))
        .no_proxy()
        .build()
        .expect("build reqwest client");

    let url = format!("http://127.0.0.1:{port}/healthz");
    let (start, timeout) = (
        std::time::Instant::now(),
        std::time::Duration::from_secs(15),
    );
    let mut healthy = false;

    while start.elapsed() < timeout {
        if client
            .get(&url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
            healthy = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    let mut child = guard.0.take().expect("child exists");
    let _ = child.kill();
    let output = child.wait_with_output().expect("wait for child output");
    let logs = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        healthy,
        "lumen serve did not answer /healthz within deadline\nlogs:\n{logs}"
    );

    let expected_addr = format!("127.0.0.1:{port}");
    let record_matches = logs
        .lines()
        .any(|l| l.contains("lumen serve listening") && l.contains(&expected_addr));
    assert!(
        record_matches,
        "single log line must contain 'lumen serve listening' and '{expected_addr}', logs:\n{logs}"
    );
}
