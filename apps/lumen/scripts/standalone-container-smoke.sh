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
#   bash apps/lumen/scripts/standalone-container-smoke.sh bind

set -euo pipefail

MODE="${1:-bind}"
if [[ "$MODE" != "bind" ]]; then
  echo "Usage: $0 bind" >&2
  exit 1
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
