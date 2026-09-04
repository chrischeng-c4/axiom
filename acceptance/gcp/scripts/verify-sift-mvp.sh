#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/kubernetes-ownership.sh"
source "$SCRIPT_DIR/sift-evidence-secrets.sh"

: "${PROJECT_ID:?PROJECT_ID is required}"
: "${REGION:?REGION is required}"
: "${GKE_ZONE:?GKE_ZONE is required}"
: "${GKE_CLUSTER_NAME:?GKE_CLUSTER_NAME is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${BACKUP_BUCKET:?BACKUP_BUCKET is required}"
: "${BACKUP_GSA_EMAIL:?BACKUP_GSA_EMAIL is required}"
: "${MANIFEST_DIR:?MANIFEST_DIR is required}"
: "${EVIDENCE_DIR:?EVIDENCE_DIR is required}"
: "${SIFT_CLI:?SIFT_CLI is required}"
: "${SIFT_IMAGE:?SIFT_IMAGE is required}"
: "${RIG_IMAGE:?RIG_IMAGE is required}"
: "${ACCEPTANCE_RUNNER_IMAGE:?ACCEPTANCE_RUNNER_IMAGE is required}"
: "${CANDIDATE_GIT_SHA:?CANDIDATE_GIT_SHA is required}"
: "${CANDIDATE_SOURCE_SHA256:?CANDIDATE_SOURCE_SHA256 is required}"
: "${CANDIDATE_CLOUD_BUILD_ID:?CANDIDATE_CLOUD_BUILD_ID is required}"
: "${CANDIDATE_SOURCE_URI:?CANDIDATE_SOURCE_URI is required}"
: "${SIFT_NODE_POOL:?SIFT_NODE_POOL is required}"
: "${ACCEPTANCE_LOCK_ACQUISITION_ID:?ACCEPTANCE_LOCK_ACQUISITION_ID is required}"

LOAD_SECONDS=1800
ITEMS_PER_SECOND=10000
EXPECTED_ITEMS=18000000
LOG_ITEMS=9000000
METRIC_ITEMS=5400000
SPAN_ITEMS=3600000
FAILOVER_SECONDS=300
FAILOVER_ITEMS=3000000
BATCH_ITEMS=1000
PROJECT=sift-mvp
PROJECT_ALT=sift-mvp-alt
NAMESPACE=sift
RESTORE_NAMESPACE=sift-restore
SIFT_PORT=17380

mkdir -p \
  "$EVIDENCE_DIR/kubernetes" \
  "$EVIDENCE_DIR/gcs" \
  "$EVIDENCE_DIR/load" \
  "$EVIDENCE_DIR/latency" \
  "$EVIDENCE_DIR/restore"

for command in awk curl date dirname gcloud gzip jq kubectl openssl python3 rg sed seq sort; do
  command -v "$command" >/dev/null 2>&1 || {
    echo "required command not found: $command" >&2
    exit 1
  }
done
[[ -x "$SIFT_CLI" ]] || {
  echo "SIFT_CLI is not executable: $SIFT_CLI" >&2
  exit 1
}
[[ "$SIFT_IMAGE" == *@sha256:* ]] || {
  echo "SIFT_IMAGE must use an immutable digest" >&2
  exit 1
}
[[ "$RIG_IMAGE" == *@sha256:* ]] || {
  echo "RIG_IMAGE must use an immutable digest" >&2
  exit 1
}
[[ "$ACCEPTANCE_RUNNER_IMAGE" == *@sha256:* ]] || {
  echo "ACCEPTANCE_RUNNER_IMAGE must use an immutable digest" >&2
  exit 1
}
[[ "$CANDIDATE_SOURCE_SHA256" =~ ^[0-9a-f]{64}$ ]] || {
  echo "CANDIDATE_SOURCE_SHA256 must be a SHA-256 hex digest" >&2
  exit 1
}
[[ "$CANDIDATE_CLOUD_BUILD_ID" =~ ^[A-Za-z0-9-]{1,128}$ ]] || {
  echo "CANDIDATE_CLOUD_BUILD_ID is invalid" >&2
  exit 1
}
[[ "$CANDIDATE_SOURCE_URI" == gs://* ]] || {
  echo "CANDIDATE_SOURCE_URI must be a gs:// object URI" >&2
  exit 1
}
candidate_gate_receipt="$EVIDENCE_DIR/candidate-gate.json"
[[ -f "$candidate_gate_receipt" && ! -L "$candidate_gate_receipt" ]] || {
  echo "the fixed candidate gate receipt is missing or unsafe" >&2
  exit 1
}
jq -e \
  --arg git_sha "$CANDIDATE_GIT_SHA" \
  --arg source_bundle_sha256 "$CANDIDATE_SOURCE_SHA256" '
    type == "object"
    and keys == ["completed_at","entrypoint","git_sha","schema","source_bundle_sha256","status"]
    and .schema == "axiom.gcp.sift.candidate-gate.v1"
    and .git_sha == $git_sha
    and .source_bundle_sha256 == $source_bundle_sha256
    and .entrypoint == "apps/sift/test.sh --candidate"
    and (.completed_at | type) == "string"
    and (.completed_at | length) > 0
    and .status == "passed"
  ' "$candidate_gate_receipt" >/dev/null || {
  echo "the fixed candidate gate receipt does not match this candidate" >&2
  exit 1
}

forward_pids=()
archive_iam_removed=0
wrong_peer_dir=""
token_file="$EVIDENCE_DIR/kubernetes/sift-rig.token"
sift_url="http://127.0.0.1:${SIFT_PORT}"

die() {
  echo "Sift MVP acceptance failed: $*" >&2
  capture_diagnostics
  exit 1
}

verify_pods_on_run_nodes() {
  local namespace="$1"
  local selector="$2"
  local stem="$3"
  local expected_count="$4"
  local pods="$EVIDENCE_DIR/kubernetes/${stem}-pods.json"
  local nodes="$EVIDENCE_DIR/kubernetes/${stem}-nodes.json"
  kubectl -n "$namespace" get pods -l "$selector" -o json > "$pods"
  kubectl get nodes -l "axiom-run-id=${RUN_ID}" -o json > "$nodes"
  jq -e \
    --arg run_id "$RUN_ID" \
    --argjson expected_count "$expected_count" \
    --slurpfile nodes "$nodes" '
    (.items | length) == $expected_count
    and all(.items[]; . as $pod
      | .spec.nodeSelector["axiom-run-id"] == $run_id
      and .spec.nodeName != null
      and any($nodes[0].items[];
        .metadata.name == $pod.spec.nodeName
        and .metadata.labels["axiom-run-id"] == $run_id))
  ' "$pods" >/dev/null \
    || die "${stem} did not have ${expected_count} pods only on this run's Sift nodes"
}

restore_archive_iam() {
  if [[ "$archive_iam_removed" == "1" ]]; then
    gcloud storage buckets add-iam-policy-binding "gs://${BACKUP_BUCKET}" \
      --member="serviceAccount:${BACKUP_GSA_EMAIL}" \
      --role="roles/storage.objectAdmin" \
      --project="$PROJECT_ID" \
      --quiet >/dev/null 2>&1 || true
    archive_iam_removed=0
  fi
}

stop_forwards() {
  local pid
  for pid in "${forward_pids[@]:-}"; do
    kill "$pid" >/dev/null 2>&1 || true
    wait "$pid" >/dev/null 2>&1 || true
  done
  forward_pids=()
}

cleanup_local() {
  restore_archive_iam
  stop_forwards
  sift_remove_ephemeral_evidence_secrets "$EVIDENCE_DIR" || {
    echo "could not remove the ephemeral Sift acceptance token" >&2
  }
  if [[ -n "$wrong_peer_dir" && -d "$wrong_peer_dir" ]]; then
    find "$wrong_peer_dir" -type f -delete
    find "$wrong_peer_dir" -depth -type d -empty -delete
  fi
}
trap cleanup_local EXIT INT
# The parent owns the 90-minute deadline and sends TERM after stopping the
# verifier's process tree. Never ignore that deadline signal.
trap 'exit 124' TERM

capture_diagnostics() {
  {
    kubectl -n "$NAMESPACE" get sift/sift -o yaml 2>&1 || true
    kubectl -n "$NAMESPACE" get deployment,statefulset,daemonset,pod,pvc,job \
      -o yaml 2>&1 || true
  } > "$EVIDENCE_DIR/kubernetes/sift-failure.yaml"
  {
    kubectl -n "$RESTORE_NAMESPACE" get sift/sift-restore -o yaml 2>&1 || true
    kubectl -n "$RESTORE_NAMESPACE" get deployment,statefulset,daemonset,pod,pvc \
      -o yaml 2>&1 || true
  } > "$EVIDENCE_DIR/kubernetes/sift-restore-failure.yaml"
  kubectl -n "$NAMESPACE" describe pods \
    > "$EVIDENCE_DIR/kubernetes/sift-pods-describe.txt" 2>&1 || true
  local pod
  for pod in $(kubectl -n "$NAMESPACE" get pods -o name 2>/dev/null); do
    kubectl -n "$NAMESPACE" logs "$pod" --all-containers --tail=300 --prefix \
      >> "$EVIDENCE_DIR/kubernetes/sift-pods.log" 2>&1 || true
    kubectl -n "$NAMESPACE" logs "$pod" --all-containers --tail=300 --prefix --previous \
      >> "$EVIDENCE_DIR/kubernetes/sift-pods-previous.log" 2>&1 || true
  done
}

refresh_token() {
  kubectl -n "$NAMESPACE" create token sift-rig \
    --audience=sift.axiom.dev \
    --duration=3600s > "$token_file"
  chmod 0600 "$token_file"
  [[ -s "$token_file" ]] || die "Kubernetes did not issue the Sift acceptance token"
}

auth_curl() {
  project_curl "$PROJECT" "$@"
}

project_curl() {
  local project="$1"
  shift
  curl --silent --show-error --fail-with-body \
    -H "authorization: Bearer $(sed -n '1p' "$token_file")" \
    -H "x-sift-project: ${project}" \
    "$@"
}

auth_curl_status() {
  project_curl_status "$PROJECT" "$@"
}

project_curl_status() {
  local project="$1"
  shift
  curl --silent --show-error \
    -H "authorization: Bearer $(sed -n '1p' "$token_file")" \
    -H "x-sift-project: ${project}" \
    "$@"
}

extract_sse_json() {
  local input="$1"
  local output="$2"
  sed -n 's/^data: //p' "$input" > "$output"
  [[ -s "$output" ]] || die "MCP response did not contain an SSE data event"
}

start_gateway_forward() {
  local deadline
  kubectl -n "$NAMESPACE" port-forward service/sift "${SIFT_PORT}:7380" \
    >> "$EVIDENCE_DIR/kubernetes/sift-port-forward.log" 2>&1 &
  forward_pids+=("$!")
  deadline=$((SECONDS + 120))
  while (( SECONDS < deadline )); do
    if curl --max-time 3 --silent --show-error --fail \
      "${sift_url}/readyz" >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  die "Sift gateway did not become ready through port-forward"
}

integrity_to() {
  local output="$1"
  mkdir -p "$(dirname "$output")"
  auth_curl "${sift_url}/admin/integrity?project=${PROJECT}" > "$output"
  jq -e --arg project "$PROJECT" '
    .version == 1
    and .project == $project
    and .event_id_digest_algorithm == "xor-sha256-v1"
  ' "$output" >/dev/null
}

verify_idempotency_retry() {
  local phase="$1"
  local before="$EVIDENCE_DIR/kubernetes/integrity-before-idempotency-${phase}.json"
  local after="$EVIDENCE_DIR/kubernetes/integrity-after-idempotency-${phase}.json"
  local query="$EVIDENCE_DIR/kubernetes/query-idempotency-${phase}.json"
  local signal response
  integrity_to "$before"
  for signal in logs metrics traces; do
    response="$EVIDENCE_DIR/kubernetes/otlp-idempotency-${signal}-${phase}-response.json"
    auth_curl -X POST "${sift_url}/v1/${signal}" \
      -H 'content-type: application/json' \
      --data-binary "@$EVIDENCE_DIR/kubernetes/otlp-idempotency-${signal}.json" \
      > "$response"
    jq -e 'type == "object" and (.partialSuccess == null)' "$response" >/dev/null \
      || die "the ${phase} ${signal} idempotency retry did not return a normal acknowledgement"
  done
  integrity_to "$after"
  jq -s -e '
    .[0] as $before | .[1] as $after
    | $after.event_count == $before.event_count
      and $after.event_id_sha256 == $before.event_id_sha256
      and $after.watermark == $before.watermark
      and $after.signals == $before.signals
  ' "$before" "$after" >/dev/null \
    || die "the ${phase} idempotency retry changed durable project identity"
  auth_curl -X POST "${sift_url}/api/v1/query" \
    -H 'content-type: application/json' \
    --data '{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"logs","filter":{"op":"eq","field":"event_id","value":"smoke-idempotency-log"}},"limit":10,"mode":"sync"}' \
    > "$query"
  jq -e '(.data.records | map(select(.event_id == "smoke-idempotency-log")) | length) == 1' \
    "$query" >/dev/null \
    || die "the ${phase} idempotency retry changed the stored row"
}

verify_load_digest() {
  local phase="$1"
  local duration="$2"
  local before="$3"
  local after="$4"
  local output="$EVIDENCE_DIR/load/${phase}/event-id-digest.json"
  local expected observed
  expected="$(python3 "$SCRIPT_DIR/sift-load-digest.py" phase \
    --name "$phase" --duration "$duration" --batch-items "$BATCH_ITEMS")"
  observed="$(python3 "$SCRIPT_DIR/sift-load-digest.py" xor \
    "$(jq -r '.event_id_sha256' "$before")" \
    "$(jq -r '.event_id_sha256' "$after")")"
  jq -n --arg expected "$expected" --arg observed "$observed" \
    '{algorithm:"xor-sha256-v1",expected:$expected,observed:$observed,match:($expected == $observed)}' \
    > "$output"
  [[ "$observed" == "$expected" ]] \
    || die "${phase} load event-ID digest did not match the independently generated digest"
}

wait_for_integrity_count() {
  local expected="$1"
  local output="$2"
  local deadline=$((SECONDS + 900))
  while (( SECONDS < deadline )); do
    if integrity_to "$output" 2>/dev/null \
      && [[ "$(jq -r '.event_count' "$output")" == "$expected" ]]; then
      return 0
    fi
    sleep 3
  done
  die "project count did not reach exactly $expected"
}

wait_for_delta() {
  local baseline="$1"
  local expected_delta="$2"
  local output="$3"
  local start expected
  start="$(jq -r '.event_count' "$baseline")"
  expected=$((start + expected_delta))
  wait_for_integrity_count "$expected" "$output"
}

number_le() {
  awk -v got="$1" -v limit="$2" 'BEGIN { exit !(got <= limit) }'
}

number_ge() {
  awk -v got="$1" -v limit="$2" 'BEGIN { exit !(got >= limit) }'
}

percentile_ms() {
  local input="$1"
  local percentile="$2"
  local sorted="${input}.sorted"
  local count rank
  sort -n "$input" > "$sorted"
  count="$(awk 'END {print NR}' "$sorted")"
  [[ "$count" -gt 0 ]] || return 1
  rank="$(awk -v n="$count" -v p="$percentile" \
    'BEGIN { r=int((n*p)+0.999999); if (r < 1) r=1; print r }')"
  sed -n "${rank}p" "$sorted" | awk '{printf "%.3f", $1 * 1000}'
}

