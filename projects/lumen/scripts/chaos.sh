#!/usr/bin/env bash
# lumen — chaos test using toxiproxy.
#
# Implements README §9 chaos gate: 30 s network partition + 5 % packet loss
# between the test client and a running lumen instance. Records baseline
# search p99, verifies 5xx during partition, verifies p99 ≤ 2× baseline
# under packet loss.
#
# Requirements:
#   - toxiproxy-server + toxiproxy-cli  (brew install toxiproxy)
#   - curl, jq, python3
#   - a reachable lumen at $LUMEN_UPSTREAM (default http://127.0.0.1:7373).
#     If unset and the upstream isn't reachable, the script tries to launch
#     `lumen serve` locally as a background process for the duration of
#     the test.

set -euo pipefail

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LUMEN_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$LUMEN_DIR/../.." && pwd)"

LUMEN_UPSTREAM="${LUMEN_UPSTREAM:-127.0.0.1:7373}"   # toxiproxy backend
TOXIPROXY_API="${TOXIPROXY_API:-127.0.0.1:8474}"
PROXY_LISTEN="${PROXY_LISTEN:-127.0.0.1:18080}"
PROXY_NAME="lumen"
COLLECTION_ID="${LUMEN_CHAOS_COLLECTION:-users}"
BASELINE_SAMPLES="${LUMEN_CHAOS_SAMPLES:-100}"
PARTITION_SECS="${LUMEN_CHAOS_PARTITION_SECS:-30}"
PACKET_LOSS_PCT="${LUMEN_CHAOS_PACKET_LOSS_PCT:-5}"
RECOVERY_BUDGET_SECS="${LUMEN_CHAOS_RECOVERY_BUDGET_SECS:-10}"

TOXIPROXY_PID=""
LUMEN_PID=""

cleanup() {
  local ec=$?
  if [[ -n "$LUMEN_PID" ]] && kill -0 "$LUMEN_PID" 2>/dev/null; then
    kill "$LUMEN_PID" 2>/dev/null || true
    wait "$LUMEN_PID" 2>/dev/null || true
  fi
  if [[ -n "$TOXIPROXY_PID" ]] && kill -0 "$TOXIPROXY_PID" 2>/dev/null; then
    kill "$TOXIPROXY_PID" 2>/dev/null || true
    wait "$TOXIPROXY_PID" 2>/dev/null || true
  fi
  exit "$ec"
}
trap cleanup EXIT INT TERM

# ---------------------------------------------------------------------------
# 1. Boot toxiproxy-server
# ---------------------------------------------------------------------------

echo ">> 1. starting toxiproxy-server on :8474"
toxiproxy-server -host "${TOXIPROXY_API%:*}" -port "${TOXIPROXY_API##*:}" \
  >/tmp/toxiproxy.log 2>&1 &
TOXIPROXY_PID=$!

for _ in $(seq 1 30); do
  if curl -fsS "http://${TOXIPROXY_API}/version" >/dev/null 2>&1; then
    break
  fi
  sleep 0.5
done

if ! curl -fsS "http://${TOXIPROXY_API}/version" >/dev/null 2>&1; then
  echo "!! toxiproxy never came up" >&2
  cat /tmp/toxiproxy.log >&2 || true
  exit 1
fi

# ---------------------------------------------------------------------------
# 1a. Ensure a lumen upstream exists (optional, opt-in)
# ---------------------------------------------------------------------------

if ! curl -fsS "http://${LUMEN_UPSTREAM}/healthz" >/dev/null 2>&1; then
  if [[ "${LUMEN_CHAOS_AUTOSTART:-0}" == "1" ]]; then
    echo ">> 1a. lumen upstream not reachable; launching lumen serve locally"
    (cd "$REPO_ROOT" && cargo run -q -p lumen --bin lumen -- serve) \
      >/tmp/lumen-server.log 2>&1 &
    LUMEN_PID=$!
    for _ in $(seq 1 60); do
      if curl -fsS "http://${LUMEN_UPSTREAM}/healthz" >/dev/null 2>&1; then
        break
      fi
      sleep 1
    done
  fi
