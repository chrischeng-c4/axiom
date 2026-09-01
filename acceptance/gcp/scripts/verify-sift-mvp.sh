#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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
NAMESPACE=sift
RESTORE_NAMESPACE=sift-restore
SIFT_PORT=17380

mkdir -p \
  "$EVIDENCE_DIR/kubernetes" \
  "$EVIDENCE_DIR/gcs" \
  "$EVIDENCE_DIR/load" \
  "$EVIDENCE_DIR/latency" \
  "$EVIDENCE_DIR/restore"

for command in awk curl date gcloud gzip jq kubectl rg sed seq sort; do
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

forward_pids=()
archive_iam_removed=0
token_file="$EVIDENCE_DIR/kubernetes/sift-rig.token"
sift_url="http://127.0.0.1:${SIFT_PORT}"

die() {
  echo "Sift MVP acceptance failed: $*" >&2
  capture_diagnostics
  exit 1
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
}
trap cleanup_local EXIT INT
# The parent owns the 90-minute deadline and the cloud cleanup trap.
trap '' TERM

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
  curl --silent --show-error --fail-with-body \
    -H "authorization: Bearer $(sed -n '1p' "$token_file")" \
    -H "x-sift-project: ${PROJECT}" \
    "$@"
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
  auth_curl "${sift_url}/admin/integrity?project=${PROJECT}" > "$output"
  jq -e --arg project "$PROJECT" '
    .version == 1
    and .project == $project
    and .event_id_digest_algorithm == "xor-sha256-v1"
  ' "$output" >/dev/null
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
        .items[] |
        {
          pod: .metadata.name,
          uid: .metadata.uid,
          restarts: ([.status.containerStatuses[]?.restartCount] | add // 0),
          waiting: [.status.containerStatuses[]?.state.waiting.reason // empty],
          terminated: [.status.containerStatuses[]?.lastState.terminated.reason // empty]
        }
      ] | sort_by(.pod)' > "$output"
}

assert_no_unexpected_restarts() {
  local before="$1"
  local after="$2"
  jq -e --slurpfile before "$before" '
    ($before[0] | map({key:.pod, value:.}) | from_entries) as $old
    | all(.[];
        (.waiting | index("CrashLoopBackOff") | not)
        and (.terminated | index("OOMKilled") | not)
        and (($old[.pod] == null) or (.restarts == $old[.pod].restarts)))
  ' "$after" >/dev/null || die "a Sift pod restarted unexpectedly or was OOM-killed"
}

wait_role_ready() {
  local kind="$1"
  local name="$2"
  local want="$3"
  local deadline=$((SECONDS + 900))
  local ready
  while (( SECONDS < deadline )); do
    ready="$(kubectl -n "$NAMESPACE" get "$kind/$name" \
      -o jsonpath='{.status.readyReplicas}' 2>/dev/null || true)"
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

echo ">> Sift MVP: validate topology and immutable candidate"
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
    axiom-run-id: ${RUN_ID}
spec:
  backoffLimit: 0
  activeDeadlineSeconds: 300
  template:
    metadata:
      labels:
        axiom-owner: gcp-operator-acceptance
        axiom-run-id: ${RUN_ID}
    spec:
      serviceAccountName: sift-rig
      automountServiceAccountToken: false
      restartPolicy: Never
      nodeSelector:
        cloud.google.com/gke-nodepool: acceptance-pool
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
}

run_mcp_smoke() {
  local init_headers="$EVIDENCE_DIR/kubernetes/mcp-init.headers"
  local init_body="$EVIDENCE_DIR/kubernetes/mcp-init.json"
  local list_body="$EVIDENCE_DIR/kubernetes/mcp-tools.json"
  local session
  local allowed_host="sift.sift.svc.cluster.local"
  local allowed_origin="http://sift.sift.svc.cluster.local:7380"
  local initialize='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"sift-gke-acceptance","version":"1"}}}'

  curl --silent --show-error --fail-with-body \
    -D "$init_headers" \
    -o "$init_body" \
    -X POST "${sift_url}/mcp" \
    -H "host: ${allowed_host}" \
    -H "origin: ${allowed_origin}" \
    -H "authorization: Bearer $(sed -n '1p' "$token_file")" \
    -H 'content-type: application/json' \
    -H 'accept: application/json, text/event-stream' \
    --data "$initialize"
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
    > "$list_body"
  jq -e '
    [.result.tools[].name] | sort ==
    ["sift_correlate","sift_get_trace","sift_list_services","sift_query","sift_tail_logs"]
  ' "$list_body" >/dev/null || die "MCP did not expose exactly five read-only tools"

  bad_origin_status="$(curl --silent --output /dev/null --write-out '%{http_code}' \
    -X POST "${sift_url}/mcp" \
    -H "host: ${allowed_host}" \
    -H 'origin: https://evil.example' \
    -H 'content-type: application/json' \
    -H 'accept: application/json, text/event-stream' \
    --data "$initialize")"
  [[ "$bad_origin_status" == "403" ]] || die "MCP accepted an untrusted Origin"
  bad_host_status="$(curl --silent --output /dev/null --write-out '%{http_code}' \
    -X POST "${sift_url}/mcp" \
    -H 'host: evil.example' \
    -H "origin: ${allowed_origin}" \
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

wait_for_integrity_count "$((smoke_start + 9))" \
  "$EVIDENCE_DIR/kubernetes/integrity-after-smoke.json"
metric_before_v2="$(jq -r '.signals.metrics.count' \
  "$EVIDENCE_DIR/kubernetes/integrity-after-smoke.json")"
remote_write_v2_status="$(auth_curl --output "$EVIDENCE_DIR/kubernetes/remote-write-v2-response.json" \
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

auth_curl "${sift_url}/prometheus/api/v1/query?project=${PROJECT}&environment=acceptance&query=sum%28sift_acceptance_total%29&time=${epoch_seconds}" \
  > "$EVIDENCE_DIR/kubernetes/prometheus-query.json"
jq -e '.status == "success" and .data.resultType == "vector"' \
  "$EVIDENCE_DIR/kubernetes/prometheus-query.json" >/dev/null \
  || die "Prometheus query endpoint failed"

cross_project_status="$(curl --silent --output "$EVIDENCE_DIR/kubernetes/cross-project.json" \
  --write-out '%{http_code}' \
  -X POST "${sift_url}/api/v1/query" \
  -H "authorization: Bearer $(sed -n '1p' "$token_file")" \
  -H 'x-sift-project: denied-project' \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"denied-project","signal":{"kind":"logs"},"limit":1,"mode":"sync"}')"
[[ "$cross_project_status" == "403" ]] || die "cross-project access was not denied"

run_mcp_smoke

make_load_scenario() {
  local phase="$1"
  local signal="$2"
  local qps="$3"
  local duration="$4"
  local output="$5"
  local body path case_name base_seconds
  case_name="${phase}_${signal}"
  base_seconds="$(date -u +%s)"
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
        --arg phase "$phase" '
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
                traceId:("c0ffee000000" + $seq + ($i|pad4)),
                spanId:$seq,
                name:"POST /load",
                startTimeUnixNano:($base + $seq6 + ($i|pad3)),
                endTimeUnixNano:($base + $seq6 + ($i|pad3)),
                status:{code:1},
                attributes:[{key:"load.phase",value:{stringValue:$phase}}]
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
  kubectl apply -f "$EVIDENCE_DIR/load/${phase}/${signal}/configmap.yaml"
  cat > "$EVIDENCE_DIR/load/${phase}/${signal}/job.yaml" <<EOF
apiVersion: batch/v1
kind: Job
metadata:
  name: ${job_name}
  namespace: ${NAMESPACE}
  labels:
    axiom-owner: gcp-operator-acceptance
    axiom-run-id: ${RUN_ID}
    sift-load-phase: ${phase}
spec:
  suspend: true
  backoffLimit: 0
  activeDeadlineSeconds: $((duration + 900))
  template:
    metadata:
      labels:
        axiom-owner: gcp-operator-acceptance
        axiom-run-id: ${RUN_ID}
        sift-load-phase: ${phase}
    spec:
      serviceAccountName: sift-rig
      automountServiceAccountToken: false
      restartPolicy: Never
      nodeSelector:
        cloud.google.com/gke-nodepool: acceptance-pool
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
    reports: .,
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
  --data-binary "@$EVIDENCE_DIR/latency/log.json" >/dev/null
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
  --data-binary "@$EVIDENCE_DIR/latency/trace.json" >/dev/null

query_payload='{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"logs","filter":{"op":"eq","field":"event_id","value":"latency-log"}},"limit":10,"mode":"sync"}'
: > "$EVIDENCE_DIR/latency/query-seconds.txt"
: > "$EVIDENCE_DIR/latency/trace-seconds.txt"
for sample in $(seq 1 40); do
  auth_curl --output /dev/null --write-out '%{time_total}\n' \
    -X POST "${sift_url}/api/v1/query" \
    -H 'content-type: application/json' \
    --data "$query_payload" \
    >> "$EVIDENCE_DIR/latency/query-seconds.txt"
  auth_curl --output /dev/null --write-out '%{time_total}\n' \
    "${sift_url}/api/v1/traces/${latency_trace}?project=${PROJECT}" \
    >> "$EVIDENCE_DIR/latency/trace-seconds.txt"
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
tail_ingest_seconds="$(auth_curl --output /dev/null --write-out '%{time_total}' \
  -X POST "${sift_url}/v1/logs" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/latency/tail-log.json")"
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
gcloud compute instances describe "$failover_node" \
  --project="$PROJECT_ID" --zone="$GKE_ZONE" --format=json \
  > "$EVIDENCE_DIR/kubernetes/failover-vm-after.json" 2>/dev/null || true
kubectl -n "$NAMESPACE" get pods -o json \
  > "$EVIDENCE_DIR/kubernetes/pods-after-failover.json"
jq -e '
  all(.items[];
    ([.status.containerStatuses[]?.state.waiting.reason // empty] | index("CrashLoopBackOff") | not)
    and ([.status.containerStatuses[]?.lastState.terminated.reason // empty] | index("OOMKilled") | not))
' "$EVIDENCE_DIR/kubernetes/pods-after-failover.json" >/dev/null \
  || die "failover caused an OOMKill or CrashLoopBackOff"

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
jq -nc \
  --arg day29 "$day29_nanos" \
  --arg day31 "$day31_nanos" \
  --arg day181 "$day181_nanos" '
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
      }
    ]}]
  }]}' > "$EVIDENCE_DIR/kubernetes/retention-boundaries.json"
