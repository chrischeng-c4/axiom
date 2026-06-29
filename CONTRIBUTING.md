# Contributing

> An agent or human should learn *what exists* and *where to act* from `ls`,
> paths, and filenames alone — without opening files. **Every file you don't
> open is a saved tool call, less context burned, and a more precise action.**

This repository is a multi-language ecosystem (Rust runtime + libraries, TS/UI,
Python conformance tests, specs, generated code, configs, docs, handoffs). Two
conventions run across all of it, both in service of the same goal — a tree an
agent can navigate cheaply and act on correctly:

1. **Authoring** — how to shape *any* artifact tree (files, paths, names) so it
   is legible from its structure alone.
2. **Ecosystem conventions** — the shapes every long-running **service** and
   every **CLI** repeats, so there is one stack to learn instead of one per
   project.

---

## Authoring principle: right-sized files, semantic paths, explicit names

This principle is **medium-agnostic** — it is about *navigability*, not any
language. Make the repository legible from its structure, so an agent can decide
where to act, and tooling can operate on the tree, without opening many files.

### The three rules

| Rule | What it asserts | Smell → fix |
|------|-----------------|-------------|
| **Right-sized files** | one coherent concern per file — one reason to exist, one reason to open. "Right-sized" ≠ "small": grain follows *access pattern* + *cohesion* (see *Balanced splitting*). | a file you open for several unrelated reasons → split |
| **Semantic paths** | the directory IS the taxonomy; the path conveys a file's role before you open it. Path ⇔ identity are mutually predictable. | you can't predict the path from what the file *is* → reclassify |
| **Explicit names** | the leaf name briefs the content; `ls <dir>/` reads as a table of contents. | a name that needs the body to grasp (`misc_cases`) → rename to the observable (`isleap_rule`) |

### Balanced splitting

Splitting must *earn its keep* — it pays off only when it improves navigation,
reviewability, reuse, or selective execution. Default to keeping.

```text
decide(file) →
  SPLIT  if ANY of:
    - a reader must search INSIDE it to reach one independent concern
    - the pieces are owned / reviewed separately
    - the pieces are executed / skipped / generated / compared independently
    - the resulting leaf names would form a useful table of contents
  KEEP   if ANY of:                                # cohesion outweighs file count
    - the pieces only make sense read together     # split → a cross-file puzzle
    - splitting would create trivial wrapper files
    - a shared setup dominates the content          # the setup, not the cases, is the file
    - the directory would only get noisier, not more discoverable
  DEFAULT → KEEP
```

Tie-break: a file needing internal headers between *unrelated* concerns wants
splitting; parts sharing one concept or one setup stay together (a single rule
over a few representative inputs → one file, as a table).

### Granularity scales with tooling

> Judgment call, called deliberately: the finer you split, the more files must
> stay mutually consistent — so **push granularity as fine as your tooling can
> keep consistent, no finer.**

```text
generated + linted  ⇒  go maximally atomic    # consistency is mechanical — you never hand-maintain the files
hand-authored       ⇒  stay coherent          # the consistency cost bites at scale — lean to cohesion
```

So a fully-tooled fixture tree goes maximally atomic (worked example below),
while hand-written source should not.

### Path grammar (a pattern, not a mandate)

```
<area>/<subject>/<concern>/<artifact>
```

- **area** — broad repo area: `tests`, `specs`, `configs`, `generated`, `docs`,
  `handoffs`, …
- **subject** — the module / feature / protocol / package / service / topic.
- **concern** — *the question this file answers, or its role*: behavior, errors,
  security, performance, integration, schema, api, migration, … (an open idea,
  not a fixed list — each tree names its own concerns).
- **artifact** — the specific case / scenario / generated unit / config concern /
  document.

Not every tree needs four levels — use the depth the tree earns. The same
grammar reads across media:

```
configs/auth/oauth_token_lifetime.yaml
specs/http/errors/malformed_header_rejected.md
generated/parser/ast/node_kinds.ts
handoffs/release/2026-05-rc1-risk-summary.md
tests/std-libs/calendar/behavior/isleap_rule.py     # worked below
```

### Organize by domain, not by tooling

