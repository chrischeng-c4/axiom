//! Static contract for the controller-owned GKE acceptance gate.
//!
//! A real GKE credential is a release-controller input. This test checks the
//! executable live slice and rejects mutations that make a green static test
//! look plausible while removing a live security or durability assertion.

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{json, Value};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf()
}

fn full_script() -> String {
    fs::read_to_string(root().join("apps/lumen/scripts/standalone-gke-acceptance.sh")).unwrap()
}

fn live_slice() -> String {
    let source = full_script();
    let (_, live) = source
        .split_once("validate_candidate_manifest_v2()")
        .expect("live gate marker");
    live.to_owned()
}

fn function_body<'a>(source: &'a str, name: &str) -> &'a str {
    let marker = format!("\n{name}() {{\n");
    let (_, tail) = source
        .split_once(&marker)
        .unwrap_or_else(|| panic!("function marker is present: {name}"));
    let (body, _) = tail
        .split_once("\n}\n")
        .unwrap_or_else(|| panic!("function terminator is present: {name}"));
    body
}

struct RequiredRuntimeOracleResult {
    output: Output,
    statefulset: Option<String>,
    diagnostic: Option<String>,
    receipt: Option<String>,
}

fn run_required_runtime_oracle(source: &str, mutation: Option<&str>) -> RequiredRuntimeOracleResult {
    let fixture = tempfile::Builder::new()
        .prefix("lumen-required-runtime-oracle-")
        .tempdir()
        .expect("required-runtime fixture");
    let live = json!({
        "apiVersion": "apps/v1",
        "kind": "StatefulSet",
        "metadata": {"name": "lumen", "labels": {"stable": "true"}},
        "spec": {
            "serviceName": "lumen",
            "replicas": 1,
            "template": {
                "metadata": {"labels": {"stable": "true"}},
                "spec": {
                    "containers": [{
                        "name": "serving",
                        "image": "example.invalid/lumen@sha256:stable",
                        "env": [
                            {"name": "LUMEN_AUTH", "value": "in-cluster"},
                            {"name": "OTHER", "value": "stable"}
                        ],
                        "resources": {"requests": {"cpu": "1", "memory": "1Gi"}}
                    }]
                }
            }
        }
    });
    let mut after = live.clone();
    after["metadata"]["creationTimestamp"] = Value::Null;
    after["spec"]["template"]["metadata"]["creationTimestamp"] =
        Value::String("2026-08-30T00:00:00Z".to_owned());
    after["status"] = json!({"observedGeneration": 1});
    after["spec"]["template"]["spec"]["containers"][0]["env"][0]["value"] =
        Value::String("required".to_owned());
    let inject_race = mutation == Some("race");
    match mutation {
        None => {}
        Some("image") | Some("race") => {
            after["spec"]["template"]["spec"]["containers"][0]["image"] =
                Value::String("example.invalid/lumen@sha256:changed".to_owned());
        }
        Some("cpu") => {
            after["spec"]["template"]["spec"]["containers"][0]["resources"]["requests"]
                ["cpu"] = Value::String("2".to_owned());
        }
        Some("memory") => {
            after["spec"]["template"]["spec"]["containers"][0]["resources"]["requests"]
                ["memory"] = Value::String("2Gi".to_owned());
        }
        Some("other-env") => {
            after["spec"]["template"]["spec"]["containers"][0]["env"][1]["value"] =
                Value::String("changed".to_owned());
        }
        Some(other) => panic!("unknown required-runtime mutation: {other}"),
    }
    let live_path = fixture.path().join("live.json");
    let after_path = fixture.path().join("after.json");
    let script_path = fixture.path().join("run.sh");
    let evidence = fixture.path().join("evidence");
    fs::create_dir(&evidence).expect("required-runtime private evidence directory");
    fs::write(&live_path, serde_json::to_vec(&live).expect("live fixture JSON"))
        .expect("write live fixture");
    fs::write(&after_path, serde_json::to_vec(&after).expect("after fixture JSON"))
        .expect("write after fixture");
    let harness = [
        r#"#!/usr/bin/env bash
set -euo pipefail
V2_RUNTIME_NAMESPACE=runtime
V2_REQUIRED_STATEFULSET=''
LUMEN_STANDALONE_GKE_IMAGE='example.invalid/lumen@sha256:stable'
die() { printf '%s\n' "$*" >&2; exit 2; }
k() {
  case "$1:$2" in
    get:statefulset) cat "$LIVE_FIXTURE" ;;
    set:env) cat "$AFTER_FIXTURE" ;;
    *) exit 99 ;;
  esac
}
ln() {
  if [[ "${RACE_INJECT:-}" == 1 ]]; then
    local destination="${!#}"
    printf '%s' "$RACE_BYTES" >"$destination"
  fi
  command ln "$@"
}
v2_write_required_continuity_diff() {
"#,
        function_body(source, "v2_write_required_continuity_diff"),
        r#"
}
v2_required_runtime() {
"#,
        function_body(source, "v2_required_runtime"),
        r#"
}
v2_required_runtime
"#,
    ]
    .concat();
    fs::write(&script_path, harness).expect("write required-runtime harness");
    let output = Command::new("bash")
        .arg(&script_path)
        .env("TMP_ROOT", fixture.path())
        .env("LUMEN_STANDALONE_GKE_EVIDENCE_DIR", &evidence)
        .env("RACE_INJECT", if inject_race { "1" } else { "0" })
        .env("RACE_BYTES", "injected-target-bytes\n")
        .env("LIVE_FIXTURE", &live_path)
        .env("AFTER_FIXTURE", &after_path)
        .output()
        .expect("run required-runtime harness");
    let statefulset = fs::read_to_string(fixture.path().join("v2-required-statefulset.json")).ok();
    let diagnostic = fs::read_to_string(
        evidence.join("lumen-standalone-gke-required-continuity-diff.json"),
    )
    .ok();
    let receipt = fs::read_to_string(evidence.join("lumen-standalone-gke-receipt.json")).ok();
    RequiredRuntimeOracleResult {
        output,
        statefulset,
        diagnostic,
        receipt,
    }
}

fn run_required_continuity_diff_oracle(
    source: &str,
    before: &Value,
    after: &Value,
    preexisting: Option<&str>,
) -> (Output, Option<String>, BTreeMap<PathBuf, Vec<u8>>, u32) {
    let fixture = tempfile::Builder::new()
        .prefix("lumen-required-continuity-diff-oracle-")
        .tempdir()
        .expect("required-continuity diff fixture");
    let before_path = fixture.path().join("before.json");
    let after_path = fixture.path().join("after.json");
    let evidence = fixture.path().join("evidence");
    let script_path = fixture.path().join("run.sh");
    fs::create_dir(&evidence).expect("required-continuity diff private evidence directory");
    if let Some(contents) = preexisting {
        let diagnostic_path = evidence.join("lumen-standalone-gke-required-continuity-diff.json");
        fs::write(&diagnostic_path, contents).expect("write pre-existing private diagnostic");
        fs::set_permissions(&diagnostic_path, fs::Permissions::from_mode(0o644))
            .expect("set pre-existing private diagnostic mode");
    }
    fs::write(&before_path, serde_json::to_vec(before).expect("before fixture JSON"))
        .expect("write before fixture");
    fs::write(&after_path, serde_json::to_vec(after).expect("after fixture JSON"))
        .expect("write after fixture");
    let harness = [
        r#"#!/usr/bin/env bash
set -euo pipefail
v2_write_required_continuity_diff() {
"#,
        function_body(source, "v2_write_required_continuity_diff"),
        r#"
}
v2_write_required_continuity_diff "$BEFORE_FIXTURE" "$AFTER_FIXTURE"
"#,
    ]
    .concat();
    fs::write(&script_path, harness).expect("write required-continuity diff harness");
    let output = Command::new("bash")
        .arg(&script_path)
        .env("BEFORE_FIXTURE", &before_path)
        .env("AFTER_FIXTURE", &after_path)
        .env("LUMEN_STANDALONE_GKE_EVIDENCE_DIR", &evidence)
        .output()
        .expect("run required-continuity diff harness");
    let diagnostic = fs::read_to_string(
        evidence.join("lumen-standalone-gke-required-continuity-diff.json"),
    )
    .ok();
    let files = collect_files(&evidence);
    let mode = fs::metadata(evidence.join("lumen-standalone-gke-required-continuity-diff.json"))
        .map(|metadata| metadata.permissions().mode() & 0o777)
        .unwrap_or(0);
    (output, diagnostic, files, mode)
}

fn has_exact_line(source: &str, expected: &str) -> bool {
    source.lines().any(|line| line.trim() == expected)
}

fn run_service_link_wait(
    source: &str,
    enable_service_links: bool,
    ready_status: &str,
    image_id: &str,
    pod_image: &str,
    status_image: &str,
    scheduled_arch: &str,
    scheduled_child: &str,
) -> Output {
    let fixture = tempfile::Builder::new()
        .prefix("lumen-service-link-oracle-")
        .tempdir()
        .expect("service-link fixture");
    let pod = r#"{
      "metadata":{"uid":"replacement"},
      "status":{
        "conditions":[{"type":"Ready","status":"READY_STATUS"}],
        "containerStatuses":[{"name":"serving","imageID":"IMAGE_ID","image":"STATUS_IMAGE"}]
      },
      "spec":{
        "enableServiceLinks":SERVICE_LINKS,
        "containers":[{
          "name":"serving",
          "image":"POD_IMAGE",
          "env":[{"name":"LUMEN_AUTH","value":"in-cluster"}],
          "resources":{"requests":{"cpu":"500m","memory":"512Mi"}}
        }]
      }
    }"#
    .replace("READY_STATUS", ready_status)
    .replace("IMAGE_ID", image_id)
    .replace("POD_IMAGE", pod_image)
    .replace("STATUS_IMAGE", status_image)
    .replace(
        "SERVICE_LINKS",
        if enable_service_links { "true" } else { "false" },
    );
    let pod_path = fixture.path().join("pod.json");
    fs::write(&pod_path, pod).expect("service-link pod fixture");
    let script_path = fixture.path().join("run.sh");
    let harness = [
        r#"#!/usr/bin/env bash
set -euo pipefail
V2_RUNTIME_NAMESPACE=runtime
V2_PVC_UID=''
V2_PV_NAME=''
V2_CHILD_DIGEST=''
V2_NODE_ARCH=''
V2_OBSERVED_RUNTIME_IMAGE_DIGEST=''
V2_LAST_POD_UID=''
ROOT_DIGEST='sha256:root'
LUMEN_STANDALONE_GKE_IMAGE='ghcr.io/chrischeng-c4/lumen@sha256:root'
die() { exit 2; }
k() {
  [[ "$1" == get && "$2" == pod ]] || return 99
  cat "$POD_FIXTURE"
}

v2_expected_child() { V2_NODE_ARCH="$SCHEDULED_ARCH"; V2_CHILD_DIGEST="$SCHEDULED_CHILD"; }
sleep() { SECONDS=300; }
v2_wait_pod() {
"#,
        function_body(source, "v2_wait_pod"),
        r#"
}
v2_wait_pod '' in-cluster 500m 512Mi
"#,
    ]
    .concat();
    fs::write(&script_path, harness).expect("service-link harness");
    Command::new("bash")
        .arg(&script_path)
        .env("POD_FIXTURE", pod_path)
        .env("TMP_ROOT", fixture.path())
        .env("SCHEDULED_ARCH", scheduled_arch)
        .env("SCHEDULED_CHILD", scheduled_child)
        .output()
        .expect("run service-link wait harness")
}

struct JobLogOracleResult {
    output: Output,
    calls: u32,
    sleeps: usize,
    log: String,
}

fn run_job_log_reader(source: &str, mode: &str) -> JobLogOracleResult {
    let fixture = tempfile::Builder::new()
        .prefix("lumen-job-log-oracle-")
        .tempdir()
        .expect("job-log fixture");
    let script = fixture.path().join("run.sh");
    let harness = [
        r#"#!/usr/bin/env bash
set -euo pipefail
V2_CLIENT_NAMESPACE=client
COUNT="$TMP_ROOT/count"
SLEEPS="$TMP_ROOT/sleeps"
die() {
  printf 'standalone GKE acceptance: %s\n' "$*" >&2
  exit 2
}
sleep() {
  [[ "$1" == 5 ]] || exit 93
  printf x >>"$SLEEPS"
}
k() {
  [[ "$1" == logs ]] || exit 91
  [[ " $* " == *' --request-timeout=10s '* ]] || exit 92
  local n
  n=$(<"$COUNT")
  n=$((n + 1))
  printf '%s' "$n" >"$COUNT"
  case "$ORACLE_MODE:$n" in
    permanent:*)
      printf 'Forbidden\n' >&2
      return 1
      ;;
    transient6:*)
      printf 'Error from server: Get "https://redacted/containerLogs/...": No agent available\n' >&2
      return 1
      ;;
    transient:1|transient:2|transient5permanent:1|transient5permanent:2|transient5permanent:3|transient5permanent:4|transient5permanent:5)
      printf 'Error from server: Get "https://redacted/containerLogs/...": No agent available\n' >&2
      return 1
      ;;
    transient5permanent:6)
      printf 'Forbidden\n' >&2
      return 1
      ;;
    *)
      printf 'row=job status=passed\n'
      return 0
      ;;
  esac
}
v2_read_job_log() {
"#,
        function_body(source, "v2_read_job_log"),
        r#"
}
printf 0 >"$COUNT"
: >"$SLEEPS"
v2_read_job_log job "$TMP_ROOT/log"
"#,
    ]
    .concat();
    fs::write(&script, harness).expect("write job-log harness");
    let output = Command::new("bash")
        .arg(&script)
        .env("TMP_ROOT", fixture.path())
        .env("ORACLE_MODE", mode)
        .output()
        .expect("run job-log harness");
    let calls = fs::read_to_string(fixture.path().join("count"))
        .expect("job-log call count")
        .parse()
        .expect("decimal job-log call count");
    let sleeps = fs::read(fixture.path().join("sleeps"))
        .expect("job-log sleep count")
        .len();
    let log = fs::read_to_string(fixture.path().join("log")).unwrap_or_default();
    JobLogOracleResult {
        output,
        calls,
        sleeps,
        log,
    }
}

fn run_api_status_oracle(source: &str, label: &str, expected: &str, log: &str) -> Output {
    let fixture = tempfile::Builder::new()
        .prefix("lumen-api-status-oracle-")
        .tempdir()
        .expect("api-status fixture");
    let log_path = fixture.path().join("job.log");
    fs::write(&log_path, log).expect("write API job log fixture");
    let script = fixture.path().join("run.sh");
    let harness = [
        r##"#!/usr/bin/env bash
set -euo pipefail
die() {
  printf 'standalone GKE acceptance: %s\n' "$*" >&2
  exit 2
}
v2_assert_api_job_log() {
"##,
        function_body(source, "v2_assert_api_job_log"),
        r#"
}
v2_assert_api_job_log "$LABEL" "$EXPECTED" "$LOG"
"#,
    ]
    .concat();
    fs::write(&script, harness).expect("write API-status harness");
    Command::new("bash")
        .arg(&script)
        .env("LABEL", label)
        .env("EXPECTED", expected)
        .env("LOG", log_path)
        .output()
        .expect("run API-status harness")
}

struct MetricDeltaOracleResult {
    output: Output,
    evidence: BTreeMap<String, String>,
}