fi

if ! curl -fsS "http://${LUMEN_UPSTREAM}/healthz" >/dev/null 2>&1; then
  echo "!! lumen upstream ${LUMEN_UPSTREAM} not reachable" >&2
  echo "   set LUMEN_UPSTREAM=host:port or LUMEN_CHAOS_AUTOSTART=1" >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# 2. Create the proxy lumen → :18080
# ---------------------------------------------------------------------------

echo ">> 2. creating toxiproxy '${PROXY_NAME}': ${PROXY_LISTEN} → ${LUMEN_UPSTREAM}"
curl -fsS -X POST "http://${TOXIPROXY_API}/proxies" \
  -H 'content-type: application/json' \
  -d "{
    \"name\": \"${PROXY_NAME}\",
    \"listen\": \"${PROXY_LISTEN}\",
    \"upstream\": \"${LUMEN_UPSTREAM}\",
    \"enabled\": true
  }" >/dev/null

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

BASE_URL_PROXY="http://${PROXY_LISTEN}"

search_once() {
  # Print: "<http_code> <elapsed_ms>"
  local out
  out=$(curl -s -o /dev/null \
    -w '%{http_code} %{time_total}\n' \
    -X POST "${BASE_URL_PROXY}/collections/${COLLECTION_ID}/search" \
    -H 'content-type: application/json' \
    --max-time 5 \
    -d '{"query": {"match": {"field": "bio", "text": "engineer"}}, "limit": 1}' \
    || echo "000 5.0")
  # Convert seconds → ms.
  local code ms
  code=${out% *}
  local sec=${out#* }
  ms=$(python3 -c "import sys; print(int(float(sys.argv[1])*1000))" "$sec")
  echo "$code $ms"
}

percentile() {
  # $1 = percentile (e.g. 99), reads space-separated numbers on stdin.
  python3 -c "
import sys, math
xs = sorted(int(x) for x in sys.stdin.read().split() if x.strip())
if not xs:
    print(0); sys.exit(0)
p = float(sys.argv[1])
k = max(0, min(len(xs)-1, int(math.ceil(p/100 * len(xs))) - 1))
print(xs[k])
" "$1"
}

run_baseline() {
  local samples=()
  local ok=0 fail=0
  for _ in $(seq 1 "$BASELINE_SAMPLES"); do
    read -r code ms < <(search_once)
    if [[ "$code" == "200" ]]; then
      ok=$((ok+1))
      samples+=("$ms")
    else
      fail=$((fail+1))
    fi
  done
  echo "   baseline: ok=$ok fail=$fail" >&2
  printf '%s\n' "${samples[@]}"
}

add_toxic() {
  # $1 = toxic name, $2 = JSON body
  curl -fsS -X POST "http://${TOXIPROXY_API}/proxies/${PROXY_NAME}/toxics" \
    -H 'content-type: application/json' \
    -d "$2" >/dev/null
  echo "   toxic '$1' attached"
}

remove_toxic() {
  curl -fsS -X DELETE \
    "http://${TOXIPROXY_API}/proxies/${PROXY_NAME}/toxics/$1" >/dev/null
  echo "   toxic '$1' removed"
}

# ---------------------------------------------------------------------------
# 3. Baseline p99
# ---------------------------------------------------------------------------

echo ">> 3. baseline: ${BASELINE_SAMPLES} searches"
BASELINE_RAW="$(run_baseline | xargs)"
BASELINE_P99=$(printf '%s\n' "$BASELINE_RAW" | percentile 99)
echo "   baseline p99 = ${BASELINE_P99}ms"
if [[ "$BASELINE_P99" -le 0 ]]; then
  echo "!! baseline p99 came back zero — upstream or proxy misconfigured" >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# 4. Inject 30 s partition (latency=∞ ≈ huge delay)
# ---------------------------------------------------------------------------

echo ">> 4. inject ${PARTITION_SECS}s network partition (latency toxic)"
add_toxic "partition" '{
  "name": "partition",
  "type": "latency",
  "stream": "downstream",
  "toxicity": 1.0,
  "attributes": {"latency": 600000, "jitter": 0}
}'