Path grammar fixes the *axis* of a tree; this fixes its *first split*. Organize a
tree by the **capability domain** it covers, not by the runner or tool that
happens to execute it. The top level should tell an agent *what boundary each
subtree pins* before it opens anything — that is what turns an `ls` into a map
instead of a guess.

A test tree, for instance, splits by what it proves — never by test framework:

```text
tests/
├── <external-contract>/   parity / replacement contract against an oracle
├── <native-libs>/         your own library contracts (no external oracle)
├── <cli>/                 CLI / tool behavior, pinned on the built binary
└── governance/            meta-gates over manifests, profiles, CI policy, inventory shape
```

(`projects/mamba/tests/` is the reference adopter: `cpython/` pins the
CPython-replacement contract, `mambalibs/` the native-library contracts,
`pkgmgr/` the CLI, `governance/` the meta-gates. The same first-split-by-domain
shape applies to specs, configs, and generated trees.)

Three rules keep such a tree legible:

- **Entrypoints vs taxonomy.** A domain root holds only entrypoints and taxonomy
  directories; concrete cases live *below* the taxonomy
  (`<domain>/<subject>/<concern>/<artifact>`). A parse-only case belongs under
  that domain's fixtures, never dumped in `governance/`.
- **Wire deep artifacts explicitly.** Many build systems auto-discover
  entrypoints only at a fixed location (Cargo discovers top-level `tests/*.rs`),
  so artifacts nested under a domain must be reached through an explicit
  entrypoint or umbrella runner — not dropped as ad-hoc root files. Domain-local
  helper scripts stay under their domain (`<domain>/tools/regen_golden.py`, not
  `tests/regen_golden.py`).
- **Pair a manifest with its checker.** Where a gate is a manifest plus a
  checker, keep the two discoverably named and co-located so finding one implies
  the other.

**Migrating an existing tree toward this shape?** Lock the legacy form with a
gate that counts old-style monoliths as a *ceiling* — it can only stay flat or
shrink — so the debt is visible and one-directional while new artifacts adopt
the finer shape.

### Where it applies (scope)

Strongest for **naturally decomposable** trees — independent test fixtures,
config entries, generated units, doc/handoff files — where each artifact is
genuinely standalone and (ideally) tool-maintained. Applied with **judgment** to
cohesive hand-written code: there, one-concept-one-file can rightly *outweigh*
file count, and a language's idioms win (Rust `#[test]` fns stay in a
`mod tests`; a cohesive module groups related items). This is a guideline for
legible structure, not a mandate to shred cohesive code into wrapper files.

---

## Example: decomposing a monolith into a navigable tree

> One worked instantiation of the principle above — **not** the definition.

The clearest adopter is a fully-tooled conformance fixture tree, where a
generator emits the structure and a linter enforces it — so it goes **maximally
atomic** (one self-contained case per file) and the path is the grammar made
concrete: `<bucket>/<subject>/<dimension>/<case>`, where `dimension` is the
*concern* for tests (surface · behavior · errors · bench · real_world ·
security).

### Before → after

**Before** — one file, eight unrelated behaviors mixed together:

```
std-libs/calendar/behavior.py        # 8 cases, one big file
```

**After** — the concern is a directory; each case is a named leaf:

```
std-libs/calendar/behavior/
  isleap_rule.py                 # leap-year rule (a few representative years, one table)
  leapdays_counts.py
  monthrange_february.py
  setfirstweekday_roundtrip.py
  ...
```

`ls behavior/` is now the spec — a reader jumps straight to the one case they
need, and no coverage is lost. Note `isleap_rule.py` keeps its several input
years *together* as one table (cohesion), rather than one-file-per-input (which
would be over-splitting).

### Keep tooling and per-tree conventions with the tree

The mechanics that make a tree fully tooled — its layout spec, the
`generate → fill → lint` loop, the file template, and tree-local authoring
conventions (hermetic per-file headers, oracle verification, performance
baselines) — belong **with the tree**, not in this general guide. For the mamba
CPython suite that source of truth is
`projects/mamba/tests/cpython/conventions/FIXTURE-LAYOUT.md`: the six-dimension
table, the `fixture_gen` → fill → `fixture_lint` loop, PEP 723 `[tool.mamba]`
headers, the CPython oracle, and the perf-baseline flow.

