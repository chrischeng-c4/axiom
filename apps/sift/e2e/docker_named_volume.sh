#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git -c core.fsmonitor=false rev-parse --show-toplevel)"
run_id="$$"
image="sift-volume-e2e:${run_id}"
volume="sift-volume-e2e-${run_id}"
container=""

for command in curl docker grep; do
  command -v "$command" >/dev/null 2>&1 || {
    echo "required command not found: $command" >&2
    exit 1
  }
done

cleanup() {
  local exit_code=$?
  if [[ -n "$container" ]]; then
    docker rm --force "$container" >/dev/null 2>&1 || true
  fi
  docker volume rm "$volume" >/dev/null 2>&1 || true
  docker image rm "$image" >/dev/null 2>&1 || true
  exit "$exit_code"
}
trap cleanup EXIT INT TERM

docker build --file "$repo_root/apps/sift/Dockerfile" --tag "$image" "$repo_root"
docker volume create "$volume" >/dev/null

start_container() {
  local suffix="$1"
  container="sift-volume-e2e-${run_id}-${suffix}"
  docker run --detach \
    --name "$container" \
    --mount "type=volume,source=${volume},target=/var/lib/sift" \
    --publish 127.0.0.1::7380 \
    "$image" \
    serve --host 0.0.0.0 --port 7380 --grpc-port 4317 >/dev/null

  local mapping
  mapping="$(docker port "$container" 7380/tcp)"
  local port="${mapping##*:}"
  base_url="http://127.0.0.1:${port}"
  for _ in $(seq 1 200); do
    if curl --fail --silent --show-error "$base_url/readyz" >/dev/null 2>&1; then
      return 0
    fi
    if ! docker inspect --format '{{.State.Running}}' "$container" 2>/dev/null | grep --quiet '^true$'; then
      docker logs "$container" >&2 || true
      echo "Sift container stopped before it became ready" >&2
      return 1
    fi
    sleep 0.1
  done
  docker logs "$container" >&2 || true
  echo "Sift container did not become ready" >&2
  return 1
}

marker="named-volume-${run_id}"
start_container first
curl --fail --silent --show-error \
  --header 'content-type: application/json' \
  --header 'x-sift-project: project-a' \
  --data "{\"resourceLogs\":[{\"resource\":{\"attributes\":[{\"key\":\"service.name\",\"value\":{\"stringValue\":\"volume-e2e\"}},{\"key\":\"deployment.environment.name\",\"value\":{\"stringValue\":\"prod\"}}]},\"scopeLogs\":[{\"logRecords\":[{\"body\":{\"stringValue\":\"${marker}\"}}]}]}]}" \
  "$base_url/v1/logs" >/dev/null

query="{\"version\":1,\"project\":\"project-a\",\"environment\":\"prod\",\"signal\":{\"kind\":\"logs\",\"filter\":{\"op\":\"text\",\"field\":\"body_text\",\"value\":\"${marker}\"}},\"limit\":10,\"mode\":\"sync\"}"
curl --fail --silent --show-error \
  --header 'content-type: application/json' \
  --header 'x-sift-project: project-a' \
  --data "$query" \
  "$base_url/api/v1/query" | grep --fixed-strings --quiet "$marker"

docker rm --force "$container" >/dev/null
container=""
docker volume inspect "$volume" >/dev/null

start_container second
curl --fail --silent --show-error \
  --header 'content-type: application/json' \
  --header 'x-sift-project: project-a' \
  --data "$query" \
  "$base_url/api/v1/query" | grep --fixed-strings --quiet "$marker"

echo "Docker named volume retained acknowledged Sift data across container replacement: ok"