fn run_metric_delta_oracle(
    source: &str,
    profile: &str,
    before: &str,
    after: &str,
) -> MetricDeltaOracleResult {
    let private_root = fs::canonicalize("/tmp").expect("canonical private temporary root");
    let fixture = tempfile::Builder::new()
        .prefix("lumen-metric-delta-oracle-")
        .tempdir_in(private_root)
        .expect("metric-delta fixture");
    let evidence = fixture.path().join("evidence");
    fs::create_dir(&evidence).expect("metric-delta evidence directory");
    let before_path = fixture.path().join("before.metrics");
    let after_path = fixture.path().join("after.metrics");
    fs::write(&before_path, before).expect("write metric before fixture");
    fs::write(&after_path, after).expect("write metric after fixture");
    let script = fixture.path().join("run.sh");
    let harness = [
        r#"#!/usr/bin/env bash
set -euo pipefail
PRIVATE_TMP_ROOT="$(cd -P /tmp && pwd -P)"
safe_private_dir() {
  local path=$1
  [[ "$path" == "$PRIVATE_TMP_ROOT"/* && "$path" != */ && -d "$path" && ! -L "$path" ]] || return 1
  [[ "$(cd "$path" && pwd -P)" == "$path" ]]
}
safe_private_file() {
  local path=$1 parent
  [[ "$path" == "$PRIVATE_TMP_ROOT"/* && -f "$path" && ! -L "$path" ]] || return 1
  parent=${path%/*}; safe_private_dir "$parent"
}
die() {
  printf "standalone GKE acceptance: %s\n" "$*" >&2
  exit 2
}
v2_metric_total() {
"#,
        function_body(source, "v2_metric_total"),
        r#"
}
v2_metric_shape() {
"#,
        function_body(source, "v2_metric_shape"),
        r#"
}
v2_write_metric_failure() {
"#,
        function_body(source, "v2_write_metric_failure"),
        r#"
}
v2_fail_metric_delta() {
"#,
        function_body(source, "v2_fail_metric_delta"),
        r#"
}
v2_metric_deltas() {
"#,
        function_body(source, "v2_metric_deltas"),
        r#"
}
v2_metric_deltas "$PROFILE" "$BEFORE" "$AFTER"
"#,
    ]
    .concat();
    fs::write(&script, harness).expect("write metric-delta harness");
    let output = Command::new("bash")
        .arg(&script)
        .env("PROFILE", profile)
        .env("BEFORE", before_path)
        .env("AFTER", after_path)
        .env("LUMEN_STANDALONE_GKE_EVIDENCE_DIR", &evidence)
        .output()
        .expect("run metric-delta harness");
    let evidence = fs::read_dir(&evidence)
        .expect("read metric-delta evidence")
        .map(|entry| {
            let entry = entry.expect("metric-delta evidence entry");
            let name = entry.file_name().to_string_lossy().into_owned();
            let value = fs::read_to_string(entry.path()).expect("read metric-delta evidence file");
            (name, value)
        })
        .collect();
    MetricDeltaOracleResult { output, evidence }
}

fn appears_before(source: &str, first: &str, second: &str) -> bool {
    match (source.find(first), source.find(second)) {
        (Some(first), Some(second)) => first < second,
        _ => false,
    }
}

fn kubectl_line_inventory_is_complete(source: &str) -> bool {
    let mut actual = source
        .lines()
        .filter(|line| line.contains("kubectl"))
        .map(str::trim)
        .collect::<Vec<_>>();
    let mut expected = vec![
        "kubectl \\",
        "KUBECTL_CACHE_DIR=\"$TMP_ROOT/kubectl-cache\"",
        "[[ ! -e \"$KUBECTL_CACHE_DIR\" && ! -L \"$KUBECTL_CACHE_DIR\" ]] || die 'kubectl cache path already exists'",
        "[[ -d \"$KUBECTL_CACHE_DIR\" && ! -L \"$KUBECTL_CACHE_DIR\" && \"$(cd \"$KUBECTL_CACHE_DIR\" && pwd -P)\" == \"$KUBECTL_CACHE_DIR\" ]] || die 'kubectl cache path is not canonical'",
        "[[ \"${KUBECTL_CACHE_DIR%/*}\" == \"$TMP_ROOT\" && \"${KUBECTL_CACHE_DIR##*/}\" == kubectl-cache ]] || die 'kubectl cache path identity is unsafe'",
        "[[ \"$(private_mode \"$KUBECTL_CACHE_DIR\")\" == 700 ]] || die 'kubectl cache path mode is not 0700'",
        "kubectl \\",
        "die \"kubectl wrapper did not select the requested context\"",
        "current_context=\"$(kubectl --kubeconfig \"$KUBECONFIG\" --cache-dir \"$KUBECTL_CACHE_DIR\" config current-context 2>\"$TMP_ROOT/current-context.err\")\" ||",
        "[[ \"$(kubectl --kubeconfig \"$KUBECONFIG\" --cache-dir \"$KUBECTL_CACHE_DIR\" config get-contexts -o name | awk 'NF {count++; name=$0} END {if (count == 1) print name}')\" == \"$LUMEN_STANDALONE_GKE_CONTEXT\" ]] ||",
    ];
    actual.sort_unstable();
    expected.sort_unstable();
    actual == expected
}

fn shared_renderer_findings(source: &str) -> Vec<&'static str> {
    let mut bad = Vec::new();
    let log_reader = function_body(source, "v2_read_job_log");
    if source.matches("k logs").count() != 1
        || log_reader.contains("apply")
        || log_reader.contains("wait")
        || log_reader.contains("create")
        || source.contains("LUMEN_STANDALONE_GKE_TEST_NO_SLEEP")
        || !appears_before(source, "grep -Fq -- 'No agent available'", "[[ \"$attempt\" -lt 6 ]]")
    {
        bad.push("CLIENT");
    }
    for required in [
        "for ((attempt = 1; attempt <= 6; attempt++)); do",
        "grep -Fq -- 'No agent available' \"$error\" || die \"job log read failed\"",
        "[[ \"$attempt\" -lt 6 ]] || die \"Konnectivity agent unavailable while reading job log\"",
        "sleep 5",
    ] {
        if !has_exact_line(log_reader, required) {
            bad.push("CLIENT");
        }
    }
    if !log_reader.contains("--request-timeout=10s") {
        bad.push("CLIENT");
    }
    for (name, component) in [
        ("v2_run_client_tooling_job", "tooling"),
        ("v2_run_api_job", "api"),
        ("v2_run_metrics_job", "metrics"),
    ] {
        let body = function_body(source, name);
        if body.matches("v2_read_job_log \"$job\" \"$log\"").count() != 1 {
            bad.push("CLIENT");
        }
        let renderer = format!("\"$KUSTOMIZE_RENDERER\" {component} \\");
        let validator = format!("ruby \"$KUSTOMIZE_VALIDATOR\" {component} \\");
        if body.lines().filter(|line| line.trim() == renderer).count() != 1
            || body.lines().filter(|line| line.trim() == validator).count() != 1
            || !body.contains("render_dir=\"$TMP_ROOT/${job}-render\"")
            || body.matches("--file \"$render_dir/rendered.yaml\"").count() != 1
            || body
                .matches("k apply -f \"$render_dir/rendered.yaml\" >/dev/null")
                .count()
                != 1
            || !body.contains("[[ ! -e \"$render_dir\" && ! -L \"$render_dir\" ]] ||")
            || body.contains("if false; then")
            || body
                .lines()
                .any(|line| line.trim_start().starts_with("false &&"))
        {
            bad.push("CLIENT");
        }
        for forbidden in [
            "jq -n",
            "read -r -d",
            "<<",
            "program=",
            "BODY64",
            "--data-binary",
            "kind:\"Job\"",
            "kind:Job",
        ] {
            if body.contains(forbidden) {
                bad.push("CLIENT");
            }
        }
    }

    let tooling = function_body(source, "v2_run_client_tooling_job");
    for required in [
        "--out-dir \"$render_dir\"",
        "--client-namespace \"$V2_CLIENT_NAMESPACE\"",
        "--run-id \"$LUMEN_STANDALONE_GKE_RUN_ID\"",
        "--job \"$job\"",
    ] {
        if !tooling.contains(required) {
            bad.push("CLIENT");
        }
    }

    let metrics = function_body(source, "v2_run_metrics_job");
    if !metrics.contains("--out-dir \"$render_dir\"") {
        bad.push("CLIENT");
    }
    for required in [
        "--client-namespace \"$V2_CLIENT_NAMESPACE\"",
        "--runtime-namespace \"$V2_RUNTIME_NAMESPACE\"",
        "--service lumen",
        "--run-id \"$LUMEN_STANDALONE_GKE_RUN_ID\"",
        "--job \"$job\"",
        "--row-label \"$label\"",
    ] {
        if metrics.matches(required).count() != 2 {
            bad.push("CLIENT");
        }
    }

    let api = function_body(source, "v2_run_api_job");
    if api.matches("--out-dir \"$render_dir\"").count() != 1 {
        bad.push("CLIENT");
    }
    for required in [
        "--client-namespace \"$V2_CLIENT_NAMESPACE\"",
        "--runtime-namespace \"$V2_RUNTIME_NAMESPACE\"",
        "--service lumen",
        "--run-id \"$LUMEN_STANDALONE_GKE_RUN_ID\"",
        "--job \"$job\"",
        "--account \"$account\"",
        "--token-mode \"$token_mode\"",
        "--method \"$method\"",
        "--path \"$path\"",
        "--request-file \"$request_file\"",
        "--expected-status \"$expected\"",
        "--required-id \"$need_id\"",
        "--rejected-id \"$reject_id\"",
        "--row-label \"$label\"",
    ] {
        if api.matches(required).count() != 2 {
            bad.push("CLIENT");
        }
    }
    for required in [
        "request_file=\"$TMP_ROOT/${job}.request.json\"",
        "[[ ! -e \"$render_dir\" && ! -L \"$render_dir\" ]] ||",
        "[[ ! -e \"$request_file\" && ! -L \"$request_file\" ]] ||",
        "printf '%s' \"$body\" >\"$request_file\"",
        "[[ -f \"$request_file\" && ! -L \"$request_file\" ]] ||",
    ] {
        if !api.contains(required) {
            bad.push("CLIENT");
        }
    }
    if api.contains("\n  file=\"$TMP_ROOT/") {
        bad.push("CLIENT");
    }

    let client = function_body(source, "v2_write_client_root");
    let client_renderer = "\"$KUSTOMIZE_RENDERER\" client \\";
    let client_validator = "ruby \"$KUSTOMIZE_VALIDATOR\" client \\";
    for required in [
        "local output=\"$TMP_ROOT/v2-client\" validated",
        "--out-dir \"$output\"",
        "--client-namespace \"$V2_CLIENT_NAMESPACE\"",
        "--run-id \"$LUMEN_STANDALONE_GKE_RUN_ID\"",
        "validated=\"$output/validated.json\"",
        "--emit-json",
        "[[ -f \"$validated\" && ! -L \"$validated\" ]] ||",
        "jq -e 'map(select(.apiVersion == \"v1\" and .kind == \"Namespace\")) | length == 1' \"$validated\" >/dev/null ||",
        "jq -e 'map(select(.apiVersion == \"v1\" and .kind == \"Namespace\")) | .[0]' \"$validated\" >\"$output/namespace.json\" ||",
        "[[ -f \"$output/namespace.json\" && ! -L \"$output/namespace.json\" ]] ||",
    ] {
        if !client.contains(required) {
            bad.push("CLIENT");
        }
    }
    if client
        .lines()
        .filter(|line| line.trim() == client_renderer)
        .count()
        != 1
        || client
            .lines()
            .filter(|line| line.trim() == client_validator)
            .count()
            != 1
        || !client.contains("[[ ! -e \"$output\" && ! -L \"$output\" ]] ||")
        || !has_exact_line(
            source,
            "if ! k apply -f \"$TMP_ROOT/v2-client/rendered.yaml\" >/dev/null; then die \"client apply failed\"; fi",
        )
    {
        bad.push("CLIENT");
    }
    for forbidden in [
        "jq -n",
        "read -r -d",
        "<<",
        "program=",
        "kind:\"Job\"",
        "kind:Job",
    ] {
        if client.contains(forbidden) {
            bad.push("CLIENT");
        }
    }
    if !appears_before(
        source,
        "ruby \"$KUSTOMIZE_VALIDATOR\" client \\",
        "if ! k apply -f \"$TMP_ROOT/v2-client/rendered.yaml\" >/dev/null; then",
    ) {
        bad.push("CLIENT");
    }
    bad
}

fn findings(source: &str) -> Vec<&'static str> {
    let mut bad = Vec::new();
    bad.extend(shared_renderer_findings(source));
    for (code, required) in [
        ("CANDIDATE", "cclab.lumen.candidate-manifest.v3"),
        ("CANDIDATE", "(.jobs|all(.[]; . == \"success\"))"),
        (
            "CANDIDATE",
            "CANDIDATE_TAG=\"lumen@$CANDIDATE_VERSION\"",
        ),
        (
            "CANDIDATE",
            "CANDIDATE_DEFAULT_IMAGE=\"ghcr.io/chrischeng-c4/lumen:$CANDIDATE_VERSION\"",
        ),
        ("CANDIDATE", ".version == $version and .tag == $tag"),
        ("CANDIDATE", "--arg version \"$CANDIDATE_VERSION\""),
        ("CANDIDATE", "--arg tag \"$CANDIDATE_TAG\""),
        (
            "CANDIDATE",
            "candidate image is not the exact receipt root digest",
        ),
        (
            "CANDIDATE",
            "[[ \"$RECEIPT_SHA256\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256\" ]]",
        ),
        (
            "CANDIDATE",
            "[[ \"$CANDIDATE_COMMIT\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_COMMIT\" ]]",
        ),
        (
            "CANDIDATE",
            "[[ \"$CANDIDATE_RUN_ID\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID\" ]]",
        ),
        (
            "CANDIDATE",
            "[[ \"$CANDIDATE_ATTEMPT\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ATTEMPT\" ]]",
        ),
        (
            "RUNTIME_IMAGE",
            "if [[ \"$image\" == \"$LUMEN_STANDALONE_GKE_IMAGE\" ]]; then",
        ),
        (
            "RUNTIME_IMAGE",
            "elif [[ \"$image\" == \"ghcr.io/chrischeng-c4/lumen@$V2_CHILD_DIGEST\" ]]; then",
        ),
        (
            "RUNTIME_IMAGE",
            "V2_OBSERVED_RUNTIME_IMAGE_DIGEST=\"$V2_CHILD_DIGEST\"",
        ),
        (
            "RUNTIME_IMAGE",
            "observed container imageID is not the exact candidate root or scheduled child digest",
        ),
        (
            "PUBLIC_IMAGE",
            "v2-public.json\" >/dev/null || die \"public runtime is not the candidate-version serving image\"",
        ),
        ("PATCH", "patch_statefulset_image \"$statefulset\" \"$label\""),
        ("PATCH", "patch_statefulset_image \"$V2_APPLY_ROOT/runtime/statefulset.yaml\" v2"),
        ("PATCH", "yaml_json \"$statefulset\" \"$canonical\""),
        ("PATCH", ".spec.template.spec.containers | length == 1"),
        ("PATCH", "--arg expected_image \"$CANDIDATE_DEFAULT_IMAGE\""),
        ("PATCH", "(.spec.template.spec.containers | length == 1)\n    and .spec.template.spec.containers[0].name == \"serving\"\n    and .spec.template.spec.containers[0].image == $expected_image"),
        ("PATCH", ".spec.template.spec.containers[0].image == $expected_image"),
        ("PATCH", ".spec.template.spec.containers[0].image != $expected_image"),
        ("PATCH", ".spec.template.spec.containers[0].name == \"serving\""),
        ("PATCH", ".spec.template.spec.containers[0].image = $image"),
        ("PATCH", "mv -f -- \"$patched\" \"$statefulset\""),
        ("PATCH", "cmp -s \"$original_canonical\" \"$patched_canonical\""),
        ("PATCH", "cmp -s \"$TMP_ROOT/v2-public-no-image.json\" \"$TMP_ROOT/v2-private-no-image.json\""),
        ("PATCH", "digest patch changed fields other than the serving image"),
        ("PATCH", "private runtime changed fields other than serving image"),
        ("ARCHIVE", "tar -xOf"),
        (
            "ARCHIVE",
            "candidate controller archive has unexpected members",
        ),
        ("ARCHIVE", "candidate controller binary is not executable"),
        ("ARCHIVE", "candidate archive hash mismatch"),
        ("ARCHIVE", "candidate archive sidecar hash mismatch"),
        ("ARCHIVE", "LC_ALL=C sort"),
        (
            "ARCHIVE",
            "candidate controller CLI bytes differ from local CLI",
        ),
        (
            "MUTATION",
            "k create -f \"$V2_APPLY_ROOT/storage/namespace.yaml\" -o json",
        ),
        (
            "MUTATION",
            "k create -f \"$V2_APPLY_ROOT/runtime/clusterrolebinding.yaml\" -o json",
        ),
        (
            "MUTATION",
            "k create -f \"$TMP_ROOT/v2-client/namespace.json\" -o json",
        ),
        ("MUTATION", "v2_stamp_private_ownership"),
        ("MUTATION", "V2_RUNTIME_NAMESPACE_UID=",),
        ("MUTATION", "V2_CLIENT_NAMESPACE_UID=",),
        ("MUTATION", "V2_CRB_UID=",),
        ("MUTATION", "V2_RUN_LABEL=\"lumen.axiom.dev/gke-acceptance-run\""),
        ("MUTATION", "k apply -k \"$V2_APPLY_ROOT/runtime\""),
        ("MUTATION", "k apply -k \"$V2_APPLY_ROOT/storage\""),
        (
            "MUTATION",
            "k apply -f \"$TMP_ROOT/v2-client/rendered.yaml\"",
        ),
        ("CLEANUP", "k delete clusterrolebinding \"$V2_CRB\""),
        ("CLEANUP", "k delete namespace \"$V2_CLIENT_NAMESPACE\""),
        ("CLEANUP", "k delete namespace \"$V2_RUNTIME_NAMESPACE\""),
        ("CLEANUP", "v2_wait_pv_gone \"$V2_PV_NAME\""),
        (
            "CLEANUP",
            "if [[ -z \"$V2_PV_NAME\" ]]; then",
        ),
        ("CLIENT", "v2_run_api_job unlisted unlisted default"),
        ("CLIENT", "v2_run_api_job missing default missing"),
        ("CLIENT", "v2_run_api_job bad unlisted bad"),
        ("CLIENT", "v2_run_client_tooling_job"),
        ("CLIENT", "[[ \"$1\" =~ ^[a-z0-9-]{1,40}$ ]]"),
        ("METRICS", "metrics-before-incluster"),
        ("METRICS", "delegated_auth_token_reviews_total"),
        ("METRICS", "delegated_auth_access_reviews_total"),
        ("METRICS", "delegated_auth_allowed_total"),
        ("METRICS", "delegated_auth_denied_total"),
        (
            "METRICS",
            "lumen-standalone-gke-metric-shape-failure.json",
        ),
        ("METRICS", "lumen.standalone-gke-metric-shape/v1"),
        ("REDACTION", "metric_values_retained:false"),
        ("REDACTION", "metric_label_values_retained:false"),
        (
            "METRICS",
            "v2_metric_deltas in-cluster \"$TMP_ROOT/metrics-before-incluster.metrics\" \"$TMP_ROOT/metrics-after-incluster.metrics\"",
        ),
        (
            "METRICS",
            "v2_metric_deltas required \"$TMP_ROOT/metrics-before-required.metrics\" \"$TMP_ROOT/metrics-after-required.metrics\"",
        ),
        (
            "METRICS",
            "db=\"$(v2_metric_total delegated_auth_denied_total \"$before\")\"",
        ),
        ("METRICS", "--argjson tokenreview_delta \"$((ta-tb))\""),
        ("RESTART", "k delete pod lumen-0"),
        ("RESTART", "v2_wait_pod \"$initial_uid\""),
        (
            "RESTART",
            "v2_wait_pod \"$replacement_uid\" in-cluster 1 1Gi",
        ),
        (
            "RESTART",
            "v2_wait_pod '' in-cluster 500m 512Mi\n  v2_capture_pvc_identity",
        ),
        ("RESTART", "PVC did not bind after the initial pod became Ready"),
        (
            "NETWORK",
            "ingresses=\"$(k get ingress --namespace \"$V2_RUNTIME_NAMESPACE\" -o name)\" || die \"Ingress inventory could not be read\"",
        ),
        (
            "NETWORK",
            "gateway_resources=\"$(k api-resources --api-group gateway.networking.k8s.io -o name)\" || die \"Gateway API inventory could not be read\"",
        ),
        (
            "NETWORK",
            "gateways=\"$(k get gateways.gateway.networking.k8s.io --namespace \"$V2_RUNTIME_NAMESPACE\" -o name)\" || die \"Gateway inventory could not be read\"",
        ),
        ("REQUIRED", "profile:\"LUMEN_AUTH=required\""),
        (
            "REQUIRED",
            "k get statefulset lumen --namespace \"$V2_RUNTIME_NAMESPACE\" -o json >\"$live\"",
        ),
        (
            "REQUIRED",
            "required continuity patch changed live desired fields other than LUMEN_AUTH",
        ),
        (
            "REQUIRED",
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata.creationTimestamp,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-before-required.json\" >\"$TMP_ROOT/v2-before-required-noauth.json\"",
        ),
        (
            "REQUIRED",
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata.creationTimestamp,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
        ),
        (
            "REQUIRED",
            "if ! cmp -s \"$TMP_ROOT/v2-before-required-noauth.json\" \"$TMP_ROOT/v2-after-required-noauth.json\"; then",
        ),
        (
            "REQUIRED",
            "v2_write_required_continuity_diff \"$TMP_ROOT/v2-before-required-noauth.json\" \"$TMP_ROOT/v2-after-required-noauth.json\" || die \"required continuity diagnostic could not be written\"",
        ),
        (
            "REQUIRED",
            "lumen-standalone-gke-required-continuity-diff.json",
        ),
        (
            "REQUIRED",
            "diagnostic=\"$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/lumen-standalone-gke-required-continuity-diff.json\"",
        ),
        (
            "REQUIRED",
            "lumen.standalone-gke-required-continuity-diff/v1",
        ),
        ("REQUIRED", "jq -n --slurpfile before \"$before\" --slurpfile after \"$after\""),
        ("REQUIRED", "(differences($before[0];$after[0];[])|unique) as $paths | {schema:\"lumen.standalone-gke-required-continuity-diff/v1\",paths:$paths}"),
        ("REQUIRED", "gsub(\"~\";\"~0\")|gsub(\"/\";\"~1\")"),
        ("REQUIRED", "if ($left|has($key)) and ($right|has($key)) then differences($left[$key];$right[$key];$path+[$key])"),
        ("REQUIRED", "if $index < ($left|length) and $index < ($right|length) then differences($left[$index];$right[$index];$path+[$index])"),
        ("REQUIRED", "(.paths == (.paths|sort|unique))"),
        ("REQUIRED", "(keys|sort) == [\"paths\",\"schema\"]"),
        ("REQUIRED", "chmod 600 \"$temporary\""),
        ("REQUIRED", "if ! ln -- \"$temporary\" \"$diagnostic\" 2>/dev/null; then"),
        ("REQUIRED", "if ! rm -f -- \"$temporary\"; then"),
        ("REQUIRED", "! -e \"$diagnostic\" && ! -L \"$diagnostic\""),
        ("REQUIRED", "required continuity diagnostic could not be written"),
        (
            "REQUIRED",
            "die \"required continuity patch changed live desired fields other than LUMEN_AUTH\"",
        ),
        (
            "REQUIRED",
            ".spec.template.spec.containers[0].image == $image and",
        ),
        (
            "REQUIRED",
            "([.spec.template.spec.containers[0].env[]|select(.name == \"LUMEN_AUTH\")|.value] == [\"required\"])",
        ),
        (
            "REQUIRED",
            "([.spec.template.spec.containers[0].resources.requests.cpu] == [\"1\"])",
        ),
        (
            "REQUIRED",
            "([.spec.template.spec.containers[0].resources.requests.memory] == [\"1Gi\"])",
        ),
        ("REQUIRED", "V2_REQUIRED_STATEFULSET=\"$TMP_ROOT/v2-required-statefulset.json\""),
        ("REQUIRED", "k apply -f \"$V2_REQUIRED_STATEFULSET\""),
        ("REQUIRED", "required-projected-app app projected"),
        ("REQUIRED", "required-default-app app default"),
        ("REQUIRED", "required-projected-unlisted unlisted projected"),
        ("REQUIRED", "audience:\"lumen.axiom.dev\""),
        (
            "REQUIRED",
            "profile:\"LUMEN_AUTH=required\",audience:\"lumen.axiom.dev\"",
        ),
        ("REQUIRED", "required-projected-app app projected POST"),
        (
            "REQUIRED",
            "required-projected-app app projected POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 2xx durable-first none",
        ),
        (
            "REQUIRED",
            "required-default-app app default POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 401 none none",
        ),
        (
            "REQUIRED",
            "required-projected-unlisted unlisted projected POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 403 none none",
        ),
        ("RECEIPT", "lumen.standalone-gke-receipt/v2"),
        ("RECEIPT", "--arg version \"$CANDIDATE_VERSION\""),
        ("RECEIPT", "cluster_identity_retained:false"),
        ("RECEIPT", "command_output_retained:false"),
        ("RECEIPT", "canary_scan:true"),
        ("RECEIPT", "receipt has unexpected keys"),
        ("RECEIPT", "receipt candidate has unexpected keys"),
        ("RECEIPT", "receipt controller CLI has unexpected keys"),
        ("RECEIPT", "receipt matrix has unexpected keys"),
        ("RECEIPT", "receipt required continuity has unexpected keys"),
        ("RECEIPT", "receipt redaction has unexpected keys"),
        (
            "RECEIPT",
            ".schema == \"lumen.standalone-gke-receipt/v2\" and .stage == \"slice-b-live\" and .complete == true",
        ),
        (
            "RECEIPT",
            "([.matrix|to_entries[]|select(.key != \"required_continuity\")|.value] | all(.[]; . == \"passed\"))",
        ),
        ("RECEIPT", ".redaction == {authorization_retained:false,canary_scan:true,cluster_identity_retained:false,command_output_retained:false,kubeconfig_retained:false,secret_retained:false,token_retained:false}"),
        (
            "RECEIPT",
            "receipt bytes must be within the 16KiB workflow transport limit",
        ),
        (
            "RECEIPT",
            "receipt_bytes=\"$(wc -c < \"$RECEIPT_TMP\" | tr -d ' ')\"",
        ),
        (
            "RECEIPT",
            "\"$receipt_bytes\" -gt 0 && \"$receipt_bytes\" -le 16384",
        ),
        (
            "RECEIPT",
            ".candidate.amd64_digest != .candidate.arm64_digest",
        ),
        (
            "RECEIPT",
            "((.matrix.required_continuity.scheduled_node_arch == \"amd64\" and .matrix.required_continuity.scheduled_runtime_child_digest == .candidate.amd64_digest and (.matrix.required_continuity.observed_runtime_image_digest == .candidate.root_digest or .matrix.required_continuity.observed_runtime_image_digest == .candidate.amd64_digest)) or (.matrix.required_continuity.scheduled_node_arch == \"arm64\" and .matrix.required_continuity.scheduled_runtime_child_digest == .candidate.arm64_digest and (.matrix.required_continuity.observed_runtime_image_digest == .candidate.root_digest or .matrix.required_continuity.observed_runtime_image_digest == .candidate.arm64_digest)))",
        ),
        ("RECEIPT", "type == \"number\" and floor == . and . > 0"),
        ("RECEIPT", "--argjson required_deltas \"$V2_REQUIRED_DELTAS\""),
        (
            "RECEIPT",
            "RECEIPT_TMP=\"$(mktemp \"$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/.receipt.XXXXXX\")\" || return 1",
        ),
        (
            "RECEIPT",
            "RECEIPT_SIDECAR_TMP=\"$(mktemp \"$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/.receipt-sidecar.XXXXXX\")\"",
        ),
        (
            "RECEIPT",
            "if ! (v2_write_receipt_body); then",
        ),
        ("CLEANUP", "v2_absent namespace \"$V2_RUNTIME_NAMESPACE\""),
        ("CLEANUP", "v2_absent namespace \"$V2_CLIENT_NAMESPACE\""),
        ("CLEANUP", "v2_absent clusterrolebinding \"$V2_CRB\""),
        (
            "CLEANUP",
            "v2_absent namespace \"$V2_RUNTIME_NAMESPACE\" || V2_CLEAN=false\n  v2_absent namespace \"$V2_CLIENT_NAMESPACE\" || V2_CLEAN=false\n  v2_absent clusterrolebinding \"$V2_CRB\" || V2_CLEAN=false",
        ),
        ("REDACTION", "receipt redaction scan failed"),
    ] {
        if !source.contains(required) {
            bad.push(code);
        }
    }
    for required in [
        "[[ \"$RECEIPT_SHA256\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256\" ]] ||",
        "[[ \"$CANDIDATE_COMMIT\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_COMMIT\" ]] ||",
        "[[ \"$CANDIDATE_RUN_ID\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID\" ]] ||",
        "[[ \"$CANDIDATE_ATTEMPT\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ATTEMPT\" ]] ||",
    ] {
        if !has_exact_line(source, required) {
            bad.push("CANDIDATE");
        }
    }
    if !has_exact_line(
        function_body(source, "v2_wait_pod"),
        ".spec.enableServiceLinks == false and",
    ) {
        bad.push("REQUIRED");
    }
    let required_runtime = function_body(source, "v2_required_runtime");
    let required_continuity_block = "if ! cmp -s \"$TMP_ROOT/v2-before-required-noauth.json\" \"$TMP_ROOT/v2-after-required-noauth.json\"; then\n    v2_write_required_continuity_diff \"$TMP_ROOT/v2-before-required-noauth.json\" \"$TMP_ROOT/v2-after-required-noauth.json\" || die \"required continuity diagnostic could not be written\"\n    die \"required continuity patch changed live desired fields other than LUMEN_AUTH\"\n  fi";
    if !required_runtime.contains(required_continuity_block) {
        bad.push("REQUIRED");
    }
    let diagnostic_writer = function_body(source, "v2_write_required_continuity_diff");
    let no_clobber_publish = "if ! ln -- \"$temporary\" \"$diagnostic\" 2>/dev/null; then\n    rm -f -- \"$temporary\"\n    return 1\n  fi\n  if ! rm -f -- \"$temporary\"; then\n    return 1\n  fi";
    if !diagnostic_writer.contains(no_clobber_publish)
        || diagnostic_writer.contains("mv ")
        || diagnostic_writer.contains("cp ")
        || diagnostic_writer.contains("ln -f")
        || diagnostic_writer.contains("ln --force")
        || diagnostic_writer.contains("rm -f -- \"$diagnostic\"")
    {
        bad.push("REQUIRED");
    }
    for required in [
        "[[ \"$RECEIPT_SHA256\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256\" ]] ||\n    die \"candidate manifest hash differs from the controller-bound expected hash\"",
        "[[ \"$CANDIDATE_COMMIT\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_COMMIT\" ]] ||\n    die \"candidate manifest commit differs from the controller-bound landed commit\"",
        "[[ \"$CANDIDATE_RUN_ID\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID\" ]] ||\n    die \"candidate manifest run id differs from the controller-bound expected run\"",
        "[[ \"$CANDIDATE_ATTEMPT\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ATTEMPT\" ]] ||\n    die \"candidate manifest run attempt differs from the controller-bound expected attempt\"",
    ] {
        if !source.contains(required) {
            bad.push("CANDIDATE");
        }
    }
    for required in [
        ".schema == \"lumen.standalone-gke-receipt/v2\" and .stage == \"slice-b-live\" and .complete == true and",
        "([.matrix|to_entries[]|select(.key != \"required_continuity\")|.value] | all(.[]; . == \"passed\")) and",
        ".redaction == {authorization_retained:false,canary_scan:true,cluster_identity_retained:false,command_output_retained:false,kubeconfig_retained:false,secret_retained:false,token_retained:false} and",
    ] {
        if !has_exact_line(source, required) {
            bad.push("RECEIPT");
        }
    }
    if source.contains("k apply -f \"$V2_APPLY_ROOT/storage\"")
        || source.contains("k apply -f \"$V2_APPLY_ROOT/runtime\"")
        || source.contains("k apply -f \"$V2_APPLY_ROOT/storage/namespace.yaml\"")
        || source.contains("k apply -f \"$V2_APPLY_ROOT/runtime/clusterrolebinding.yaml\"")
        || source.contains("k apply -f \"$TMP_ROOT/v2-client/namespace.json\"")
    {
        bad.push("MUTATION");
    }
    if source.contains("kubectl create token")
        || source.contains("curl http://")
        || source.contains("-d BODY")
        || source.contains("@$AMD64_DIGEST\" ||")
        || source.contains("$AMD64_DIGEST\" || \"$ARM64_DIGEST")
        || source.contains("grep -qx \"status=$expected\"")
        || source.contains("tar -xzf")
        || source.contains("gateway.networking.k8s.io --namespace \"$V2_RUNTIME_NAMESPACE\" -o name 2>/dev/null || true")
    {
        bad.push("CLIENT");
    }
    if source.contains("case \"$arch\" in amd64|arm64)") {
        bad.push("CLIENT");
    }
    let metric_failure = function_body(source, "v2_write_metric_failure");
    for required in [
        "case \"$profile\" in in-cluster|required) ;; *) return 1 ;; esac",
        "case \"$reason\" in missing_metric|non_positive_delta) ;; *) return 1 ;; esac",
        "report=\"$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/lumen-standalone-gke-metric-shape-failure.json\"",
        "tmp=\"$(mktemp \"$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/.metric-shape.XXXXXX\")\" || return 1",
        "chmod 600 \"$tmp\"",
        "safe_private_file \"$report\"",
        "metric_values_retained:false",
        "metric_label_values_retained:false",
        "(keys | sort) == [\"failure\",\"observations\",\"redaction\",\"schema\"]",
        "before_token=\"$(v2_metric_shape delegated_auth_token_reviews_total \"$before\")\" || return 1",
        "after_denied=\"$(v2_metric_shape delegated_auth_denied_total \"$after\")\" || return 1",
    ] {
        if !metric_failure.contains(required) {
            bad.push("METRICS");
        }
    }
    if metric_failure.contains("cat \"$before\"")
        || metric_failure.contains("cat \"$after\"")
        || metric_failure.contains("cp \"$before\"")
        || metric_failure.contains("cp \"$after\"")
    {
        bad.push("REDACTION");
    }
    let metric_deltas = function_body(source, "v2_metric_deltas");
    for required in [
        "local profile=\"$1\" before=\"$2\" after=\"$3\"",
        "v2_fail_metric_delta \"$profile\" missing_metric \"$before\" \"$after\"",
        "v2_fail_metric_delta \"$profile\" non_positive_delta \"$before\" \"$after\"",
    ] {
        if !metric_deltas.contains(required) {
            bad.push("METRICS");
        }
    }
    let metric_fail = function_body(source, "v2_fail_metric_delta");
    for required in [
        "v2_write_metric_failure \"$profile\" \"$reason\" \"$before\" \"$after\" ||",
        "die \"could not retain redacted metric shape evidence\"",
    ] {
        if !metric_fail.contains(required) {
            bad.push("METRICS");
        }
    }
    let run = function_body(source, "run_live_acceptance_v2");
    for required in [
        "v2_run_metrics_job metrics-before-incluster",
        "v2_run_api_job marker-after-resize app default POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 2xx durable-first none",
        "v2_run_api_job metrics-denied-after-resize unlisted default POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 403 none none",
        "v2_run_metrics_job metrics-after-incluster",
        "v2_run_metrics_job metrics-before-required",
        "v2_run_metrics_job metrics-after-required",
    ] {
        if !has_exact_line(run, required) {
            bad.push("METRICS");
        }
    }
    if !appears_before(run, "v2_wait_pod \"$replacement_uid\" in-cluster 1 1Gi", "v2_run_metrics_job metrics-before-incluster")
        || !appears_before(run, "v2_run_metrics_job metrics-before-incluster", "v2_run_api_job marker-after-resize")
        || !appears_before(run, "v2_run_api_job marker-after-resize", "v2_run_api_job metrics-denied-after-resize")
        || !appears_before(run, "v2_run_api_job metrics-denied-after-resize", "v2_run_metrics_job metrics-after-incluster")
        || !appears_before(run, "v2_run_metrics_job metrics-after-incluster", "v2_metric_deltas in-cluster")
        || !appears_before(run, "v2_wait_pod \"$resized_uid\" required 1 1Gi", "v2_run_metrics_job metrics-before-required")
        || !appears_before(run, "v2_run_metrics_job metrics-before-required", "v2_run_api_job required-projected-app")
        || !appears_before(run, "v2_run_api_job required-projected-app", "v2_run_api_job required-default-app")
        || !appears_before(run, "v2_run_api_job required-default-app", "v2_run_api_job required-projected-unlisted")
        || !appears_before(run, "v2_run_api_job required-projected-unlisted", "v2_run_metrics_job metrics-after-required")
        || !appears_before(run, "v2_run_metrics_job metrics-after-required", "v2_metric_deltas required")
    {
        bad.push("METRICS");
    }
    if source.contains("--selector") || source.contains("get all,") {
        bad.push("CLEANUP");
    }
    if source.contains("cp -R \"$V2_APPLY_ROOT/runtime\"")
        || source.contains("V2_REQUIRED_RUNTIME")
        || source.contains("jq '.resources |= map")
    {
        bad.push("REQUIRED");
    }
    if source.contains("$undefined") {
        bad.push("RECEIPT");
    }
    if source.matches("patch_statefulset_image").count() != 3 || source.contains("k set image") {
        bad.push("PATCH");
    }

    let state = function_body(source, "v2_get_state");
    if state.matches("--ignore-not-found -o json").count() != 2 {
        bad.push("CLEANUP");
    }
    for required in [
        "[[ \"$status\" -eq 0 ]] || return 2",
        "if [[ ! -s \"$response\" ]]; then",
        "return 1",
        "jq -e 'type == \"object\"' \"$response\" >/dev/null 2>&1 || return 2",
        "return 0",
    ] {
        if !has_exact_line(state, required) {
            bad.push("CLEANUP");
        }
    }
    let absent = function_body(source, "v2_absent");
    for required in ["[[ \"$state\" -eq 1 ]] && return 0", "return 2"] {
        if !has_exact_line(absent, required) {
            bad.push("CLEANUP");
        }
    }

    for required in [
        "V2_RUNTIME_ARMED=true\n  k create -f \"$V2_APPLY_ROOT/storage/namespace.yaml\" -o json",
        "V2_CRB_ARMED=true\n  k create -f \"$V2_APPLY_ROOT/runtime/clusterrolebinding.yaml\" -o json",
        "V2_CLIENT_ARMED=true\n  if ! k create -f \"$TMP_ROOT/v2-client/namespace.json\" -o json",
        ".metadata.uid == $uid and\n    .metadata.name == $name",
        ".metadata.uid == $uid and .metadata.name == $ns",
        ".metadata.labels[\"app.kubernetes.io/managed-by\"] == \"lumen-standalone-gke-acceptance\"",
        "v2_recover_created_uids",
        "v2_recover_created_uids\n  if [[ \"$V2_CRB_ARMED\" == true ]]",
        "v2_assert_runtime_namespace\n  V2_CRB_ARMED=true",
        "k apply -k \"$V2_APPLY_ROOT/storage\" >/dev/null; then die \"storage apply failed\"; fi\n  v2_assert_runtime_namespace",
        "if ! k apply -f \"$TMP_ROOT/v2-client/rendered.yaml\" >/dev/null; then die \"client apply failed\"; fi",
    ] {
        if !source.contains(required) {
            bad.push("MUTATION");
        }
    }

    let recovery = function_body(source, "v2_recover_created_uids");
    for required in [
        "v2-runtime-namespace-create.json",
        "v2-crb-create.json",
        "v2-client-namespace-create.json",
    ] {
        if !recovery.contains(required) {
            bad.push("MUTATION");
        }
    }
    for required in [
        "V2_RUNTIME_NAMESPACE_UID=\"$recovered\"",
        "V2_CRB_UID=\"$recovered\"",
        "V2_CLIENT_NAMESPACE_UID=\"$recovered\"",
    ] {
        if !has_exact_line(recovery, required) {
            bad.push("MUTATION");
        }
    }
    let crb = function_body(source, "v2_crb_owned");
    for required in [
        ".metadata.uid == $uid and",
        ".metadata.labels[\"app.kubernetes.io/managed-by\"] == \"lumen-standalone-gke-acceptance\" and",
        ".roleRef == {apiGroup:\"rbac.authorization.k8s.io\",kind:\"ClusterRole\",name:\"system:auth-delegator\"} and",
    ] {
        if !has_exact_line(crb, required) {
            bad.push("MUTATION");
        }
    }

    let run = function_body(source, "run_live_acceptance_v2");
    for line in run.lines().map(str::trim) {
        if line.starts_with("v2_run_api_job ") || line.starts_with("v2_run_metrics_job ") {
            let label = line.split_whitespace().nth(1).unwrap_or_default();
            if label.is_empty()
                || label.len() > 40
                || !label
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            {
                bad.push("CLIENT");
            }
        }
    }
    for required in [
        "v2_run_api_job unlisted unlisted default POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 403 none none",
        "v2_run_api_job missing default missing POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 401 none none",
        "v2_run_api_job bad unlisted bad POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 401 none none",
        "v2_run_api_job application-admin app default GET /admin/backup '' 403 none none",
        "v2_run_api_job required-projected-app app projected POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 2xx durable-first none",
        "v2_run_api_job required-default-app app default POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 401 none none",
        "v2_run_api_job required-projected-unlisted unlisted projected POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 403 none none",
    ] {
        if !has_exact_line(run, required) {
            bad.push("EXECUTION");
        }
    }
    if run.lines().any(|line| {
        let line = line.trim();
        line == "return"
            || line.starts_with("return ")
            || line == "exit"
            || line.starts_with("exit ")
            || line == "if false; then"
            || line.starts_with("false &&")
            || line.contains("<<")
    }) {
        bad.push("EXECUTION");
    }
    let expected_tail =
        "run_live_acceptance() {\n  run_live_acceptance_v2\n}\n\nrun_live_acceptance\n\nexit 0\n";
    if !source.ends_with(expected_tail) {
        bad.push("EXECUTION");
    }

    if !appears_before(
        source,
        "mv -f -- \"$RECEIPT_SIDECAR_TMP\" \"$receipt.sha256\"",
        "mv -f -- \"$RECEIPT_TMP\" \"$receipt\"",
    ) {
        bad.push("RECEIPT");
    }
    let receipt = function_body(source, "v2_write_receipt");
    if !receipt.contains(
        "if ! (v2_write_receipt_body); then\n    rm -f -- \"$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/lumen-standalone-gke-receipt.json\" \"$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/lumen-standalone-gke-receipt.json.sha256\" \"$RECEIPT_TMP\" \"$RECEIPT_SIDECAR_TMP\"",
    ) {
        bad.push("RECEIPT");
    }
    bad
}