auth_curl -X POST "${sift_url}/v1/logs" \
  -H 'content-type: application/json' \
  --data-binary "@$EVIDENCE_DIR/kubernetes/retention-boundaries.json" \
  > "$EVIDENCE_DIR/kubernetes/retention-boundaries-response.json"
jq -e '
  .partialSuccess.rejectedLogRecords == 1
  and (.partialSuccess.errorMessage | contains("180-day retention"))
' "$EVIDENCE_DIR/kubernetes/retention-boundaries-response.json" >/dev/null \
  || die "181-day telemetry was not rejected as non-retryable partial success"
wait_for_integrity_count "$((retention_start + 2))" \
  "$EVIDENCE_DIR/kubernetes/integrity-after-retention.json"
auth_curl -X POST "${sift_url}/api/v1/query" \
  -H 'content-type: application/json' \
  --data '{"version":1,"project":"sift-mvp","environment":"gke","signal":{"kind":"logs","filter":{"op":"in","field":"event_id","values":["retention-29d","retention-31d","retention-181d"]}},"limit":10,"mode":"sync"}' \
  > "$EVIDENCE_DIR/kubernetes/query-retention-boundaries.json"
jq -e '
  ([.data.records[].event_id] | sort) == ["retention-29d","retention-31d"]
  and .partial == false