---

## Service archetype: HA, HTTP/2 + OpenAPI, k8s-native

> The ecosystem's long-running network services share one shape. A new service
> of this kind — a broker, a store, an orchestrator, an index — **copies the
> archetype** rather than reinventing its transport, consensus, deployment, or
> gates. The wins are concrete: one transport stack to learn, one HA engine to
> harden, one set of gate files an agent can find by `ls` before opening
> anything.

Reference instantiations: **`keep`** (KV / claim-check store), **`relay`**
(durable-log broker), **`lumen`** (search / dedup index). `loom` is a
control-plane variant on the same runtime (greenfield — it shares the stack but
its governance files are not yet grown; treat it as aspirational, not as a
template for the gate files).

### The shared service kit — compose these libs, do not hand-roll

A service is mostly *wiring* four shared libs together. **Adopting them is
mandatory** — a service that re-implements any of these is a defect, not a
variation:

| Lib | Role |
|-----|------|
| **`libs/raft-core`** | the step-driven raft **consensus core** (serde-only; replaced openraft). |
| **`libs/raft-host`** | the raft **host**: h2c peer transport, the single apply loop, **snapshot + log compaction** (the "backup layer"), read-your-write `propose`, and **k8s topology + auto-mode** (`cluster::ClusterTopology::from_env` + `replica_mode`). A service supplies a `RaftStateMachine` (`apply`/`snapshot`/`restore`/`applied_index`) and gets HA + backup for free. |
| **`libs/h2c`** | the **transport**: `h2c::serve` (server, feature `server`) + `h2c_client`/`H2cPool` (client). |
| **`libs/cli-std`** | the **standard CLI** commands (`llm` / `upgrade` / `issue`). |

**k8s-native auto-mode + discovery.** A service defaults to single-node and turns
on raft **only when the StatefulSet scales out** — `raft_host::cluster::
replica_mode()` is `true` when `REPLICAS_PER_SHARD > 1` (a downward-API value). So
`<svc> serve` needs **no flags or cluster env** for local/single-node dev; k8s
scaling flips it to replica mode automatically, with node id / membership / peers
derived from the downward API by `ClusterTopology::from_env` (a local
`<SVC>_PEERS=host:port,…` override runs a multi-node group on one machine). Do not
hand-roll the pod-ordinal or peer-DNS math.

A service is not "done" until it satisfies every row:

| Dimension | Requirement | Reference / gotcha |
|-----------|-------------|--------------------|
| **Shape** | Workspace member that is **both `lib` and `bin`** — embeddable as a crate, runnable as a server. Metadata via `version/edition/authors/license = .workspace`. | every service `Cargo.toml` |
| **Transport** | HTTP/2 cleartext (**h2c**) **+** HTTP/1.1 on **one port**, with an OpenAPI surface (`utoipa`). | Serve via **`libs/h2c`'s `h2c::serve` (feature `server`)** — built on `hyper-util` `auto::Builder`, **not `axum::serve`** (HTTP/1-only). The same crate's client side (`h2c_client`/`H2cPool`) is the in-tree client. |
| **Standard endpoints** | The same operational surface on the one port: **`/healthz`** (liveness), **`/readyz`** (readiness), **`/metrics`** (Prometheus), **`/openapi.json`** (machine OpenAPI), **`/docs`** (Swagger UI). Probes + scrape **depend** on these, so they stay auth-exempt and always-on. | `lumen` is the reference (all five). The contract is reachable three ways — **`<cli> spec`** (offline) ≡ **`/openapi.json`** (served) ≡ **`/docs`** (browsable) — one OpenAPI, three access paths. |
| **OpenAPI client codegen** | Generate typed clients from the service's **own** OpenAPI via **`libs/openapi-codegen`** (`cclab-openapi-codegen`) — **never** hand-rolled or an external tool. Expose it on the CLI: `<cli> spec gen --lang ts\|py\|rust --out <dir>`. Adopters get a typed client with **no external codegen step**. | `lumen spec gen` is the reference; the polyglot core (ts/py/rust) was extracted so any CLI composes it. |
| **HA / consensus** | **Mandatory for any stateful service:** sharded, strongly-consistent state replicated with **`libs/raft-core`** driven by **`libs/raft-host`** — the replication path **wired** (a `RaftStateMachine` impl), not a DTO-only / "later slice" stub. Follower tails the leader over h2c; snapshot/compaction comes from the host. | Use `raft-core`+`raft-host`, **not `openraft`** and **not** a hand-rolled driver. The raft path may be a Cargo feature (`keep`); `lumen` is the reference adopter (`EngineSm`). |
| **Core neutrality** | Keep domain/payload knowledge **out of the transport core** where feasible, so the core is reusable. | `relay` carries an opaque JSON body and "knows nothing about workflows" (#120). |
| **Deploy** | `Dockerfile` (+ `.release` / `.bench` variants); `<cli> dockerfile render`; **k8s-native** kustomize tree (`k8s/base` + `k8s/overlays`); `<cli> k8s crd/operator/instance`; StatefulSet identity/peers from the **downward API**; an `HA.md`. | `keep/k8s`, `lumen k8s` (+ `operator` feature). `loom` currently ships only a flat `deploy/k8s.yaml` — that's the un-grown form, not the target. |
| **SDD-managed** | `aw.toml` + `tech-design/` + `SPEC-MANAGED` / `HANDWRITE` markers in source. Drive changes through the `aw` lifecycle. | see the SDD rules in `CLAUDE.md`. |
| **EC gates** | Evidence-contract gates wired below. | see *EC gates* next. |
| **CLI** | The bin ships `llm` / `upgrade` / `issue`. | see the *CLI convention* below. |

### Transport — h2c + OpenAPI on one port

`axum::serve` speaks HTTP/1 only. To serve h2c (HTTP/2 cleartext, no TLS — the
in-cluster default) alongside HTTP/1.1 on a single port, build the connection
with `hyper-util`'s `auto::Builder`. In-tree clients are `reqwest` over h2c
(rustls, no openssl). Describe the surface with `utoipa` so the OpenAPI doc is
generated from the handlers, never hand-maintained.

### OpenAPI client codegen — typed clients from the spec

Because the OpenAPI doc is the source of truth, the typed clients adopters use
are **generated from it**, never hand-written and never produced by an external
tool. The shared `libs/openapi-codegen` (`cclab-openapi-codegen`) is the polyglot
core — a language-neutral IR feeding per-language emitters (TypeScript: types +
fetch/axios client + TanStack Query hooks; Python: pydantic + generated
sync/async HTTP/2 runtime that speaks h2c for `http://` and ALPN h2 for
`https://`; Rust: serde + reqwest). A service **composes** it
behind a CLI verb:

```
<cli> spec gen --lang ts|py|rust --out <dir>
```

so an adopter goes from "the service is up" to "a typed client in my language"
with no external codegen step. Reference: `lumen spec gen` (feeds the binary's
own `openapi_json()` into `cclab_openapi_codegen::generate`). Do **not** add a
second codegen path — extend the shared crate (a new emitter / capability) so
every service benefits.

### Deploy artifacts — image, cluster API, operator, instance

Image construction is not a Kubernetes subcommand. Every k8s-native service CLI
ships:

```
<cli> dockerfile render --variant source|release [--version <tag>] [--out <path-or-dir>]
```

`source` renders the workspace-build Dockerfile used by dev/CI; `release`
renders the small production image that fetches a published `<project>@<version>`
binary. The same image artifact feeds compose, kind, and real registries.

Kubernetes output is split by lifecycle layer:

```
<cli> k8s crd render [--out <path>]
<cli> k8s operator render [--namespace <ns>] [--out <path-or-dir>]
<cli> k8s operator run
<cli> k8s instance render --profile dev|staging|prod|template [--out <path-or-dir>]
```

`crd` owns the cluster-scoped API. `operator` owns the control plane, normally in
an independent namespace such as `<svc>-system`; `operator run` is the controller
process/container entrypoint. `instance` renders the app-namespace custom
resource that an application team applies next to the app it integrates with.

### Standard endpoints — one operational surface, one contract three ways

Every service exposes the same five endpoints on its one port, so an operator,
a probe, a scraper, or an agent finds the same surface without per-service
lookup:

```
/healthz       liveness  — k8s livenessProbe
/readyz        readiness — k8s readinessProbe (gates traffic)
/metrics       Prometheus exposition — the ServiceMonitor scrape target
/openapi.json  the machine-readable OpenAPI (utoipa-generated)
/docs          Swagger UI — the human-browsable render of that OpenAPI
```

`/healthz` and `/readyz` are **not optional**: the StatefulSet/Deployment the
operator renders points its probes at them, and `/metrics` is the scrape target —
so all three stay **auth-exempt and always-on** (a 401 on `/healthz` fails the
probe). They are the intersection of the *Transport* and *Deploy* rows.

The integration contract is reachable **three equivalent ways** — same OpenAPI,
different access path:

| Path | Context |
|------|---------|
| **`<cli> spec [--format openapi\|openapi-yaml\|json-schema] [--shapes] [--fields]`** | **offline** (no server) — for agents, CI, codegen input. The offline twin of `/openapi.json`; `spec gen` is its codegen sub-verb. |
| **`/openapi.json`** | served, machine-readable |
| **`/docs`** | served, human-browsable (Swagger UI) |

So `spec` belongs to **this** archetype (a service has an OpenAPI to emit), **not**
`cli-std` — which carries only the universal `llm`/`upgrade`/`issue` that *every*
CLI ships, services and non-services alike. `lumen` is the reference for all of it.

### HA — `raft-core`, sharded and strongly consistent

**HA is mandatory for any stateful service — not a "wire it later" slice.** A
DTO / cluster-state surface (`/debug/cluster`) without the `raft-core` replication
path actually wired does **not** satisfy the HA row; the service is not
production-ready until writes are ordered and replicated through `raft-core`.

State is **sharded** and **strongly consistent**, replicated by the shared
`libs/raft-core` engine (serde-only; it replaced `openraft` across the
ecosystem). The leader owns writes; followers tail it over h2c. Node identity
and the peer set come from the Kubernetes **downward API** on a StatefulSet —
nothing is hand-configured per replica. Gate consensus behind a Cargo feature
only when a single-node mode is a legitimate deployment (e.g. `keep`).

### EC gates — `vat`-driven, evidence under `external-contracts/`

Every service carries a fixed set of **evidence-contract (EC) gate files**, each
`SPEC-MANAGED` and pointed at a contract under `external-contracts/`. They are
named so `ls` tells you what is enforced before you open anything:

- **`vat.toml`** — the EC test runner; backs integration tests with **real
  services / emulators** (never hand-rolled mocks) and lists the setup steps.
- **`meter*.toml`** — performance/efficiency/stability EC gates
  (`meter.toml` + `meter-<scope>-<dimension>.toml`, e.g.
  `meter-keep-performance.toml`, `meter-search-{efficiency,stability}.toml`),
  run via `vat run meter-*`; evidence under
  `external-contracts/competitor-performance/`.
- **`guard*.toml`** — the security EC gate (`guard-<scope>-security.toml`),
  run via `vat run guard-security`; evidence under
  `external-contracts/security-hardening/`.

A breach is a non-zero-exit finding that blocks the `aw td merge` gate. Keep
these files `SPEC-MANAGED` — regenerate them from their contract; do not
hand-edit the `AW-EC-TOOL` block.

## CLI convention: every CLI ships `llm`, `upgrade`, `issue`

> Every binary a human or agent runs must answer three questions without prior
> knowledge: *how do I drive this?* (`llm`), *am I current?* (`upgrade`), and
> *what's broken — what's already reported, and how do I file it?* (`issue`).
> These three are
> **mandatory** on every CLI surface in the ecosystem (`mamba`, `jet`, `lumen`,
> `vat`, `aw`/`cclab`, and any new tool) — the agent-facing contract that lets a
> tool an agent has never seen self-onboard, self-update, and file a structured
> defect using the binary alone.

A new CLI is not "done" until all three appear in `--help`.

**Positionals are for subcommands and a verb's one primary object; structured
parameters are flags.** A positional names a *subcommand* (`jet build`, `jet
issue search`) or the verb's single natural object — an id, a query, or free-form
prose (`issue view <n>`, `issue search [query]`, `issue create [msg…]`). Anything
the command *selects or configures* — a topic, title, version, tag, state — is a
named flag (`--topic`, `--title`, `--version`, `--state`), so the grammar stays
unambiguous as the surface grows.

| Subcommand | Signature | Contract |
|------------|-----------|----------|
| `llm` | `<cli> llm [--topic <topic>] [--format md\|json]` | Offline (no server/network) docs that teach an agent to drive the tool. Topic via `--topic` (not positional); default `outline` (a topic map); per-tool topics follow its domain. Markdown default, `--format json` for machine-readable. |
| `upgrade` | `<cli> upgrade [--version <tag>] [--check]` | Self-update to the latest `<project>@*` GitHub release. `--check` = report whether newer exists, no install; `--version` = pin a tag. |
| `issue` | `<cli> issue search [query]` · `view <n>` · `create [--title <t>] [msg…]` | Read **and** write the tool's issues on the tracker. `search` finds this tool's issues (filtered to `project:<name>`; omit the query to list recent), `view <n>` prints one, `create` files a structured issue (auto-attaching `--version` + OS/arch + context, tagged with the `project:<name>` label). |

Implementation notes not obvious from the signature:

The logic for all three lives in the shared **`libs/cli-std`** crate (`cli_std`),
which is **clap-agnostic**: each CLI keeps its own clap registration — so it owns
the convention's flag shape (`--topic`, not a positional) — and delegates the
behavior to the crate, parameterized by a `cli_std::ToolInfo` it fills from its
own `build.rs` stamps (project, repo, target triple, version, git sha). A tool
provides only its clap surface, that `ToolInfo`, and (for `llm`) its topic list;
the crate does the rest. The network paths (`upgrade` install, `issue`
search/view/create) sit behind cli-std's `online` feature — enable it in release
builds. Reference adopter: `projects/jet` (keep/loom/lumen still on the
deprecated `report-issue` shim, migrating to `issue`).

- **`llm`** — `cli_std::llm::render(project, version, topics, topic, format)`. The
  tool supplies `&[cli_std::llm::Topic]` (`id`/`summary`/`body` — the one in-code
  source of truth) and cli-std renders the `outline` topic map + the
  standard-command footer. Pure offline; always builds.
- **`upgrade`** — `cli_std::upgrade::run(&tool, opts)`: the in-binary form of
  `projects/<project>/install.sh` — detect target (`<arch>-<os>`) → download the
  matching `*.tar.gz` → verify `.sha256` → **atomically** replace the binary.
  Fail loudly on checksum mismatch; never leave a half-written binary.
- **`issue`** — `cli_std::issue::{search, view, create}`. `search`/`view` are
  read-only GitHub API GETs (tokenless on public repos); `create` submits via the
  API when `GITHUB_TOKEN` is set, else prints a pre-filled `issues/new` URL. Pass
  the tracker's `project:<name>` label in `CreateOptions.label` so it is applied
  on submit **and** carried into the URL fallback's `&labels=`; `search` filters
  to that same label. The group is named `issue` (**not** `report`), leaving
  domain `report` verbs (`jet report` = HTML **test** reports) untouched.

## Releasing: each project owns its version and `<project>@X.Y.Z` tag

Projects release **independently**. Each project crate sets its **own**
`version` in its `Cargo.toml` (not `version.workspace`), so bumping one project
never version-bumps the others. A release ships via the project's
`.github/workflows/<project>-release.yml`, triggered by pushing a matching tag:

```
<project>@X.Y.Z        # e.g. lumen@0.4.4, vat@0.3.62
```

To cut a release:

1. Bump the project crate's own `version` (e.g. `projects/lumen/Cargo.toml`
   `version = "0.4.4"`), regenerate `Cargo.lock`, and commit
   `release(<project>): <project>@X.Y.Z`.
2. Merge to `main`.
3. Tag that commit `<project>@X.Y.Z` and push the tag — the workflow builds the
   per-target artifacts (`<project>-<target>.tar.gz` + `.sha256`) and publishes
   the GitHub release that `<project> upgrade` consumes.

Do **not** bump `[workspace.package].version` to release one project — that
version is shared, so it would bump every crate still inheriting it. A few
crates still inherit it (`version.workspace = true`); when a project starts
releasing independently, give it its own `version` first (matching its last
published release).