fn preflight_findings(source: &str) -> Vec<&'static str> {
    let mut bad = Vec::new();
    for (code, required) in [
        ("PREFLIGHT", "set +x"),
        ("PREFLIGHT", "umask 077"),
        (
            "PREFLIGHT",
            "SCRIPT_DIR=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")\" && pwd -P)\"",
        ),
        (
            "PREFLIGHT",
            "REPO_ROOT=\"$(cd \"$SCRIPT_DIR/../../..\" && pwd -P)\"",
        ),
        (
            "PREFLIGHT",
            "KUSTOMIZE_SOURCE_ROOT=\"$REPO_ROOT/kustomize/lumen-standalone-acceptance\"",
        ),
        (
            "PREFLIGHT",
            "KUSTOMIZE_RENDERER_SOURCE=\"$KUSTOMIZE_SOURCE_ROOT/scripts/render.sh\"",
        ),
        (
            "PREFLIGHT",
            "KUSTOMIZE_VALIDATOR_SOURCE=\"$KUSTOMIZE_SOURCE_ROOT/scripts/validate.rb\"",
        ),
        (
            "PREFLIGHT",
            "KUSTOMIZE_RENDERER_SHA256=\"f83e347b5f66c6cad049595a776230ea559f80efc265129f407032bf5a93dd74\"",
        ),
        (
            "PREFLIGHT",
            "KUSTOMIZE_VALIDATOR_SHA256=\"43355d4a083303c9ffadade98f4add46958d7a7e625100dea97d979a3d1d294e\"",
        ),
        ("PREFLIGHT", "LUMEN_STANDALONE_GKE_MUTATION=1"),
        ("PREFLIGHT", "reject_token_env\n\nrequire_tool"),
        ("PREFLIGHT", "ruby; do"),
        (
            "PREFLIGHT",
            "[[ -d \"$KUSTOMIZE_SOURCE_ROOT\" && ! -L \"$KUSTOMIZE_SOURCE_ROOT\" ]] ||",
        ),
        (
            "PREFLIGHT",
            "[[ -f \"$KUSTOMIZE_RENDERER_SOURCE\" && -x \"$KUSTOMIZE_RENDERER_SOURCE\" && ! -L \"$KUSTOMIZE_RENDERER_SOURCE\" ]] ||",
        ),
        (
            "PREFLIGHT",
            "[[ -f \"$KUSTOMIZE_VALIDATOR_SOURCE\" && -x \"$KUSTOMIZE_VALIDATOR_SOURCE\" && ! -L \"$KUSTOMIZE_VALIDATOR_SOURCE\" ]] ||",
        ),
        (
            "PREFLIGHT",
            "[[ -z \"$(find \"$KUSTOMIZE_SOURCE_ROOT\" -type l -print -quit)\" ]] ||",
        ),
        (
            "PREFLIGHT",
            "KUSTOMIZE_ROOT=\"$PRIVATE_REPOSITORY_ROOT/kustomize/lumen-standalone-acceptance\"",
        ),
        (
            "PREFLIGHT",
            "cp -R -- \"$KUSTOMIZE_SOURCE_ROOT\" \"$KUSTOMIZE_ROOT\"",
        ),
        (
            "PREFLIGHT",
            "KUSTOMIZE_RENDERER=\"$KUSTOMIZE_ROOT/scripts/render.sh\"",
        ),
        (
            "PREFLIGHT",
            "KUSTOMIZE_VALIDATOR=\"$KUSTOMIZE_ROOT/scripts/validate.rb\"",
        ),
        (
            "PREFLIGHT",
            "[[ -f \"$KUSTOMIZE_RENDERER\" && -x \"$KUSTOMIZE_RENDERER\" && ! -L \"$KUSTOMIZE_RENDERER\" ]] ||",
        ),
        (
            "PREFLIGHT",
            "[[ -f \"$KUSTOMIZE_VALIDATOR\" && -x \"$KUSTOMIZE_VALIDATOR\" && ! -L \"$KUSTOMIZE_VALIDATOR\" ]] ||",
        ),
        (
            "PREFLIGHT",
            "[[ \"$(sha256_file \"$KUSTOMIZE_RENDERER\")\" == \"$KUSTOMIZE_RENDERER_SHA256\" ]] ||",
        ),
        (
            "PREFLIGHT",
            "[[ \"$(sha256_file \"$KUSTOMIZE_VALIDATOR\")\" == \"$KUSTOMIZE_VALIDATOR_SHA256\" ]] ||",
        ),
        (
            "PREFLIGHT",
            "safe_private_file \"$KUBECONFIG\" ||",
        ),
        ("PREFLIGHT", "config current-context"),
        (
            "PREFLIGHT",
            "task-local kubeconfig current context must equal LUMEN_STANDALONE_GKE_CONTEXT",
        ),
        (
            "PREFLIGHT",
            "task-local kubeconfig must contain exactly the requested context",
        ),
        (
            "PREFLIGHT",
            "requested StorageClass must exist and use reclaimPolicy Delete for acceptance cleanup",
        ),
        ("PREFLIGHT", ".reclaimPolicy == \"Delete\""),
        (
            "PREFLIGHT",
            "  LUMEN_STANDALONE_GKE_EXPECTED_COMMIT \\\n",
        ),
        (
            "PREFLIGHT",
            "  LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID \\\n",
        ),
        (
            "PREFLIGHT",
            "  LUMEN_STANDALONE_GKE_EXPECTED_RUN_ATTEMPT \\\n",
        ),
        (
            "PREFLIGHT",
            "  LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256 \\\n",
        ),
        (
            "PREFLIGHT",
            "APPROVED_CLIENT_IMAGE=\"docker.io/curlimages/curl@sha256:7c12af72ceb38b7432ab85e1a265cff6ae58e06f95539d539b654f2cfa64bb13\"",
        ),
        (
            "PREFLIGHT",
            "[[ \"$LUMEN_STANDALONE_GKE_CLIENT_IMAGE\" == \"$APPROVED_CLIENT_IMAGE\" ]]",
        ),
    ] {
        if !source.contains(required) {
            bad.push(code);
        }
    }
    for required in [
        "CDPATH=",
        "PRIVATE_TMP_ROOT=\"$(cd -P /tmp && pwd -P)\"",
        "case \"$PRIVATE_TMP_ROOT\" in /tmp|/private/tmp) ;; *) die 'unsupported private temp root' ;; esac",
        "safe_private_dir() {",
        "[[ \"$path\" == \"$PRIVATE_TMP_ROOT\"/* && \"$path\" != */ && \"$path\" != *'/../'* && \"$path\" != */.. && \"$path\" != *'/./'* && -d \"$path\" && ! -L \"$path\" ]] || return 1",
        "[[ \"$(cd \"$path\" && pwd -P)\" == \"$path\" ]]",
        "safe_private_file() {",
        "[[ \"$path\" == \"$PRIVATE_TMP_ROOT\"/* && \"$path\" != *'/../'* && \"$path\" != */.. && \"$path\" != *'/./'* && -f \"$path\" && ! -L \"$path\" ]] || return 1",
        "parent=${path%/*}; safe_private_dir \"$parent\"",
        "private_mode() {",
        "safe_private_file \"$KUBECONFIG\" ||",
        "safe_private_dir \"$LUMEN_STANDALONE_GKE_EVIDENCE_DIR\" ||",
        "TMP_ROOT=\"$(mktemp -d \"$PRIVATE_TMP_ROOT/lumen-standalone-gke.XXXXXX\")\"",
        "[[ \"$(private_mode \"$TMP_ROOT\")\" == 700 ]] || die 'private temporary root mode is not 0700'",
        "KUBECTL_CACHE_DIR=\"$TMP_ROOT/kubectl-cache\"",
        "[[ ! -e \"$KUBECTL_CACHE_DIR\" && ! -L \"$KUBECTL_CACHE_DIR\" ]] || die 'kubectl cache path already exists'",
        "mkdir -m 700 \"$KUBECTL_CACHE_DIR\"",
        "[[ -d \"$KUBECTL_CACHE_DIR\" && ! -L \"$KUBECTL_CACHE_DIR\" && \"$(cd \"$KUBECTL_CACHE_DIR\" && pwd -P)\" == \"$KUBECTL_CACHE_DIR\" ]] || die 'kubectl cache path is not canonical'",
        "[[ \"${KUBECTL_CACHE_DIR%/*}\" == \"$TMP_ROOT\" && \"${KUBECTL_CACHE_DIR##*/}\" == kubectl-cache ]] || die 'kubectl cache path identity is unsafe'",
        "[[ \"$(private_mode \"$KUBECTL_CACHE_DIR\")\" == 700 ]] || die 'kubectl cache path mode is not 0700'",
        "SCRIPT_DIR=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")\" && pwd -P)\"",
        "REPO_ROOT=\"$(cd \"$SCRIPT_DIR/../../..\" && pwd -P)\"",
        "KUSTOMIZE_SOURCE_ROOT=\"$REPO_ROOT/kustomize/lumen-standalone-acceptance\"",
        "KUSTOMIZE_RENDERER_SOURCE=\"$KUSTOMIZE_SOURCE_ROOT/scripts/render.sh\"",
        "KUSTOMIZE_VALIDATOR_SOURCE=\"$KUSTOMIZE_SOURCE_ROOT/scripts/validate.rb\"",
        "PRIVATE_REPOSITORY_ROOT=\"$TMP_ROOT/repository\"",
        "[[ ! -e \"$PRIVATE_REPOSITORY_ROOT\" && ! -L \"$PRIVATE_REPOSITORY_ROOT\" ]] ||",
        "mkdir -m 700 \"$PRIVATE_REPOSITORY_ROOT\"",
        "mkdir -m 700 \"$PRIVATE_REPOSITORY_ROOT/kustomize\"",
        "[[ -d \"$PRIVATE_REPOSITORY_ROOT\" && ! -L \"$PRIVATE_REPOSITORY_ROOT\" && -d \"$PRIVATE_REPOSITORY_ROOT/kustomize\" && ! -L \"$PRIVATE_REPOSITORY_ROOT/kustomize\" ]] ||",
        "PRIVATE_REPOSITORY_ROOT=\"$(cd \"$PRIVATE_REPOSITORY_ROOT\" && pwd -P)\" || die \"private repository root cannot be canonicalized\"",
        "KUSTOMIZE_ROOT=\"$PRIVATE_REPOSITORY_ROOT/kustomize/lumen-standalone-acceptance\"",
        "KUSTOMIZE_RENDERER=\"$KUSTOMIZE_ROOT/scripts/render.sh\"",
        "KUSTOMIZE_VALIDATOR=\"$KUSTOMIZE_ROOT/scripts/validate.rb\"",
        "KUSTOMIZE_RENDERER_SHA256=\"f83e347b5f66c6cad049595a776230ea559f80efc265129f407032bf5a93dd74\"",
        "KUSTOMIZE_VALIDATOR_SHA256=\"43355d4a083303c9ffadade98f4add46958d7a7e625100dea97d979a3d1d294e\"",
        "ruby; do",
        "[[ -d \"$KUSTOMIZE_SOURCE_ROOT\" && ! -L \"$KUSTOMIZE_SOURCE_ROOT\" ]] ||",
        "[[ -f \"$KUSTOMIZE_RENDERER_SOURCE\" && -x \"$KUSTOMIZE_RENDERER_SOURCE\" && ! -L \"$KUSTOMIZE_RENDERER_SOURCE\" ]] ||",
        "[[ -f \"$KUSTOMIZE_VALIDATOR_SOURCE\" && -x \"$KUSTOMIZE_VALIDATOR_SOURCE\" && ! -L \"$KUSTOMIZE_VALIDATOR_SOURCE\" ]] ||",
        "[[ -z \"$(find \"$KUSTOMIZE_SOURCE_ROOT\" -type l -print -quit)\" ]] ||",
        "[[ ! -e \"$KUSTOMIZE_ROOT\" && ! -L \"$KUSTOMIZE_ROOT\" ]] ||",
        "cp -R -- \"$KUSTOMIZE_SOURCE_ROOT\" \"$KUSTOMIZE_ROOT\"",
        "[[ -d \"$KUSTOMIZE_ROOT\" && ! -L \"$KUSTOMIZE_ROOT\" ]] ||",
        "[[ -z \"$(find \"$KUSTOMIZE_ROOT\" -type l -print -quit)\" ]] ||",
        "[[ -f \"$KUSTOMIZE_RENDERER\" && -x \"$KUSTOMIZE_RENDERER\" && ! -L \"$KUSTOMIZE_RENDERER\" ]] ||",
        "[[ -f \"$KUSTOMIZE_VALIDATOR\" && -x \"$KUSTOMIZE_VALIDATOR\" && ! -L \"$KUSTOMIZE_VALIDATOR\" ]] ||",
        "[[ \"$(sha256_file \"$KUSTOMIZE_RENDERER\")\" == \"$KUSTOMIZE_RENDERER_SHA256\" ]] ||",
        "[[ \"$(sha256_file \"$KUSTOMIZE_VALIDATOR\")\" == \"$KUSTOMIZE_VALIDATOR_SHA256\" ]] ||",
        "LUMEN_STANDALONE_GKE_EXPECTED_COMMIT \\",
        "LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID \\",
        "LUMEN_STANDALONE_GKE_EXPECTED_RUN_ATTEMPT \\",
        "LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256 \\",
        "APPROVED_CLIENT_IMAGE=\"docker.io/curlimages/curl@sha256:7c12af72ceb38b7432ab85e1a265cff6ae58e06f95539d539b654f2cfa64bb13\"",
        "[[ \"$LUMEN_STANDALONE_GKE_CLIENT_IMAGE\" == \"$APPROVED_CLIENT_IMAGE\" ]] ||",
    ] {
        if !has_exact_line(source, required) {
            bad.push("PREFLIGHT");
        }
    }
    if !appears_before(
        source,
        "TMP_ROOT=\"$(mktemp -d \"$PRIVATE_TMP_ROOT/lumen-standalone-gke.XXXXXX\")\"",
        "PRIVATE_REPOSITORY_ROOT=\"$TMP_ROOT/repository\"",
    ) || !appears_before(
        source,
        "PRIVATE_REPOSITORY_ROOT=\"$TMP_ROOT/repository\"",
        "[[ ! -e \"$PRIVATE_REPOSITORY_ROOT\" && ! -L \"$PRIVATE_REPOSITORY_ROOT\" ]] ||",
    ) || !appears_before(
        source,
        "[[ ! -e \"$PRIVATE_REPOSITORY_ROOT\" && ! -L \"$PRIVATE_REPOSITORY_ROOT\" ]] ||",
        "mkdir -m 700 \"$PRIVATE_REPOSITORY_ROOT\"",
    ) || !appears_before(
        source,
        "mkdir -m 700 \"$PRIVATE_REPOSITORY_ROOT\"",
        "mkdir -m 700 \"$PRIVATE_REPOSITORY_ROOT/kustomize\"",
    ) || !appears_before(
        source,
        "mkdir -m 700 \"$PRIVATE_REPOSITORY_ROOT/kustomize\"",
        "[[ -d \"$PRIVATE_REPOSITORY_ROOT\" && ! -L \"$PRIVATE_REPOSITORY_ROOT\" && -d \"$PRIVATE_REPOSITORY_ROOT/kustomize\" && ! -L \"$PRIVATE_REPOSITORY_ROOT/kustomize\" ]] ||",
    ) || !appears_before(
        source,
        "[[ -d \"$PRIVATE_REPOSITORY_ROOT\" && ! -L \"$PRIVATE_REPOSITORY_ROOT\" && -d \"$PRIVATE_REPOSITORY_ROOT/kustomize\" && ! -L \"$PRIVATE_REPOSITORY_ROOT/kustomize\" ]] ||",
        "PRIVATE_REPOSITORY_ROOT=\"$(cd \"$PRIVATE_REPOSITORY_ROOT\" && pwd -P)\" || die \"private repository root cannot be canonicalized\"",
    ) || !appears_before(
        source,
        "PRIVATE_REPOSITORY_ROOT=\"$(cd \"$PRIVATE_REPOSITORY_ROOT\" && pwd -P)\" || die \"private repository root cannot be canonicalized\"",
        "KUSTOMIZE_ROOT=\"$PRIVATE_REPOSITORY_ROOT/kustomize/lumen-standalone-acceptance\"",
    ) || !appears_before(
        source,
        "KUSTOMIZE_ROOT=\"$PRIVATE_REPOSITORY_ROOT/kustomize/lumen-standalone-acceptance\"",
        "cp -R -- \"$KUSTOMIZE_SOURCE_ROOT\" \"$KUSTOMIZE_ROOT\"",
    ) || !appears_before(
        source,
        "cp -R -- \"$KUSTOMIZE_SOURCE_ROOT\" \"$KUSTOMIZE_ROOT\"",
        "[[ \"$(sha256_file \"$KUSTOMIZE_RENDERER\")\" == \"$KUSTOMIZE_RENDERER_SHA256\" ]] ||",
    ) || !appears_before(
        source,
        "[[ \"$(sha256_file \"$KUSTOMIZE_RENDERER\")\" == \"$KUSTOMIZE_RENDERER_SHA256\" ]] ||",
        "\"$KUSTOMIZE_RENDERER\" tooling \\",
    ) || !appears_before(
        source,
        "[[ \"$(sha256_file \"$KUSTOMIZE_VALIDATOR\")\" == \"$KUSTOMIZE_VALIDATOR_SHA256\" ]] ||",
        "ruby \"$KUSTOMIZE_VALIDATOR\" tooling \\",
    ) {
        bad.push("PREFLIGHT");
    }
    let before_first_cleanup_trap = source
        .split_once("trap cleanup EXIT")
        .map(|(before, _)| before)
        .unwrap_or_default();
    if !before_first_cleanup_trap.contains("cleanup() {")
        || !before_first_cleanup_trap.contains("rm -rf -- \"$TMP_ROOT\"")
        || before_first_cleanup_trap.contains("v2_cleanup")
    {
        bad.push("PREFLIGHT");
    }
    for call in [
        "safe_private_file \"$KUBECONFIG\" ||",
        "safe_private_dir \"$LUMEN_STANDALONE_GKE_EVIDENCE_DIR\" ||",
    ] {
        if !appears_before(
            source,
            call,
            "TMP_ROOT=\"$(mktemp -d \"$PRIVATE_TMP_ROOT/lumen-standalone-gke.XXXXXX\")\"",
        ) {
            bad.push("PREFLIGHT");
        }
    }
    if !kubectl_line_inventory_is_complete(source)
        || source.matches("--cache-dir \"$KUBECTL_CACHE_DIR\"").count() != 3
        || !appears_before(
            source,
            "[[ \"$(private_mode \"$KUBECTL_CACHE_DIR\")\" == 700 ]] || die 'kubectl cache path mode is not 0700'",
            "if ! context_name=\"$(k config get-contexts",
        )
    {
        bad.push("PREFLIGHT");
    }
    for variable in ["HOME=", "XDG_CACHE_HOME=", "XDG_CONFIG_HOME=", "CLOUDSDK_CONFIG="] {
        if source.lines().any(|line| line.contains(variable)) {
            bad.push("PREFLIGHT");
        }
    }
    bad
}