' "$EVIDENCE_DIR/kubernetes/query-retention-boundaries.json" >/dev/null \
  || die "retention boundary query returned the wrong event set"

wait_archive_covers_project "$archive_leader" \
  "$EVIDENCE_DIR/gcs/integrity-before-iam-outage.json"
gcloud storage ls --recursive "gs://${BACKUP_BUCKET}/sift/${RUN_ID}/**" \
  > "$EVIDENCE_DIR/gcs/objects-before-iam-outage.txt"
archive_manifest_before="$(jq -r '.storage.archive.manifest_uri' \
  "$EVIDENCE_DIR/gcs/integrity-before-iam-outage.json")"
archive_hash_before="$(jq -r '.storage.archive.manifest_sha256' \
  "$EVIDENCE_DIR/gcs/integrity-before-iam-outage.json")"
archive_wal_before="$(jq -r '.storage.wal_bytes.logs' \
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
sleep 15
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
  --data-binary "@$EVIDENCE_DIR/gcs/archive-outage-log.json" >/dev/null
# The normal archive period is 300 seconds. Waiting 330 seconds proves at
# least one complete worker opportunity happened while GCS access was absent.
sleep 330
[[ "$(wait_store_leader)" == "$archive_leader" ]] \
  || die "archive leadership changed during the isolated GCS outage"