snapshot_restarts() {
  local output="$1"
  kubectl -n "$NAMESPACE" get pods \
    -l app.kubernetes.io/name=sift -o json \
    | jq '[
        .items[]
        | select((.metadata.labels["sift.axiom.dev/role"] // "") as $role
            | ["store","control","gateway","query","agent"] | index($role))
        |
        {
          pod: .metadata.name,
          uid: .metadata.uid,
          node: .spec.nodeName,
          role: (.metadata.labels["sift.axiom.dev/role"] // ""),
          restarts: ([.status.containerStatuses[]?.restartCount] | add // 0),
          waiting: [.status.containerStatuses[]?.state.waiting.reason // empty],
          terminated: [.status.containerStatuses[]?.lastState.terminated.reason // empty]
        }
      ] | sort_by(.pod)' > "$output"
}

assert_no_unexpected_restarts() {
  local before="$1"
  local after="$2"
  jq -n -e --slurpfile before "$before" --slurpfile after "$after" '
    ($after[0] | map({key:.pod, value:.}) | from_entries) as $new
    | ($before[0] | length) == ($after[0] | length)
      and all($after[0][];
        (.waiting | index("CrashLoopBackOff") | not)
        and (.terminated | index("OOMKilled") | not))
      and all($before[0][];
        $new[.pod] != null
        and $new[.pod].uid == .uid
        and $new[.pod].restarts == .restarts)
  ' >/dev/null || die "a Sift pod was replaced, restarted, crash-looped, or OOM-killed"
}

assert_failover_restart_evidence() {
  local before="$1"
  local after="$2"
  local failed_node="$3"
  jq -n -e \
    --arg failed_node "$failed_node" \
    --slurpfile before "$before" \
    --slurpfile after "$after" '
    ($after[0] | map({key:.pod, value:.}) | from_entries) as $new
    | ($before[0] | map(.uid)) as $old_uids
    | ([ $after[0][] | select(.role == "store") ] | length) == 3
      and ([ $after[0][] | select(.role == "control") ] | length) == 3
      and ([ $after[0][] | select(.role == "gateway") ] | length) == 1
      and ([ $after[0][] | select(.role == "query") ] | length) == 1
      and ([ $after[0][] | select(.role == "agent") ] | length) == 3
      and all($after[0][];
        (.waiting | index("CrashLoopBackOff") | not)
        and (.terminated | index("OOMKilled") | not))
      and all($before[0][];
        if .node == $failed_node then
          if .role == "agent" then
            any($after[0][];
              .role == "agent"
              and (.uid as $uid | $old_uids | index($uid) == null))
          else
            $new[.pod] != null
            and $new[.pod].uid != .uid
            and $new[.pod].restarts == 0
          end
        else
          $new[.pod] != null
          and $new[.pod].uid == .uid
          and $new[.pod].restarts == .restarts
        end)
  ' >/dev/null \
    || die "failover changed a pod outside the stopped VM or caused a hidden restart"
}

wait_role_ready() {
  local kind="$1"
  local name="$2"
  local want="$3"
  local deadline=$((SECONDS + 900))
  local ready
  while (( SECONDS < deadline )); do
    if [[ "$kind" == "daemonset" ]]; then
      ready="$(kubectl -n "$NAMESPACE" get "$kind/$name" \
        -o jsonpath='{.status.numberReady}' 2>/dev/null || true)"
    else
      ready="$(kubectl -n "$NAMESPACE" get "$kind/$name" \
        -o jsonpath='{.status.readyReplicas}' 2>/dev/null || true)"
    fi
    [[ "$ready" == "$want" ]] && return 0
    sleep 5
  done
  die "$kind/$name did not reach $want ready replicas"
}

store_http_port() {
  echo $((17400 + $1))
}

store_peer_port() {
  echo $((18400 + $1))
}

start_store_forwards() {
  local ordinal
  local http_port
  local peer_port
  for ordinal in 0 1 2; do
    http_port="$(store_http_port "$ordinal")"
    peer_port="$(store_peer_port "$ordinal")"
    kubectl -n "$NAMESPACE" port-forward "pod/sift-store-${ordinal}" \
      "${http_port}:7380" \
      >> "$EVIDENCE_DIR/kubernetes/store-${ordinal}-http-forward.log" 2>&1 &
    forward_pids+=("$!")
    kubectl -n "$NAMESPACE" port-forward "pod/sift-store-${ordinal}" \
      "${peer_port}:7381" \
      >> "$EVIDENCE_DIR/kubernetes/store-${ordinal}-peer-forward.log" 2>&1 &
    forward_pids+=("$!")
  done
  sleep 3
}

store_integrity_to() {
  local ordinal="$1"
  local output="$2"
  local port
  port="$(store_http_port "$ordinal")"
  curl --max-time 30 --silent --show-error --fail-with-body \
    -H "authorization: Bearer $(sed -n '1p' "$token_file")" \
    -H "x-sift-project: ${PROJECT}" \
    "http://127.0.0.1:${port}/admin/integrity?project=${PROJECT}" > "$output"
}

store_raftz() {
  local ordinal="$1"
  local port host peer_dir
  port="$(store_peer_port "$ordinal")"
  host="sift-store-${ordinal}.sift-store-headless.sift.svc.cluster.local"
  peer_dir="$MANIFEST_DIR/sift/peer-pki"
  curl --noproxy '*' --max-time 5 --silent --show-error --fail \
    --cacert "$peer_dir/ca.crt" \
    --cert "$peer_dir/tls.crt" \
    --key "$peer_dir/tls.key" \
    --resolve "${host}:${port}:127.0.0.1" \
    "https://${host}:${port}/raftz"
}

verify_peer_mtls_rejection() {
  local ordinal=0
  local port host peer_dir
  port="$(store_peer_port "$ordinal")"
  host="sift-store-${ordinal}.sift-store-headless.sift.svc.cluster.local"
  peer_dir="$MANIFEST_DIR/sift/peer-pki"

  if curl --noproxy '*' --max-time 5 --silent --show-error \
      --cacert "$peer_dir/ca.crt" \
      --resolve "${host}:${port}:127.0.0.1" \
      --output "$EVIDENCE_DIR/kubernetes/peer-mtls-no-client.body" \
      "https://${host}:${port}/raftz" \
      2> "$EVIDENCE_DIR/kubernetes/peer-mtls-no-client.stderr"; then
    die "peer port accepted a client without a certificate"
  fi

  wrong_peer_dir="$(mktemp -d "${TMPDIR:-/tmp}/sift-wrong-peer.XXXXXX")"
  openssl req -x509 -newkey rsa:2048 -nodes -days 1 \
    -subj '/CN=sift-untrusted-acceptance-client' \
    -keyout "$wrong_peer_dir/tls.key" \
    -out "$wrong_peer_dir/tls.crt" >/dev/null 2>&1
  if curl --noproxy '*' --max-time 5 --silent --show-error \
      --cacert "$peer_dir/ca.crt" \
      --cert "$wrong_peer_dir/tls.crt" \
      --key "$wrong_peer_dir/tls.key" \
      --resolve "${host}:${port}:127.0.0.1" \
      --output "$EVIDENCE_DIR/kubernetes/peer-mtls-wrong-ca.body" \
      "https://${host}:${port}/raftz" \
      2> "$EVIDENCE_DIR/kubernetes/peer-mtls-wrong-ca.stderr"; then
    die "peer port accepted a client certificate from an untrusted CA"
  fi

  store_raftz "$ordinal" \
    > "$EVIDENCE_DIR/kubernetes/peer-mtls-positive-after-negatives.json"
  jq -e '.durability_error == null' \
    "$EVIDENCE_DIR/kubernetes/peer-mtls-positive-after-negatives.json" >/dev/null \
    || die "peer port was unavailable after the mTLS rejection probes"
  find "$wrong_peer_dir" -type f -delete
  find "$wrong_peer_dir" -depth -type d -empty -delete
  wrong_peer_dir=""
}

verify_sift_image_provenance() {
  local pod="sift-candidate-provenance-${RUN_ID}"
  local phase deadline
  "$SIFT_CLI" acceptance-build-info \
    > "$EVIDENCE_DIR/kubernetes/sift-local-build-info.json"
  jq -e --arg expected "$CANDIDATE_GIT_SHA" \
    '.git_sha == $expected and .next == "done"' \
    "$EVIDENCE_DIR/kubernetes/sift-local-build-info.json" >/dev/null \
    || die "local Sift CLI does not match the candidate Git revision"

  kubectl -n "$NAMESPACE" delete "pod/${pod}" --ignore-not-found \
    --wait=true --timeout=60s >/dev/null
  kubectl -n "$NAMESPACE" run "$pod" \
    --image="$SIFT_IMAGE" \
    --restart=Never \
    --labels="axiom-owner=gcp-operator-acceptance,axiom-run-id=${RUN_ID},sift-acceptance-probe=provenance" \
    --overrides="$(jq -nc --arg run_id "$RUN_ID" \
      '{spec:{nodeSelector:{"axiom-run-id":$run_id}}}')" \
    --command -- /usr/local/bin/sift acceptance-build-info >/dev/null
  deadline=$((SECONDS + 180))
  phase=""
  while (( SECONDS < deadline )); do
    phase="$(kubectl -n "$NAMESPACE" get "pod/${pod}" \
      -o jsonpath='{.status.phase}' 2>/dev/null || true)"
    [[ "$phase" == "Succeeded" || "$phase" == "Failed" ]] && break
    sleep 2
  done
  kubectl -n "$NAMESPACE" logs "pod/${pod}" \
    > "$EVIDENCE_DIR/kubernetes/sift-image-build-info.json" 2>/dev/null || true
  [[ "$phase" == "Succeeded" ]] \
    || die "candidate Sift image could not report its build provenance"
  jq -e --arg expected "$CANDIDATE_GIT_SHA" \
    '.git_sha == $expected and .next == "done"' \
    "$EVIDENCE_DIR/kubernetes/sift-image-build-info.json" >/dev/null \
    || die "candidate Sift image does not match the candidate Git revision"
  verify_pods_on_run_nodes "$NAMESPACE" \
    'sift-acceptance-probe=provenance' sift-candidate-provenance 1
  kubectl -n "$NAMESPACE" delete "pod/${pod}" --wait=true --timeout=60s >/dev/null
}

wait_for_store_convergence() {
  local expected="$1"
  local output_dir="$2"
  local require_snapshot="${3:-true}"
  local deadline=$((SECONDS + 900))
  local ordinal
  mkdir -p "$output_dir"
  while (( SECONDS < deadline )); do
    local complete=true
    for ordinal in 0 1 2; do
      store_integrity_to "$ordinal" "$output_dir/store-${ordinal}-integrity.json" \
        2>/dev/null || complete=false
      store_raftz "$ordinal" > "$output_dir/store-${ordinal}-raftz.json" \
        2>/dev/null || complete=false
    done
    if [[ "$complete" == true ]] \
      && jq -s -e --slurpfile expected "$expected" '
        length == 3
        and (.[0] as $first | all(.[];
          .event_count == $first.event_count
          and .event_id_sha256 == $first.event_id_sha256
          and .watermark == $first.watermark
          and .signals == $first.signals))
        and all(.[];
          .event_count == $expected[0].event_count
          and .event_id_sha256 == $expected[0].event_id_sha256
          and .watermark == $expected[0].watermark
          and .signals == $expected[0].signals)
      ' "$output_dir"/store-*-integrity.json >/dev/null \
      && jq -s -e --argjson require_snapshot "$require_snapshot" '
        length == 3
        and (map(.committed_voters | length == 3) | all)
        and (map(.durability_error == null) | all)
        and (map(.applied_index) | unique | length == 1)
        and (map(.commit_index) | unique | length == 1)
        and all(.[];
          .applied_index == .commit_index
          and .snapshot_index <= .applied_index
          and ((($require_snapshot | not) or .snapshot_index > 0))
          and .resident_log_bytes <= 536870912
          and .resident_log_bytes < .max_resident_log_bytes)
      ' "$output_dir"/store-*-raftz.json >/dev/null; then
      return 0
    fi
    sleep 5
  done
  die "the three store voters did not converge on integrity and bounded Raft state"
}

find_store_leader() {
  local ordinal
  for ordinal in 0 1 2; do
    if store_raftz "$ordinal" 2>/dev/null \
      | jq -e '.is_leader == true and .durability_error == null' >/dev/null 2>&1; then
      printf '%s\n' "$ordinal"
      return 0
    fi
  done
  return 1
}

wait_store_leader() {
  local deadline=$((SECONDS + 180))
  local leader
  while (( SECONDS < deadline )); do
    if leader="$(find_store_leader)" && [[ -n "$leader" ]]; then
      printf '%s\n' "$leader"
      return 0
    fi
    sleep 3
  done
  die "the three-store Raft group did not expose a durable leader"
}

verify_outage_quorum_recovery() {
  local stopped_leader="$1"
  local expected="$2"
  local manifest="$3"
  local manifest_hash="$4"
  local old_uid new_uid new_uid_deadline new_ready voter

  wait_for_store_convergence "$expected" \
    "$EVIDENCE_DIR/gcs/voters-after-outage-ack" false
  for voter in 0 1 2; do
    jq -e \
      --arg manifest "$manifest" \
      --arg hash "$manifest_hash" '
      .storage.archive.manifest_uri == $manifest
      and .storage.archive.manifest_sha256 == $hash
      and .storage.wal_bytes.logs > 0
      and .storage.archive.watermarks.logs < .signals.logs.watermark
    ' "$EVIDENCE_DIR/gcs/voters-after-outage-ack/store-${voter}-integrity.json" >/dev/null \
      || die "store voter ${voter} did not retain the acknowledged outage WAL"
  done

  old_uid="$(kubectl -n "$NAMESPACE" get "pod/sift-store-${stopped_leader}" \
    -o jsonpath='{.metadata.uid}')"
  stop_forwards
  kubectl -n "$NAMESPACE" delete "pod/sift-store-${stopped_leader}" \
    --wait=true --timeout=240s
  new_uid_deadline=$((SECONDS + 300))
  new_uid=""
  while (( SECONDS < new_uid_deadline )); do
    new_uid="$(kubectl -n "$NAMESPACE" get "pod/sift-store-${stopped_leader}" \
      -o jsonpath='{.metadata.uid}' 2>/dev/null || true)"
    new_ready="$(kubectl -n "$NAMESPACE" get "pod/sift-store-${stopped_leader}" \
      -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}' 2>/dev/null || true)"
    if [[ -n "$new_uid" && "$new_uid" != "$old_uid" && "$new_ready" == "True" ]]; then
      break
    fi
    sleep 3
  done
  [[ -n "$old_uid" && -n "$new_uid" && "$old_uid" != "$new_uid" ]] \
    || die "the GCS-outage leader process was not replaced"
  wait_role_ready statefulset sift-store 3

  start_gateway_forward
  start_store_forwards
  refresh_token
  wait_for_store_convergence "$expected" \
    "$EVIDENCE_DIR/gcs/voters-after-outage-leader-restart" false
  archive_leader="$(wait_store_leader)"
  for voter in 0 1 2; do
    jq -e \
      --arg manifest "$manifest" \
      --arg hash "$manifest_hash" \
      --slurpfile before "$EVIDENCE_DIR/gcs/voters-after-outage-ack/store-${voter}-integrity.json" '
      .storage.archive.manifest_uri == $manifest
      and .storage.archive.manifest_sha256 == $hash
      and .storage.wal_bytes.logs >= $before[0].storage.wal_bytes.logs
      and .storage.archive.watermarks.logs < .signals.logs.watermark
    ' "$EVIDENCE_DIR/gcs/voters-after-outage-leader-restart/store-${voter}-integrity.json" >/dev/null \
      || die "store voter ${voter} compacted or lost outage WAL after leader restart"
  done
  jq -n \
    --arg stopped_ordinal "$stopped_leader" \
    --arg old_uid "$old_uid" \
    --arg new_uid "$new_uid" \
    --arg leader_after "$archive_leader" \
    '{stopped_ordinal:$stopped_ordinal,old_uid:$old_uid,new_uid:$new_uid,leader_after:$leader_after,quorum_recovered:true}' \
    > "$EVIDENCE_DIR/gcs/outage-leader-restart.json"
}

echo ">> Sift MVP: validate topology and immutable candidate"
gcloud container node-pools describe "$SIFT_NODE_POOL" \
  --cluster="$GKE_CLUSTER_NAME" \
  --zone="$GKE_ZONE" \
  --project="$PROJECT_ID" \
  --format=json > "$EVIDENCE_DIR/kubernetes/sift-node-pool.json"
jq -e \
  --arg name "$SIFT_NODE_POOL" \
  --arg run_id "$RUN_ID" '
  .name == $name
  and .initialNodeCount == 3
  and .config.machineType == "e2-standard-4"
  and .config.labels["axiom-run-id"] == $run_id
  and .management.autoRepair == true
' "$EVIDENCE_DIR/kubernetes/sift-node-pool.json" >/dev/null \
  || die "run-scoped Sift node pool did not match the three-node MVP contract"
wait_role_ready statefulset sift-store 3
wait_role_ready statefulset sift-control 3
wait_role_ready deployment sift-gateway 1
wait_role_ready deployment sift-query 1
kubectl -n "$NAMESPACE" rollout status daemonset/sift-agent --timeout=900s

kubectl -n "$NAMESPACE" get deployment,statefulset,daemonset,pod,pvc,serviceaccount \
  -o json > "$EVIDENCE_DIR/kubernetes/sift-topology.json"
jq -e --arg image "$SIFT_IMAGE" '
  def workload($kind; $name):
    .items[] | select(.kind == $kind and .metadata.name == $name);
  ([.items[] | select(.kind == "PersistentVolumeClaim")
       | {key:.metadata.name, value:.spec.resources.requests.storage}] | from_entries) as $pvc
  | (workload("StatefulSet"; "sift-store").spec.replicas == 3)
  and (workload("StatefulSet"; "sift-control").spec.replicas == 3)
  and (workload("Deployment"; "sift-gateway").spec.replicas == 1)
  and (workload("Deployment"; "sift-query").spec.replicas == 1)
  and (workload("StatefulSet"; "sift-store").spec.template.spec.containers[0].image == $image)
  and (workload("StatefulSet"; "sift-control").spec.template.spec.containers[0].image == $image)
  and (workload("Deployment"; "sift-gateway").spec.template.spec.containers[0].image == $image)
  and (workload("Deployment"; "sift-query").spec.template.spec.containers[0].image == $image)
  and (workload("DaemonSet"; "sift-agent").spec.template.spec.containers[0].image == $image)
  and ($pvc["data-sift-store-0"] == "50Gi")
    and ($pvc["data-sift-control-0"] == "5Gi")
    and ($pvc["sift-gateway-data"] == "2Gi")
    and ($pvc["sift-query-data"] == "2Gi")
' "$EVIDENCE_DIR/kubernetes/sift-topology.json" >/dev/null \
  || die "Sift topology, PVC sizes, or immutable image did not match the MVP contract"
verify_pods_on_run_nodes "$NAMESPACE" \
  'app.kubernetes.io/name=sift' sift-topology 11
verify_sift_image_provenance

delegator="sift.${NAMESPACE}.sift.auth-delegator"
kubectl get clusterrolebinding "$delegator" -o json \
  > "$EVIDENCE_DIR/kubernetes/auth-delegator-binding.json"
jq -e --arg namespace "$NAMESPACE" --arg instance sift \
  -f "$SCRIPT_DIR/sift-auth-delegator.jq" \
  "$EVIDENCE_DIR/kubernetes/auth-delegator-binding.json" >/dev/null \
  || die "operator-managed auth-delegator binding does not match the exact runtime and store subjects"

kubectl -n "$NAMESPACE" get fqdnnetworkpolicy \
  -o json > "$EVIDENCE_DIR/kubernetes/fqdn-network-policies.json"
jq -e --arg namespace "$NAMESPACE" --arg instance sift \
  -f "$SCRIPT_DIR/sift-archive-fqdn-policy.jq" \
  "$EVIDENCE_DIR/kubernetes/fqdn-network-policies.json" >/dev/null \
  || die "GCS archive FQDN policy is absent, broad, or selects another Sift instance"

refresh_token
start_gateway_forward
start_store_forwards
initial_leader="$(wait_store_leader)"
for ordinal in 0 1 2; do
  store_raftz "$ordinal" \
    > "$EVIDENCE_DIR/kubernetes/store-${ordinal}-raftz-initial.json"
done
jq -s -e '
  length == 3
  and (map(.committed_voters | length == 3) | all)
  and (map(.durability_error == null) | all)
  and (map(select(.is_leader == true)) | length == 1)
' "$EVIDENCE_DIR"/kubernetes/store-*-raftz-initial.json >/dev/null \
  || die "Sift did not form one three-voter durable Raft group"
verify_peer_mtls_rejection

run_grpc_smoke() {
  local job="sift-grpc-${RUN_ID}"
  cat > "$EVIDENCE_DIR/kubernetes/${job}.yaml" <<EOF
apiVersion: batch/v1
kind: Job
metadata:
  name: ${job}
  namespace: ${NAMESPACE}
  labels:
    axiom-owner: gcp-operator-acceptance
    axiom-run-id: "${RUN_ID}"
spec:
  backoffLimit: 0
  activeDeadlineSeconds: 300
  template:
    metadata:
      labels:
        axiom-owner: gcp-operator-acceptance
        axiom-run-id: "${RUN_ID}"
    spec:
      serviceAccountName: sift-rig
      automountServiceAccountToken: false
      restartPolicy: Never
      nodeSelector:
        axiom-run-id: "${RUN_ID}"
      containers:
        - name: sift
          image: ${SIFT_IMAGE}
          args:
            - acceptance-grpc
            - --endpoint
            - http://sift.sift.svc.cluster.local:4317
            - --project
            - ${PROJECT}
            - --token-file
            - /var/run/secrets/sift/token
          volumeMounts:
            - name: token
              mountPath: /var/run/secrets/sift
              readOnly: true
      volumes:
        - name: token
          projected:
            sources:
              - serviceAccountToken:
                  audience: sift.axiom.dev
                  expirationSeconds: 600
                  path: token
EOF
  kubectl apply -f "$EVIDENCE_DIR/kubernetes/${job}.yaml"
  kubectl -n "$NAMESPACE" wait --for=condition=Complete "job/${job}" --timeout=300s \
    || die "OTLP/gRPC acceptance job did not complete"
  kubectl -n "$NAMESPACE" logs "job/${job}" \
    > "$EVIDENCE_DIR/kubernetes/sift-grpc-smoke.json"
  jq -e '
    .signal == "logs"
    and .accepted == 1
    and .rejected == 1
    and .compression == "gzip"
  ' "$EVIDENCE_DIR/kubernetes/sift-grpc-smoke.json" >/dev/null \
    || die "OTLP/gRPC gzip and partial-success contract failed"
  verify_pods_on_run_nodes "$NAMESPACE" "job-name=${job}" "${job}" 1
}

run_mcp_smoke() {
  local init_headers="$EVIDENCE_DIR/kubernetes/mcp-init.headers"
  local init_body="$EVIDENCE_DIR/kubernetes/mcp-init.json"
  local init_sse="$EVIDENCE_DIR/kubernetes/mcp-init.sse"
  local list_body="$EVIDENCE_DIR/kubernetes/mcp-tools.json"
  local list_sse="$EVIDENCE_DIR/kubernetes/mcp-tools.sse"
  local session
  local mcp_last_result=""
  local allowed_host="sift.sift.svc.cluster.local"
  local allowed_origin="http://sift.sift.svc.cluster.local:7380"
  local initialize='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"sift-gke-acceptance","version":"1"}}}'

  mcp_call() {
    local request_id="$1"
    local tool_name="$2"
    local arguments="$3"
    local stem="$EVIDENCE_DIR/kubernetes/mcp-call-${request_id}-${tool_name}"
    local payload
    payload="$(jq -nc \
      --argjson id "$request_id" \
      --arg name "$tool_name" \
      --argjson arguments "$arguments" \
      '{jsonrpc:"2.0",id:$id,method:"tools/call",params:{name:$name,arguments:$arguments}}')"
    curl --silent --show-error --fail-with-body \
      -X POST "${sift_url}/mcp" \
      -H "host: ${allowed_host}" \
      -H "origin: ${allowed_origin}" \
      -H "authorization: Bearer $(sed -n '1p' "$token_file")" \
      -H "mcp-session-id: ${session}" \
      -H 'mcp-protocol-version: 2025-11-25' \
      -H 'content-type: application/json' \
      -H 'accept: application/json, text/event-stream' \
      --data "$payload" > "${stem}.sse"
    extract_sse_json "${stem}.sse" "${stem}.json"
    jq -e --argjson id "$request_id" '
      .jsonrpc == "2.0"
      and .id == $id
      and (has("error") | not)
      and .result != null
      and ((.result.isError // false) == false)
      and ((.result.content // []) | length) > 0
    ' "${stem}.json" >/dev/null \
      || die "MCP tool ${tool_name} did not return a successful result"
    mcp_last_result="${stem}.result.json"
    jq -er '[.result.content[]? | select(.type == "text") | .text][0]' \
      "${stem}.json" > "$mcp_last_result" \
      || die "MCP tool ${tool_name} did not return JSON text content"
    jq -e 'type == "object"' "$mcp_last_result" >/dev/null \
      || die "MCP tool ${tool_name} returned text that was not a JSON object"
  }

  curl --silent --show-error --fail-with-body \
    -D "$init_headers" \
    -o "$init_sse" \
    -X POST "${sift_url}/mcp" \
    -H "host: ${allowed_host}" \
    -H "origin: ${allowed_origin}" \
    -H "authorization: Bearer $(sed -n '1p' "$token_file")" \
    -H 'content-type: application/json' \
    -H 'accept: application/json, text/event-stream' \
    --data "$initialize"
  extract_sse_json "$init_sse" "$init_body"
  jq -e '.jsonrpc == "2.0" and .id == 1 and .result != null' "$init_body" >/dev/null \
    || die "MCP initialize did not return a valid result"
  session="$(awk '
    tolower($1) == "mcp-session-id:" {
      gsub("\r", "", $2)
      print $2
      exit
    }
  ' "$init_headers")"
  [[ -n "$session" ]] || die "MCP initialize did not return a session ID"

  curl --silent --show-error --fail-with-body \
    -X POST "${sift_url}/mcp" \
    -H "host: ${allowed_host}" \
    -H "origin: ${allowed_origin}" \
    -H "authorization: Bearer $(sed -n '1p' "$token_file")" \
    -H "mcp-session-id: ${session}" \
    -H 'mcp-protocol-version: 2025-11-25' \
    -H 'content-type: application/json' \
    -H 'accept: application/json, text/event-stream' \
    --data '{"jsonrpc":"2.0","method":"notifications/initialized"}' >/dev/null

  curl --silent --show-error --fail-with-body \
    -X POST "${sift_url}/mcp" \
    -H "host: ${allowed_host}" \
    -H "origin: ${allowed_origin}" \
    -H "authorization: Bearer $(sed -n '1p' "$token_file")" \
    -H "mcp-session-id: ${session}" \
    -H 'mcp-protocol-version: 2025-11-25' \
    -H 'content-type: application/json' \
    -H 'accept: application/json, text/event-stream' \
    --data '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
    > "$list_sse"
  extract_sse_json "$list_sse" "$list_body"
  jq -e '
    [.result.tools[].name] | sort ==
    ["sift_correlate","sift_get_trace","sift_list_services","sift_query","sift_tail_logs"]
  ' "$list_body" >/dev/null || die "MCP did not expose exactly five read-only tools"

  mcp_call 3 sift_query \
    '{"request":{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"logs","filter":{"op":"eq","field":"event_id","value":"smoke-log"}},"limit":10,"mode":"sync"}}'
  jq -e '.partial == false and any(.data.records[]?; .event_id == "smoke-log")' \
    "$mcp_last_result" >/dev/null \
    || die "MCP sift_query did not return the known smoke log"
  mcp_call 4 sift_get_trace \
    '{"project":"sift-mvp","trace_id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}'
  jq -e '
    .trace_id == "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    and any(.spans[]?; .span_id == "bbbbbbbbbbbbbbbb")
  ' "$mcp_last_result" >/dev/null \
    || die "MCP sift_get_trace did not return the known smoke span"
  mcp_call 5 sift_correlate \
    '{"request":{"version":1,"project":"sift-mvp","environment":"gke","trace_id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","limit":10}}'
  jq -e '
    .partial == false
    and any(.logs[]?; .event_id == "smoke-log")
    and any(.traces[]?; .trace_id == "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
  ' "$mcp_last_result" >/dev/null \
    || die "MCP sift_correlate did not return known correlated signals"
  mcp_call 6 sift_list_services \
    '{"project":"sift-mvp","environment":"gke"}'
  jq -e '
    any(.services[]?;
      .name == "sift-acceptance"
      and (.signals | sort) == ["logs","metrics","traces"])
  ' "$mcp_last_result" >/dev/null \
    || die "MCP sift_list_services did not return the three-signal smoke service"
  mcp_call 7 sift_tail_logs \
    '{"request":{"version":1,"project":"sift-mvp","environment":"gke","filter":{"op":"eq","field":"event_id","value":"smoke-log"},"wait_ms":0,"limit":10}}'
  jq -e '.partial == false and any(.data.records[]?; .event_id == "smoke-log")' \
    "$mcp_last_result" >/dev/null \
    || die "MCP sift_tail_logs did not return the known smoke log"
  mcp_call 8 sift_query \
    '{"request":{"version":1,"project":"sift-mvp-alt","environment":"gke","signal":{"kind":"logs","filter":{"op":"eq","field":"event_id","value":"smoke-log"}},"limit":10,"mode":"sync"}}'
  jq -e '.partial == false and (.data.records | length) == 0' \
    "$mcp_last_result" >/dev/null \
    || die "MCP sift_query leaked the smoke log into another project"

  bad_origin_status="$(curl --silent --output /dev/null --write-out '%{http_code}' \
    -X POST "${sift_url}/mcp" \
    -H "host: ${allowed_host}" \
    -H 'origin: https://evil.example' \
    -H "authorization: Bearer $(sed -n '1p' "$token_file")" \
    -H 'content-type: application/json' \
    -H 'accept: application/json, text/event-stream' \
    --data "$initialize")"
  [[ "$bad_origin_status" == "403" ]] || die "MCP accepted an untrusted Origin"
  bad_host_status="$(curl --silent --output /dev/null --write-out '%{http_code}' \
    -X POST "${sift_url}/mcp" \
    -H 'host: evil.example' \
    -H "origin: ${allowed_origin}" \
    -H "authorization: Bearer $(sed -n '1p' "$token_file")" \
    -H 'content-type: application/json' \
    -H 'accept: application/json, text/event-stream' \
    --data "$initialize")"
  [[ "$bad_host_status" == "403" ]] || die "MCP accepted an untrusted Host"
}

echo ">> Sift MVP: protocol and public-surface smoke tests"
integrity_to "$EVIDENCE_DIR/kubernetes/integrity-before-smoke.json"
smoke_start="$(jq -r '.event_count' "$EVIDENCE_DIR/kubernetes/integrity-before-smoke.json")"
epoch_seconds="$(date -u +%s)"
timestamp_nanos="${epoch_seconds}000000000"
trace_id=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
span_id=bbbbbbbbbbbbbbbb

jq -nc \
  --arg ts "$timestamp_nanos" \
  --arg trace "$trace_id" \
  --arg span "$span_id" '
  {
    resourceLogs: [{
      resource: {attributes: [
        {key:"service.name",value:{stringValue:"sift-acceptance"}},
        {key:"deployment.environment.name",value:{stringValue:"gke"}}
      ]},
      scopeLogs: [{scope:{name:"sift.acceptance"},logRecords:[
        {
          timeUnixNano:$ts,
          severityText:"ERROR",
          traceId:$trace,
          spanId:$span,
          body:{stringValue:"smoke accepted"},
          attributes:[{key:"sift.event_id",value:{stringValue:"smoke-log"}}]
        },
        {timeUnixNano:$ts,body:null}
      ]}]
    }]
  }' > "$EVIDENCE_DIR/kubernetes/otlp-logs-partial.json"
auth_curl -X POST "${sift_url}/v1/logs" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/kubernetes/otlp-logs-partial.json" \
  > "$EVIDENCE_DIR/kubernetes/otlp-logs-partial-response.json"
jq -e '
  .partialSuccess.rejectedLogRecords == 1
  and (.partialSuccess.errorMessage | contains("body is required"))
' "$EVIDENCE_DIR/kubernetes/otlp-logs-partial-response.json" >/dev/null \
  || die "OTLP log partial-success was not explicit"

jq -nc --arg ts "$((timestamp_nanos + 10))" '
  {
    resourceMetrics:[{
      resource:{attributes:[
        {key:"service.name",value:{stringValue:"sift-acceptance"}},
        {key:"deployment.environment.name",value:{stringValue:"gke"}}
      ]},
      scopeMetrics:[{scope:{name:"sift.acceptance"},metrics:[{
        name:"sift.acceptance.gauge",
        unit:"1",
        gauge:{dataPoints:[{
          timeUnixNano:$ts,
          asDouble:42.5,
          attributes:[{key:"fixture",value:{stringValue:"smoke"}}],
          exemplars:[{
            timeUnixNano:$ts,
            asDouble:42.5,
            traceId:"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            spanId:"bbbbbbbbbbbbbbbb"
          }]
        }]}
      }]}]
    }]
  }' > "$EVIDENCE_DIR/kubernetes/otlp-metrics.json"
auth_curl -X POST "${sift_url}/v1/metrics" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/kubernetes/otlp-metrics.json" \
  > "$EVIDENCE_DIR/kubernetes/otlp-metrics-response.json"
jq -e 'type == "object" and (.partialSuccess == null)' \
  "$EVIDENCE_DIR/kubernetes/otlp-metrics-response.json" >/dev/null \
  || die "OTLP metrics returned unexpected partial success"

jq -nc \
  --arg ts "$((timestamp_nanos + 20))" \
  --arg end "$((timestamp_nanos + 1000020))" \
  --arg trace "$trace_id" \
  --arg span "$span_id" '
  {
    resourceSpans:[{
      resource:{attributes:[
        {key:"service.name",value:{stringValue:"sift-acceptance"}},
        {key:"deployment.environment.name",value:{stringValue:"gke"}}
      ]},
      scopeSpans:[{scope:{name:"sift.acceptance"},spans:[{
        traceId:$trace,
        spanId:$span,
        name:"GET /smoke",
        startTimeUnixNano:$ts,
        endTimeUnixNano:$end,
        status:{code:2},
        attributes:[{key:"http.route",value:{stringValue:"/smoke"}}]
      }]}]
    }]
  }' > "$EVIDENCE_DIR/kubernetes/otlp-traces.json"
auth_curl -X POST "${sift_url}/v1/traces" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/kubernetes/otlp-traces.json" \
  > "$EVIDENCE_DIR/kubernetes/otlp-traces-response.json"
jq -e 'type == "object" and (.partialSuccess == null)' \
  "$EVIDENCE_DIR/kubernetes/otlp-traces-response.json" >/dev/null \
  || die "OTLP traces returned unexpected partial success"

jq -nc --arg ts "$((timestamp_nanos + 30))" '
  {resourceLogs:[{
    resource:{attributes:[{key:"service.name",value:{stringValue:"sift-acceptance"}}]},
    scopeLogs:[{logRecords:[{
      timeUnixNano:$ts,
      body:{stringValue:"gzip accepted"},
      attributes:[{key:"sift.event_id",value:{stringValue:"smoke-gzip"}}]
    }]}]
  }]}' > "$EVIDENCE_DIR/kubernetes/otlp-gzip.json"
gzip -c "$EVIDENCE_DIR/kubernetes/otlp-gzip.json" \
  > "$EVIDENCE_DIR/kubernetes/otlp-gzip.json.gz"
auth_curl -X POST "${sift_url}/v1/logs" \
  -H 'content-type: application/json' \
  -H 'content-encoding: gzip' \
  --data-binary "@$EVIDENCE_DIR/kubernetes/otlp-gzip.json.gz" \
  > "$EVIDENCE_DIR/kubernetes/otlp-gzip-response.json"
jq -e 'type == "object" and (.partialSuccess == null)' \
  "$EVIDENCE_DIR/kubernetes/otlp-gzip-response.json" >/dev/null \
  || die "OTLP gzip returned unexpected partial success"

jq -nc --arg ts "$((timestamp_nanos + 31))" '
  {resourceLogs:[{
    resource:{attributes:[
      {key:"service.name",value:{stringValue:"sift-acceptance"}},
      {key:"deployment.environment.name",value:{stringValue:"gke"}}
    ]},
    scopeLogs:[{logRecords:[{
      timeUnixNano:$ts,
      body:{stringValue:"idempotency probe"},
      attributes:[{key:"sift.event_id",value:{stringValue:"smoke-idempotency-log"}}]
    }]}]
  }]}' > "$EVIDENCE_DIR/kubernetes/otlp-idempotency-logs.json"
auth_curl -X POST "${sift_url}/v1/logs" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/kubernetes/otlp-idempotency-logs.json" \
  > "$EVIDENCE_DIR/kubernetes/otlp-idempotency-logs-first-response.json"
jq -e 'type == "object" and (.partialSuccess == null)' \
  "$EVIDENCE_DIR/kubernetes/otlp-idempotency-logs-first-response.json" >/dev/null \
  || die "the first log idempotency probe was not accepted"

jq -nc --arg ts "$((timestamp_nanos + 32))" '
  {resourceMetrics:[{
    resource:{attributes:[
      {key:"service.name",value:{stringValue:"sift-acceptance"}},
      {key:"deployment.environment.name",value:{stringValue:"gke"}}
    ]},
    scopeMetrics:[{metrics:[{
      name:"sift.idempotency.gauge",
      gauge:{dataPoints:[{
        timeUnixNano:$ts,
        asDouble:1.0,
        attributes:[{key:"sift.event_id",value:{stringValue:"smoke-idempotency-metric"}}]
      }]}
    }]}]
  }]}' > "$EVIDENCE_DIR/kubernetes/otlp-idempotency-metrics.json"
auth_curl -X POST "${sift_url}/v1/metrics" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/kubernetes/otlp-idempotency-metrics.json" \
  > "$EVIDENCE_DIR/kubernetes/otlp-idempotency-metrics-first-response.json"
jq -e 'type == "object" and (.partialSuccess == null)' \
  "$EVIDENCE_DIR/kubernetes/otlp-idempotency-metrics-first-response.json" >/dev/null \
  || die "the first metric idempotency probe was not accepted"

jq -nc \
  --arg ts "$((timestamp_nanos + 33))" \
  --arg end "$((timestamp_nanos + 1000033))" '
  {resourceSpans:[{
    resource:{attributes:[
      {key:"service.name",value:{stringValue:"sift-acceptance"}},
      {key:"deployment.environment.name",value:{stringValue:"gke"}}
    ]},
    scopeSpans:[{spans:[{
      traceId:"33333333333333333333333333333333",
      spanId:"4444444444444444",
      name:"GET /idempotency",
      startTimeUnixNano:$ts,
      endTimeUnixNano:$end,
      status:{code:1},
      attributes:[{key:"sift.event_id",value:{stringValue:"smoke-idempotency-span"}}]
    }]}]
  }]}' > "$EVIDENCE_DIR/kubernetes/otlp-idempotency-traces.json"
auth_curl -X POST "${sift_url}/v1/traces" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/kubernetes/otlp-idempotency-traces.json" \
  > "$EVIDENCE_DIR/kubernetes/otlp-idempotency-traces-first-response.json"
jq -e 'type == "object" and (.partialSuccess == null)' \
  "$EVIDENCE_DIR/kubernetes/otlp-idempotency-traces-first-response.json" >/dev/null \
  || die "the first trace idempotency probe was not accepted"

"$SIFT_CLI" acceptance-payload \
  --kind otlp-logs-protobuf \
  --items 2 \
  --project "$PROJECT" \
  --event-prefix smoke-protobuf \
  --timestamp-unix-nano "$((timestamp_nanos + 40))" \
  > "$EVIDENCE_DIR/kubernetes/otlp-logs.pb"
auth_curl -X POST "${sift_url}/v1/logs" \
  -H 'content-type: application/x-protobuf' \
  --data-binary "@$EVIDENCE_DIR/kubernetes/otlp-logs.pb" \
  > "$EVIDENCE_DIR/kubernetes/otlp-logs-protobuf-response.pb"

run_grpc_smoke

"$SIFT_CLI" acceptance-payload \
  --kind prometheus-remote-write-v1 \
  --items 2 \
  --project "$PROJECT" \
  --event-prefix smoke-remote-write \
  --timestamp-unix-nano "$((timestamp_nanos + 50))" \
  > "$EVIDENCE_DIR/kubernetes/remote-write-v1.snappy"
remote_write_status="$(auth_curl --output /dev/null --write-out '%{http_code}' \
  -X POST "${sift_url}/prometheus/api/v1/write" \
  -H 'content-type: application/x-protobuf' \
  -H 'content-encoding: snappy' \
  -H 'x-prometheus-remote-write-version: 0.1.0' \
  --data-binary "@$EVIDENCE_DIR/kubernetes/remote-write-v1.snappy")"
[[ "$remote_write_status" == "204" ]] || die "Remote Write 1.0 did not return 204"

wait_for_integrity_count "$((smoke_start + 12))" \
  "$EVIDENCE_DIR/kubernetes/integrity-after-smoke.json"
verify_idempotency_retry immediate-retry
metric_before_v2="$(jq -r '.signals.metrics.count' \
  "$EVIDENCE_DIR/kubernetes/integrity-after-smoke.json")"
remote_write_v2_status="$(auth_curl_status --output "$EVIDENCE_DIR/kubernetes/remote-write-v2-response.json" \
  --write-out '%{http_code}' \
  -X POST "${sift_url}/prometheus/api/v1/write" \
  -H 'content-type: application/x-protobuf;proto=io.prometheus.write.v2.Request' \
  -H 'content-encoding: snappy' \
  -H 'x-prometheus-remote-write-version: 2.0.0' \
  --data-binary '')"
[[ "$remote_write_v2_status" == "415" ]] \
  || die "Remote Write 2.0 was not rejected with 415"
integrity_to "$EVIDENCE_DIR/kubernetes/integrity-after-remote-write-v2.json"
[[ "$(jq -r '.signals.metrics.count' \
  "$EVIDENCE_DIR/kubernetes/integrity-after-remote-write-v2.json")" == "$metric_before_v2" ]] \
  || die "Remote Write 2.0 rejection wrote metric data"

auth_curl -X POST "${sift_url}/api/v1/query" \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"logs","filter":{"op":"regex","field":"body_text","pattern":"smoke.*accepted"}},"limit":10,"mode":"sync"}' \
  > "$EVIDENCE_DIR/kubernetes/query-logs.json"
jq -e '
  (.data.records | map(select(.event_id == "smoke-log")) | length) == 1
  and .partial == false
  and (.warnings | length) == 0
  and (.watermark > 0)
  and (.stats.returned >= 1)
' "$EVIDENCE_DIR/kubernetes/query-logs.json" >/dev/null \
  || die "structured log regex query failed"

auth_curl -X POST "${sift_url}/api/v1/query" \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"logs","filter":{"op":"eq","field":"event_id","value":"smoke-idempotency-log"}},"limit":10,"mode":"sync"}' \
  > "$EVIDENCE_DIR/kubernetes/query-idempotency.json"
jq -e '(.data.records | map(select(.event_id == "smoke-idempotency-log")) | length) == 1' \
  "$EVIDENCE_DIR/kubernetes/query-idempotency.json" >/dev/null \
  || die "idempotency retry produced zero or multiple stored rows"

auth_curl -X POST "${sift_url}/api/v1/query" \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"metrics","name":"sift.acceptance.gauge","function":"sum","step_seconds":1,"group_by":["service.name"]},"limit":10,"mode":"sync"}' \
  > "$EVIDENCE_DIR/kubernetes/query-metrics.json"
jq -e '(.data.series | length) == 1 and .data.series[0].aggregate == 42.5' \
  "$EVIDENCE_DIR/kubernetes/query-metrics.json" >/dev/null \
  || die "metric aggregate query failed"

auth_curl -X POST "${sift_url}/api/v1/query" \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"traces","service":"sift-acceptance","operation":"GET /smoke","min_duration_ms":1},"limit":10,"mode":"sync"}' \
  > "$EVIDENCE_DIR/kubernetes/query-traces.json"
jq -e --arg trace "$trace_id" \
  '(.data.traces | map(select(.trace_id == $trace)) | length) == 1' \
  "$EVIDENCE_DIR/kubernetes/query-traces.json" >/dev/null \
  || die "trace search failed"

auth_curl "${sift_url}/api/v1/traces/${trace_id}?project=${PROJECT}" \
  > "$EVIDENCE_DIR/kubernetes/trace-read.json"
jq -e --arg trace "$trace_id" '
  .trace_id == $trace
  and (.spans | length) == 1
  and (.critical_path_span_ids | length) == 1
' "$EVIDENCE_DIR/kubernetes/trace-read.json" >/dev/null \
  || die "trace read or critical path failed"

auth_curl -X POST "${sift_url}/api/v1/correlate" \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"sift-mvp","environment":"gke","trace_id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","limit":10}' \
  > "$EVIDENCE_DIR/kubernetes/correlate.json"