fn integrity_findings(source: &str) -> Vec<&'static str> {
    let mut bad = Vec::new();
    for required in [
        "checksum sidecar must contain exactly one hash and its exact filename",
        "cmp -s \"$sidecar\" <(printf",
        "LC_ALL=C sort",
        "tar -tzf \"$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR/$archive\" | LC_ALL=C sort",
        "README.md\\n' \"$target\" \"$target\" \"$target\" | LC_ALL=C sort",
        "VERIFIED_CLI=\"$TMP_ROOT/lumen-controller-verified\"",
        "private verified controller CLI hash changed",
        "\"$VERIFIED_CLI\" standalone gke render",
        "\"$VERIFIED_CLI\" standalone backup",
        "\"$VERIFIED_CLI\" standalone restore",
        "[[ \"$RECEIPT_SHA256\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256\" ]]",
        "[[ \"$CANDIDATE_COMMIT\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_COMMIT\" ]]",
        "[[ \"$CANDIDATE_RUN_ID\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID\" ]]",
        "[[ \"$CANDIDATE_ATTEMPT\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ATTEMPT\" ]]",
    ] {
        if !source.contains(required) {
            bad.push("INTEGRITY");
        }
    }
    let validated = source.find("validate_candidate_manifest_v2\n");
    let render = source.find("\"$VERIFIED_CLI\" standalone gke render");
    let backup = source.find("\"$VERIFIED_CLI\" standalone backup");
    let restore = source.find("\"$VERIFIED_CLI\" standalone restore");
    if validated.is_none()
        || render.is_none()
        || backup.is_none()
        || restore.is_none()
        || !(validated < render && render < backup && backup < restore)
        || source.contains("\"$LUMEN_STANDALONE_GKE_CLI\" standalone")
    {
        bad.push("INTEGRITY");
    }
    bad
}