PARTITION_START=$(date +%s)
PARTITION_5XX=0
PARTITION_SAMPLES=0
while true; do
  now=$(date +%s)
  elapsed=$((now - PARTITION_START))
  if (( elapsed >= PARTITION_SECS )); then
    break
  fi
  read -r code _ms < <(search_once)
  PARTITION_SAMPLES=$((PARTITION_SAMPLES+1))
  # 5xx OR connection timeout (000) both count as "service unavailable".
  if [[ "$code" =~ ^5 ]] || [[ "$code" == "000" ]]; then
    PARTITION_5XX=$((PARTITION_5XX+1))
  fi
done
echo "   during partition: ${PARTITION_5XX}/${PARTITION_SAMPLES} requests failed"

# ---------------------------------------------------------------------------
# 5. Expect 5xx during partition
# ---------------------------------------------------------------------------

echo ">> 5. assert 5xx during partition"
if (( PARTITION_5XX == 0 )); then
  echo "!! expected at least one 5xx/timeout during partition, got 0" >&2
  exit 1
fi
echo "   PASS — saw ${PARTITION_5XX} failures while partitioned"

# ---------------------------------------------------------------------------
# 6. Remove partition; assert recovery within budget
# ---------------------------------------------------------------------------

echo ">> 6. remove partition; assert recovery within ${RECOVERY_BUDGET_SECS}s"
remove_toxic "partition"
RECOVERY_DEADLINE=$(( $(date +%s) + RECOVERY_BUDGET_SECS ))
recovered_at=""
while [[ $(date +%s) -lt $RECOVERY_DEADLINE ]]; do
  read -r code ms < <(search_once)
  # "recovered" = 200 AND latency ≤ 2× baseline.
  if [[ "$code" == "200" ]] && (( ms <= 2 * BASELINE_P99 )); then
    recovered_at=$ms
    break
  fi
  sleep 0.5
done

if [[ -z "$recovered_at" ]]; then
  echo "!! recovery never converged within ${RECOVERY_BUDGET_SECS}s" >&2
  exit 1
fi
echo "   PASS — recovered with ${recovered_at}ms ≤ 2×${BASELINE_P99}ms"

# ---------------------------------------------------------------------------
# 7. Inject 5% packet loss; p99 ≤ 2× baseline
# ---------------------------------------------------------------------------

echo ">> 7. inject ${PACKET_LOSS_PCT}% packet loss"
TOXICITY=$(python3 -c "print(${PACKET_LOSS_PCT}/100)")
add_toxic "loss" "{
  \"name\": \"loss\",
  \"type\": \"timeout\",
  \"stream\": \"downstream\",
  \"toxicity\": ${TOXICITY},
  \"attributes\": {\"timeout\": 0}
}"

LOSS_SAMPLES=()
for _ in $(seq 1 "$BASELINE_SAMPLES"); do
  read -r code ms < <(search_once)
  if [[ "$code" == "200" ]]; then
    LOSS_SAMPLES+=("$ms")
  fi
done

remove_toxic "loss"

LOSS_RAW="${LOSS_SAMPLES[*]:-}"
LOSS_P99=$(printf '%s\n' "$LOSS_RAW" | percentile 99)
echo "   under ${PACKET_LOSS_PCT}% loss: p99=${LOSS_P99}ms (baseline ${BASELINE_P99}ms)"

if (( LOSS_P99 > 2 * BASELINE_P99 )); then
  echo "!! p99 ${LOSS_P99}ms exceeds 2× baseline ${BASELINE_P99}ms" >&2
  exit 1
fi

echo ">> chaos PASS"