jq -e '
  (.logs | map(select(.event_id == "smoke-log")) | length) == 1
  and (.traces | length) == 1
  and .partial == false
' "$EVIDENCE_DIR/kubernetes/correlate.json" >/dev/null \
  || die "cross-signal correlation failed"

auth_curl "${sift_url}/api/v1/services?project=${PROJECT}&environment=gke" \
  > "$EVIDENCE_DIR/kubernetes/services.json"
jq -e '
  [.services[] | select(.name == "sift-acceptance")][0] as $service
  | $service != null
    and ($service.signals | index("logs") != null)
    and ($service.signals | index("metrics") != null)
    and ($service.signals | index("traces") != null)
' "$EVIDENCE_DIR/kubernetes/services.json" >/dev/null \
  || die "service inventory did not correlate all three signals"

  auth_curl --get "${sift_url}/prometheus/api/v1/query" \
    --data-urlencode "project=${PROJECT}" \
    --data-urlencode 'environment=acceptance' \
    --data-urlencode 'query=sift_acceptance_total{fixture="smoke-remote-write"}' \
    --data-urlencode "time=$((epoch_seconds + 1))" \
    > "$EVIDENCE_DIR/kubernetes/prometheus-query.json"
  jq -e '
    .status == "success"
    and .data.resultType == "vector"
    and (.data.result | length) == 1
    and .data.result[0].metric.__name__ == "sift_acceptance_total"
    and .data.result[0].metric.fixture == "smoke-remote-write"
    and .data.result[0].value[1] == "1"
  ' \
    "$EVIDENCE_DIR/kubernetes/prometheus-query.json" >/dev/null \
    || die "Prometheus query did not return the newly written series and value"

  auth_curl --get "${sift_url}/prometheus/api/v1/query_range" \
    --data-urlencode "project=${PROJECT}" \
    --data-urlencode 'environment=acceptance' \
    --data-urlencode 'query=sift_acceptance_total{fixture="smoke-remote-write"}' \
    --data-urlencode "start=$((epoch_seconds - 1))" \
    --data-urlencode "end=$((epoch_seconds + 2))" \
    --data-urlencode 'step=1' \
    > "$EVIDENCE_DIR/kubernetes/prometheus-query-range.json"
  jq -e --argjson epoch "$epoch_seconds" \
    -f "$SCRIPT_DIR/sift-prometheus-range-smoke.jq" \
    "$EVIDENCE_DIR/kubernetes/prometheus-query-range.json" >/dev/null \
    || die "Prometheus query_range did not evaluate and carry the series at every step"