fn collect_files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(base: &Path, current: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        let mut entries = fs::read_dir(current)
            .expect("read fixture directory")
            .collect::<Result<Vec<_>, _>>()
            .expect("read fixture entries");
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let relative = path.strip_prefix(base).expect("fixture relative path");
            if entry.file_type().expect("fixture entry type").is_dir() {
                walk(base, &path, files);
            } else {
                files.insert(relative.to_owned(), fs::read(path).expect("read fixture file"));
            }
        }
    }

    let mut files = BTreeMap::new();
    walk(root, root, &mut files);
    files
}

struct GkeOracleFixture {
    _root: tempfile::TempDir,
    physical_root: PathBuf,
    kubeconfig: PathBuf,
    evidence: PathBuf,
    candidate: PathBuf,
    cli: PathBuf,
    fake_bin: PathBuf,
    cwd: PathBuf,
    kubectl_marker: PathBuf,
    kubectl_calls: PathBuf,
    gcloud_marker: PathBuf,
}

fn executable(path: &Path) {
    let mut permissions = fs::metadata(path).expect("executable metadata").permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(path, permissions).expect("make executable");
}

fn install_fake(path: &Path, body: &str) {
    fs::write(path, body).expect("write fake executable");
    executable(path);
}

fn make_gke_oracle_fixture() -> GkeOracleFixture {
    let physical_root = fs::canonicalize("/tmp").expect("canonical physical /tmp");
    assert!(matches!(physical_root.to_str(), Some("/tmp" | "/private/tmp")));
    let root = tempfile::Builder::new()
        .prefix("lumen-gke-physical-oracle-")
        .tempdir_in(&physical_root)
        .expect("physical temporary fixture");
    let kubeconfig = root.path().join("kubeconfig");
    let evidence = root.path().join("evidence");
    let candidate = root.path().join("candidate");
    let fake_bin = root.path().join("fake-bin");
    let cwd = root.path().join("cwd");
    fs::create_dir(&evidence).expect("evidence directory");
    fs::create_dir(&fake_bin).expect("fake bin directory");
    fs::create_dir(&cwd).expect("isolated current directory");
    fs::write(&kubeconfig, b"not parsed by the fake kubectl\n").expect("kubeconfig");
    let kubectl_marker = root.path().join("kubectl.marker");
    let kubectl_calls = root.path().join("kubectl.calls");
    let gcloud_marker = root.path().join("gcloud.marker");
    install_fake(
        &fake_bin.join("kubectl"),
        "#!/bin/sh\nset -eu\ncache=\ncount=0\nprevious=\nfor arg in \"$@\"; do\n  if [ \"$previous\" = --cache-dir ]; then cache=$arg; count=$((count + 1)); fi\n  previous=$arg\ndone\n[ \"$count\" -eq 1 ] || exit 18\ncache=$(cd \"$cache\" && pwd -P) || exit 19\nprintf '%s\\n' \"$cache\" >>\"$FAKE_KUBECTL_CALLS\"\nmkdir -p \"$cache/discovery\" \"$cache/http\"\n: >\"$cache/discovery/response\"\n: >\"$cache/http/response\"\nif [ \"$(wc -l <\"$FAKE_KUBECTL_CALLS\" | tr -d ' ')\" -eq 1 ]; then\n  { printf '%s\\n' \"${cache%/kubectl-cache}\"; printf '%s\\n' --tmpdir; find \"$TMPDIR\" -maxdepth 1 -name 'lumen-standalone-gke.*' -print; } >\"$FAKE_KUBECTL_MARKER\"\nfi\nprintf '%s\\n' oracle-context\n",
    );
    install_fake(
        &fake_bin.join("gcloud"),
        "#!/bin/sh\nif [ \"${HOME+x}\" = x ]; then printf '%s\\n' home-set >\"$FAKE_GCLOUD_MARKER\"; else printf '%s\\n' home-unset >\"$FAKE_GCLOUD_MARKER\"; fi\nexit 17\n",
    );
    fs::create_dir(&candidate).expect("empty candidate receipt directory");
    let fake_cli = fake_bin.join("lumen");
    install_fake(&fake_cli, "#!/bin/sh\nexit 17\n");
    GkeOracleFixture {
        _root: root,
        physical_root,
        kubeconfig,
        evidence,
        candidate,
        cli: fake_cli,
        fake_bin,
        cwd,
        kubectl_marker,
        kubectl_calls,
        gcloud_marker,
    }
}

fn run_gke_oracle(fixture: &GkeOracleFixture, kubeconfig: &Path, evidence: &Path) -> Output {
    let ambient_path = std::env::var_os("PATH").expect("ambient PATH");
    let path = format!(
        "{}:{}",
        fixture.fake_bin.display(),
        PathBuf::from(ambient_path).display()
    );
    let unrelated_tmp = fixture._root.path().join("unrelated-tmp");
    fs::create_dir_all(&unrelated_tmp).expect("unrelated TMPDIR");
    let mut command = Command::new("bash");
    command
        .current_dir(&fixture.cwd)
        .env_clear()
        .env("PATH", path)
        .env("TMPDIR", &unrelated_tmp)
        .env("KUBECONFIG", kubeconfig)
        .env("LUMEN_STANDALONE_GKE_CONTEXT", "oracle-context")
        .env("LUMEN_STANDALONE_GKE_PROJECT_ID", "oracle-project")
        .env("LUMEN_STANDALONE_GKE_LOCATION", "oracle-location")
        .env("LUMEN_STANDALONE_GKE_CLUSTER", "oracle-cluster")
        .env("LUMEN_STANDALONE_GKE_CLI", &fixture.cli)
        .env(
            "LUMEN_STANDALONE_GKE_IMAGE",
            format!("ghcr.io/chrischeng-c4/lumen@sha256:{}", "0".repeat(64)),
        )
        .env(
            "LUMEN_STANDALONE_GKE_CLIENT_IMAGE",
            "docker.io/curlimages/curl@sha256:7c12af72ceb38b7432ab85e1a265cff6ae58e06f95539d539b654f2cfa64bb13",
        )
        .env("LUMEN_STANDALONE_GKE_CLI_TARGET", "aarch64-apple-darwin")
        .env("LUMEN_STANDALONE_GKE_STORAGE_CLASS", "premium-rwo")
        .env("LUMEN_STANDALONE_GKE_NODE_POOL", "oracle-pool")
        .env("LUMEN_STANDALONE_GKE_RUN_ID", "424242")
        .env(
            "LUMEN_STANDALONE_GKE_EXPECTED_COMMIT",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .env("LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID", "424242")
        .env("LUMEN_STANDALONE_GKE_EXPECTED_RUN_ATTEMPT", "1")
        .env("LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256", "0".repeat(64))
        .env("LUMEN_STANDALONE_GKE_EVIDENCE_DIR", evidence)
        .env(
            "LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR",
            &fixture.candidate,
        )
        .env("LUMEN_STANDALONE_GKE_MUTATION", "1")
        .env("FAKE_KUBECTL_MARKER", &fixture.kubectl_marker)
        .env("FAKE_KUBECTL_CALLS", &fixture.kubectl_calls)
        .env("FAKE_GCLOUD_MARKER", &fixture.gcloud_marker)
        .env("FAKE_PHYSICAL_ROOT", &fixture.physical_root)
        .arg(root().join("apps/lumen/scripts/standalone-gke-acceptance.sh"))
        .args(["--mode", "gke"]);
    command.output().expect("run GKE acceptance oracle")
}

fn replace_once(source: &str, from: &str, to: &str) -> String {
    assert_eq!(
        source.matches(from).count(),
        1,
        "mutation anchor is unique: {from}"
    );
    let changed = source.replacen(from, to, 1);
    assert_ne!(source, changed, "mutation changes bytes");
    changed
}

fn replace_first(source: &str, from: &str, to: &str) -> String {
    assert!(source.contains(from), "mutation anchor is present: {from}");
    let changed = source.replacen(from, to, 1);
    assert_ne!(source, changed, "mutation changes bytes");
    changed
}

#[test]
fn live_gate_has_the_complete_candidate_bound_contract() {
    let full = full_script();
    assert!(
        preflight_findings(&full).is_empty(),
        "preflight findings: {:?}",
        preflight_findings(&full)
    );
    assert!(
        integrity_findings(&full).is_empty(),
        "integrity findings: {:?}",
        integrity_findings(&full)
    );
    for required in [
        "LUMEN_STANDALONE_GKE_CONTEXT",
        ".networkConfig.datapathProvider == \"ADVANCED_DATAPATH\"",
        "final-candidate-manifest.json",
    ] {
        assert!(
            full.contains(required),
            "missing controller preflight: {required}"
        );
    }
    let live = live_slice();
    assert!(
        findings(&live).is_empty(),
        "findings: {:?}",
        findings(&live)
    );
}

#[test]
fn required_runtime_accepts_only_serializer_metadata_and_auth_change() {
    let source = full_script();
    let accepted = run_required_runtime_oracle(&source, None);
    assert!(
        accepted.output.status.success(),
        "serializer-only required transition was rejected: {}",
        String::from_utf8_lossy(&accepted.output.stderr)
    );
    let statefulset: Value = serde_json::from_str(
        accepted
            .statefulset
            .as_deref()
            .expect("successful required transition writes StatefulSet"),
    )
    .expect("required StatefulSet JSON");
    assert_eq!(
        statefulset["spec"]["template"]["spec"]["containers"][0]["env"][0]["value"],
        "required",
        "required transition must retain the exact auth profile"
    );
    assert!(accepted.diagnostic.is_none(), "accepted transition wrote a diagnostic");
    assert!(accepted.receipt.is_none(), "required-runtime oracle wrote a receipt");
    assert!(
        accepted.output.stdout.is_empty(),
        "accepted transition wrote unexpected stdout: {:?}",
        String::from_utf8_lossy(&accepted.output.stdout)
    );

    for (mutation, expected_path) in [
        ("image", "/spec/template/spec/containers/0/image"),
        ("cpu", "/spec/template/spec/containers/0/resources/requests/cpu"),
        ("memory", "/spec/template/spec/containers/0/resources/requests/memory"),
        ("other-env", "/spec/template/spec/containers/0/env/0/value"),
    ] {
        let rejected = run_required_runtime_oracle(&source, Some(mutation));
        assert_eq!(
            rejected.output.status.code(),
            Some(2),
            "business-field mutation passed: {mutation}; stderr={} ",
            String::from_utf8_lossy(&rejected.output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&rejected.output.stderr)
                .contains("required continuity patch changed live desired fields other than LUMEN_AUTH"),
            "business-field mutation did not fail at continuity comparator: {mutation}"
        );
        assert!(
            rejected.statefulset.is_none(),
            "business-field mutation wrote the required StatefulSet: {mutation}"
        );
        assert!(rejected.receipt.is_none(), "business-field mutation wrote a receipt: {mutation}");
        assert!(
            rejected.output.stdout.is_empty(),
            "business-field mutation wrote stdout: {mutation}; stdout={:?}",
            String::from_utf8_lossy(&rejected.output.stdout)
        );
        let diagnostic: Value = serde_json::from_str(
            rejected
                .diagnostic
                .as_deref()
                .expect("business-field mutation writes private path-only diagnostic"),
        )
        .expect("private required-continuity diagnostic JSON");
        assert_eq!(
            diagnostic,
            json!({
                "schema": "lumen.standalone-gke-required-continuity-diff/v1",
                "paths": [expected_path],
            }),
            "business-field mutation wrote an unexpected diagnostic: {mutation}"
        );
        assert!(
            !rejected
                .diagnostic
                .as_deref()
                .expect("diagnostic text")
                .contains("changed"),
            "private diagnostic retained a changed fixture value: {mutation}"
        );
    }

    let raced = run_required_runtime_oracle(&source, Some("race"));
    assert_eq!(
        raced.output.status.code(),
        Some(2),
        "injected target race did not fail closed: {}",
        String::from_utf8_lossy(&raced.output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&raced.output.stderr),
        "required continuity diagnostic could not be written\n",
        "injected target race must fail only with the generic diagnostic error"
    );
    assert!(raced.output.stdout.is_empty(), "injected target race wrote stdout");
    assert_eq!(
        raced.diagnostic.as_deref(),
        Some("injected-target-bytes\n"),
        "injected target bytes must not be overwritten or altered"
    );
    assert!(raced.statefulset.is_none(), "injected target race wrote required StatefulSet");
    assert!(raced.receipt.is_none(), "injected target race wrote a public receipt");
}

#[test]
fn required_continuity_diff_reports_only_sorted_rfc6901_paths() {
    let source = full_script();
    let before = json!({
        "array": ["same"],
        "canary": "do-not-retain-this-private-value",
        "removed": {"member": "private"},
        "scalar": "before-private-value",
        "slash/key": "before",
        "structure": {"kept": true},
        "tilde~key": "before",
        "type": "string",
    });
    let after = json!({
        "array": ["same", "private-added"],
        "canary": "do-not-retain-this-private-value",
        "missing": null,
        "removed": {},
        "scalar": "after-private-value",
        "slash/key": "after",
        "structure": {"added": "private", "kept": true},
        "tilde~key": "after",
        "type": {"nested": "private"},
    });
    let (output, diagnostic, files, mode) =
        run_required_continuity_diff_oracle(&source, &before, &after, None);
    assert!(
        output.status.success(),
        "path-only diagnostic writer rejected a composite difference: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty(), "diagnostic writer emitted stdout");
    let diagnostic_text = diagnostic.expect("private path-only diagnostic");
    let diagnostic: Value = serde_json::from_str(&diagnostic_text).expect("diagnostic JSON");
    assert_eq!(
        diagnostic,
        json!({
            "schema": "lumen.standalone-gke-required-continuity-diff/v1",
            "paths": [
                "/array/1",
                "/missing",
                "/removed/member",
                "/scalar",
                "/slash~1key",
                "/structure/added",
                "/tilde~0key",
                "/type",
            ],
        })
    );
    assert_eq!(
        diagnostic.as_object().expect("diagnostic object").len(),
        2,
        "diagnostic object has only schema and paths"
    );
    for private_value in [
        "do-not-retain-this-private-value",
        "before-private-value",
        "after-private-value",
        "private-added",
    ] {
        assert!(
            !diagnostic_text.contains(private_value),
            "diagnostic retained a private value: {private_value}"
        );
    }
    assert_eq!(files.len(), 1, "diagnostic writer retained only one private artifact");
    assert!(
        files.contains_key(&PathBuf::from("lumen-standalone-gke-required-continuity-diff.json")),
        "diagnostic writer used the exact private file name"
    );
    assert_eq!(mode, 0o600, "diagnostic mode must be 0600");
}

#[test]
fn required_continuity_diff_fails_closed_without_replacing_existing_private_artifact() {
    let source = full_script();
    let before = json!({"private": "before"});
    let after = json!({"private": "after"});
    let (output, diagnostic, files, mode) = run_required_continuity_diff_oracle(
        &source,
        &before,
        &after,
        Some("pre-existing-private-artifact\n"),
    );
    assert!(!output.status.success(), "existing diagnostic path must fail closed");
    assert!(output.stdout.is_empty(), "failed diagnostic writer emitted stdout");
    assert!(output.stderr.is_empty(), "failed diagnostic writer emitted private diagnostics");
    assert_eq!(
        diagnostic.as_deref(),
        Some("pre-existing-private-artifact\n"),
        "failed writer must not replace the pre-existing private artifact"
    );
    assert_eq!(files.len(), 1, "failed writer must not leave a temporary artifact");
    assert_eq!(mode, 0o644, "failed writer must not change the existing artifact mode");
}

#[test]
fn required_runtime_continuity_mutations_fail_the_static_contract() {
    let source = live_slice();
    for (from, to) in [
        (
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata.creationTimestamp,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
            "jq -S '(.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
        ),
        (
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata.creationTimestamp,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
            "jq -S 'del(.spec.template.metadata.creationTimestamp,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
        ),
        (
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata.creationTimestamp,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-before-required.json\" >\"$TMP_ROOT/v2-before-required-noauth.json\"",
            "jq -S 'del(.metadata.creationTimestamp,.status,.spec.template.metadata.creationTimestamp) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-before-required.json\" >\"$TMP_ROOT/v2-before-required-noauth.json\"",
        ),
        (
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata.creationTimestamp,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
            "jq -S 'del(.metadata.creationTimestamp,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
        ),
        (
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata.creationTimestamp,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata.creationTimestamp) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
        ),
        (
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata.creationTimestamp,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
            "jq -S 'del(.metadata,.spec.template.metadata.creationTimestamp,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
        ),
        (
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata.creationTimestamp,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata.creationTimestamp,.status?) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
        ),
        (
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata.creationTimestamp,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
        ),
        (
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata.creationTimestamp,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata.creationTimestamp,.status,.spec.template.spec.serviceAccountName) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
        ),
        (
            "jq -S 'del(.metadata.creationTimestamp,.spec.template.metadata.creationTimestamp,.status) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
            "jq -S 'del(.metadata,.spec) | (.spec.template.spec.containers[0].env |= map(select(.name != \"LUMEN_AUTH\")))' \"$TMP_ROOT/v2-after-required.json\" >\"$TMP_ROOT/v2-after-required-noauth.json\"",
        ),
        (
            "if ! cmp -s \"$TMP_ROOT/v2-before-required-noauth.json\" \"$TMP_ROOT/v2-after-required-noauth.json\"; then\n    v2_write_required_continuity_diff \"$TMP_ROOT/v2-before-required-noauth.json\" \"$TMP_ROOT/v2-after-required-noauth.json\" || die \"required continuity diagnostic could not be written\"\n    die \"required continuity patch changed live desired fields other than LUMEN_AUTH\"\n  fi",
            "cmp -s \"$TMP_ROOT/v2-before-required-noauth.json\" \"$TMP_ROOT/v2-after-required-noauth.json\" || true",
        ),
        (
            "v2_write_required_continuity_diff \"$TMP_ROOT/v2-before-required-noauth.json\" \"$TMP_ROOT/v2-after-required-noauth.json\" || die \"required continuity diagnostic could not be written\"",
            "true # required-continuity diagnostic disabled",
        ),
        (
            "v2_write_required_continuity_diff \"$TMP_ROOT/v2-before-required-noauth.json\" \"$TMP_ROOT/v2-after-required-noauth.json\" || die \"required continuity diagnostic could not be written\"\n    die \"required continuity patch changed live desired fields other than LUMEN_AUTH\"",
            "die \"required continuity patch changed live desired fields other than LUMEN_AUTH\"\n    v2_write_required_continuity_diff \"$TMP_ROOT/v2-before-required-noauth.json\" \"$TMP_ROOT/v2-after-required-noauth.json\" || die \"required continuity diagnostic could not be written\"",
        ),
        (
            "lumen-standalone-gke-required-continuity-diff.json",
            "lumen-standalone-gke-receipt.json",
        ),
        (
            "diagnostic=\"$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/lumen-standalone-gke-required-continuity-diff.json\"",
            "diagnostic=\"$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR/lumen-standalone-gke-required-continuity-diff.json\"",
        ),
        (
            "--slurpfile before \"$before\" --slurpfile after \"$after\"",
            "--arg before \"$before\" --arg after \"$after\"",
        ),
        (
            "(differences($before[0];$after[0];[])|unique) as $paths | {schema:\"lumen.standalone-gke-required-continuity-diff/v1\",paths:$paths}",
            "(differences($before[0];$after[0];[])|unique) as $paths | {schema:\"lumen.standalone-gke-required-continuity-diff/v1\",paths:$paths,before:$before}",
        ),
        (
            "gsub(\"~\";\"~0\")|gsub(\"/\";\"~1\")",
            "tostring",
        ),
        (
            "if ($left|has($key)) and ($right|has($key)) then differences($left[$key];$right[$key];$path+[$key])",
            "if ($left[$key] != null) and ($right[$key] != null) then differences($left[$key];$right[$key];$path+[$key])",
        ),
        (
            "differences($left[$key];$right[$key];$path+[$key])",
            "[pointer($path+[$key])]",
        ),
        (
            "if $index < ($left|length) and $index < ($right|length) then differences($left[$index];$right[$index];$path+[$index])",
            "[pointer($path+[$index])]",
        ),
        (
            "(.paths == (.paths|sort|unique))",
            "true # path uniqueness unchecked",
        ),
        (
            "if ! ln -- \"$temporary\" \"$diagnostic\" 2>/dev/null; then\n    rm -f -- \"$temporary\"\n    return 1\n  fi",
            "true # diagnostic commit bypassed",
        ),
        (
            "ln -- \"$temporary\" \"$diagnostic\" 2>/dev/null",
            "mv -f -- \"$temporary\" \"$diagnostic\"",
        ),
        (
            "ln -- \"$temporary\" \"$diagnostic\" 2>/dev/null",
            "cp -f -- \"$temporary\" \"$diagnostic\"",
        ),
        (
            "ln -- \"$temporary\" \"$diagnostic\" 2>/dev/null",
            "ln -f -- \"$temporary\" \"$diagnostic\"",
        ),
        (
            "if ! ln -- \"$temporary\" \"$diagnostic\" 2>/dev/null; then\n    rm -f -- \"$temporary\"\n    return 1\n  fi",
            "ln -- \"$temporary\" \"$diagnostic\" || true",
        ),
        (
            ".spec.template.spec.containers[0].image == $image and",
            "true and",
        ),
        (
            "([.spec.template.spec.containers[0].env[]|select(.name == \"LUMEN_AUTH\")|.value] == [\"required\"])",
            "([.spec.template.spec.containers[0].env[]|select(.name == \"LUMEN_AUTH\")|.value] == [\"optional\"])",
        ),
        (
            "([.spec.template.spec.containers[0].resources.requests.cpu] == [\"1\"])",
            "([.spec.template.spec.containers[0].resources.requests.cpu] == [\"2\"])",
        ),
        (
            "([.spec.template.spec.containers[0].resources.requests.memory] == [\"1Gi\"])",
            "([.spec.template.spec.containers[0].resources.requests.memory] == [\"2Gi\"])",
        ),
    ] {
        let changed = replace_once(&source, from, to);
        assert!(
            findings(&changed).contains(&"REQUIRED"),
            "required continuity mutation did not trigger REQUIRED; findings={:?}",
            findings(&changed)
        );
    }
}

