#!/usr/bin/env bash
# Standalone container smoke test for Lumen.
#
# Validates container listener binding against host loopback:
# 1. Image default: without LUMEN_HOST, the container listens on 0.0.0.0 and answers
#    requests forwarded from a host-loopback published port.
# 2. Negative control: with explicit LUMEN_HOST=127.0.0.1, the container starts and logs
#    its 127.0.0.1:7373 listener while the host-published port remains unreachable.
#
# Usage:
#   bash apps/lumen/scripts/standalone-container-smoke.sh bind|durable

set -euo pipefail

MODE="${1:-bind}"
if [[ "$MODE" != "bind" && "$MODE" != "durable" ]]; then
  echo "Usage: $0 bind|durable" >&2
  exit 1
fi

if [[ "$MODE" == "durable" ]]; then
  # DURABLE-CONTRACT-BEGIN
  [[ "${LUMEN_STANDALONE_DURABLE_IMAGE:-}" =~ ^ghcr\.io/chrischeng-c4/lumen@sha256:[0-9a-f]{64}$ ]] || {
    echo "ERROR: LUMEN_STANDALONE_DURABLE_IMAGE must be an exact GHCR root digest" >&2
    exit 1
  }
  OLD_IMAGE="ghcr.io/chrischeng-c4/lumen@sha256:59a85c96d807428c424ec8889ac830b14e02869da49c4b44ae12dcce3786d03d"
  ID_SUFFIX="$(date +%s)_$$_${RANDOM}"
  VOLUME="lumen-smoke-durable-${ID_SUFFIX}"
  OLD_CONTAINER="lumen-smoke-durable-old-${ID_SUFFIX}"
  CANDIDATE_CONTAINER="lumen-smoke-durable-candidate-${ID_SUFFIX}"
  REPLACEMENT_CONTAINER="lumen-smoke-durable-replacement-${ID_SUFFIX}"
  TEMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/lumen-durable.XXXXXX")"
  CREATED_VOLUME=0
  CREATED_OLD=""
  CREATED_CANDIDATE=""
  CREATED_REPLACEMENT=""
  # shellcheck disable=SC2329 # invoked by the EXIT trap below
  cleanup_durable() {
    local exit_code=$?
    local cleanup_failed=0
    trap - EXIT
    for container in "$CREATED_OLD" "$CREATED_CANDIDATE" "$CREATED_REPLACEMENT"; do
      if [[ -n "$container" ]]; then
        if ! docker rm -f "$container" >/dev/null 2>&1; then
          echo "ERROR: Failed to remove container $container" >&2
          cleanup_failed=1
        fi
      fi
    done
    if [[ "$CREATED_VOLUME" == 1 ]]; then
      if ! docker volume rm "$VOLUME" >/dev/null 2>&1; then
        echo "ERROR: Failed to remove volume $VOLUME" >&2
        cleanup_failed=1
      fi
    fi
    if [[ -n "$TEMP_DIR" ]] && ! rm -rf -- "$TEMP_DIR"; then
      echo "ERROR: Failed to remove temporary directory $TEMP_DIR" >&2
      cleanup_failed=1
    fi
    if [[ "$exit_code" -ne 0 ]]; then
      exit "$exit_code"
    fi
    if [[ "$cleanup_failed" -ne 0 ]]; then
      exit 1
    fi
    exit 0
  }
  trap cleanup_durable EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM

  if docker volume inspect "$VOLUME" >/dev/null 2>&1; then
    echo "ERROR: volume unexpectedly preexists" >&2
    exit 1
  fi
  docker volume create "$VOLUME" >/dev/null
  CREATED_VOLUME=1
  wait_ready() {
    local container="$1" port="$2"
    for _ in $(seq 1 60); do
      curl -fsS --noproxy '*' --connect-timeout 2 --max-time 5 "http://127.0.0.1:${port}/readyz" -o "$TEMP_DIR/ready" >/dev/null 2>&1 && return 0
      sleep 0.5
    done
    echo "ERROR: container failed /readyz" >&2
    if ! docker logs "$container" >&2; then :; fi
    return 1
  }
  has_legacy_generation() {
    local path base
    [[ -d "$1" ]] || return 1
    [[ ! -e "$1/CURRENT" ]] || return 1
    while IFS= read -r -d '' path; do
      base="${path##*/}"
      [[ "$base" =~ ^gen-[0-9]+$ ]] && return 0
    done < <(find "$1" -mindepth 1 -maxdepth 1 -type d -print0)
    return 1
  }
  request() { curl -fsS --noproxy '*' --connect-timeout 2 --max-time 5 "$@"; }
  assert_search() {
    local url="$1"
    local value="$2"
    local response="$TEMP_DIR/search-${value}"
    request -X POST "$url/collections/durable/search" -H 'Content-Type: application/json' \
      -d "{\"query\":{\"term\":{\"field\":\"tag\",\"value\":\"$value\"}},\"limit\":10}" -o "$response"
    jq -e --arg id "durable-${value}" '.total == 1 and (.hits | length) == 1 and .hits[0].external_id == $id' "$response" >/dev/null
  }
  CREATED_OLD="$OLD_CONTAINER"
  docker run -d --name "$OLD_CONTAINER" --mount "type=volume,src=$VOLUME,dst=/var/lib/lumen/data" -e LUMEN_AUTH=off -e LUMEN_SNAPSHOT_SECS=1 -e LUMEN_GRACE_SECS=1 \
    -p 127.0.0.1::7373 "$OLD_IMAGE" >/dev/null
  OLD_PORT="$(docker inspect --format '{{(index (index .NetworkSettings.Ports "7373/tcp") 0).HostPort}}' "$OLD_CONTAINER")"
  wait_ready "$OLD_CONTAINER" "$OLD_PORT"
  OLD_URL="http://127.0.0.1:${OLD_PORT}"
  request -X PUT "$OLD_URL/collections/durable" -H 'Content-Type: application/json' -d '{"fields":{"tag":{"type":"keyword"}}}' -o "$TEMP_DIR/create"
  request -X POST "$OLD_URL/collections/durable/index" -H 'Content-Type: application/json' -d '{"items":[{"external_id":"durable-first","field":"tag","value":"first"}]}' -o "$TEMP_DIR/index-first"
  assert_search "$OLD_URL" first
  for _ in $(seq 1 60); do
    if ! docker cp "$OLD_CONTAINER:/var/lib/lumen/data" "$TEMP_DIR/old-data" >/dev/null 2>&1; then :; fi
    has_legacy_generation "$TEMP_DIR/old-data" && break
    rm -rf -- "$TEMP_DIR/old-data"
    sleep 0.5
  done
  [[ -d "$TEMP_DIR/old-data" && ! -e "$TEMP_DIR/old-data/CURRENT" ]] || { echo "ERROR: no legacy checkpoint" >&2; exit 1; }
  docker rm -f "$OLD_CONTAINER" >/dev/null
  CREATED_OLD=""
  CREATED_CANDIDATE="$CANDIDATE_CONTAINER"
  docker run -d --name "$CANDIDATE_CONTAINER" --mount "type=volume,src=$VOLUME,dst=/var/lib/lumen/data" -e LUMEN_AUTH=off \
    -p 127.0.0.1::7373 "$LUMEN_STANDALONE_DURABLE_IMAGE" >/dev/null
  CANDIDATE_PORT="$(docker inspect --format '{{(index (index .NetworkSettings.Ports "7373/tcp") 0).HostPort}}' "$CANDIDATE_CONTAINER")"
  wait_ready "$CANDIDATE_CONTAINER" "$CANDIDATE_PORT"
  CANDIDATE_URL="http://127.0.0.1:${CANDIDATE_PORT}"
  docker cp "$CANDIDATE_CONTAINER:/var/lib/lumen/data/CURRENT" "$TEMP_DIR/current-first"
  python3 -c 'import pathlib,re,sys; b=pathlib.Path(sys.argv[1]).read_bytes(); sys.exit(0 if re.fullmatch(rb"generation:gen-[0-9]+\n",b) else 1)' "$TEMP_DIR/current-first"
  assert_search "$CANDIDATE_URL" first
  request -X POST "$CANDIDATE_URL/collections/durable/index" -H 'Content-Type: application/json' -d '{"items":[{"external_id":"durable-second","field":"tag","value":"second"}]}' -o "$TEMP_DIR/index-second"
  assert_search "$CANDIDATE_URL" second
  request -X POST "$CANDIDATE_URL/admin/checkpoint" -H 'Content-Type: application/json' -d '{}' -o "$TEMP_DIR/checkpoint"
  jq -e '.persisted == true' "$TEMP_DIR/checkpoint" >/dev/null
  docker cp "$CANDIDATE_CONTAINER:/var/lib/lumen/data/CURRENT" "$TEMP_DIR/current-rev"
  python3 -c 'import pathlib,re,sys; b=pathlib.Path(sys.argv[1]).read_bytes(); sys.exit(0 if re.fullmatch(rb"generation:gen-[0-9]+-rev-[1-9][0-9]*\n",b) else 1)' "$TEMP_DIR/current-rev"
  docker kill "$CANDIDATE_CONTAINER" >/dev/null
  docker rm "$CANDIDATE_CONTAINER" >/dev/null
  CREATED_CANDIDATE=""
  CREATED_REPLACEMENT="$REPLACEMENT_CONTAINER"
  docker run -d --name "$REPLACEMENT_CONTAINER" --mount "type=volume,src=$VOLUME,dst=/var/lib/lumen/data" -e LUMEN_AUTH=off \
    -p 127.0.0.1::7373 "$LUMEN_STANDALONE_DURABLE_IMAGE" >/dev/null
  REPLACEMENT_PORT="$(docker inspect --format '{{(index (index .NetworkSettings.Ports "7373/tcp") 0).HostPort}}' "$REPLACEMENT_CONTAINER")"
  wait_ready "$REPLACEMENT_CONTAINER" "$REPLACEMENT_PORT"
  REPLACEMENT_URL="http://127.0.0.1:${REPLACEMENT_PORT}"
  docker cp "$REPLACEMENT_CONTAINER:/var/lib/lumen/data/CURRENT" "$TEMP_DIR/current-replacement"
  python3 -c 'import pathlib,re,sys; b=pathlib.Path(sys.argv[1]).read_bytes(); sys.exit(0 if re.fullmatch(rb"generation:gen-[0-9]+-rev-[1-9][0-9]*\n",b) else 1)' "$TEMP_DIR/current-replacement"
  assert_search "$REPLACEMENT_URL" first
  assert_search "$REPLACEMENT_URL" second
  echo "==> Standalone container smoke (durable) passed successfully."
  exit 0
  # DURABLE-CONTRACT-END