cross_project_status="$(curl --silent --output "$EVIDENCE_DIR/kubernetes/cross-project.json" \
  --write-out '%{http_code}' \
  -X POST "${sift_url}/api/v1/query" \
  -H "authorization: Bearer $(sed -n '1p' "$token_file")" \
  -H 'x-sift-project: denied-project' \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"denied-project","signal":{"kind":"logs"},"limit":1,"mode":"sync"}')"
  [[ "$cross_project_status" == "403" ]] || die "cross-project access was not denied"

  allowed_header_denied_body_status="$(project_curl_status "$PROJECT" \
    --output "$EVIDENCE_DIR/kubernetes/cross-project-allowed-header-denied-body.json" \
    --write-out '%{http_code}' \
    -X POST "${sift_url}/api/v1/query" \
    -H 'content-type: application/json' \
    --data '{"version":1,"project":"denied-project","signal":{"kind":"logs"},"limit":1,"mode":"sync"}')"
  [[ "$allowed_header_denied_body_status" == "403" ]] \
    || die "allowed project header could authorize a different body project"
  denied_header_allowed_body_status="$(project_curl_status denied-project \
    --output "$EVIDENCE_DIR/kubernetes/cross-project-denied-header-allowed-body.json" \
    --write-out '%{http_code}' \
    -X POST "${sift_url}/api/v1/query" \
    -H 'content-type: application/json' \
    --data '{"version":1,"project":"sift-mvp","signal":{"kind":"logs"},"limit":1,"mode":"sync"}')"
  [[ "$denied_header_allowed_body_status" == "403" ]] \
    || die "denied project header could query an allowed body project"

jq -nc --arg ts "$((timestamp_nanos + 60))" '
  {resourceLogs:[{
    resource:{attributes:[
      {key:"service.name",value:{stringValue:"sift-project-isolation"}},
      {key:"deployment.environment.name",value:{stringValue:"gke"}}
    ]},
    scopeLogs:[{logRecords:[{
      timeUnixNano:$ts,
      body:{stringValue:"same ID in two authorized projects"},
      attributes:[{key:"sift.event_id",value:{stringValue:"cross-project-same-id"}}]
    }]}]
  }]}' > "$EVIDENCE_DIR/kubernetes/cross-project-same-id.json"
for allowed_project in "$PROJECT" "$PROJECT_ALT"; do
  project_curl "$allowed_project" -X POST "${sift_url}/v1/logs" \
    -H 'content-type: application/json' \
    --data-binary "@$EVIDENCE_DIR/kubernetes/cross-project-same-id.json" \
    > "$EVIDENCE_DIR/kubernetes/cross-project-${allowed_project}-ingest.json"
  jq -e 'type == "object" and (.partialSuccess == null)' \
    "$EVIDENCE_DIR/kubernetes/cross-project-${allowed_project}-ingest.json" >/dev/null \
    || die "same-ID ingest failed for authorized project ${allowed_project}"
  project_curl "$allowed_project" -X POST "${sift_url}/api/v1/query" \
    -H 'content-type: application/json' \
    --data "{\"version\":1,\"project\":\"${allowed_project}\",\"environment\":\"gke\",\"signal\":{\"kind\":\"logs\",\"filter\":{\"op\":\"eq\",\"field\":\"event_id\",\"value\":\"cross-project-same-id\"}},\"limit\":10,\"mode\":\"sync\"}" \
    > "$EVIDENCE_DIR/kubernetes/cross-project-${allowed_project}-query.json"
  jq -e '(.data.records | map(select(.event_id == "cross-project-same-id")) | length) == 1' \
    "$EVIDENCE_DIR/kubernetes/cross-project-${allowed_project}-query.json" >/dev/null \
    || die "same event ID was not retained independently for ${allowed_project}"
