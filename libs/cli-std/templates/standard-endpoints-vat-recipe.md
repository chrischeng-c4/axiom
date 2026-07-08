# Standard-endpoints vat recipe

Copy-usable EC recipe for a project's `standard_endpoints` archetype trait
baseline capability (`CONTRIBUTING.md` § "Standard endpoints — one
operational surface, one contract three ways", trait table row
`standard_endpoints` → baseline cap `standard-operational-endpoints`). Follows
the meter/guard recipe format already in `apps/relay/vat.toml` /
`apps/keep/vat.toml`: vat owns the build + workspace isolation, a plain
curl-based probe script is the assertion, and one `aw.toml` `ec.*` binding
wires it into `aw ec check` / `aw health --verify-ec`.

Home: `libs/cli-std/templates/` (not `apps/vat/`) — vat itself has no
`docs/`/`templates/` convention today (checked: `apps/vat/README.md` has
no such section), and this recipe is a cross-project adoption artifact for
*consumers* of vat, not part of vat's own source tree. `libs/cli-std` is
already where the sibling `chainable`/`llm`/`upgrade`/`issue` gate templates
for the archetype-as-traits work live, so a project standardizing on the
traits finds both gate templates in one place.

Substitute `<name>` (the project directory under `projects/`), `<bin>` (the
binary that serves `/healthz` etc.), and `<port>` (the port `<bin> serve`
binds) below.

## 1. `projects/<name>/vat.toml` — probe the operational surface

```toml
[[services]]
id = "svc"
cmd = ["target/release/<bin>", "serve"]
ready_http = "http://127.0.0.1:<port>/healthz"
timeout_s = 60

[[runners]]
id = "standard-endpoints"
requires = ["svc"]
cmd = [
  "sh",
  "-c",
  "curl -fsS http://127.0.0.1:<port>/healthz && curl -fsS http://127.0.0.1:<port>/readyz && curl -fsS http://127.0.0.1:<port>/metrics | grep -q '^# HELP' && curl -fsS http://127.0.0.1:<port>/openapi.json | python3 -c 'import json,sys; json.load(sys.stdin)'",
]
timeout_s = 60
```

`/healthz` and `/readyz` are asserted by plain `curl -fsS` (non-2xx fails the
runner); `/metrics` is asserted to be real Prometheus exposition format
(`grep -q '^# HELP'`); `/openapi.json` is asserted to be parseable JSON. All
four stay auth-exempt and always-on per the archetype contract, so no
credentials belong in this probe. If the project already has a `vat.toml`
(e.g. it also runs `meter-*`/`guard-*` EC gates), add the `[[runners]]` block
to it rather than creating a second file — one `vat.toml` per project.

## 2. `projects/<name>/aw.toml` — the `ec.*` binding

```toml
[ec.standard_endpoints]
tool = "vat"
dir = "standard-endpoints"
command = "cd projects/<name> && ../../target/debug/vat run standard-endpoints"
```

`command` is the resolved form `aw ec check`'s tier-1b binding validator
expects (`src/cli/chain.rs::parse_vat_runner_invocation` — the literal shape
`cd <dir> && <path-to-vat> run <runner-id>`); `dir`/`tool` are set for parity
with the other `ec.*` rows (`ec.efficiency`, `ec.security`) in the same file,
but `command` is what actually resolves since it is present.

## 3. `external-contracts/` — the evidence layout

Add the TD stub that `aw ec gen` reads to populate the EC inventory, at
`projects/<name>/external-contracts/standard-operational-endpoints/behavior/endpoints-probe.md`
(same `<capability-id>/<category>/<file>.md` layout as
`apps/relay/external-contracts/competitor-performance/efficiency/perf-gate.md`):

```markdown
---
id: <name>-standard-operational-endpoints-ec
summary: <name> exposes the standard /healthz /readyz /metrics /openapi.json operational surface on its one port, probed inside a vat workspace.
fill_sections: [e2e-test, tool-contract]
---

# EC: Standard Operational Endpoints

## External Contract
<!-- type: e2e-test lang: yaml -->

\`\`\`yaml
e2e_tests:
  - id: <name>-standard-endpoints-vat-probe
    capability_id: standard-operational-endpoints
    claim_id: standard-endpoints-reachable
    contract_id: <name>-standard-endpoints-probe
    category: behavior
    test_path: projects/<name>/tests/behavior_standard_endpoints_probe.rs
    command: "cd projects/<name> && ../../target/debug/vat run standard-endpoints"
    assertions:
      - "/healthz and /readyz return 2xx while the service is up (auth-exempt, always-on)."
      - "/metrics returns Prometheus exposition format (`# HELP`/`# TYPE` lines)."
      - "/openapi.json returns machine-readable JSON that parses."
\`\`\`

## Tool Contract
<!-- type: tool-contract lang: yaml -->

\`\`\`yaml
tool_contracts:
  - id: <name>-standard-endpoints-vat
    tool: vat
    manifest: vat.toml
    category: behavior
    command: "cd projects/<name> && ../../target/debug/vat run standard-endpoints"
\`\`\`
```

Run `aw ec gen --project <name>` after adding the stub to project it into
`projects/<name>/aw.toml`'s generated `[[aw.ec.generated.cases]]` /
`[[aw.ec.generated.tool_manifests]]` blocks (the `AW-EC-BEGIN`/`AW-EC-END`
markers); the hand-written `[ec.standard_endpoints]` table from step 2 is the
cross-CLI binding `aw ec check`/`aw health` dispatch through, independent of
that generated inventory.

## Validate

`aw ec check --project <name>` from the repo root exercises tier-1b (#921):
it parses the resolved `ec.standard_endpoints` command, opens
`projects/<name>/vat.toml`, and confirms the `standard-endpoints` runner id
exists there — a missing/misspelled runner id is a blocker; a `cmd[0]`
binary that isn't built yet is warn-only ("buildable, not built"). This is a
static parse check, not a live probe: `aw health --project <name>
--verify-ec` (or `vat run standard-endpoints` directly, once `<bin>` is
built) is what actually curls the running service.