fi

ID_SUFFIX="$(date +%s)_$$_${RANDOM}"
IMAGE_TAG="lumen-smoke-bind:${ID_SUFFIX}"
POS_CONTAINER="lumen-smoke-pos-${ID_SUFFIX}"
NEG_CONTAINER="lumen-smoke-neg-${ID_SUFFIX}"

CREATED_POS_CONTAINER=""
CREATED_NEG_CONTAINER=""
CREATED_IMAGE=""

cleanup() {
  local exit_code=$?
  trap - EXIT

  local cleanup_failed=0

  if [[ -n "$CREATED_POS_CONTAINER" ]]; then
    if ! docker rm -f "$CREATED_POS_CONTAINER" >/dev/null 2>&1; then
      echo "ERROR: Failed to remove container $CREATED_POS_CONTAINER" >&2
      cleanup_failed=1
    fi
  fi

  if [[ -n "$CREATED_NEG_CONTAINER" ]]; then
    if ! docker rm -f "$CREATED_NEG_CONTAINER" >/dev/null 2>&1; then
      echo "ERROR: Failed to remove container $CREATED_NEG_CONTAINER" >&2
      cleanup_failed=1
    fi
  fi

  if [[ -n "$CREATED_IMAGE" ]]; then
    if ! docker rmi -f "$CREATED_IMAGE" >/dev/null 2>&1; then
      echo "ERROR: Failed to remove image $CREATED_IMAGE" >&2
      cleanup_failed=1
    fi
  fi

  if [[ "$exit_code" -ne 0 ]]; then
    exit "$exit_code"
  elif [[ "$cleanup_failed" -ne 0 ]]; then
    exit 1
  fi
  exit 0
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

echo "==> Building task-local Lumen image: ${IMAGE_TAG}"
CREATED_IMAGE="$IMAGE_TAG"
docker build -f apps/lumen/Dockerfile -t "$IMAGE_TAG" .

echo "==> Running positive container: ${POS_CONTAINER}"
CREATED_POS_CONTAINER="$POS_CONTAINER"
docker run -d \
  --name "$POS_CONTAINER" \
  -e LUMEN_AUTH=off \
  -p 127.0.0.1::7373 \
  "$IMAGE_TAG"

POS_HOST_IP="$(docker inspect --format '{{(index (index .NetworkSettings.Ports "7373/tcp") 0).HostIp}}' "$POS_CONTAINER")"
POS_HOST_PORT="$(docker inspect --format '{{(index (index .NetworkSettings.Ports "7373/tcp") 0).HostPort}}' "$POS_CONTAINER")"

if [[ "$POS_HOST_IP" != "127.0.0.1" ]]; then
  echo "ERROR: Expected published HostIp to be 127.0.0.1, got '$POS_HOST_IP'" >&2
  docker logs "$POS_CONTAINER" >&2 || true
  exit 1
fi

if [[ -z "$POS_HOST_PORT" ]]; then
  echo "ERROR: Could not resolve published HostPort for positive container" >&2
  docker logs "$POS_CONTAINER" >&2 || true
  exit 1
fi

POS_URL="http://127.0.0.1:${POS_HOST_PORT}"
echo "==> Waiting for positive container /healthz on ${POS_URL}"
READY=0
for _ in $(seq 1 60); do
  if curl -fsS --noproxy '*' --connect-timeout 2 --max-time 5 "${POS_URL}/healthz" >/dev/null 2>&1; then
    READY=1
    break
  fi
  sleep 0.5
done

if [[ "$READY" -ne 1 ]]; then
  echo "ERROR: Positive container failed to answer /healthz within deadline" >&2
  docker logs "$POS_CONTAINER" >&2 || true
  exit 1
fi

echo "==> Exercising CRUD journey on positive container"
# Create collection containerbind
curl -fsS --noproxy '*' --connect-timeout 2 --max-time 5 -X PUT "${POS_URL}/collections/containerbind" \
  -H "Content-Type: application/json" \
  -d '{"fields":{"tag":{"type":"keyword"}}}'

# Index item
curl -fsS --noproxy '*' --connect-timeout 2 --max-time 5 -X POST "${POS_URL}/collections/containerbind/index" \
  -H "Content-Type: application/json" \
  -d '{"items":[{"external_id":"containerbind1","field":"tag","value":"ready"}]}'

# Search item
SEARCH_RESP="$(curl -fsS --noproxy '*' --connect-timeout 2 --max-time 5 -X POST "${POS_URL}/collections/containerbind/search" \
  -H "Content-Type: application/json" \
  -d '{"query":{"term":{"field":"tag","value":"ready"}}}')"

if [[ "$SEARCH_RESP" != *'"external_id":"containerbind1"'* ]]; then
  echo "ERROR: Search response did not contain exact external_id token: $SEARCH_RESP" >&2
  docker logs "$POS_CONTAINER" >&2 || true
  exit 1
fi
echo "==> Positive container search verified: found external_id:containerbind1"

echo "==> Running negative container with LUMEN_HOST=127.0.0.1: ${NEG_CONTAINER}"
CREATED_NEG_CONTAINER="$NEG_CONTAINER"
docker run -d \
  --name "$NEG_CONTAINER" \
  -e LUMEN_HOST=127.0.0.1 \
  -e LUMEN_AUTH=off \
  -p 127.0.0.1::7373 \
  "$IMAGE_TAG"

NEG_HOST_IP="$(docker inspect --format '{{(index (index .NetworkSettings.Ports "7373/tcp") 0).HostIp}}' "$NEG_CONTAINER")"
NEG_HOST_PORT="$(docker inspect --format '{{(index (index .NetworkSettings.Ports "7373/tcp") 0).HostPort}}' "$NEG_CONTAINER")"

if [[ "$NEG_HOST_IP" != "127.0.0.1" ]]; then
  echo "ERROR: Negative container HostIp expected 127.0.0.1, got '$NEG_HOST_IP'" >&2
  docker logs "$NEG_CONTAINER" >&2 || true
  exit 1
fi

echo "==> Verifying negative container reached running state and logged 127.0.0.1:7373 listener"
LOGS_READY=0
for _ in $(seq 1 60); do
  NEG_LOGS="$(docker logs "$NEG_CONTAINER" 2>&1 || true)"
  IS_RUNNING="$(docker inspect --format '{{.State.Running}}' "$NEG_CONTAINER" 2>/dev/null || echo "false")"

  if [[ "$IS_RUNNING" != "true" ]]; then
    echo "ERROR: Negative container exited unexpectedly before proving startup" >&2
    echo "$NEG_LOGS" >&2
    exit 1
  fi

  while IFS= read -r line; do
    if [[ "$line" == *"lumen serve listening"* && "$line" == *"127.0.0.1:7373"* ]]; then
      LOGS_READY=1
      break
    fi
  done <<< "$NEG_LOGS"

  if [[ "$LOGS_READY" -eq 1 ]]; then
    break
  fi
  sleep 0.25
done

if [[ "$LOGS_READY" -ne 1 ]]; then
  echo "ERROR: Negative container logs did not confirm 'lumen serve listening' on 127.0.0.1:7373 within deadline" >&2
  docker logs "$NEG_CONTAINER" >&2 || true
  exit 1
fi

echo "==> Proving published host port http://127.0.0.1:${NEG_HOST_PORT} is unreachable"
NEG_URL="http://127.0.0.1:${NEG_HOST_PORT}"
for _ in $(seq 1 10); do
  if curl --noproxy '*' --connect-timeout 1 --max-time 2 -sS "${NEG_URL}/healthz" >/dev/null 2>&1; then
    echo "ERROR: Negative container host port was unexpectedly reachable!" >&2
    docker logs "$NEG_CONTAINER" >&2 || true
    exit 1
  fi
  sleep 0.5
done

FINAL_STATE="$(docker inspect --format '{{.State.Running}}' "$NEG_CONTAINER" 2>/dev/null || echo "false")"
if [[ "$FINAL_STATE" != "true" ]]; then
  echo "ERROR: Negative container is not running after failed host probe (State.Running=$FINAL_STATE)" >&2
  docker logs "$NEG_CONTAINER" >&2 || true
  exit 1
fi

echo "==> Standalone container smoke (bind) passed successfully."