done

run_mcp_smoke

make_load_scenario() {
  local phase="$1"
  local signal="$2"
  local qps="$3"
  local duration="$4"
  local output="$5"
  local body path case_name base_seconds trace_prefix span_prefix
  case_name="${phase}_${signal}"
  base_seconds="$(date -u +%s)"
  case "$phase" in
    steady)
      trace_prefix=57ead0000000
      span_prefix=57ea
      ;;
    failover)
      trace_prefix=fa1100000000
      span_prefix=fa11
      ;;
    *)
      die "unknown load phase: $phase"
      ;;
  esac
  case "$signal" in
    logs)
      path=v1/logs
      body="$(jq -nc \
        --arg base "$base_seconds" \
        --arg seq '{{rig.sequence_016x}}' \
        --arg seq6 '{{rig.sequence_06}}' \
        --arg phase "$phase" '
        def pad3: tostring as $s | ("000" + $s)[-3:];
        {
          resourceLogs:[{
            resource:{attributes:[
              {key:"service.name",value:{stringValue:"sift-load"}},
              {key:"deployment.environment.name",value:{stringValue:"gke"}},
              {key:"service.version",value:{stringValue:"mvp"}}
            ]},
            scopeLogs:[{scope:{name:"sift.load"},logRecords:[
              range(0;1000) as $i | {
                timeUnixNano:($base + $seq6 + ($i|pad3)),
                severityText:"INFO",
                body:{stringValue:("sift " + $phase + " load log")},
                attributes:[
                  {key:"sift.event_id",value:{stringValue:($phase + "-log-" + $seq + "-" + ($i|tostring))}},
                  {key:"load.phase",value:{stringValue:$phase}}
                ]
              }
            ]}]
          }]
        }')"
      ;;
    metrics)
      path=v1/metrics
      body="$(jq -nc \
        --arg base "$base_seconds" \
        --arg seq '{{rig.sequence_016x}}' \
        --arg seq6 '{{rig.sequence_06}}' \
        --arg phase "$phase" '
        def pad3: tostring as $s | ("000" + $s)[-3:];
        {
          resourceMetrics:[{
            resource:{attributes:[
              {key:"service.name",value:{stringValue:"sift-load"}},
              {key:"deployment.environment.name",value:{stringValue:"gke"}},
              {key:"service.version",value:{stringValue:"mvp"}}
            ]},
            scopeMetrics:[{scope:{name:"sift.load"},metrics:[{
              name:"sift.load.counter",
              unit:"1",
              sum:{
                aggregationTemporality:2,
                isMonotonic:true,
                dataPoints:[
                  range(0;1000) as $i | {
                    startTimeUnixNano:($base + "000000000"),
                    timeUnixNano:($base + $seq6 + ($i|pad3)),
                    asDouble:(($i + 1) * 1.0),
                    attributes:[
                      {key:"sift.event_id",value:{stringValue:($phase + "-metric-" + $seq + "-" + ($i|tostring))}},
                      {key:"load.phase",value:{stringValue:$phase}},
                      {key:"load.request",value:{stringValue:$seq}}
                    ]
                  }
                ]
              }
            }]}]
          }]
        }')"
      ;;
    traces)
      path=v1/traces
      body="$(jq -nc \
        --arg base "$base_seconds" \
        --arg seq '{{rig.sequence_016x}}' \
        --arg seq6 '{{rig.sequence_06}}' \
        --arg phase "$phase" \
        --arg trace_prefix "$trace_prefix" \
        --arg span_prefix "$span_prefix" '
        def pad3: tostring as $s | ("000" + $s)[-3:];
        def pad4: tostring as $s | ("0000" + $s)[-4:];
        {
          resourceSpans:[{
            resource:{attributes:[
              {key:"service.name",value:{stringValue:"sift-load"}},
              {key:"deployment.environment.name",value:{stringValue:"gke"}},
              {key:"service.version",value:{stringValue:"mvp"}}
            ]},
            scopeSpans:[{scope:{name:"sift.load"},spans:[
              range(0;1000) as $i | {
                traceId:($trace_prefix + $seq + ($i|pad4)),
                spanId:($span_prefix + $seq[-8:] + ($i|pad4)),
                name:"POST /load",
                startTimeUnixNano:($base + $seq6 + ($i|pad3)),
                endTimeUnixNano:($base + $seq6 + ($i|pad3)),
                status:{code:1},
                attributes:[
                  {key:"sift.event_id",value:{stringValue:($phase + "-span-" + $seq + "-" + ($i|tostring))}},
                  {key:"load.phase",value:{stringValue:$phase}}
                ]
              }
            ]}]
          }]
        }')"
      ;;
    *)
      die "unknown load signal: $signal"
      ;;
  esac

  cat > "$output" <<EOF
[record]
suite = "sift"
dimension = "load"
case = "${case_name}"
subject = "${phase} ${signal} sends ${BATCH_ITEMS} unique telemetry items per acknowledged request"
kind = "load"
expected = "pass"
required = true

[load]
target_qps = ${qps}
workers = 8
duration_secs = ${duration}
warmup_secs = 0

[load.request]
method = "POST"
url = "http://sift.sift.svc.cluster.local:7380/${path}"
bearer_token_file = "/var/run/secrets/sift/token"
body = '''${body}'''

[load.request.headers]
x-sift-project = "${PROJECT}"

[load.request.expect]
status = 200
timeout_ms = 5000

[load.request.expect.jsonpath]
"$.partialSuccess" = "absent"
EOF
}

create_load_job() {
  local phase="$1"
  local signal="$2"
  local qps="$3"
  local duration="$4"
  local case_name="${phase}_${signal}"
  local config_name="sift-${phase}-${signal}"
  local job_name="sift-${phase}-${signal}"
  local scenario_dir="$EVIDENCE_DIR/load/${phase}/${signal}/load"
  local scenario_file="${scenario_dir}/${case_name}.toml"
  mkdir -p "$scenario_dir"
  make_load_scenario "$phase" "$signal" "$qps" "$duration" "$scenario_file"
  kubectl -n "$NAMESPACE" create configmap "$config_name" \
    --from-file="$scenario_file" \
    --dry-run=client -o yaml \
    > "$EVIDENCE_DIR/load/${phase}/${signal}/configmap.yaml"
  kubectl create -f "$EVIDENCE_DIR/load/${phase}/${signal}/configmap.yaml"
  cat > "$EVIDENCE_DIR/load/${phase}/${signal}/job.yaml" <<EOF
apiVersion: batch/v1
kind: Job
metadata:
  name: ${job_name}
  namespace: ${NAMESPACE}
  labels:
    axiom-owner: gcp-operator-acceptance
    axiom-run-id: "${RUN_ID}"
    sift-load-phase: ${phase}
spec:
  suspend: true
  backoffLimit: 0
  activeDeadlineSeconds: $((duration + 900))
  template:
    metadata:
      labels:
        axiom-owner: gcp-operator-acceptance
        axiom-run-id: "${RUN_ID}"
        sift-load-phase: ${phase}
    spec:
      serviceAccountName: sift-rig
      automountServiceAccountToken: false
      restartPolicy: Never
      nodeSelector:
        axiom-run-id: "${RUN_ID}"
      securityContext:
        runAsNonRoot: true
        runAsUser: 65532
        runAsGroup: 65532
        seccompProfile:
          type: RuntimeDefault
      containers:
        - name: rig
          image: ${RIG_IMAGE}
          args:
            - run
            - --scenario
            - /scenarios/load/${case_name}.toml
          resources:
            requests:
              cpu: 250m
              memory: 128Mi
            limits:
              cpu: "2"
              memory: 1Gi
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: true
            capabilities:
              drop: ["ALL"]
          volumeMounts:
            - name: scenario
              mountPath: /scenarios/load
              readOnly: true
            - name: token
              mountPath: /var/run/secrets/sift
              readOnly: true
      volumes:
        - name: scenario
          configMap:
            name: ${config_name}
        - name: token
          projected:
            sources:
              - serviceAccountToken:
                  audience: sift.axiom.dev
                  expirationSeconds: 600
                  path: token
EOF
  kubectl apply -f "$EVIDENCE_DIR/load/${phase}/${signal}/job.yaml"
}

start_load_phase() {
  local phase="$1"
  local duration="$2"
  create_load_job "$phase" logs 5 "$duration"
  create_load_job "$phase" metrics 3 "$duration"
  create_load_job "$phase" traces 2 "$duration"
  kubectl -n "$NAMESPACE" patch job "sift-${phase}-logs" \
    --type=merge --patch '{"spec":{"suspend":false}}'
  kubectl -n "$NAMESPACE" patch job "sift-${phase}-metrics" \
    --type=merge --patch '{"spec":{"suspend":false}}'
  kubectl -n "$NAMESPACE" patch job "sift-${phase}-traces" \
    --type=merge --patch '{"spec":{"suspend":false}}'
  date -u +%Y-%m-%dT%H:%M:%SZ > "$EVIDENCE_DIR/load/${phase}/started-at.txt"
}

validate_load_report() {
  local signal="$1"
  local qps="$2"
  local duration="$3"
  local report="$4"
  local expected_requests=$((qps * duration))
  jq -e \
    --argjson expected "$expected_requests" \
    --argjson floor "$(awk -v qps="$qps" 'BEGIN {print qps * 0.95}')" '
    .schema_version == "rig.report/1"
    and .clean == true
    and .exit_code == 0
    and .scenarios.pass == 1
    and ([.findings[] | select(.kind == "load_observation")] | length) == 1
    and ([.findings[] | select(.kind == "load_observation")][0].evidence) as $load
    | ($load.total == $expected)
      and ($load.failed == 0)
      and ($load.error_rate == 0)
      and ($load.achieved_qps >= $floor)
      and ($load.p95_ms <= 250)
      and ($load.p99_ms <= 1000)
  ' "$report" >/dev/null || die "$signal load did not meet count, rate, error, or latency gates"
}

wait_load_phase() {
  local phase="$1"
  local duration="$2"
  local timeout="$((duration + 900))s"
  local signal qps report
  for signal in logs metrics traces; do
    kubectl -n "$NAMESPACE" wait --for=condition=Complete \
      "job/sift-${phase}-${signal}" --timeout="$timeout" \
      || die "${phase} ${signal} load job did not complete"
    report="$EVIDENCE_DIR/load/${phase}/${signal}/report.json"
    kubectl -n "$NAMESPACE" logs "job/sift-${phase}-${signal}" > "$report"
    case "$signal" in
      logs) qps=5 ;;
      metrics) qps=3 ;;
      traces) qps=2 ;;
    esac
    validate_load_report "$signal" "$qps" "$duration" "$report"
  done
  jq -s '{
    requests: ([.[].findings[] | select(.kind == "load_observation") | .evidence.total] | add),
    failed: ([.[].findings[] | select(.kind == "load_observation") | .evidence.failed] | add),
    achieved_items_per_second:
      (([.[].findings[] | select(.kind == "load_observation") | .evidence.achieved_qps] | add) * 1000),
    p95_ms: ([.[].findings[] | select(.kind == "load_observation") | .evidence.p95_ms] | max),
    p99_ms: ([.[].findings[] | select(.kind == "load_observation") | .evidence.p99_ms] | max)
  }' \
    "$EVIDENCE_DIR/load/${phase}/logs/report.json" \
    "$EVIDENCE_DIR/load/${phase}/metrics/report.json" \
    "$EVIDENCE_DIR/load/${phase}/traces/report.json" \
    > "$EVIDENCE_DIR/load/${phase}/summary.json"
  verify_pods_on_run_nodes "$NAMESPACE" \
    "sift-load-phase=${phase}" "sift-load-${phase}" 3
}

echo ">> Sift MVP: persistent async query job and PVC restart"
auth_curl -X POST "${sift_url}/api/v1/query" \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"logs","filter":{"op":"eq","field":"event_id","value":"smoke-log"}},"limit":10,"mode":"async"}' \
  > "$EVIDENCE_DIR/kubernetes/query-job-create.json"
query_id="$(jq -r '.query_id // empty' "$EVIDENCE_DIR/kubernetes/query-job-create.json")"
[[ -n "$query_id" ]] || die "async query did not return a query ID"
query_pod="$(kubectl -n "$NAMESPACE" get pod \
  -l sift.axiom.dev/role=query -o jsonpath='{.items[0].metadata.name}')"
kubectl -n "$NAMESPACE" delete "pod/${query_pod}" --wait=true --timeout=180s
wait_role_ready deployment sift-query 1
query_deadline=$((SECONDS + 180))
while (( SECONDS < query_deadline )); do
  if auth_curl "${sift_url}/api/v1/queries/${query_id}?project=${PROJECT}" \
      > "$EVIDENCE_DIR/kubernetes/query-job-after-restart.json" 2>/dev/null \
    && jq -e '.status == "succeeded" or .status == "failed"' \
      "$EVIDENCE_DIR/kubernetes/query-job-after-restart.json" >/dev/null 2>&1; then
    break
  fi
  sleep 2
done
jq -e '.status == "succeeded" or .status == "failed"' \
  "$EVIDENCE_DIR/kubernetes/query-job-after-restart.json" >/dev/null \
  || die "query job disappeared or remained ambiguous after query-role restart"

kubectl -n "$NAMESPACE" get pvc -o json \
  | jq '[.items[] | {name:.metadata.name, uid:.metadata.uid}] | sort_by(.name)' \
  > "$EVIDENCE_DIR/kubernetes/pvc-before-restart.json"
integrity_to "$EVIDENCE_DIR/kubernetes/integrity-before-pvc-restart.json"
restart_gateway="$(kubectl -n "$NAMESPACE" get pod \
  -l sift.axiom.dev/role=gateway -o jsonpath='{.items[0].metadata.name}')"
restart_query="$(kubectl -n "$NAMESPACE" get pod \
  -l sift.axiom.dev/role=query -o jsonpath='{.items[0].metadata.name}')"
stop_forwards
kubectl -n "$NAMESPACE" delete pod/sift-store-0 \
  "pod/${restart_gateway}" "pod/${restart_query}" --wait=true --timeout=240s
wait_role_ready statefulset sift-store 3
wait_role_ready deployment sift-gateway 1
wait_role_ready deployment sift-query 1
start_gateway_forward
start_store_forwards
refresh_token
integrity_to "$EVIDENCE_DIR/kubernetes/integrity-after-pvc-restart.json"
jq -e --slurpfile before "$EVIDENCE_DIR/kubernetes/integrity-before-pvc-restart.json" '
  .event_count == $before[0].event_count
  and .event_id_sha256 == $before[0].event_id_sha256
  and .watermark == $before[0].watermark
' "$EVIDENCE_DIR/kubernetes/integrity-after-pvc-restart.json" >/dev/null \
  || die "PVC restart changed acknowledged project data"
kubectl -n "$NAMESPACE" get pvc -o json \
  | jq '[.items[] | {name:.metadata.name, uid:.metadata.uid}] | sort_by(.name)' \
  > "$EVIDENCE_DIR/kubernetes/pvc-after-restart.json"
jq -e --slurpfile before "$EVIDENCE_DIR/kubernetes/pvc-before-restart.json" \
  '. == $before[0]' "$EVIDENCE_DIR/kubernetes/pvc-after-restart.json" >/dev/null \
  || die "PVC restart replaced a persistent volume claim"

echo ">> Sift MVP: 30-minute, 10,000-items-per-second steady load"
snapshot_restarts "$EVIDENCE_DIR/kubernetes/restarts-before-steady.json"
integrity_to "$EVIDENCE_DIR/load/steady/integrity-before.json"
start_load_phase steady "$LOAD_SECONDS"
wait_load_phase steady "$LOAD_SECONDS"
wait_for_delta "$EVIDENCE_DIR/load/steady/integrity-before.json" "$EXPECTED_ITEMS" \
  "$EVIDENCE_DIR/load/steady/integrity-after.json"
jq -e \
  --slurpfile before "$EVIDENCE_DIR/load/steady/integrity-before.json" \
  --argjson total "$EXPECTED_ITEMS" \
  --argjson logs "$LOG_ITEMS" \
  --argjson metrics "$METRIC_ITEMS" \
  --argjson traces "$SPAN_ITEMS" '
  (.event_count - $before[0].event_count) == $total
  and (.signals.logs.count - $before[0].signals.logs.count) == $logs
  and (.signals.metrics.count - $before[0].signals.metrics.count) == $metrics
  and (.signals.traces.count - $before[0].signals.traces.count) == $traces