#[test]
fn job_log_reader_retries_only_konnectivity_transients() {
    let source = full_script();
    for (mode, code, calls, sleeps, log, needle) in [
        ("transient", 0, 3, 2, "row=job status=passed\n", ""),
        (
            "transient6",
            2,
            6,
            5,
            "",
            "Konnectivity agent unavailable while reading job log",
        ),
        ("permanent", 2, 1, 0, "", "job log read failed"),
        (
            "transient5permanent",
            2,
            6,
            5,
            "",
            "job log read failed",
        ),
    ] {
        let result = run_job_log_reader(&source, mode);
        let stderr = String::from_utf8_lossy(&result.output.stderr);
        assert_eq!(result.output.status.code(), Some(code), "mode={mode}: {stderr}");
        assert_eq!(result.calls, calls, "mode={mode}: wrong call count");
        assert_eq!(result.sleeps, sleeps, "mode={mode}: wrong sleep count");
        assert_eq!(result.log, log, "mode={mode}: wrong log bytes");
        if needle.is_empty() {
            assert!(stderr.is_empty(), "mode={mode}: {stderr}");
        } else {
            assert!(stderr.contains(needle), "mode={mode}: {stderr}");
        }
    }
}

#[test]
fn api_job_status_contract_accepts_only_a_single_concrete_2xx_log_line() {
    let source = full_script();
    for status in ["200", "201", "299"] {
        let result = run_api_status_oracle(&source, "create", "2xx", &format!("row=create status={status}\n"));
        assert!(
            result.status.success(),
            "concrete 2xx status {status} was rejected: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    for (name, log) in [
        ("literal", "row=create status=2xx\n"),
        ("1xx", "row=create status=199\n"),
        ("3xx", "row=create status=300\n"),
        ("4xx", "row=create status=403\n"),
        ("5xx", "row=create status=500\n"),
        ("short", "row=create status=20\n"),
        ("long", "row=create status=2000\n"),
        ("prefix", "note row=create status=200\n"),
        ("suffix", "row=create status=200 note\n"),
        ("wrong-label", "row=other status=200\n"),
        ("extra-line", "row=create status=200\nrow=create status=201\n"),
    ] {
        let result = run_api_status_oracle(&source, "create", "2xx", log);
        assert!(
            !result.status.success(),
            "invalid 2xx log passed: {name}; stderr: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let exact = run_api_status_oracle(&source, "unlisted", "403", "row=unlisted status=403\n");
    assert!(exact.status.success(), "exact status was rejected");
    for log in ["row=unlisted status=2xx\n", "row=unlisted status=401\n", "row=unlisted status=403\nextra\n"] {
        assert!(
            !run_api_status_oracle(&source, "unlisted", "403", log)
                .status
                .success(),
            "exact-status contract accepted invalid log: {log:?}"
        );
    }

    let literal_mutation = replace_once(&source, "status=2[0-9][0-9]", "status=2xx");
    assert!(
        !run_api_status_oracle(&literal_mutation, "create", "2xx", "row=create status=200\n")
            .status
            .success(),
        "literal-status mutation still accepted a concrete 2xx response"
    );
    assert!(
        run_api_status_oracle(&literal_mutation, "create", "2xx", "row=create status=2xx\n")
            .status
            .success(),
        "literal-status mutation fixture did not expose the bypass"
    );
}

#[test]
fn metric_delta_failures_retain_only_redacted_shape_evidence() {
    let source = full_script();
    let missing_before = concat!(
        "delegated_auth_access_reviews_total 0\n",
        "delegated_auth_allowed_total 0\n",
        "delegated_auth_denied_total 0\n",
    );
    let complete_after = concat!(
        "delegated_auth_token_reviews_total 1\n",
        "delegated_auth_access_reviews_total 1\n",
        "delegated_auth_allowed_total 1\n",
        "delegated_auth_denied_total 1\n",
    );
    let missing = run_metric_delta_oracle(&source, "in-cluster", missing_before, complete_after);
    assert_eq!(missing.output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&missing.output.stderr)
            .contains("missing delegated_auth_token_reviews_total"),
        "missing-metric failure did not retain and report its cause: stderr={}; evidence={:?}",
        String::from_utf8_lossy(&missing.output.stderr),
        missing.evidence
    );
    assert_eq!(
        missing.evidence.len(),
        1,
        "a metric failure must retain exactly one final artifact"
    );
    let missing_body = missing
        .evidence
        .get("lumen-standalone-gke-metric-shape-failure.json")
        .expect("redacted missing-metric artifact");
    assert!(
        !missing_body.contains("delegated_auth_token_reviews_total 1"),
        "failure artifact must not retain raw metric rows: {missing_body}"
    );
    let missing_json: Value = serde_json::from_str(missing_body).expect("missing-metric JSON");
    assert_eq!(missing_json["schema"], "lumen.standalone-gke-metric-shape/v1");
    assert_eq!(missing_json["failure"]["profile"], "in-cluster");
    assert_eq!(missing_json["failure"]["reason"], "missing_metric");
    assert_eq!(
        missing_json["observations"][0]["metrics"]["delegated_auth_token_reviews_total"]["shape"],
        "absent"
    );
    assert_eq!(
        missing_json["observations"][1]["metrics"]["delegated_auth_token_reviews_total"]["value_class"],
        "positive"
    );
    assert_eq!(missing_json["redaction"]["metric_values_retained"], false);
    assert_eq!(missing_json["redaction"]["metric_label_values_retained"], false);

    let labeled_zero = concat!(
        "delegated_auth_token_reviews_total{private_label=\"secret-value\"} 0\n",
        "delegated_auth_access_reviews_total{private_label=\"secret-value\"} 0\n",
        "delegated_auth_allowed_total{private_label=\"secret-value\"} 0\n",
        "delegated_auth_denied_total{private_label=\"secret-value\"} 0\n",
    );
    let non_positive = run_metric_delta_oracle(&source, "required", labeled_zero, labeled_zero);
    assert_eq!(non_positive.output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&non_positive.output.stderr)
            .contains("measured auth metric deltas are not positive")
    );
    assert_eq!(non_positive.evidence.len(), 1);
    let non_positive_body = non_positive
        .evidence
        .get("lumen-standalone-gke-metric-shape-failure.json")
        .expect("redacted non-positive artifact");
    assert!(
        !non_positive_body.contains("secret-value"),
        "failure artifact must not retain metric label values: {non_positive_body}"
    );
    let non_positive_json: Value =
        serde_json::from_str(non_positive_body).expect("non-positive metric JSON");
    assert_eq!(non_positive_json["failure"]["profile"], "required");
    assert_eq!(non_positive_json["failure"]["reason"], "non_positive_delta");
    for phase in [0, 1] {
        assert_eq!(
            non_positive_json["observations"][phase]["metrics"]["delegated_auth_denied_total"]["shape"],
            "labeled"
        );
        assert_eq!(
            non_positive_json["observations"][phase]["metrics"]["delegated_auth_denied_total"]["value_class"],
            "zero"
        );
    }
}

#[test]
fn metric_shape_and_restart_window_mutations_fail_the_static_contract() {
    let source = live_slice();
    for (from, to, expected) in [
        (
            "v2_write_metric_failure \"$profile\" \"$reason\" \"$before\" \"$after\" ||",
            "true # redacted failure evidence disabled",
            "METRICS",
        ),
        (
            "safe_private_file \"$report\"",
            "cp \"$before\" \"$report\"",
            "REDACTION",
        ),
        (
            "before_token=\"$(v2_metric_shape delegated_auth_token_reviews_total \"$before\")\" || return 1",
            "before_token=\"$(v2_metric_shape delegated_auth_token_reviews \"$before\")\" || return 1",
            "METRICS",
        ),
        (
            "v2_metric_deltas in-cluster \"$TMP_ROOT/metrics-before-incluster.metrics\" \"$TMP_ROOT/metrics-after-incluster.metrics\"",
            "v2_metric_deltas required \"$TMP_ROOT/metrics-before-incluster.metrics\" \"$TMP_ROOT/metrics-after-incluster.metrics\"",
            "METRICS",
        ),
        (
            "v2_run_api_job metrics-denied-after-resize unlisted default POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 403 none none",
            "# post-resize denial removed",
            "METRICS",
        ),
        (
            "v2_run_metrics_job metrics-before-incluster\n  v2_run_api_job marker-after-resize",
            "v2_run_api_job marker-after-resize\n  v2_run_metrics_job metrics-before-incluster",
            "METRICS",
        ),
        (
            "v2_wait_pod \"$resized_uid\" required 1 1Gi\n  required_uid=\"$V2_LAST_POD_UID\"\n  [[ \"$required_uid\" != \"$resized_uid\" ]] || die \"required profile did not replace pod\"\n  v2_run_metrics_job metrics-before-required",
            "v2_run_metrics_job metrics-before-required\n  v2_wait_pod \"$resized_uid\" required 1 1Gi\n  required_uid=\"$V2_LAST_POD_UID\"\n  [[ \"$required_uid\" != \"$resized_uid\" ]] || die \"required profile did not replace pod\"",
            "METRICS",
        ),
        (
            "v2_run_metrics_job metrics-before-required\n  v2_run_api_job required-projected-app app projected POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 2xx durable-first none\n  v2_run_api_job required-default-app app default POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 401 none none",
            "v2_run_api_job required-default-app app default POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 401 none none\n  v2_run_metrics_job metrics-before-required\n  v2_run_api_job required-projected-app app projected POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 2xx durable-first none",
            "METRICS",
        ),
    ] {
        let changed = replace_once(&source, from, to);
        assert!(
            findings(&changed).contains(&expected),
            "metric mutation {from:?} did not trigger {expected}; findings={:?}",
            findings(&changed)
        );
    }
    let changed = source.replace("metric_values_retained:false", "metric_values_retained:true");
    assert_ne!(source, changed, "redaction mutation changes bytes");
    assert!(
        findings(&changed).contains(&"REDACTION"),
        "metric-value retention mutation did not fail the redaction contract: {:?}",
        findings(&changed)
    );
}

#[test]
fn runtime_image_identity_binds_root_or_scheduled_child_inside_pod_wait_predicate() {
    let source = live_slice();
    let root = "ghcr.io/chrischeng-c4/lumen@sha256:root";
    let amd64_child = "ghcr.io/chrischeng-c4/lumen@sha256:amd64-child";
    let arm64_child = "ghcr.io/chrischeng-c4/lumen@sha256:arm64-child";
    let config = "ghcr.io/other/config@sha256:config";

    for (arch, child, other_child) in [
        ("amd64", amd64_child, arm64_child),
        ("arm64", arm64_child, amd64_child),
    ] {
        for image_id in [root, child] {
            let result = run_service_link_wait(
                &source,
                false,
                "True",
                image_id,
                root,
                config,
                arch,
                child.rsplit_once('@').unwrap().1,
            );
            assert!(
                result.status.success(),
                "{arch} accepted runtime image was rejected: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }

        let cross_arch = run_service_link_wait(
            &source,
            false,
            "True",
            other_child,
            root,
            config,
            arch,
            child.rsplit_once('@').unwrap().1,
        );
        assert!(
            !cross_arch.status.success(),
            "{arch} accepted the other architecture child"
        );
    }

    let enabled = run_service_link_wait(
        &source,
        true,
        "True",
        root,
        root,
        config,
        "amd64",
        "sha256:amd64-child",
    );
    assert!(
        !enabled.status.success(),
        "enabled service links bypassed the executable pod assertion"
    );

    let prose_bypass = replace_once(
        &source,
        "          .spec.enableServiceLinks == false and",
        "          true and",
    );
    let prose_bypass = replace_once(
        &prose_bypass,
        "v2_wait_pod() {\n",
        "v2_wait_pod() {\n  : <<'SERVICE_LINK_PROSE'\n          .spec.enableServiceLinks == false and\nSERVICE_LINK_PROSE\n",
    );
    assert!(
        !findings(&prose_bypass).contains(&"REQUIRED"),
        "prose fixture must prove why the executable oracle is required"
    );
    assert!(
        run_service_link_wait(
            &prose_bypass,
            true,
            "True",
            root,
            root,
            config,
            "amd64",
            "sha256:amd64-child",
        )
        .status
        .success(),
        "prose fixture did not remove the executable service-link assertion"
    );

    let not_ready = run_service_link_wait(
        &source,
        false,
        "False",
        root,
        root,
        config,
        "amd64",
        "sha256:amd64-child",
    );
    assert!(
        !not_ready.status.success(),
        "non-Ready pod bypassed the executable pod wait predicate"
    );

    for (name, image_id) in [
        ("wrong repo", config),
        ("wrong repo with root digest", "ghcr.io/chrischeng-c4/other@sha256:root"),
        (
            "wrong repo with scheduled child digest",
            "ghcr.io/chrischeng-c4/other@sha256:amd64-child",
        ),
        ("tag", "ghcr.io/chrischeng-c4/lumen:0.4.29"),
        (
            "tag plus root digest",
            "ghcr.io/chrischeng-c4/lumen:0.4.29@sha256:root",
        ),
        (
            "tag plus scheduled child digest",
            "ghcr.io/chrischeng-c4/lumen:0.4.29@sha256:amd64-child",
        ),
        ("malformed", "sha256:amd64-child"),
        ("substring", "ghcr.io/chrischeng-c4/lumen@sha256:root-extra"),
        ("prefix", "prefixghcr.io/chrischeng-c4/lumen@sha256:root"),
        ("suffix", "ghcr.io/chrischeng-c4/lumen@sha256:root-suffix"),
    ] {
        let result = run_service_link_wait(
            &source,
            false,
            "True",
            image_id,
            root,
            config,
            "amd64",
            "sha256:amd64-child",
        );
        assert!(!result.status.success(), "identity mutation passed: {name}");
    }

    for (name, pod_image) in [
        ("wrong repository", "ghcr.io/chrischeng-c4/other@sha256:root"),
        ("child instead of root", amd64_child),
    ] {
        let result = run_service_link_wait(
            &source,
            false,
            "True",
            root,
            pod_image,
            config,
            "amd64",
            "sha256:amd64-child",
        );
        assert!(
            !result.status.success(),
            "{name} PodSpec image bypassed the exact root assertion"
        );
    }

    let exact_runtime_identity = r#"        if [[ "$image" == "$LUMEN_STANDALONE_GKE_IMAGE" ]]; then
          V2_OBSERVED_RUNTIME_IMAGE_DIGEST="$ROOT_DIGEST"
        elif [[ "$image" == "ghcr.io/chrischeng-c4/lumen@$V2_CHILD_DIGEST" ]]; then
          V2_OBSERVED_RUNTIME_IMAGE_DIGEST="$V2_CHILD_DIGEST"
        else
          die "observed container imageID is not the exact candidate root or scheduled child digest"
        fi"#;
    let unchecked_identity = replace_once(
        &source,
        exact_runtime_identity,
        "        V2_OBSERVED_RUNTIME_IMAGE_DIGEST=\"$image\"",
    );
    assert!(findings(&unchecked_identity).contains(&"RUNTIME_IMAGE"));
    assert!(
        run_service_link_wait(
            &unchecked_identity,
            false,
            "True",
            "prefixghcr.io/chrischeng-c4/lumen@sha256:root",
            root,
            config,
            "amd64",
            "sha256:amd64-child",
        )
        .status
        .success(),
        "identity bypass fixture did not remove the runtime image gate"
    );

    let unbound_child = replace_once(
        &source,
        "elif [[ \"$image\" == \"ghcr.io/chrischeng-c4/lumen@$V2_CHILD_DIGEST\" ]]; then",
        "elif [[ \"$image\" == \"ghcr.io/chrischeng-c4/lumen@sha256:wrong-child\" ]]; then",
    );
    assert!(findings(&unbound_child).contains(&"RUNTIME_IMAGE"));
    assert!(
        !run_service_link_wait(
            &unbound_child,
            false,
            "True",
            amd64_child,
            root,
            config,
            "amd64",
            "sha256:amd64-child",
        )
        .status
        .success(),
        "runtime child identity was not bound to the scheduled child"
    );

    let read_status_image = replace_once(
        &source,
        ".status.containerStatuses[]|select(.name == \"serving\")|.imageID",
        ".status.containerStatuses[]|select(.name == \"serving\")|.image",
    );
    assert!(
        !run_service_link_wait(
            &read_status_image,
            false,
            "True",
            root,
            root,
            config,
            "amd64",
            "sha256:amd64-child",
        )
        .status
        .success(),
        "status.image was accepted as the runtime image identity"
    );

    let unchecked_pod_image = replace_once(
        &source,
        "          ([.spec.containers[] | select(.name == \"serving\") | .image] == [$image]) and",
        "          true and",
    );
    assert!(
        run_service_link_wait(
            &unchecked_pod_image,
            false,
            "True",
            root,
            amd64_child,
            config,
            "amd64",
            "sha256:amd64-child",
        )
        .status
        .success(),
        "PodSpec image bypass fixture did not remove the exact root assertion"
    );
}

#[test]
fn negative_mutations_remove_real_gate_obligations() {
    let source = live_slice();
    for (from, to, expected) in [
        (
            "cclab.lumen.candidate-manifest.v3",
            "candidate-v2",
            "CANDIDATE",
        ),
        (
            "CANDIDATE_TAG=\"lumen@$CANDIDATE_VERSION\"",
            "CANDIDATE_TAG=\"lumen@0.4.29\"",
            "CANDIDATE",
        ),
        (
            "CANDIDATE_DEFAULT_IMAGE=\"ghcr.io/chrischeng-c4/lumen:$CANDIDATE_VERSION\"",
            "CANDIDATE_DEFAULT_IMAGE=\"ghcr.io/chrischeng-c4/lumen:0.4.29\"",
            "CANDIDATE",
        ),
        (
            ".version == $version and .tag == $tag",
            ".tag == $tag",
            "CANDIDATE",
        ),
        ("tar -xOf", "tar -xzf", "ARCHIVE"),
        (
            "v2-public.json\" >/dev/null || die \"public runtime is not the candidate-version serving image\"",
            "v2-public.json\" >/dev/null || true",
            "PUBLIC_IMAGE",
        ),
        (
            "patch_statefulset_image \"$statefulset\" \"$label\"",
            "k set image -f \"$statefulset\" serving=\"$LUMEN_STANDALONE_GKE_IMAGE\" --local -o yaml >\"$statefulset\"",
            "PATCH",
        ),
        (
            "patch_statefulset_image \"$V2_APPLY_ROOT/runtime/statefulset.yaml\" v2",
            "k set image -f \"$V2_APPLY_ROOT/runtime/statefulset.yaml\" serving=\"$LUMEN_STANDALONE_GKE_IMAGE\" --local -o yaml >\"$V2_APPLY_ROOT/runtime/statefulset.yaml\"",
            "PATCH",
        ),
        (
            "(.spec.template.spec.containers | length == 1)\n    and .spec.template.spec.containers[0].name == \"serving\"\n    and .spec.template.spec.containers[0].image == $expected_image",
            "(.spec.template.spec.containers | length >= 1)\n    and .spec.template.spec.containers[0].name == \"serving\"\n    and .spec.template.spec.containers[0].image == $expected_image",
            "PATCH",
        ),
        (
            ".spec.template.spec.containers[0].image = $image",
            ".spec.template.spec.containers[].image = $image",
            "PATCH",
        ),
        (
            "cmp -s \"$original_canonical\" \"$patched_canonical\"",
            "true # field comparison bypassed",
            "PATCH",
        ),
        (
            "cmp -s \"$TMP_ROOT/v2-public-no-image.json\" \"$TMP_ROOT/v2-private-no-image.json\"",
            "true # v2 field comparison bypassed",
            "PATCH",
        ),
        (
            "candidate archive hash mismatch",
            "archive hash optional",
            "ARCHIVE",
        ),
        (
            "candidate controller CLI bytes differ from local CLI",
            "controller CLI hash optional",
            "ARCHIVE",
        ),
        (
            "k create -f \"$V2_APPLY_ROOT/storage/namespace.yaml\" -o json",
            "k apply -f \"$V2_APPLY_ROOT/storage/namespace.yaml\"",
            "MUTATION",
        ),
        (
            "k create -f \"$V2_APPLY_ROOT/runtime/clusterrolebinding.yaml\" -o json",
            "k apply -f \"$V2_APPLY_ROOT/runtime/clusterrolebinding.yaml\"",
            "MUTATION",
        ),
        (
            "k create -f \"$TMP_ROOT/v2-client/namespace.json\" -o json",
            "k apply -f \"$TMP_ROOT/v2-client/namespace.json\"",
            "MUTATION",
        ),
        (
            "V2_RUNTIME_ARMED=true\n  k create -f \"$V2_APPLY_ROOT/storage/namespace.yaml\" -o json",
            "k create -f \"$V2_APPLY_ROOT/storage/namespace.yaml\" -o json\n  V2_RUNTIME_ARMED=true",
            "MUTATION",
        ),
        (
            ".metadata.uid == $uid and\n    .metadata.name == $name and",
            ".metadata.name == $name and",
            "MUTATION",
        ),
        (
            "if k get \"$kind\" \"$name\" --ignore-not-found -o json >\"$response\" 2>\"$error\"; then status=0; else status=$?; fi",
            "if k get \"$kind\" \"$name\" -o json >\"$response\" 2>\"$error\"; then status=0; else status=$?; fi",
            "CLEANUP",
        ),
        (
            "[[ \"$status\" -eq 0 ]] || return 2",
            "[[ \"$status\" -eq 0 ]] || return 1",
            "CLEANUP",
        ),
        (
            "if [[ ! -s \"$response\" ]]; then\n    return 1",
            "if [[ ! -s \"$response\" ]]; then\n    return 0",
            "CLEANUP",
        ),
        (
            "[[ \"$state\" -eq 1 ]] && return 0",
            "[[ \"$state\" -ne 0 ]] && return 0",
            "CLEANUP",
        ),
        (
            ".metadata.labels[\"app.kubernetes.io/managed-by\"] == \"lumen-standalone-gke-acceptance\" and\n    .metadata.labels[\"lumen.axiom.dev/owner-namespace\"] == $ns",
            ".metadata.labels[\"app.kubernetes.io/managed-by\"] == \"lumen-standalone\" and\n    .metadata.labels[\"lumen.axiom.dev/owner-namespace\"] == $ns",
            "MUTATION",
        ),
        (
            "v2_recover_created_uids\n  if [[ \"$V2_CRB_ARMED\" == true ]]",
            "true # UID recovery disabled\n  if [[ \"$V2_CRB_ARMED\" == true ]]",
            "MUTATION",
        ),
        (
            "V2_CRB_UID=\"$recovered\"",
            "# V2_CRB_UID=\"$recovered\"",
            "MUTATION",
        ),
        (
            "\"$KUSTOMIZE_RENDERER\" tooling \\",
            "# \"$KUSTOMIZE_RENDERER\" tooling \\",
            "CLIENT",
        ),
        (
            "\"$KUSTOMIZE_RENDERER\" tooling \\",
            "echo \"$KUSTOMIZE_RENDERER\" tooling \\",
            "CLIENT",
        ),
        (
            "\"$KUSTOMIZE_RENDERER\" tooling \\",
            "false && \"$KUSTOMIZE_RENDERER\" tooling \\",
            "CLIENT",
        ),
        (
            "\"$KUSTOMIZE_RENDERER\" tooling \\",
            "\"$KUSTOMIZE_RENDERER\" api \\",
            "CLIENT",
        ),
        (
            "\"$KUSTOMIZE_RENDERER\" tooling \\",
            "jq -n --arg job \"$job\"",
            "CLIENT",
        ),
        (
            "\"$KUSTOMIZE_RENDERER\" api \\",
            "jq -n --arg job \"$job\"",
            "CLIENT",
        ),
        (
            "\"$KUSTOMIZE_RENDERER\" metrics \\",
            "read -r -d '' program <<'SH'",
            "CLIENT",
        ),
        (
            "ruby \"$KUSTOMIZE_VALIDATOR\" tooling \\",
            "# ruby \"$KUSTOMIZE_VALIDATOR\" tooling \\",
            "CLIENT",
        ),
        (
            "    --emit-json >\"$validated\"",
            "",
            "CLIENT",
        ),
        (
            "if ! k apply -f \"$TMP_ROOT/v2-client/rendered.yaml\" >/dev/null; then die \"client apply failed\"; fi",
            "if ! k apply -f \"$TMP_ROOT/v2-client/namespace.json\" >/dev/null; then die \"client apply failed\"; fi",
            "CLIENT",
        ),
        (
            "for ((attempt = 1; attempt <= 6; attempt++))",
            "for ((attempt = 1; attempt <= 60; attempt++))",
            "CLIENT",
        ),
        (
            "--request-timeout=10s",
            "--request-timeout=60s",
            "CLIENT",
        ),
        (
            "grep -Fq -- 'No agent available'",
            "grep -Fqx -- 'No agent available'",
            "CLIENT",
        ),
        ("sleep 5", "sleep 1", "CLIENT"),
        (
            "grep -Fq -- 'No agent available' \"$error\" || die \"job log read failed\"",
            "grep -Fq -- 'No agent available' \"$error\" || true # retry every log error",
            "CLIENT",
        ),
        (
            "v2_read_job_log \"$job\" \"$log\"\n  grep -Fx 'row=client-tools status=passed'",
            "k logs \"job/$job\" --namespace \"$V2_CLIENT_NAMESPACE\" >\"$log\"\n  grep -Fx 'row=client-tools status=passed'",
            "CLIENT",
        ),
        (
            "v2_read_job_log \"$job\" \"$log\"\n  grep -Fx 'row=client-tools status=passed'",
            "k apply -f \"$render_dir/rendered.yaml\" >/dev/null\n  v2_read_job_log \"$job\" \"$log\"\n  grep -Fx 'row=client-tools status=passed'",
            "CLIENT",
        ),
        (
            "v2_read_job_log() {\n",
            "v2_read_job_log() {\n  LUMEN_STANDALONE_GKE_TEST_NO_SLEEP=1\n",
            "CLIENT",
        ),
        (
            "db=\"$(v2_metric_total delegated_auth_denied_total \"$before\")\"",
            "db=\"$(v2_metric_total delegated_auth_denied \"$before\")\"",
            "METRICS",
        ),
        (
            "--argjson tokenreview_delta \"$((ta-tb))\"",
            "--argjson tokenreview_delta \"1\"",
            "METRICS",
        ),
        (
            "v2_wait_pod \"$initial_uid\"",
            "v2_wait_pod \"\"",
            "RESTART",
        ),
        (
            "v2_wait_pod '' in-cluster 500m 512Mi\n  v2_capture_pvc_identity",
            "v2_capture_pvc_identity\n  v2_wait_pod '' in-cluster 500m 512Mi",
            "RESTART",
        ),
        (
            "ingresses=\"$(k get ingress --namespace \"$V2_RUNTIME_NAMESPACE\" -o name)\" || die \"Ingress inventory could not be read\"",
            "ingresses=\"$(k get ingress --namespace \"$V2_RUNTIME_NAMESPACE\" -o name 2>/dev/null || true)\"",
            "NETWORK",
        ),
        (
            "gateways=\"$(k get gateways.gateway.networking.k8s.io --namespace \"$V2_RUNTIME_NAMESPACE\" -o name)\" || die \"Gateway inventory could not be read\"",
            "gateways=\"$(k get gateways.gateway.networking.k8s.io --namespace \"$V2_RUNTIME_NAMESPACE\" -o name 2>/dev/null || true)\"",
            "NETWORK",
        ),
        (
            "[[ \"$1\" =~ ^[a-z0-9-]{1,40}$ ]]",
            "[[ \"$1\" =~ ^[a-z0-9-]{1,22}$ ]]",
            "CLIENT",
        ),
        (
            "v2_run_metrics_job metrics-before-incluster",
            "v2_run_metrics_job aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "CLIENT",
        ),
        (
            "case \"$arch\" in amd64)",
            "case \"$arch\" in amd64|arm64)",
            "CLIENT",
        ),
        (
            "profile:\"LUMEN_AUTH=required\"",
            "profile:\"LUMEN_AUTH=off\"",
            "REQUIRED",
        ),
        (
            "k get statefulset lumen --namespace \"$V2_RUNTIME_NAMESPACE\" -o json >\"$live\"",
            "cp \"$V2_APPLY_ROOT/runtime/statefulset.yaml\" \"$live\"",
            "REQUIRED",
        ),
        (
            ".spec.enableServiceLinks == false",
            ".spec.enableServiceLinks == true",
            "REQUIRED",
        ),
        (
            ".spec.enableServiceLinks == false",
            "true",
            "REQUIRED",
        ),
        (
            ".spec.enableServiceLinks == false and",
            "# .spec.enableServiceLinks == false and\n          true and",
            "REQUIRED",
        ),
        (
            "profile:\"LUMEN_AUTH=required\",audience:\"lumen.axiom.dev\"",
            "profile:\"LUMEN_AUTH=required\",audience:\"other\"",
            "REQUIRED",
        ),
        (
            "required-default-app app default POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 401 none none",
            "required-default-app app projected POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 401 none none",
            "REQUIRED",
        ),
        (
            "v2_run_api_job unlisted unlisted default POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 403 none none",
            "v2_run_api_job unlisted unlisted default POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 401 none none",
            "EXECUTION",
        ),
        (
            "v2_run_api_job missing default missing POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 401 none none",
            "v2_run_api_job missing default missing POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 403 none none",
            "EXECUTION",
        ),
        (
            "v2_run_api_job application-admin app default GET /admin/backup '' 403 none none",
            "# v2_run_api_job application-admin app default GET /admin/backup '' 403 none none",
            "EXECUTION",
        ),
        (
            "v2_run_api_job required-default-app app default POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 401 none none",
            "v2_run_api_job required-default-app app default POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 403 none none",
            "EXECUTION",
        ),
        (
            "v2_run_api_job required-projected-unlisted unlisted projected POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 403 none none",
            "v2_run_api_job required-projected-unlisted unlisted projected POST /collections/gke/search '{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"first\"}},\"limit\":10}' 401 none none",
            "EXECUTION",
        ),
        (
            "[[ \"$RECEIPT_SHA256\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_MANIFEST_SHA256\" ]]",
            "[[ -n \"$RECEIPT_SHA256\" ]]",
            "CANDIDATE",
        ),
        (
            "[[ \"$CANDIDATE_COMMIT\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_COMMIT\" ]]",
            "# [[ \"$CANDIDATE_COMMIT\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_COMMIT\" ]]",
            "CANDIDATE",
        ),
        (
            "[[ \"$CANDIDATE_RUN_ID\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID\" ]] ||\n    die \"candidate manifest run id differs from the controller-bound expected run\"",
            "[[ \"$CANDIDATE_RUN_ID\" == \"$LUMEN_STANDALONE_GKE_EXPECTED_RUN_ID\" ]] ||\n    true # controller binding bypassed",
            "CANDIDATE",
        ),
        (
            ".redaction == {authorization_retained:false",
            ".redaction == {authorization_retained:true",
            "RECEIPT",
        ),
        (
            "receipt matrix has unexpected keys",
            "receipt keys unchecked",
            "RECEIPT",
        ),
        (
            ".schema == \"lumen.standalone-gke-receipt/v2\" and .stage == \"slice-b-live\" and .complete == true",
            ".schema == \"lumen.standalone-gke-receipt/v2\" and true",
            "RECEIPT",
        ),
        (
            "([.matrix|to_entries[]|select(.key != \"required_continuity\")|.value] | all(.[]; . == \"passed\"))",
            "true",
            "RECEIPT",
        ),
        (
            "--argjson required_deltas \"$V2_REQUIRED_DELTAS\"",
            "--argjson required_deltas \"$undefined\"",
            "RECEIPT",
        ),
        (
            "type == \"number\" and floor == . and . > 0",
            "type == \"number\" and floor == . and . >= 0",
            "RECEIPT",
        ),
        (
            "((.matrix.required_continuity.scheduled_node_arch == \"amd64\" and .matrix.required_continuity.scheduled_runtime_child_digest == .candidate.amd64_digest and (.matrix.required_continuity.observed_runtime_image_digest == .candidate.root_digest or .matrix.required_continuity.observed_runtime_image_digest == .candidate.amd64_digest)) or (.matrix.required_continuity.scheduled_node_arch == \"arm64\" and .matrix.required_continuity.scheduled_runtime_child_digest == .candidate.arm64_digest and (.matrix.required_continuity.observed_runtime_image_digest == .candidate.root_digest or .matrix.required_continuity.observed_runtime_image_digest == .candidate.arm64_digest)))",
            "true # nested child unchecked",
            "RECEIPT",
        ),
        (
            "\"$receipt_bytes\" -gt 0 && \"$receipt_bytes\" -le 16384",
            "true # receipt size unchecked",
            "RECEIPT",
        ),
        (
            "v2_wait_pv_gone \"$V2_PV_NAME\"",
            "true # PV reclaim unchecked",
            "CLEANUP",
        ),
        (
            "k delete namespace \"$V2_RUNTIME_NAMESPACE\"",
            "k delete namespace --selector app=lumen",
            "CLEANUP",
        ),
        (
            "v2_absent namespace \"$V2_RUNTIME_NAMESPACE\" || V2_CLEAN=false\n  v2_absent namespace \"$V2_CLIENT_NAMESPACE\" || V2_CLEAN=false\n  v2_absent clusterrolebinding \"$V2_CRB\" || V2_CLEAN=false",
            "true",
            "CLEANUP",
        ),
        (
            "mv -f -- \"$RECEIPT_SIDECAR_TMP\" \"$receipt.sha256\" || die \"receipt sidecar commit failed\"\n  mv -f -- \"$RECEIPT_TMP\" \"$receipt\" || die \"receipt commit failed\"",
            "mv -f -- \"$RECEIPT_TMP\" \"$receipt\" || die \"receipt commit failed\"\n  mv -f -- \"$RECEIPT_SIDECAR_TMP\" \"$receipt.sha256\" || die \"receipt sidecar commit failed\"",
            "RECEIPT",
        ),
        (
            "if ! (v2_write_receipt_body); then\n    rm -f -- \"$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/lumen-standalone-gke-receipt.json\" \"$LUMEN_STANDALONE_GKE_EVIDENCE_DIR/lumen-standalone-gke-receipt.json.sha256\" \"$RECEIPT_TMP\" \"$RECEIPT_SIDECAR_TMP\"",
            "if ! (v2_write_receipt_body); then\n    true # failed receipt bytes retained",
            "RECEIPT",
        ),
        (
            "run_live_acceptance_v2() {\n  local initial_uid replacement_uid resized_uid required_uid",
            "run_live_acceptance_v2() {\n  exit 0\n  local initial_uid replacement_uid resized_uid required_uid",
            "EXECUTION",
        ),
        (
            "run_live_acceptance_v2() {\n  local initial_uid replacement_uid resized_uid required_uid",
            "run_live_acceptance_v2() {\n  return 0\n  local initial_uid replacement_uid resized_uid required_uid",
            "EXECUTION",
        ),
        (
            "run_live_acceptance_v2() {\n  local initial_uid replacement_uid resized_uid required_uid",
            "run_live_acceptance_v2() {\n  if false; then\n  local initial_uid replacement_uid resized_uid required_uid",
            "EXECUTION",
        ),
        (
            "run_live_acceptance() {\n  run_live_acceptance_v2\n}",
            "run_live_acceptance() {\n  true\n}",
            "EXECUTION",
        ),
        (
            "\nrun_live_acceptance\n\nexit 0\n",
            "\n# run_live_acceptance\n\nexit 0\n",
            "EXECUTION",
        ),
    ] {
        let changed = replace_once(&source, from, to);
        assert!(
            findings(&changed).contains(&expected),
            "mutation {from:?} did not trigger {expected}; findings={:?}",
            findings(&changed)
        );
    }

    let changed = replace_first(
        &source,
        "--token-mode \"$token_mode\" \\",
        "--token-mode default \\",
    );
    assert!(
        findings(&changed).contains(&"CLIENT"),
        "wrong API token argument did not fail the shared renderer contract"
    );
    let changed = replace_first(
        &source,
        "--file \"$render_dir/rendered.yaml\"",
        "--file \"$render_dir/other.yaml\"",
    );
    assert!(
        findings(&changed).contains(&"CLIENT"),
        "validator/apply file drift did not fail the shared renderer contract"
    );
    let changed = replace_first(
        &source,
        "k apply -f \"$render_dir/rendered.yaml\" >/dev/null",
        "k apply -f \"$file\" >/dev/null",
    );
    assert!(
        findings(&changed).contains(&"CLIENT"),
        "apply of a different rendered file did not fail the shared renderer contract"
    );

    let full = full_script();
    let changed = replace_once(
        &full,
        "reject_token_env\n\nrequire_tool",
        "allow_token_env\n\nrequire_tool",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "token-environment bypass did not fail the preflight contract"
    );
    let changed = replace_once(
        &full,
        ".reclaimPolicy == \"Delete\"",
        ".reclaimPolicy == \"Retain\"",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "Retain StorageClass did not fail the preflight contract"
    );
    let changed = replace_once(&full, "config current-context", "config get-contexts");
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "ambient CLI current-context bypass did not fail the preflight contract"
    );
    let changed = replace_once(
        &full,
        "if [[ -n \"${TMP_ROOT:-}\" && -d \"$TMP_ROOT\" ]]; then\n    rm -rf -- \"$TMP_ROOT\"\n  fi",
        "if false; then\n    true\n  fi",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "preflight private-directory cleanup bypass did not fail the contract"
    );
    let changed = replace_once(&full, "  LUMEN_STANDALONE_GKE_EXPECTED_COMMIT \\\n", "");
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "missing controller-bound expected commit input did not fail preflight"
    );
    let changed = replace_once(
        &full,
        "APPROVED_CLIENT_IMAGE=\"docker.io/curlimages/curl@sha256:7c12af72ceb38b7432ab85e1a265cff6ae58e06f95539d539b654f2cfa64bb13\"",
        "APPROVED_CLIENT_IMAGE=\"docker.io/curlimages/curl@sha256:ac12af72ceb38b7432ab85e1a265cff6ae58e06f95539d539b654f2cfa64bb13\"",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "client-oracle digest drift did not fail preflight"
    );
    let changed = replace_once(
        &full,
        "[[ \"$LUMEN_STANDALONE_GKE_CLIENT_IMAGE\" == \"$APPROVED_CLIENT_IMAGE\" ]]",
        "[[ -n \"$LUMEN_STANDALONE_GKE_CLIENT_IMAGE\" ]]",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "arbitrary client image did not fail preflight"
    );
    let changed = replace_once(
        &full,
        "KUSTOMIZE_RENDERER=\"$KUSTOMIZE_ROOT/scripts/render.sh\"",
        "KUSTOMIZE_RENDERER=\"$CUSTOM_RENDERER\"",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "environment-overridable renderer path did not fail preflight"
    );
    let changed = replace_once(
        &full,
        "KUSTOMIZE_VALIDATOR=\"$KUSTOMIZE_ROOT/scripts/validate.rb\"",
        "KUSTOMIZE_VALIDATOR=\"$CUSTOM_VALIDATOR\"",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "environment-overridable validator path did not fail preflight"
    );
    let changed = replace_once(
        &full,
        "KUSTOMIZE_ROOT=\"$PRIVATE_REPOSITORY_ROOT/kustomize/lumen-standalone-acceptance\"",
        "KUSTOMIZE_ROOT=\"$TMP_ROOT/kustomize-harness\"",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "repository renderer use did not fail the private-copy contract"
    );
    let changed = replace_once(
        &full,
        "cp -R -- \"$KUSTOMIZE_SOURCE_ROOT\" \"$KUSTOMIZE_ROOT\"",
        "true # private harness copy bypassed",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "missing private harness copy did not fail preflight"
    );
    for (anchor, replacement, message) in [
        (
            "[[ ! -e \"$PRIVATE_REPOSITORY_ROOT\" && ! -L \"$PRIVATE_REPOSITORY_ROOT\" ]] ||",
            "true # repository existence check bypassed",
            "repository existence check bypass did not fail preflight",
        ),
        (
            "mkdir -m 700 \"$PRIVATE_REPOSITORY_ROOT\"",
            "true # repository mkdir bypassed",
            "repository mkdir bypass did not fail preflight",
        ),
        (
            "mkdir -m 700 \"$PRIVATE_REPOSITORY_ROOT/kustomize\"",
            "true # kustomize mkdir bypassed",
            "kustomize mkdir bypass did not fail preflight",
        ),
        (
            "PRIVATE_REPOSITORY_ROOT=\"$(cd \"$PRIVATE_REPOSITORY_ROOT\" && pwd -P)\" || die \"private repository root cannot be canonicalized\"",
            "true # canonicalization bypassed",
            "canonicalization bypass did not fail preflight",
        ),
        (
            "[[ -d \"$PRIVATE_REPOSITORY_ROOT\" && ! -L \"$PRIVATE_REPOSITORY_ROOT\" && -d \"$PRIVATE_REPOSITORY_ROOT/kustomize\" && ! -L \"$PRIVATE_REPOSITORY_ROOT/kustomize\" ]] ||",
            "true # parent safety bypassed",
            "parent safety bypass did not fail preflight",
        ),
    ] {
        let changed = replace_once(&full, anchor, replacement);
        assert!(preflight_findings(&changed).contains(&"PREFLIGHT"), "{message}");
    }
    let changed = replace_once(
        &full,
        "[[ \"$(sha256_file \"$KUSTOMIZE_RENDERER\")\" == \"$KUSTOMIZE_RENDERER_SHA256\" ]]",
        "[[ \"$(sha256_file \"$KUSTOMIZE_RENDERER_SOURCE\")\" == \"$KUSTOMIZE_RENDERER_SHA256\" ]]",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "hashing the mutable source renderer did not fail preflight"
    );
    let changed = replace_once(
        &full,
        "KUSTOMIZE_RENDERER_SHA256=\"f83e347b5f66c6cad049595a776230ea559f80efc265129f407032bf5a93dd74\"",
        "KUSTOMIZE_RENDERER_SHA256=\"0000000000000000000000000000000000000000000000000000000000000000\"",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "wrong renderer hash did not fail preflight"
    );
    let changed = replace_once(
        &full,
        "KUSTOMIZE_VALIDATOR_SHA256=\"43355d4a083303c9ffadade98f4add46958d7a7e625100dea97d979a3d1d294e\"",
        "KUSTOMIZE_VALIDATOR_SHA256=\"0000000000000000000000000000000000000000000000000000000000000000\"",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "wrong validator hash did not fail preflight"
    );
    let changed = replace_once(
        &full,
        "cmp -s \"$sidecar\" <(printf",
        "test -f \"$sidecar\" # checksum optional",
    );
    assert!(
        integrity_findings(&changed).contains(&"INTEGRITY"),
        "non-exact checksum sidecar did not fail the integrity contract"
    );
    let changed = replace_once(
        &full,
        "tar -tzf \"$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR/$archive\" | LC_ALL=C sort",
        "tar -tzf \"$LUMEN_STANDALONE_GKE_CANDIDATE_RECEIPT_DIR/$archive\"",
    );
    assert!(
        integrity_findings(&changed).contains(&"INTEGRITY"),
        "archive member order bypass did not fail the integrity contract"
    );
    let changed = replace_once(
        &full,
        "\"$VERIFIED_CLI\" standalone gke render",
        "\"$LUMEN_STANDALONE_GKE_CLI\" standalone gke render",
    );
    assert!(
        integrity_findings(&changed).contains(&"INTEGRITY"),
        "unverified render execution did not fail the integrity contract"
    );
    let invocation = "validate_candidate_manifest_v2\n[[ \"$LUMEN_STANDALONE_GKE_IMAGE\" == \"ghcr.io/chrischeng-c4/lumen@$ROOT_DIGEST\" ]] ||\n  die \"candidate image is not the exact receipt root digest\"\n";
    let without_invocation = replace_once(&full, invocation, "");
    let render_at = without_invocation
        .find("[[ -f \"$RENDERED/.lumen-standalone-managed\" ]]")
        .expect("verified render postcondition");
    let mut reordered = without_invocation;
    reordered.insert_str(render_at, invocation);
    assert!(
        integrity_findings(&reordered).contains(&"INTEGRITY"),
        "render-before-validation did not fail the integrity contract"
    );
}

#[test]
fn physical_temp_root_and_path_validation_are_executable_contracts() {
    let fixture = make_gke_oracle_fixture();
    let harness = root().join("kustomize/lumen-standalone-acceptance");
    let harness_before = collect_files(&harness);

    let output = run_gke_oracle(&fixture, &fixture.kubeconfig, &fixture.evidence);
    assert!(!output.status.success(), "fake gcloud unexpectedly passed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("standalone GKE acceptance: could not describe the requested GKE cluster"),
        "positive canonical path did not reach the expected gcloud error: {stderr}"
    );
    assert!(fixture.kubectl_marker.exists(), "fake kubectl was not reached");
    let calls = fs::read_to_string(&fixture.kubectl_calls).expect("read kubectl calls");
    let call_paths: Vec<_> = calls.lines().collect();
    assert_eq!(call_paths.len(), 4, "expected four config kubectl calls: {calls}");
    assert!(call_paths.windows(2).all(|pair| pair[0] == pair[1]));
    let cache = Path::new(call_paths[0]);
    assert!(cache.ends_with("kubectl-cache"));
    let run_root = cache.parent().expect("cache run root");
    assert_eq!(run_root.parent(), Some(fixture.physical_root.as_path()));
    assert!(!run_root.exists(), "cleanup did not remove the acceptance run root");
    let marker = fs::read_to_string(&fixture.kubectl_marker).expect("read fake kubectl marker");
    let mut marker_lines = marker.lines();
    let observed_run_root = PathBuf::from(marker_lines.next().expect("marker run root"));
    assert_eq!(observed_run_root, run_root);
    assert_eq!(marker_lines.next(), Some("--tmpdir"));
    for line in marker_lines {
        if !line.is_empty() {
            panic!("TMPDIR was used for the private acceptance root: {marker}");
        }
    }
    let added_name = run_root
        .file_name()
        .expect("physical temp child name")
        .to_string_lossy();
    assert!(added_name.starts_with("lumen-standalone-gke."));
    assert_eq!(added_name.len(), "lumen-standalone-gke.".len() + 6);
    assert_eq!(
        fs::read_to_string(&fixture.gcloud_marker).expect("read gcloud marker"),
        "home-unset\n"
    );
    assert_eq!(collect_files(&harness), harness_before, "checked-in harness changed");
    assert!(
        !fixture.cwd.join(".kube").exists(),
        "kubectl wrote its default cache below the isolated current directory"
    );

    let kubeconfig_dot = fixture
        .kubeconfig
        .parent()
        .unwrap()
        .join(".")
        .join(fixture.kubeconfig.file_name().unwrap());
    let cases = [
        (
            "kubeconfig-dot",
            kubeconfig_dot,
            fixture.evidence.clone(),
            "KUBECONFIG must be an existing regular non-symlink file",
        ),
        (
            "evidence-dot",
            fixture.kubeconfig.clone(),
            fixture.evidence.parent().unwrap().join("./evidence"),
            "LUMEN_STANDALONE_GKE_EVIDENCE_DIR must be an existing non-symlink directory",
        ),
    ];
    for (label, kubeconfig, evidence, expected_error) in cases {
        let _ = fs::remove_file(&fixture.kubectl_marker);
        let _ = fs::remove_file(&fixture.kubectl_calls);
        let _ = fs::remove_file(&fixture.gcloud_marker);
        let output = run_gke_oracle(&fixture, &kubeconfig, &evidence);
        assert!(!output.status.success(), "{label} unexpectedly passed");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected_error),
            "{label} returned the wrong error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!fixture.kubectl_marker.exists(), "{label} reached fake kubectl");
        assert!(!fixture.kubectl_calls.exists(), "{label} reached fake kubectl");
        assert!(!fixture.gcloud_marker.exists(), "{label} reached fake gcloud");
        assert_eq!(collect_files(&harness), harness_before, "{label} changed checked-in harness");
        assert!(!fixture.cwd.join(".kube").exists(), "{label} wrote a default kubectl cache");
    }

    let real_parent = fixture._root.path().join("real-parent");
    let symlink_parent = fixture._root.path().join("symlink-parent");
    fs::create_dir(&real_parent).expect("real kubeconfig parent");
    symlink(&real_parent, &symlink_parent).expect("kubeconfig parent symlink");
    let symlink_kubeconfig = symlink_parent.join("kubeconfig");
    fs::write(real_parent.join("kubeconfig"), b"not parsed\n").expect("symlink kubeconfig");

    let evidence_target = fixture._root.path().join("evidence-target");
    let evidence_link = fixture._root.path().join("evidence-link");
    fs::create_dir(&evidence_target).expect("evidence target");
    symlink(&evidence_target, &evidence_link).expect("evidence symlink");
    let nonempty = fixture._root.path().join("evidence-nonempty");
    fs::create_dir(&nonempty).expect("non-empty evidence");
    fs::write(nonempty.join("unexpected"), b"x").expect("non-empty evidence marker");

    for (label, kubeconfig, evidence, expected_error) in [
        (
            "kubeconfig-parent-symlink",
            symlink_kubeconfig,
            fixture.evidence.clone(),
            "KUBECONFIG must be an existing regular non-symlink file",
        ),
        (
            "evidence-symlink",
            fixture.kubeconfig.clone(),
            evidence_link,
            "LUMEN_STANDALONE_GKE_EVIDENCE_DIR must be an existing non-symlink directory",
        ),
        (
            "evidence-nonempty",
            fixture.kubeconfig.clone(),
            nonempty,
            "LUMEN_STANDALONE_GKE_EVIDENCE_DIR must be empty",
        ),
    ] {
        let _ = fs::remove_file(&fixture.kubectl_marker);
        let _ = fs::remove_file(&fixture.kubectl_calls);
        let _ = fs::remove_file(&fixture.gcloud_marker);
        let output = run_gke_oracle(&fixture, &kubeconfig, &evidence);
        assert!(!output.status.success(), "{label} unexpectedly passed");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected_error),
            "{label} returned the wrong error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!fixture.kubectl_marker.exists(), "{label} reached fake kubectl");
        assert!(!fixture.kubectl_calls.exists(), "{label} reached fake kubectl");
        assert!(!fixture.gcloud_marker.exists(), "{label} reached fake gcloud");
        assert_eq!(collect_files(&harness), harness_before, "{label} changed checked-in harness");
        assert!(!fixture.cwd.join(".kube").exists(), "{label} wrote a default kubectl cache");
    }
}

