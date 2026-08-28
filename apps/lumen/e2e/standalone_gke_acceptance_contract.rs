//! Static contract for the controller-owned GKE acceptance gate.
//!
//! A real GKE credential is a release-controller input. This test checks the
//! executable live slice and rejects mutations that make a green static test
//! look plausible while removing a live security or durability assertion.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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

fn has_exact_line(source: &str, expected: &str) -> bool {
    source.lines().any(|line| line.trim() == expected)
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
    for (name, component) in [
        ("v2_run_client_tooling_job", "tooling"),
        ("v2_run_api_job", "api"),
        ("v2_run_metrics_job", "metrics"),
    ] {
        let body = function_body(source, name);
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
            "PUBLIC_IMAGE",
            "v2-public.json\" >/dev/null || die \"public runtime is not the fixed 0.4.29 serving image\"",
        ),
        ("PATCH", "patch_statefulset_image \"$statefulset\" \"$label\""),
        ("PATCH", "patch_statefulset_image \"$V2_APPLY_ROOT/runtime/statefulset.yaml\" v2"),
        ("PATCH", "yaml_json \"$statefulset\" \"$canonical\""),
        ("PATCH", ".spec.template.spec.containers | length == 1"),
        ("PATCH", "(.spec.template.spec.containers | length == 1)\n    and .spec.template.spec.containers[0].name == \"serving\"\n    and .spec.template.spec.containers[0].image == \"ghcr.io/chrischeng-c4/lumen:0.4.29\""),
        ("PATCH", ".spec.template.spec.containers[0].name == \"serving\""),
        ("PATCH", ".spec.template.spec.containers[0].image == \"ghcr.io/chrischeng-c4/lumen:0.4.29\""),
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
        ("RECEIPT", "lumen.standalone-gke-receipt/v1"),
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
            ".schema == \"lumen.standalone-gke-receipt/v1\" and .stage == \"slice-b-live\" and .complete == true",
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
            ".matrix.required_continuity.observed_runtime_child_digest == .candidate.observed_runtime_child_digest",
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
        ".schema == \"lumen.standalone-gke-receipt/v1\" and .stage == \"slice-b-live\" and .complete == true and",
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
    if source.find("metrics-before-incluster") > source.find("v2_run_api_job create")
        || source.find("metrics-before-required") > source.find("required-projected-app")
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

fn physical_temp_children(root: &Path) -> BTreeSet<PathBuf> {
    fs::read_dir(root)
        .expect("read physical temporary root")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("lumen-standalone-gke.")
        })
        .map(|entry| entry.path())
        .collect()
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
        "#!/bin/sh\nset -eu\ncache=\ncount=0\nprevious=\nfor arg in \"$@\"; do\n  if [ \"$previous\" = --cache-dir ]; then cache=$arg; count=$((count + 1)); fi\n  previous=$arg\ndone\n[ \"$count\" -eq 1 ] || exit 18\ncache=$(cd \"$cache\" && pwd -P) || exit 19\ncase \"$cache\" in\n  /tmp/lumen-standalone-gke.??????/kubectl-cache|/private/tmp/lumen-standalone-gke.??????/kubectl-cache) ;;\n  *) exit 20 ;;\nesac\nprintf '%s\\n' \"$cache\" >>\"$FAKE_KUBECTL_CALLS\"\nmkdir -p \"$cache/discovery\" \"$cache/http\"\n: >\"$cache/discovery/response\"\n: >\"$cache/http/response\"\nif [ \"$(wc -l <\"$FAKE_KUBECTL_CALLS\" | tr -d ' ')\" -eq 1 ]; then\n  { find \"$FAKE_PHYSICAL_ROOT\" -maxdepth 1 -name 'lumen-standalone-gke.*' -print; printf '%s\\n' --tmpdir; find \"$TMPDIR\" -maxdepth 1 -name 'lumen-standalone-gke.*' -print; } >\"$FAKE_KUBECTL_MARKER\"\nfi\nprintf '%s\\n' oracle-context\n",
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
fn negative_mutations_remove_real_gate_obligations() {
    let source = live_slice();
    for (from, to, expected) in [
        (
            "cclab.lumen.candidate-manifest.v3",
            "candidate-v2",
            "CANDIDATE",
        ),
        ("tar -xOf", "tar -xzf", "ARCHIVE"),
        (
            "v2-public.json\" >/dev/null || die \"public runtime is not the fixed 0.4.29 serving image\"",
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
            "(.spec.template.spec.containers | length == 1)\n    and .spec.template.spec.containers[0].name == \"serving\"\n    and .spec.template.spec.containers[0].image == \"ghcr.io/chrischeng-c4/lumen:0.4.29\"",
            "(.spec.template.spec.containers | length >= 1)\n    and .spec.template.spec.containers[0].name == \"serving\"\n    and .spec.template.spec.containers[0].image == \"ghcr.io/chrischeng-c4/lumen:0.4.29\"",
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
            ".schema == \"lumen.standalone-gke-receipt/v1\" and .stage == \"slice-b-live\" and .complete == true",
            ".schema == \"lumen.standalone-gke-receipt/v1\" and true",
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
            ".matrix.required_continuity.observed_runtime_child_digest == .candidate.observed_runtime_child_digest",
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

    let before_children = physical_temp_children(&fixture.physical_root);
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
    let mut observed_children = BTreeSet::new();
    let mut in_tmpdir_section = false;
    for line in marker.lines() {
        if line == "--tmpdir" {
            in_tmpdir_section = true;
        } else if !in_tmpdir_section {
            observed_children.insert(PathBuf::from(line));
        } else {
            panic!("TMPDIR was used for the private acceptance root: {marker}");
        }
    }
    let added_children: Vec<_> = observed_children
        .difference(&before_children)
        .collect();
    assert_eq!(added_children.len(), 1, "fake kubectl saw the wrong physical temp set: {marker}");
    let added_name = added_children[0]
        .file_name()
        .expect("physical temp child name")
        .to_string_lossy();
    assert!(added_name.starts_with("lumen-standalone-gke."));
    assert_eq!(added_name.len(), "lumen-standalone-gke.".len() + 6);
    assert!(
        in_tmpdir_section,
        "fake kubectl did not inspect the unrelated TMPDIR: {marker}"
    );
    assert_eq!(
        fs::read_to_string(&fixture.gcloud_marker).expect("read gcloud marker"),
        "home-unset\n"
    );
    assert_eq!(
        physical_temp_children(&fixture.physical_root),
        before_children,
        "acceptance left a physical temporary child"
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
        let children = physical_temp_children(&fixture.physical_root);
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
        assert_eq!(physical_temp_children(&fixture.physical_root), children, "{label} created a temporary child");
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
        let children = physical_temp_children(&fixture.physical_root);
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
        assert_eq!(physical_temp_children(&fixture.physical_root), children, "{label} created a temporary child");
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