' "$EVIDENCE_DIR/load/steady/integrity-after.json" >/dev/null \
  || die "steady load did not retain the exact 50/30/20 signal mix"
verify_load_digest steady "$LOAD_SECONDS" \
  "$EVIDENCE_DIR/load/steady/integrity-before.json" \
  "$EVIDENCE_DIR/load/steady/integrity-after.json"
jq -e --argjson expected "$EXPECTED_ITEMS" --argjson rate "$ITEMS_PER_SECOND" '
  .failed == 0
  and (.requests * 1000) == $expected
  and .achieved_items_per_second >= ($rate * 0.95)
  and .p95_ms <= 250
  and .p99_ms <= 1000
' "$EVIDENCE_DIR/load/steady/summary.json" >/dev/null \
  || die "steady load summary missed throughput or acknowledgement gates"
snapshot_restarts "$EVIDENCE_DIR/kubernetes/restarts-after-steady.json"
assert_no_unexpected_restarts \
  "$EVIDENCE_DIR/kubernetes/restarts-before-steady.json" \
  "$EVIDENCE_DIR/kubernetes/restarts-after-steady.json"
refresh_token
verify_idempotency_retry after-steady-load

echo ">> Sift MVP: recent query, trace read, and log-tail latency"
latency_epoch="$(date -u +%s)"
latency_nanos="${latency_epoch}000000000"
latency_trace=dddddddddddddddddddddddddddddddd
latency_span=eeeeeeeeeeeeeeee
jq -nc --arg ts "$latency_nanos" '
  {resourceLogs:[{
    resource:{attributes:[
      {key:"service.name",value:{stringValue:"sift-latency"}},
      {key:"deployment.environment.name",value:{stringValue:"gke"}}
    ]},
    scopeLogs:[{logRecords:[{
      timeUnixNano:$ts,
      body:{stringValue:"recent latency probe"},
      attributes:[{key:"sift.event_id",value:{stringValue:"latency-log"}}]
    }]}]
  }]}' > "$EVIDENCE_DIR/latency/log.json"
auth_curl -X POST "${sift_url}/v1/logs" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/latency/log.json" \
  > "$EVIDENCE_DIR/latency/log-ingest-response.json"
jq -e 'type == "object" and (.partialSuccess == null)' \
  "$EVIDENCE_DIR/latency/log-ingest-response.json" >/dev/null \
  || die "latency log was not fully accepted"
jq -nc \
  --arg ts "$((latency_nanos + 10))" \
  --arg end "$((latency_nanos + 2000010))" \
  --arg trace "$latency_trace" \
  --arg span "$latency_span" '
  {resourceSpans:[{
    resource:{attributes:[
      {key:"service.name",value:{stringValue:"sift-latency"}},
      {key:"deployment.environment.name",value:{stringValue:"gke"}}
    ]},
    scopeSpans:[{spans:[{
      traceId:$trace,
      spanId:$span,
      name:"GET /latency",
      startTimeUnixNano:$ts,
      endTimeUnixNano:$end,
      status:{code:1}
    }]}]
  }]}' > "$EVIDENCE_DIR/latency/trace.json"
auth_curl -X POST "${sift_url}/v1/traces" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/latency/trace.json" \
  > "$EVIDENCE_DIR/latency/trace-ingest-response.json"
jq -e 'type == "object" and (.partialSuccess == null)' \
  "$EVIDENCE_DIR/latency/trace-ingest-response.json" >/dev/null \
  || die "latency trace was not fully accepted"

query_payload='{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"logs","filter":{"op":"eq","field":"event_id","value":"latency-log"}},"limit":10,"mode":"sync"}'
: > "$EVIDENCE_DIR/latency/query-seconds.txt"
: > "$EVIDENCE_DIR/latency/trace-seconds.txt"
for sample in $(seq 1 40); do
  query_seconds="$(auth_curl \
    --output "$EVIDENCE_DIR/latency/query-response-${sample}.json" \
    --write-out '%{time_total}' \
    -X POST "${sift_url}/api/v1/query" \
    -H 'content-type: application/json' \
    --data "$query_payload")"
  jq -e '
    (.data.records | map(select(.event_id == "latency-log")) | length) == 1
    and .partial == false
  ' "$EVIDENCE_DIR/latency/query-response-${sample}.json" >/dev/null \
    || die "recent query latency sample ${sample} returned the wrong body"
  printf '%s\n' "$query_seconds" >> "$EVIDENCE_DIR/latency/query-seconds.txt"

  trace_seconds="$(auth_curl \
    --output "$EVIDENCE_DIR/latency/trace-response-${sample}.json" \
    --write-out '%{time_total}' \
    "${sift_url}/api/v1/traces/${latency_trace}?project=${PROJECT}")"
  jq -e --arg trace "$latency_trace" '
    .trace_id == $trace
    and (.spans | length) == 1
    and .partial == false
  ' "$EVIDENCE_DIR/latency/trace-response-${sample}.json" >/dev/null \
    || die "trace-read latency sample ${sample} returned the wrong body"
  printf '%s\n' "$trace_seconds" >> "$EVIDENCE_DIR/latency/trace-seconds.txt"
done
query_p95_ms="$(percentile_ms "$EVIDENCE_DIR/latency/query-seconds.txt" 0.95)"
trace_p95_ms="$(percentile_ms "$EVIDENCE_DIR/latency/trace-seconds.txt" 0.95)"
number_le "$query_p95_ms" 2000 || die "recent query p95 exceeded 2 seconds"
number_le "$trace_p95_ms" 1000 || die "trace read p95 exceeded 1 second"

tail_nanos="$((latency_nanos + 3000000))"
jq -nc --arg ts "$tail_nanos" '
  {resourceLogs:[{
    resource:{attributes:[
      {key:"service.name",value:{stringValue:"sift-latency"}},
      {key:"deployment.environment.name",value:{stringValue:"gke"}}
    ]},
    scopeLogs:[{logRecords:[{
      timeUnixNano:$ts,
      body:{stringValue:"tail latency probe"},
      attributes:[{key:"sift.event_id",value:{stringValue:"latency-tail"}}]
    }]}]
  }]}' > "$EVIDENCE_DIR/latency/tail-log.json"
tail_ingest_seconds="$(auth_curl \
  --output "$EVIDENCE_DIR/latency/tail-ingest-response.json" \
  --write-out '%{time_total}' \
  -X POST "${sift_url}/v1/logs" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/latency/tail-log.json")"
jq -e 'type == "object" and (.partialSuccess == null)' \
  "$EVIDENCE_DIR/latency/tail-ingest-response.json" >/dev/null \
  || die "tail latency log was not fully accepted"
tail_query_seconds="$(auth_curl --output "$EVIDENCE_DIR/latency/tail-response.json" \
  --write-out '%{time_total}' \
  -X POST "${sift_url}/api/v1/logs/tail" \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"sift-mvp","environment":"gke","filter":{"op":"eq","field":"event_id","value":"latency-tail"},"wait_ms":2000,"limit":10}')"
jq -e '(.data.records | map(select(.event_id == "latency-tail")) | length) == 1' \
  "$EVIDENCE_DIR/latency/tail-response.json" >/dev/null \
  || die "log tail did not observe the new record"
tail_visible_ms="$(awk -v ingest="$tail_ingest_seconds" -v tail="$tail_query_seconds" \
  'BEGIN {printf "%.3f", (ingest + tail) * 1000}')"
number_le "$tail_visible_ms" 2000 || die "log tail visibility exceeded 2 seconds"
jq -n \
  --argjson query "$query_p95_ms" \
  --argjson trace "$trace_p95_ms" \
  --argjson tail "$tail_visible_ms" \
  '{query_p95_ms:$query,trace_read_p95_ms:$trace,tail_visible_ms:$tail}' \
  > "$EVIDENCE_DIR/latency/summary.json"

echo ">> Sift MVP: five-minute load while stopping the Raft leader VM"
integrity_to "$EVIDENCE_DIR/load/failover/integrity-before.json"
stop_forwards
start_gateway_forward
start_store_forwards
refresh_token
failover_leader="$(wait_store_leader)"
failover_pod="sift-store-${failover_leader}"
failover_node="$(kubectl -n "$NAMESPACE" get "pod/${failover_pod}" \
  -o jsonpath='{.spec.nodeName}')"
[[ -n "$failover_node" ]] || die "could not resolve the leader VM"
kubectl cordon "$failover_node"
kubectl -n "$NAMESPACE" get pods -l sift.axiom.dev/frontend=true -o json \
  | jq -r --arg node "$failover_node" \
      '.items[] | select(.spec.nodeName == $node) | .metadata.name' \
  > "$EVIDENCE_DIR/kubernetes/frontends-on-failover-node.txt"
while IFS= read -r frontend; do
  [[ -n "$frontend" ]] || continue
  kubectl -n "$NAMESPACE" delete "pod/${frontend}" --wait=true --timeout=180s
done < "$EVIDENCE_DIR/kubernetes/frontends-on-failover-node.txt"
wait_role_ready deployment sift-gateway 1
wait_role_ready deployment sift-query 1
stop_forwards
start_gateway_forward
start_store_forwards
refresh_token
[[ "$(wait_store_leader)" == "$failover_leader" ]] \
  || die "Raft leadership changed before the failover drill began"
snapshot_restarts "$EVIDENCE_DIR/kubernetes/restarts-before-failover.json"

gcloud compute instances describe "$failover_node" \
  --project="$PROJECT_ID" --zone="$GKE_ZONE" --format=json \
  > "$EVIDENCE_DIR/kubernetes/failover-vm-before.json"
start_load_phase failover "$FAILOVER_SECONDS"
sleep 30
gcloud compute instances stop "$failover_node" \
  --project="$PROJECT_ID" --zone="$GKE_ZONE" --quiet
date -u +%Y-%m-%dT%H:%M:%SZ \
  > "$EVIDENCE_DIR/kubernetes/failover-vm-stopped-at.txt"

new_leader_deadline=$((SECONDS + 180))
new_leader=""
while (( SECONDS < new_leader_deadline )); do
  if candidate="$(find_store_leader 2>/dev/null)" \
    && [[ -n "$candidate" && "$candidate" != "$failover_leader" ]]; then
    new_leader="$candidate"
    break
  fi
  sleep 3
done
[[ -n "$new_leader" ]] || die "the surviving voters did not elect a new leader"
store_raftz "$new_leader" \
  > "$EVIDENCE_DIR/kubernetes/store-new-leader-after-vm-stop.json"
jq -e '
  .is_leader == true
  and .durability_error == null
  and (.committed_voters | length) == 3
' \
  "$EVIDENCE_DIR/kubernetes/store-new-leader-after-vm-stop.json" >/dev/null \
  || die "the surviving Raft quorum was not healthy"

wait_load_phase failover "$FAILOVER_SECONDS"
wait_for_delta "$EVIDENCE_DIR/load/failover/integrity-before.json" "$FAILOVER_ITEMS" \
  "$EVIDENCE_DIR/load/failover/integrity-after.json"
jq -e --slurpfile before "$EVIDENCE_DIR/load/failover/integrity-before.json" '
  (.event_count - $before[0].event_count) == 3000000
  and (.signals.logs.count - $before[0].signals.logs.count) == 1500000
  and (.signals.metrics.count - $before[0].signals.metrics.count) == 900000
  and (.signals.traces.count - $before[0].signals.traces.count) == 600000
' "$EVIDENCE_DIR/load/failover/integrity-after.json" >/dev/null \
  || die "failover load lost acknowledged signal data"
verify_load_digest failover "$FAILOVER_SECONDS" \
  "$EVIDENCE_DIR/load/failover/integrity-before.json" \
  "$EVIDENCE_DIR/load/failover/integrity-after.json"

node_deadline=$((SECONDS + 1200))
while (( SECONDS < node_deadline )); do
  ready_run_nodes="$(kubectl get nodes -l "axiom-run-id=${RUN_ID}" -o json \
    | jq '[.items[] | select(
        any(.status.conditions[]?;
          .type == "Ready" and .status == "True")
      )] | length')"
  [[ "$ready_run_nodes" == "3" ]] && break
  sleep 10
done
[[ "${ready_run_nodes:-0}" == "3" ]] || die "GKE auto-repair did not restore three ready run nodes"
kubectl uncordon "$failover_node" >/dev/null 2>&1 || true
wait_role_ready statefulset sift-store 3
wait_role_ready statefulset sift-control 3
wait_role_ready deployment sift-gateway 1
wait_role_ready deployment sift-query 1
wait_role_ready daemonset sift-agent 3
stop_forwards
start_gateway_forward
start_store_forwards
refresh_token
wait_store_leader >/dev/null
wait_for_store_convergence \
  "$EVIDENCE_DIR/load/failover/integrity-after.json" \
  "$EVIDENCE_DIR/load/failover/voters-after-recovery" \
  true
gcloud compute instances describe "$failover_node" \
  --project="$PROJECT_ID" --zone="$GKE_ZONE" --format=json \
  > "$EVIDENCE_DIR/kubernetes/failover-vm-after.json" 2>/dev/null || true
kubectl -n "$NAMESPACE" get pods -o json \
  > "$EVIDENCE_DIR/kubernetes/pods-after-failover.json"
snapshot_restarts "$EVIDENCE_DIR/kubernetes/restarts-after-failover.json"
assert_failover_restart_evidence \
  "$EVIDENCE_DIR/kubernetes/restarts-before-failover.json" \
  "$EVIDENCE_DIR/kubernetes/restarts-after-failover.json" \
  "$failover_node"
