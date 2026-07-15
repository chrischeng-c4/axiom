#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

legacy=(
  operator raft-host h2c server-core tcp-server http-server
  service-durability service-metrics service-tls claimtoken
)
canonical=(
  service-k8s raft-runtime transport-h2c server-lifecycle server-tcp server-http
  storage-durable metrics-prometheus peer-tls claim-token
)

for index in "${!legacy[@]}"; do
  old="libs/${legacy[$index]}"
  new="libs/${canonical[$index]}"
  if [[ -e "$old" ]]; then
    printf 'retired library directory still exists: %s\n' "$old" >&2
    exit 1
  fi
  if [[ ! -f "$new/Cargo.toml" ]]; then
    printf 'canonical library manifest is missing: %s/Cargo.toml\n' "$new" >&2
    exit 1
  fi
  if ! rg -Fq "name = \"${canonical[$index]}\"" "$new/Cargo.toml"; then
    printf 'canonical package identity is missing from %s/Cargo.toml\n' "$new" >&2
    exit 1
  fi
done

migration_td='apps/agentic-workflow/tech-design/semantic/rename-shared-service-libraries-around-semantic-responsibility-t.md'
fixture='apps/agentic-workflow/tests/fixtures/shared_service_library_names/assert_semantic_names.sh'
legacy_content='(^|[^[:alnum:]_-])libs/(operator|raft-host|h2c|server-core|tcp-server|http-server|service-durability|service-metrics|service-tls|claimtoken)|\bh2c::|\bclaimtoken::|\b(raft_host|server_core|tcp_server|service_durability|service_metrics|service_tls)::|^\s*(operator|h2c|claimtoken|raft-host|server-core|tcp-server|http-server|service-durability|service-metrics|service-tls)\s*=\s*\{|\bpackage\s*=\s*"(operator|h2c|claimtoken|raft-host|server-core|tcp-server|http-server|service-durability|service-metrics|service-tls)"|(^|[[:space:]])-p[[:space:]]+(operator|h2c|claimtoken|raft-host|server-core|tcp-server|http-server|service-durability|service-metrics|service-tls)([[:space:]]|$)|name = "(operator|h2c|claimtoken|raft-host|server-core|tcp-server|http-server|service-durability|service-metrics|service-tls)"'

if rg -n --pcre2 "$legacy_content" . \
  --glob '!target/**' \
  --glob '!.git/**' \
  --glob "!$migration_td" \
  --glob "!$fixture"; then
  printf 'retired shared-library identity remains in active source or docs\n' >&2
  exit 1
fi

legacy_filename='(raft-host|server-core|tcp-server|http-server|service-durability|service-metrics|service-tls|claimtoken|libs-h2c|h2c-libs-h2c|libs-operator|operator-libs-operator)'
if rg --files \
  libs/service-k8s libs/raft-runtime libs/transport-h2c \
  libs/server-lifecycle libs/server-tcp libs/server-http \
  libs/storage-durable libs/metrics-prometheus libs/peer-tls libs/claim-token \
  | rg "$legacy_filename"; then
  printf 'retired shared-library identity remains in a canonical library filename\n' >&2
  exit 1
fi

rg -Fq 'The prefix is the map:' README.md
rg -Fq '[service-k8s](libs/service-k8s/Cargo.toml)' README.md
rg -Fq '[transport-h2c](libs/transport-h2c/Cargo.toml)' README.md
rg -Fq '[claim-token](libs/claim-token/Cargo.toml)' README.md
rg -Fq '### Shared-library naming grammar' CONTRIBUTING.md
rg -Fq '`server-*`' CONTRIBUTING.md
rg -Fq '`service-*`' CONTRIBUTING.md
rg -Fq 'Directory, Cargo package, and Rust crate identities move' README.md

cargo metadata --no-deps --format-version 1 >/dev/null