#[test]
fn physical_temp_root_mutations_fail_closed() {
    let full = full_script();
    let changed = replace_once(
        &full,
        "PRIVATE_TMP_ROOT=\"$(cd -P /tmp && pwd -P)\"",
        "PRIVATE_TMP_ROOT=\"$(cd /tmp && pwd -P)\"",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "removing physical path resolution did not fail preflight"
    );
    let changed = replace_once(
        &full,
        "PRIVATE_TMP_ROOT=\"$(cd -P /tmp && pwd -P)\"",
        "PRIVATE_TMP_ROOT=\"$(cd -P \"$TMPDIR\" && pwd -P)\"",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "TMPDIR-derived private root did not fail preflight"
    );
    let changed = replace_once(
        &full,
        "TMP_ROOT=\"$(mktemp -d \"$PRIVATE_TMP_ROOT/lumen-standalone-gke.XXXXXX\")\"",
        "TMP_ROOT=\"$(mktemp -d /tmp/lumen-standalone-gke.XXXXXX)\"",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "raw /tmp mktemp did not fail preflight"
    );
    let changed = replace_once(
        &full,
        "TMP_ROOT=\"$(mktemp -d \"$PRIVATE_TMP_ROOT/lumen-standalone-gke.XXXXXX\")\"",
        "TMP_ROOT=\"$(mktemp -d \"$TMPDIR/lumen-standalone-gke.XXXXXX\")\"",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "TMPDIR-derived mktemp did not fail preflight"
    );
    let changed = replace_once(
        &full,
        "safe_private_file \"$KUBECONFIG\" ||",
        "true ||",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "kubeconfig safe_private_file bypass did not fail preflight"
    );
    let changed = replace_once(
        &full,
        "safe_private_dir \"$LUMEN_STANDALONE_GKE_EVIDENCE_DIR\" ||",
        "true ||",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "evidence safe_private_dir bypass did not fail preflight"
    );
    let changed = replace_once(
        &full,
        "[[ \"$path\" == \"$PRIVATE_TMP_ROOT\"/* && \"$path\" != */ && \"$path\" != *'/../'* && \"$path\" != */.. && \"$path\" != *'/./'* && -d \"$path\" && ! -L \"$path\" ]] || return 1",
        "[[ \"$path\" == /tmp/* || \"$path\" == /private/tmp/* ]] || return 1",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "broad /tmp or /private/tmp input prefix did not fail preflight"
    );
    let changed = replace_once(
        &full,
        "[[ \"$path\" == \"$PRIVATE_TMP_ROOT\"/* && \"$path\" != *'/../'* && \"$path\" != */.. && \"$path\" != *'/./'* && -f \"$path\" && ! -L \"$path\" ]] || return 1",
        "[[ \"$path\" == /tmp/* || \"$path\" == /private/tmp/* ]] || return 1",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "broad /tmp or /private/tmp file prefix did not fail preflight"
    );
    for (anchor, replacement, message) in [
        (
            "[[ \"${KUBECTL_CACHE_DIR%/*}\" == \"$TMP_ROOT\" && \"${KUBECTL_CACHE_DIR##*/}\" == kubectl-cache ]] || die 'kubectl cache path identity is unsafe'",
            "true # kubectl cache identity bypassed",
            "kubectl cache identity bypass did not fail preflight",
        ),
        (
            "    --context \"$LUMEN_STANDALONE_GKE_CONTEXT\" \\\n    --cache-dir \"$KUBECTL_CACHE_DIR\" \\",
            "    --context \"$LUMEN_STANDALONE_GKE_CONTEXT\" \\",
            "wrapper cache flag removal did not fail preflight",
        ),
        (
            "kubectl --kubeconfig \"$KUBECONFIG\" --cache-dir \"$KUBECTL_CACHE_DIR\" config current-context",
            "kubectl --kubeconfig \"$KUBECONFIG\" config current-context",
            "current-context cache flag removal did not fail preflight",
        ),
        (
            "kubectl --kubeconfig \"$KUBECONFIG\" --cache-dir \"$KUBECTL_CACHE_DIR\" config get-contexts",
            "kubectl --kubeconfig \"$KUBECONFIG\" config get-contexts",
            "raw get-contexts cache flag removal did not fail preflight",
        ),
    ] {
        let changed = replace_once(&full, anchor, replacement);
        assert!(preflight_findings(&changed).contains(&"PREFLIGHT"), "{message}");
    }
    let changed = replace_once(
        &full,
        "validate_candidate_manifest_v2\n",
        "kubectl --kubeconfig \"$KUBECONFIG\" get pods >/dev/null\nvalidate_candidate_manifest_v2\n",
    );
    assert!(
        preflight_findings(&changed).contains(&"PREFLIGHT"),
        "a later uncached raw kubectl invocation did not fail preflight"
    );
    for variable in ["HOME=", "XDG_CACHE_HOME=", "XDG_CONFIG_HOME=", "CLOUDSDK_CONFIG="] {
        let changed = replace_once(&full, "CDPATH=", &format!("CDPATH=\n{variable}/tmp"));
        assert!(
            preflight_findings(&changed).contains(&"PREFLIGHT"),
            "{variable} assignment did not fail preflight"
        );
    }
}