jq -e '
  all(.items[];
    ([.status.containerStatuses[]?.state.waiting.reason // empty] | index("CrashLoopBackOff") | not)
    and ([.status.containerStatuses[]?.lastState.terminated.reason // empty] | index("OOMKilled") | not))
' "$EVIDENCE_DIR/kubernetes/pods-after-failover.json" >/dev/null \
  || die "failover caused an OOMKill or CrashLoopBackOff"
verify_idempotency_retry after-vm-failover

wait_archive_covers_project() {
  local leader="$1"
  local output="$2"
  local deadline=$((SECONDS + 900))
  while (( SECONDS < deadline )); do
    if store_integrity_to "$leader" "$output" 2>/dev/null \
      && jq -e '
        .storage.archive.manifest_uri != null
        and .storage.archive.watermarks.logs >= .signals.logs.watermark
        and .storage.archive.watermarks.metrics >= .signals.metrics.watermark
        and .storage.archive.watermarks.traces >= .signals.traces.watermark
      ' "$output" >/dev/null 2>&1; then
      return 0
    fi
    sleep 5
  done
  die "archive manifest did not cover every project signal watermark"
}

echo ">> Sift MVP: 29-day, 31-day, and 181-day retention boundaries"
stop_forwards
start_gateway_forward
start_store_forwards
refresh_token
archive_leader="$(wait_store_leader)"
integrity_to "$EVIDENCE_DIR/kubernetes/integrity-before-retention.json"
retention_start="$(jq -r '.event_count' \
  "$EVIDENCE_DIR/kubernetes/integrity-before-retention.json")"
now_seconds="$(date -u +%s)"
day29_nanos="$(((now_seconds - 29 * 86400) * 1000000000))"
day31_nanos="$(((now_seconds - 31 * 86400) * 1000000000))"
day181_nanos="$(((now_seconds - 181 * 86400) * 1000000000))"
rollover_nanos="$(((now_seconds - 180 * 86400 + 120) * 1000000000))"
jq -nc \
  --arg day29 "$day29_nanos" \
  --arg day31 "$day31_nanos" \
  --arg day181 "$day181_nanos" \
  --arg rollover "$rollover_nanos" '
  {resourceLogs:[{
    resource:{attributes:[
      {key:"service.name",value:{stringValue:"sift-retention"}},
      {key:"deployment.environment.name",value:{stringValue:"gke"}}
    ]},
    scopeLogs:[{logRecords:[
      {
        timeUnixNano:$day29,
        body:{stringValue:"29-day boundary"},
        attributes:[{key:"sift.event_id",value:{stringValue:"retention-29d"}}]
      },
      {
        timeUnixNano:$day31,
        body:{stringValue:"31-day boundary"},
        attributes:[{key:"sift.event_id",value:{stringValue:"retention-31d"}}]
      },
      {
        timeUnixNano:$day181,
        body:{stringValue:"181-day boundary"},
        attributes:[{key:"sift.event_id",value:{stringValue:"retention-181d"}}]
      },
      {
        timeUnixNano:$rollover,
        body:{stringValue:"retention rollover"},
        attributes:[{key:"sift.event_id",value:{stringValue:"retention-rollover"}}]
      }
    ]}]
  }]}' > "$EVIDENCE_DIR/kubernetes/retention-boundaries.json"
jq -nc --arg rollover "$rollover_nanos" '
  {resourceLogs:[{
    resource:{attributes:[
      {key:"service.name",value:{stringValue:"sift-retention"}},
      {key:"deployment.environment.name",value:{stringValue:"gke"}}
    ]},
    scopeLogs:[{logRecords:[{
      timeUnixNano:$rollover,
      body:{stringValue:"retention rollover"},
      attributes:[{key:"sift.event_id",value:{stringValue:"retention-rollover"}}]
    }]}]
  }]}' > "$EVIDENCE_DIR/kubernetes/retention-rollover.json"
auth_curl -X POST "${sift_url}/v1/logs" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/kubernetes/retention-boundaries.json" \
  > "$EVIDENCE_DIR/kubernetes/retention-boundaries-response.json"
jq -e '
  .partialSuccess.rejectedLogRecords == 1
  and (.partialSuccess.errorMessage | contains("180-day retention"))
' "$EVIDENCE_DIR/kubernetes/retention-boundaries-response.json" >/dev/null \
  || die "181-day telemetry was not rejected as non-retryable partial success"
wait_for_integrity_count "$((retention_start + 3))" \
  "$EVIDENCE_DIR/kubernetes/integrity-after-retention.json"
auth_curl -X POST "${sift_url}/api/v1/query" \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"logs","filter":{"op":"in","field":"event_id","values":["retention-29d","retention-31d","retention-181d","retention-rollover"]}},"limit":10,"mode":"sync"}' \
  > "$EVIDENCE_DIR/kubernetes/query-retention-boundaries.json"
jq -e '
  ([.data.records[].event_id] | sort) == ["retention-29d","retention-31d","retention-rollover"]
  and .partial == false
' "$EVIDENCE_DIR/kubernetes/query-retention-boundaries.json" >/dev/null \
  || die "retention boundary query returned the wrong event set"

wait_archive_covers_project "$archive_leader" \
  "$EVIDENCE_DIR/gcs/integrity-before-rollover-expiry.json"
retention_generation_before="$(jq -r '.storage.archive.retention_generation' \
  "$EVIDENCE_DIR/gcs/integrity-before-rollover-expiry.json")"
rollover_deadline=$((SECONDS + 900))
rollover_removed=0
while (( SECONDS < rollover_deadline )); do
  if integrity_to "$EVIDENCE_DIR/gcs/integrity-after-rollover-expiry.json" 2>/dev/null; then
    rollover_query_status="$(auth_curl_status \
      --output "$EVIDENCE_DIR/kubernetes/query-retention-rollover-expired.json" \
      --write-out '%{http_code}' \
      -X POST "${sift_url}/api/v1/query" \
      -H 'content-type: application/json' \
      --data '{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"logs","filter":{"op":"eq","field":"event_id","value":"retention-rollover"}},"limit":10,"mode":"sync"}' \
      || true)"
    if [[ "$rollover_query_status" == "200" ]] \
      && jq -e \
        --argjson expected "$((retention_start + 2))" \
        --argjson before "$retention_generation_before" '
          .event_count == $expected
          and .storage.archive.retention_generation > $before
          and .storage.archive.retention_scan_pending == false
        ' "$EVIDENCE_DIR/gcs/integrity-after-rollover-expiry.json" >/dev/null 2>&1 \
      && jq -e '
        (.data.records | map(select(.event_id == "retention-rollover")) | length) == 0
        and .partial == false
      ' "$EVIDENCE_DIR/kubernetes/query-retention-rollover-expired.json" >/dev/null 2>&1; then
      rollover_removed=1
      break
    fi
  fi
  sleep 5
done
[[ "$rollover_removed" == "1" ]] \
  || die "the 180-day rollover did not produce a complete retention generation"

cp "$EVIDENCE_DIR/gcs/integrity-after-rollover-expiry.json" \
  "$EVIDENCE_DIR/kubernetes/integrity-before-expired-idempotency-retry.json"
auth_curl -X POST "${sift_url}/v1/logs" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/kubernetes/retention-rollover.json" \
  > "$EVIDENCE_DIR/kubernetes/retention-rollover-retry-response.json"
jq -e 'type == "object" and (.partialSuccess == null)' \
  "$EVIDENCE_DIR/kubernetes/retention-rollover-retry-response.json" >/dev/null \
  || die "the active six-hour receipt did not win over telemetry retention"
integrity_to "$EVIDENCE_DIR/kubernetes/integrity-after-expired-idempotency-retry.json"
jq -s -e '
  .[0] as $before | .[1] as $after
  | $after.event_count == $before.event_count
    and $after.event_id_sha256 == $before.event_id_sha256
    and $after.watermark == $before.watermark
    and $after.signals == $before.signals
' "$EVIDENCE_DIR/kubernetes/integrity-before-expired-idempotency-retry.json" \
  "$EVIDENCE_DIR/kubernetes/integrity-after-expired-idempotency-retry.json" >/dev/null \
  || die "retrying an expired event changed durable project identity"
auth_curl -X POST "${sift_url}/api/v1/query" \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"logs","filter":{"op":"eq","field":"event_id","value":"retention-rollover"}},"limit":10,"mode":"sync"}' \
  > "$EVIDENCE_DIR/kubernetes/query-retention-rollover-after-retry.json"
jq -e '
  (.data.records | map(select(.event_id == "retention-rollover")) | length) == 0
  and .partial == false
' "$EVIDENCE_DIR/kubernetes/query-retention-rollover-after-retry.json" >/dev/null \
  || die "retrying an expired event reintroduced telemetry"

archive_leader="$(wait_store_leader)"
wait_archive_covers_project "$archive_leader" \
  "$EVIDENCE_DIR/gcs/integrity-before-iam-outage.json"
gcloud storage ls --recursive "gs://${BACKUP_BUCKET}/sift/${RUN_ID}/**" \
  > "$EVIDENCE_DIR/gcs/objects-before-iam-outage.txt"
archive_manifest_before="$(jq -r '.storage.archive.manifest_uri' \
  "$EVIDENCE_DIR/gcs/integrity-before-iam-outage.json")"
archive_hash_before="$(jq -r '.storage.archive.manifest_sha256' \
  "$EVIDENCE_DIR/gcs/integrity-before-iam-outage.json")"
echo ">> Sift MVP: GCS outage preserves local WAL"
gcloud storage buckets remove-iam-policy-binding "gs://${BACKUP_BUCKET}" \
  --member="serviceAccount:${BACKUP_GSA_EMAIL}" \
  --role="roles/storage.objectAdmin" \
  --project="$PROJECT_ID" \
  --quiet > "$EVIDENCE_DIR/gcs/archive-iam-remove.json"
archive_iam_removed=1
printf '%s\n' archive-iam-disabled \
  > "$EVIDENCE_DIR/gcs/archive-iam-disabled"
sleep 1
archive_outage_started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
printf '%s\n' "$archive_outage_started_at" \
  > "$EVIDENCE_DIR/gcs/archive-outage-started-at.txt"
outage_nanos="$((now_seconds * 1000000000 + 9000000))"
jq -nc --arg ts "$outage_nanos" '
  {resourceLogs:[{
    resource:{attributes:[
      {key:"service.name",value:{stringValue:"sift-archive-outage"}},
      {key:"deployment.environment.name",value:{stringValue:"gke"}}
    ]},
    scopeLogs:[{logRecords:[{
      timeUnixNano:$ts,
      body:{stringValue:"archive IAM outage"},
      attributes:[{key:"sift.event_id",value:{stringValue:"archive-outage-log"}}]
    }]}]
  }]}' > "$EVIDENCE_DIR/gcs/archive-outage-log.json"
auth_curl -X POST "${sift_url}/v1/logs" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/gcs/archive-outage-log.json" \
  > "$EVIDENCE_DIR/gcs/archive-outage-log-response.json"
jq -e 'type == "object" and (.partialSuccess == null)' \
  "$EVIDENCE_DIR/gcs/archive-outage-log-response.json" >/dev/null \
  || die "the outage log was not fully acknowledged"
store_integrity_to "$archive_leader" \
  "$EVIDENCE_DIR/gcs/integrity-after-outage-ack.json"
archive_wal_after_ack="$(jq -r '.storage.wal_bytes.logs' \
  "$EVIDENCE_DIR/gcs/integrity-after-outage-ack.json")"
jq -e '
  .storage.wal_bytes.logs > 0
  and .storage.archive.watermarks.logs < .signals.logs.watermark
' "$EVIDENCE_DIR/gcs/integrity-after-outage-ack.json" >/dev/null \
  || die "the acknowledged outage event did not create unarchived WAL"
# Wait for direct proof that a lifecycle attempt happened after IAM was
# removed. A fixed sleep alone cannot prove that the worker reached GCS.
archive_failure_deadline=$((SECONDS + 150))
while (( SECONDS < archive_failure_deadline )); do
  kubectl -n "$NAMESPACE" logs "pod/sift-store-${archive_leader}" \
    --since-time="$archive_outage_started_at" \
    > "$EVIDENCE_DIR/gcs/archive-worker-outage.log" 2>&1 || true
  if rg -F "archive attempt failed" \
      "$EVIDENCE_DIR/gcs/archive-worker-outage.log" >/dev/null; then
    break
  fi
  sleep 5
done
rg -F "archive attempt failed" "$EVIDENCE_DIR/gcs/archive-worker-outage.log" >/dev/null \
  || die "the leader did not report an archive failure after IAM removal"
[[ "$(wait_store_leader)" == "$archive_leader" ]] \
  || die "archive leadership changed during the isolated GCS outage"
store_integrity_to "$archive_leader" \
  "$EVIDENCE_DIR/gcs/integrity-during-iam-outage.json"
jq -e \
  --arg manifest "$archive_manifest_before" \
  --arg hash "$archive_hash_before" \
  --argjson wal "$archive_wal_after_ack" '
  .storage.archive.manifest_uri == $manifest
  and .storage.archive.manifest_sha256 == $hash
  and .storage.wal_bytes.logs >= $wal
  and .storage.archive.watermarks.logs < .signals.logs.watermark
' "$EVIDENCE_DIR/gcs/integrity-during-iam-outage.json" >/dev/null \
  || die "GCS outage changed the manifest, compacted WAL, or hid archive lag"

verify_outage_quorum_recovery "$archive_leader" \
  "$EVIDENCE_DIR/gcs/integrity-after-outage-ack.json" \
  "$archive_manifest_before" "$archive_hash_before"

cold_query_start="$(jq -nr --argjson epoch "$((now_seconds - 32 * 86400))" \
  '$epoch | strftime("%Y-%m-%dT%H:%M:%SZ")')"
jq -nc --arg start "$cold_query_start" '
  {
    version:1,
    project:"sift-mvp",
    environment:"gke",
    time_range:{start:$start},
    signal:{kind:"logs",filter:{op:"eq",field:"event_id",value:"retention-31d"}},
    limit:10,
    mode:"sync"
  }' > "$EVIDENCE_DIR/gcs/cold-query-request.json"
auth_curl -X POST "${sift_url}/api/v1/query" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/gcs/cold-query-request.json" \
  > "$EVIDENCE_DIR/gcs/cold-query-during-outage.json"
jq -e '
  .partial == true
  and (.warnings | map(ascii_downcase) | join(" ") | contains("archive"))
' "$EVIDENCE_DIR/gcs/cold-query-during-outage.json" >/dev/null \
  || die "cold query silently hid the GCS outage"

restore_archive_iam
gcloud storage buckets get-iam-policy "gs://${BACKUP_BUCKET}" \
  --project="$PROJECT_ID" --format=json \
  > "$EVIDENCE_DIR/gcs/archive-iam-restored.json"
wait_archive_covers_project "$archive_leader" \
  "$EVIDENCE_DIR/gcs/integrity-after-iam-restore.json"
archive_manifest="$(jq -r '.storage.archive.manifest_uri' \
  "$EVIDENCE_DIR/gcs/integrity-after-iam-restore.json")"
[[ "$archive_manifest" != "$archive_manifest_before" ]] \
  || die "archive did not advance after GCS IAM was restored"
[[ "$archive_manifest" == gs://* ]] || die "archive manifest URI is not a GCS URI"
gcloud storage objects describe "$archive_manifest" --format=json \
  > "$EVIDENCE_DIR/gcs/final-manifest-object.json"
gcloud storage cat "$archive_manifest" \
  > "$EVIDENCE_DIR/gcs/final-manifest.json"
jq -e '
  .format_version == 10
  and .event_count > 0
  and (.catalog_uri | startswith("gs://"))
  and .catalog_root.format_version == 1
  and .catalog_root.entry_count == (.segment_count + .blob_count + .dedupe_receipt_count)
  and .segment_count > 0
  and (has("segments") | not)
  and (has("blobs") | not)
  and (has("gc_object_uris") | not)
  and (.event_id_sha256 | length) == 64
' "$EVIDENCE_DIR/gcs/final-manifest.json" >/dev/null \
  || die "final GCS archive manifest is incomplete"
store_integrity_to "$archive_leader" \
  "$EVIDENCE_DIR/gcs/source-integrity-for-restore.json"
jq -e --slurpfile archived "$EVIDENCE_DIR/gcs/integrity-after-iam-restore.json" '
  .event_count == $archived[0].event_count
  and .event_id_sha256 == $archived[0].event_id_sha256
  and .watermark == $archived[0].watermark
' "$EVIDENCE_DIR/gcs/source-integrity-for-restore.json" >/dev/null \
  || die "source changed after the final archive manifest was selected"

echo ">> Sift MVP: fresh-PVC restore from the committed manifest"
printf '%s\n' fresh-pvc-restore \
  > "$EVIDENCE_DIR/restore/fresh-pvc-restore"
create_owned_namespace \
  "$RESTORE_NAMESPACE" "$EVIDENCE_DIR/kubernetes/ownership" \
  "$PROJECT_ID" "$RUN_ID" "$ACCEPTANCE_LOCK_ACQUISITION_ID"
cat > "$EVIDENCE_DIR/restore/identity.yaml" <<EOF
apiVersion: v1
kind: ServiceAccount
metadata:
  name: sift-restore
  namespace: ${RESTORE_NAMESPACE}
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: sift-restore-store
  namespace: ${RESTORE_NAMESPACE}
  annotations:
    iam.gke.io/gcp-service-account: ${BACKUP_GSA_EMAIL}
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: sift-source-project
  namespace: ${RESTORE_NAMESPACE}
rules:
  - apiGroups: ["sift.axiom.dev"]
    resources: ["projects"]
    resourceNames: ["${PROJECT}"]
    verbs: ["get", "create", "update"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: sift-source-project
  namespace: ${RESTORE_NAMESPACE}
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: Role
  name: sift-source-project
subjects:
  - kind: ServiceAccount
    name: sift-rig
    namespace: ${NAMESPACE}
EOF
kubectl apply -f "$EVIDENCE_DIR/restore/identity.yaml"
peer_dir="$MANIFEST_DIR/sift/peer-pki"
kubectl -n "$RESTORE_NAMESPACE" create secret generic sift-peer-tls \
  --from-file=tls.crt="$peer_dir/tls.crt" \
  --from-file=tls.key="$peer_dir/tls.key" \
  --from-file=ca.crt="$peer_dir/ca.crt" \
  --dry-run=client -o yaml \
  > "$EVIDENCE_DIR/restore/peer-secret.yaml"
kubectl apply -f "$EVIDENCE_DIR/restore/peer-secret.yaml"
cat > "$EVIDENCE_DIR/restore/sift-restore.yaml" <<EOF
apiVersion: sift.axiom.dev/v1alpha1
kind: Sift
metadata:
  name: sift-restore
  namespace: ${RESTORE_NAMESPACE}
spec:
  image: ${SIFT_IMAGE}
  peerTlsSecret: sift-peer-tls
  replicasPerShard: 3
  voterCount: 3
  auth: kubernetes
  bootstrap:
    archiveManifestUri: ${archive_manifest}
  storage:
    storeSize: 50Gi
    controlSize: 5Gi
    gatewaySize: 2Gi
    querySize: 2Gi
  ingest:
    maxItemsPerMinute: 720000
    maxConcurrentRequests: 32
  placement:
    nodeSelector:
      axiom-run-id: "${RUN_ID}"
  gcpProjectId: ${PROJECT_ID}
  gkeClusterName: ${GKE_CLUSTER_NAME}
  gkeLocation: ${GKE_ZONE}
EOF
kubectl apply -f "$EVIDENCE_DIR/restore/sift-restore.yaml"

restore_deadline=$((SECONDS + 1200))
while (( SECONDS < restore_deadline )); do
  restore_generation="$(kubectl -n "$RESTORE_NAMESPACE" get sift/sift-restore \
    -o jsonpath='{.metadata.generation}' 2>/dev/null || true)"
  restore_observed="$(kubectl -n "$RESTORE_NAMESPACE" get sift/sift-restore \
    -o jsonpath='{.status.observedGeneration}' 2>/dev/null || true)"
  restore_phase="$(kubectl -n "$RESTORE_NAMESPACE" get sift/sift-restore \
    -o jsonpath='{.status.phase}' 2>/dev/null || true)"
  restore_archive_phase="$(kubectl -n "$RESTORE_NAMESPACE" get sift/sift-restore \
    -o jsonpath='{.status.restorePhase}' 2>/dev/null || true)"
  if [[ -n "$restore_generation" \
    && "$restore_observed" == "$restore_generation" \
    && "$restore_phase" == "Ready" \
    && "$restore_archive_phase" == "Restored" ]]; then
    break
  fi
  sleep 5
done
[[ "${restore_phase:-}" == "Ready" && "${restore_archive_phase:-}" == "Restored" ]] \
  || die "fresh-PVC restore did not reach Ready and Restored"
kubectl -n "$RESTORE_NAMESPACE" rollout status \
  statefulset/sift-restore-store --timeout=900s
kubectl -n "$RESTORE_NAMESPACE" rollout status \
  statefulset/sift-restore-control --timeout=900s
kubectl -n "$RESTORE_NAMESPACE" rollout status \
  deployment/sift-restore-gateway --timeout=900s
kubectl -n "$RESTORE_NAMESPACE" rollout status \
  deployment/sift-restore-query --timeout=900s
kubectl -n "$RESTORE_NAMESPACE" get \
  deployment,statefulset,daemonset,pod,pvc,serviceaccount -o json \
  > "$EVIDENCE_DIR/restore/topology.json"
jq -e '
  [.items[] | select(.kind == "PersistentVolumeClaim")] as $pvcs
  | ($pvcs | length) == 8
    and (all($pvcs[]; .status.phase == "Bound"))
    and (all($pvcs[]; .metadata.creationTimestamp != null))
' "$EVIDENCE_DIR/restore/topology.json" >/dev/null \
  || die "restore did not use eight fresh bound PVCs"
verify_pods_on_run_nodes "$RESTORE_NAMESPACE" \
  'app.kubernetes.io/name=sift' sift-restore-topology 11

RESTORE_PORT=17580
kubectl -n "$RESTORE_NAMESPACE" port-forward service/sift-restore \
  "${RESTORE_PORT}:7380" \
  >> "$EVIDENCE_DIR/restore/port-forward.log" 2>&1 &
forward_pids+=("$!")
restore_url="http://127.0.0.1:${RESTORE_PORT}"
restore_forward_deadline=$((SECONDS + 120))
while (( SECONDS < restore_forward_deadline )); do
  if curl --max-time 3 --silent --show-error --fail \
    "${restore_url}/readyz" >/dev/null 2>&1; then
    break
  fi
  sleep 2
done
curl --max-time 3 --silent --show-error --fail \
  "${restore_url}/readyz" >/dev/null 2>&1 \
  || die "restored Sift gateway did not become ready"

restore_auth_curl() {
  curl --silent --show-error --fail-with-body \
    -H "authorization: Bearer $(sed -n '1p' "$token_file")" \
    -H "x-sift-project: ${PROJECT}" \
    "$@"
}
restore_auth_curl "${restore_url}/admin/integrity?project=${PROJECT}" \
  > "$EVIDENCE_DIR/restore/integrity.json"
jq -e \
  --slurpfile source "$EVIDENCE_DIR/gcs/source-integrity-for-restore.json" \
  --arg manifest "$archive_manifest" '
  .restored_from == $manifest
  and .cluster_id != $source[0].cluster_id
  and .event_count == $source[0].event_count
  and .event_id_sha256 == $source[0].event_id_sha256
  and .watermark == $source[0].watermark
  and .signals == $source[0].signals
' "$EVIDENCE_DIR/restore/integrity.json" >/dev/null \
  || die "fresh-PVC restore count, digest, watermark, signals, or cluster identity did not match"

cp "$EVIDENCE_DIR/restore/integrity.json" \
  "$EVIDENCE_DIR/restore/integrity-before-expired-idempotency-retry.json"
restore_auth_curl -X POST "${restore_url}/v1/logs" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/kubernetes/retention-rollover.json" \
  > "$EVIDENCE_DIR/restore/retention-rollover-retry-response.json"
jq -e 'type == "object" and (.partialSuccess == null)' \
  "$EVIDENCE_DIR/restore/retention-rollover-retry-response.json" >/dev/null \
  || die "fresh-PVC restore lost the active six-hour receipt"
restore_auth_curl "${restore_url}/admin/integrity?project=${PROJECT}" \
  > "$EVIDENCE_DIR/restore/integrity-after-expired-idempotency-retry.json"
jq -s -e '
  .[0] as $before | .[1] as $after
  | $after.event_count == $before.event_count
    and $after.event_id_sha256 == $before.event_id_sha256
    and $after.watermark == $before.watermark
    and $after.signals == $before.signals
' "$EVIDENCE_DIR/restore/integrity-before-expired-idempotency-retry.json" \
  "$EVIDENCE_DIR/restore/integrity-after-expired-idempotency-retry.json" >/dev/null \
  || die "fresh-PVC retry changed restored project identity"
restore_auth_curl -X POST "${restore_url}/api/v1/query" \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"logs","filter":{"op":"eq","field":"event_id","value":"retention-rollover"}},"limit":10,"mode":"sync"}' \
  > "$EVIDENCE_DIR/restore/query-retention-rollover-after-retry.json"
jq -e '
  (.data.records | map(select(.event_id == "retention-rollover")) | length) == 0
  and .partial == false
' "$EVIDENCE_DIR/restore/query-retention-rollover-after-retry.json" >/dev/null \
  || die "fresh-PVC retry reintroduced expired telemetry"

for ordinal in 0 1 2; do
  restore_http_port=$((17600 + ordinal))
  restore_peer_port=$((18600 + ordinal))
  kubectl -n "$RESTORE_NAMESPACE" port-forward \
    "pod/sift-restore-store-${ordinal}" "${restore_http_port}:7380" \
    >> "$EVIDENCE_DIR/restore/store-${ordinal}-http-forward.log" 2>&1 &
  forward_pids+=("$!")
  kubectl -n "$RESTORE_NAMESPACE" port-forward \
    "pod/sift-restore-store-${ordinal}" "${restore_peer_port}:7381" \
    >> "$EVIDENCE_DIR/restore/store-${ordinal}-peer-forward.log" 2>&1 &
  forward_pids+=("$!")
done
sleep 3
restore_peer_dir="$MANIFEST_DIR/sift/peer-pki"
restore_voters_deadline=$((SECONDS + 900))
restore_voters_converged=false
while (( SECONDS < restore_voters_deadline )); do
  restore_voters_complete=true
  for ordinal in 0 1 2; do
    restore_http_port=$((17600 + ordinal))
    restore_peer_port=$((18600 + ordinal))
    restore_host="sift-restore-store-${ordinal}.sift-restore-store-headless.sift-restore.svc.cluster.local"
    curl --max-time 30 --silent --show-error --fail-with-body \
      -H "authorization: Bearer $(sed -n '1p' "$token_file")" \
      -H "x-sift-project: ${PROJECT}" \
      "http://127.0.0.1:${restore_http_port}/admin/integrity?project=${PROJECT}" \
      > "$EVIDENCE_DIR/restore/store-${ordinal}-integrity.json" \
      2>/dev/null || restore_voters_complete=false
    curl --noproxy '*' --max-time 5 --silent --show-error --fail \
      --cacert "$restore_peer_dir/ca.crt" \
      --cert "$restore_peer_dir/tls.crt" \
      --key "$restore_peer_dir/tls.key" \
      --resolve "${restore_host}:${restore_peer_port}:127.0.0.1" \
      "https://${restore_host}:${restore_peer_port}/raftz" \
      > "$EVIDENCE_DIR/restore/store-${ordinal}-raftz.json" \
      2>/dev/null || restore_voters_complete=false
  done
  if [[ "$restore_voters_complete" == true ]] \
    && jq -s -e --slurpfile source "$EVIDENCE_DIR/gcs/source-integrity-for-restore.json" \
      --arg manifest "$archive_manifest" '
        length == 3
        and (map(.cluster_id) | unique | length == 1)
        and all(.[];
          .restored_from == $manifest
          and .cluster_id != $source[0].cluster_id
          and .event_count == $source[0].event_count
          and .event_id_sha256 == $source[0].event_id_sha256
          and .watermark == $source[0].watermark
          and .signals == $source[0].signals)
      ' "$EVIDENCE_DIR"/restore/store-*-integrity.json >/dev/null \
    && jq -s -e '
        length == 3
        and (map(.committed_voters | length == 3) | all)
        and (map(.durability_error == null) | all)
        and (map(.applied_index) | unique | length == 1)
        and (map(.commit_index) | unique | length == 1)
        and all(.[];
          .applied_index == .commit_index
          and .snapshot_index <= .applied_index
          and .resident_log_bytes <= 536870912
          and .resident_log_bytes < .max_resident_log_bytes)
      ' "$EVIDENCE_DIR"/restore/store-*-raftz.json >/dev/null; then
    restore_voters_converged=true
    break
  fi
  sleep 5
done
[[ "$restore_voters_converged" == true ]] \
  || die "fresh-PVC restore voters did not converge on integrity and bounded Raft state"
jq -s -e --slurpfile source "$EVIDENCE_DIR/gcs/source-integrity-for-restore.json" '
  length == 3
  and all(.[];
    .event_count == $source[0].event_count
    and .event_id_sha256 == $source[0].event_id_sha256
    and .watermark == $source[0].watermark
    and .signals == $source[0].signals)
' "$EVIDENCE_DIR"/restore/store-*-integrity.json >/dev/null \
  || die "fresh-PVC restore voters disagree with the source archive"

restore_auth_curl -X POST "${restore_url}/api/v1/query" \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"logs","filter":{"op":"eq","field":"event_id","value":"retention-31d"}},"limit":10,"mode":"sync"}' \
  > "$EVIDENCE_DIR/restore/log-sample.json"