store_integrity_to "$archive_leader" \
  "$EVIDENCE_DIR/gcs/integrity-during-iam-outage.json"
jq -e \
  --arg manifest "$archive_manifest_before" \
  --arg hash "$archive_hash_before" \
  --argjson wal "$archive_wal_before" '
  .storage.archive.manifest_uri == $manifest
  and .storage.archive.manifest_sha256 == $hash
  and .storage.wal_bytes.logs >= $wal
  and .storage.archive.watermarks.logs < .signals.logs.watermark
' "$EVIDENCE_DIR/gcs/integrity-during-iam-outage.json" >/dev/null \
  || die "GCS outage changed the manifest, compacted WAL, or hid archive lag"
kubectl -n "$NAMESPACE" logs "pod/sift-store-${archive_leader}" \
  --since=7m > "$EVIDENCE_DIR/gcs/archive-worker-outage.log"
rg -F "archive attempt failed" "$EVIDENCE_DIR/gcs/archive-worker-outage.log" >/dev/null \
  || die "the leader did not report its failed archive attempt"

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
  .format_version >= 1
  and .event_count > 0
  and (.segments | length) > 0
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
kubectl create namespace "$RESTORE_NAMESPACE"
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
      cloud.google.com/gke-nodepool: acceptance-pool
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

echo ">> Sift MVP: write acceptance evidence"
jq -n \
  --arg project_id "$PROJECT_ID" \
  --arg region "$REGION" \
  --arg zone "$GKE_ZONE" \
  --arg run_id "$RUN_ID" \
  --arg bucket "$BACKUP_BUCKET" \
  --arg sift_image "$SIFT_IMAGE" \
  --arg rig_image "$RIG_IMAGE" \
  --arg manifest "$archive_manifest" \
  --arg leader_before "$failover_leader" \
  --arg leader_after "$new_leader" \
  --arg stopped_vm "$failover_node" \
  --slurpfile steady "$EVIDENCE_DIR/load/steady/summary.json" \
  --slurpfile failover "$EVIDENCE_DIR/load/failover/summary.json" \
  --slurpfile latency "$EVIDENCE_DIR/latency/summary.json" \
  --slurpfile source "$EVIDENCE_DIR/gcs/source-integrity-for-restore.json" \
  --slurpfile restored "$EVIDENCE_DIR/restore/integrity.json" '
  {
    schema:"axiom.gcp.operator.acceptance.v1",
    project_id:$project_id,
    region:$region,
    gke_zone:$zone,
    run_id:$run_id,
    backup_bucket:$bucket,
    acceptance:{
      sift:{
        schema:"axiom.gcp.sift.mvp.acceptance.v1",
        status:"passed",
        candidate:{
          sift_image:$sift_image,
          rig_image:$rig_image,
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
          remote_write_2_rejected_415:"passed",
          mcp_read_only_tools:"passed",
          mcp_host_origin:"passed",
          cross_project_denied:"passed"
        },
        load:{
          duration_seconds:1800,
          offered_items_per_second:10000,
          expected_unique_items:18000000,
          expected_logs:9000000,
          expected_metric_points:5400000,
          expected_spans:3600000,
          observed:$steady[0]
        },
        latency:$latency[0],
        failover:{
          duration_seconds:300,
          expected_unique_items:3000000,
          leader_before:$leader_before,
          leader_after:$leader_after,
          stopped_vm:$stopped_vm,
          observed:$failover[0],
          acknowledged_data_loss:0,
          auto_repair:"passed"
        },
        archive:{
          gcs_iam_outage:"passed",
          wal_preserved:"passed",
          manifest_uri:$manifest
        },
        retention:{
          day_29:"hot-query-passed",
          day_31:"cold-query-passed",
          day_181:"non-retryable-partial-rejection-passed"
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
        cleanup_evidence:"pending-exit-trap"
      }
    }
  }' > "$EVIDENCE_DIR/sift-mvp-acceptance.json"
cp "$EVIDENCE_DIR/sift-mvp-acceptance.json" "$EVIDENCE_DIR/acceptance.json"
date -u +%Y-%m-%dT%H:%M:%SZ \
  > "$EVIDENCE_DIR/kubernetes/sift-mvp-acceptance-completed-at.txt"
echo "Sift MVP cloud acceptance passed. Mandatory cleanup will now produce cleanup.json."