jq -e '(.data.records | map(select(.event_id == "retention-31d")) | length) == 1' \
  "$EVIDENCE_DIR/restore/log-sample.json" >/dev/null \
  || die "restored log sample was not queryable"
restore_auth_curl -X POST "${restore_url}/api/v1/query" \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"metrics","name":"sift.load.counter","function":"count"},"limit":10,"mode":"sync"}' \
  > "$EVIDENCE_DIR/restore/metric-sample.json"
jq -e '(.data.series | length) >= 1' \
  "$EVIDENCE_DIR/restore/metric-sample.json" >/dev/null \
  || die "restored metric sample was not queryable"
restore_auth_curl -X POST "${restore_url}/api/v1/query" \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"traces","service":"sift-load","operation":"POST /load"},"limit":10,"mode":"sync"}' \
  > "$EVIDENCE_DIR/restore/trace-sample.json"
jq -e '(.data.traces | length) >= 1' \
  "$EVIDENCE_DIR/restore/trace-sample.json" >/dev/null \
  || die "restored trace sample was not queryable"

echo ">> Sift MVP: write pre-cleanup verification evidence"
jq -n \
  --arg project_id "$PROJECT_ID" \
  --arg region "$REGION" \
  --arg zone "$GKE_ZONE" \
  --arg run_id "$RUN_ID" \
  --arg bucket "$BACKUP_BUCKET" \
  --arg sift_image "$SIFT_IMAGE" \
  --arg rig_image "$RIG_IMAGE" \
  --arg acceptance_runner_image "$ACCEPTANCE_RUNNER_IMAGE" \
  --arg candidate_git_sha "$CANDIDATE_GIT_SHA" \
  --arg source_bundle_sha256 "$CANDIDATE_SOURCE_SHA256" \
  --arg cloud_build_id "$CANDIDATE_CLOUD_BUILD_ID" \
  --arg source_object_uri "$CANDIDATE_SOURCE_URI" \
  --arg manifest "$archive_manifest" \
  --arg leader_before "$failover_leader" \
  --arg leader_after "$new_leader" \
  --arg stopped_vm "$failover_node" \
  --slurpfile steady "$EVIDENCE_DIR/load/steady/summary.json" \
  --slurpfile steady_digest "$EVIDENCE_DIR/load/steady/event-id-digest.json" \
  --slurpfile failover "$EVIDENCE_DIR/load/failover/summary.json" \
  --slurpfile failover_digest "$EVIDENCE_DIR/load/failover/event-id-digest.json" \
  --slurpfile latency "$EVIDENCE_DIR/latency/summary.json" \
  --slurpfile outage_restart "$EVIDENCE_DIR/gcs/outage-leader-restart.json" \
  --slurpfile source "$EVIDENCE_DIR/gcs/source-integrity-for-restore.json" \
  --slurpfile restored "$EVIDENCE_DIR/restore/integrity.json" '
  {
    schema:"axiom.gcp.operator.verification.v1",
    project_id:$project_id,
    region:$region,
    gke_zone:$zone,
    run_id:$run_id,
    backup_bucket:$bucket,
    acceptance:{
      sift:{
        schema:"axiom.gcp.sift.mvp.verification.v1",
        status:"verification-passed",
        candidate:{
          sift_image:$sift_image,
          rig_image:$rig_image,
          acceptance_runner_image:$acceptance_runner_image,
          git_sha:$candidate_git_sha,
          source_bundle_sha256:$source_bundle_sha256,
          cloud_build_id:$cloud_build_id,
          source_object_uri:$source_object_uri,
          immutable:true
        },
        topology:{
          store_voters:3,
          control_replicas:3,
          gateway_replicas:1,
          query_replicas:1,
          peer_mtls:"passed",
          pvc_restart:"passed"
        },
        protocols:{
          otlp_http_json:"passed",
          otlp_http_protobuf:"passed",
          otlp_gzip:"passed",
          otlp_grpc:"passed",
          partial_success:"passed",
          remote_write_1:"passed",
          prometheus_query_range:"passed",
          remote_write_2_rejected_415:"passed",
          mcp_read_only_tools:"passed",
          mcp_host_origin:"passed",
          cross_project_denied:"passed",
          cross_project_same_id:"passed"
        },
        load:{
          duration_seconds:1800,
          offered_items_per_second:10000,
          expected_unique_items:18000000,
          expected_logs:9000000,
          expected_metric_points:5400000,
          expected_spans:3600000,
          observed:$steady[0],
          event_id_digest:$steady_digest[0]
        },
        latency:$latency[0],
        failover:{
          duration_seconds:300,
          expected_unique_items:3000000,
          leader_before:$leader_before,
          leader_after:$leader_after,
          stopped_vm:$stopped_vm,
          observed:$failover[0],
          event_id_digest:$failover_digest[0],
          acknowledged_data_loss:0,
          auto_repair:"passed"
        },
        archive:{
          gcs_iam_outage:"passed",
          wal_preserved:"passed",
          quorum_recovered_after_leader_restart:$outage_restart[0].quorum_recovered,
          manifest_uri:$manifest
        },
        idempotency:{
          signals:["logs","metrics","traces"],
          immediate_retry:"passed",
          after_steady_load:"passed",
          after_vm_failover:"passed",
          after_telemetry_expiration:"passed"
        },
        retention:{
          day_29:"hot-query-passed",
          day_31:"cold-query-passed",
          day_181:"non-retryable-partial-rejection-passed",
          day_180_rollover:"bounded-generation-passed",
          scan_completed:true
        },
        restore:{
          fresh_pvc:"passed",
          new_cluster_id:($restored[0].cluster_id != $source[0].cluster_id),
          restored_from:$restored[0].restored_from,
          source_count:$source[0].event_count,
          restored_count:$restored[0].event_count,
          source_digest:$source[0].event_id_sha256,
          restored_digest:$restored[0].event_id_sha256,
          source_watermark:$source[0].watermark,
          restored_watermark:$restored[0].watermark
        },
        cleanup_evidence:null
      }
    }
  }' > "$EVIDENCE_DIR/sift-mvp-verification.json"
python3 "$SCRIPT_DIR/validate-sift-mvp-evidence.py" \
  --schema "$SCRIPT_DIR/../evidence/schema.json" \
  --document "$EVIDENCE_DIR/sift-mvp-verification.json" \
  --mode verification
date -u +%Y-%m-%dT%H:%M:%SZ \
  > "$EVIDENCE_DIR/kubernetes/sift-mvp-verification-completed-at.txt"
echo "Sift MVP cloud checks passed. Cleanup must finish before terminal evidence exists."
