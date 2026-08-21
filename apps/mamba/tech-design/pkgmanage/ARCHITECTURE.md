# pkgmanage — architecture (as-is, 2026-07-15)

Scope: `apps/mamba/src/pkgmanage/` — the C4 (uv-like package manager) bounded context.
All refs relative to `apps/mamba/`. Fix TDs in this directory are cross-referenced by filename, not restated.

## Responsibilities

- All `mamba` package-manager CLI verbs (init/add/remove/lock/sync/install/venv/python/auth/audit/cache/hash/index/export/tree/version/workspace/pip/tool/package/shell + `pkgmgr-validate`), wired in `src/main.rs:6-28`.
- `mamba.toml` manifest and `mamba.lock` lockfile read/write with byte-identical-replay determinism.
- PyPI-compatible index access: JSON API + Simple API (PEP 503/691) with retry/backoff, ETag+TTL metadata cache, sha256-verified streaming artifact download (`pkgmgr/http.rs`).
- Dependency resolution: frozen-local-index exact-pin BFS (`lock.rs`) and live-registry resolver (`pkgmgr/resolver/`).
- `mamba build` orchestration + native `[crates.*]` force-link (`builder/`, `manifest/schema.rs`).
- uv-parity primitive library: ~110 single-concern "tick" modules under `pkgmgr/` (DDD map lives in the `pkgmgr/mod.rs` header comment — 10 logical contexts, deliberately flat on disk to preserve blame).

## Key structures & invariants

| Structure | file:symbol | Rule that must hold |
|---|---|---|
| `ManifestState` | `add.rs:ManifestState` | write-side view of mamba.toml: `[project]` deps + `[tool.mamba.sources]` only; `render()` regenerates just those tables |
| `MambaConfig` | `manifest/schema.rs:MambaConfig` | read-only build-side view of the *same* mamba.toml (`[crates]`,`[expose]`,`[build]`,`[paths]`); disjoint from `ManifestState` |
| `ResolvedDep`/`SourceMeta` | `add.rs` | source is `Default` \| `DirectFile{path}` \| `MambaProvider{...}`; carries optional `sha256`+`url` so sync can fetch without re-resolving |
| `Pin`/`Resolved` | `lock.rs:Pin` | frozen-index path demands `NAME==VERSION` (`Pin::parse` rejects unpinned) |
| `Lockfile`/`Package`/`SourceRef` | `pkgmgr/lockfile/mod.rs` | resolver schema; `MAX_SUPPORTED_FORMAT_VERSION` gate; `format_version = 1` emitted by `lock.rs:render_lockfile` |
| user-lock adapter | `lockfile/mod.rs:parse_user_lockfile` | adapts on-disk mamba.lock into the pkgmgr schema for read-only verbs (export/tree); `url` field wins over legacy `source` |
| `IndexClient`/`PackageMetadata`/`IndexError` | `pkgmgr/types.rs` | one client handle per verb invocation; `auth_header` from `auth.rs:authorization_for_url` |
| `Resolver` | `pkgmgr/resolver/mod.rs:Resolver` | eager BFS (NOT PubGrub yet — see hazards); name-sorted deterministic `ResolvedGraph`; yanked + marker filtering |
| frozen index layout | `index.rs` header | `<INDEX>/<pep503-name>/<version>/<file>.whl` + optional `metadata.toml` `requires=[...]` for transitive edges |
| `MambaProviderPackage` | `provider.rs:catalog` | first-party providers (`mamba-httpx-compat`); `locked_mamba_package` verifies lock metadata verbatim against catalog |

Cross-cutting invariants:
- **Determinism**: replaying add/lock yields byte-identical mamba.toml/mamba.lock (sorted deps; `lock.rs:compute_input_hash` = sha256 over sorted direct deps).
- **No partial writes**: resolve fully before writing; `add.rs:atomic_write` for both files; `index.rs` parses every wheel filename before creating output dirs.
- **Source policy**: frozen index (`--index`/`MAMBA_FROZEN_INDEX`) > offline pin > explicit registry (`--index-url`/`MAMBA_INDEX_URL`) > fail fast. **No implicit pypi.org default** — contract in `mamba-pkgmanage-remove-implicit-pypi-default-from-add-lock.md`.
- **sync never rewrites the lockfile**; second run emits `no_op` to stderr, exit 0.
- **Every artifact download is sha256-verified** (`http.rs:download_artifact`); sync aborts all in-flight tasks on first failure.

## Control flow

1. `mamba add SPEC` → `add.rs:cmd_add`: `--provider mamba`? → `provider.rs`; `looks_like_wheel_path`? → local-wheel sha256; else `resolve_dep` (frozen index → `--offline`+pin → `--index-url` via `IndexClient::fetch_metadata` + `pick_best_wheel` PEP 425 scoring) → `ManifestState::upsert_dependency` → render manifest+lockfile → two atomic writes.
2. `mamba lock` → `lock.rs:cmd_lock`: parse manifest → provider deps + registry deps → frozen: `resolve_transitive` (metadata.toml BFS, exact pins) | live: `resolve_via_pypi` (`parse_requirement` → `Resolver::resolve` over `pubgrub_glue.rs:IndexClientProvider`, host `MarkerEnv`, `pick_artifact_url` per node) → `render_lockfile` → `--check` compares bytes, else atomic write.
3. `mamba sync` → `sync.rs:cmd_sync`: parse mamba.lock → `plan_install` diff vs `.venv/site-packages` → `download_and_verify_parallel` (tokio `JoinSet` + `Semaphore`, jobs default 8) → `materialize_stub` / `materialize_mamba_provider` (stub `__init__.py` + dist-info marker — NOT real wheel extraction, deliberate MVP per header).
4. `mamba run FILE` → `run.rs:preflight` before compilation (`src/main.rs:1104`): inside a project, unsynced env fails; synced env injects site-packages via `PYTHONPATH` on the current process only; outside → `Mode::Legacy` untouched.
5. `mamba install` (tools) → `install.rs`: frozen-index only, materializes into `$MAMBA_TOOLS_DIR`/`~/.local/share/mamba/tools`; `--list`/`--uninstall`; idempotent `no_op`.
6. `mamba index build` → `index.rs:cmd_build`: collect wheels → parse all filenames → `copy_if_changed` into PEP 503 layout.
7. `mamba pkgmgr-validate` → `validate.rs`: spawns the current binary in self-cleaning `ScratchDir` tempdirs per `validation/profiles/package_manager.toml` family; JSON summary per `[runner_contract]`.
8. Metadata fetch → `http.rs:fetch_metadata`: JSON API `/pypi/{name}/json` → on `NotFound` fall back to `simple/{name}/` with PEP 691 content negotiation; ETag revalidation + TTL cache; 429/5xx exponential backoff + jitter.

## Known hazards

- **Lossy manifest rewrite** — `add.rs:ManifestState::render` regenerates mamba.toml from `[project]`+`[tool.mamba.sources]` only. `mamba add/remove` on a manifest carrying `[crates.*]`/`[build]`/`[paths]` (the `manifest/schema.rs` view) or comments silently destroys them.
- **Resolver is not PubGrub** — `pkgmgr/resolver/mod.rs` HANDWRITE header: eager BFS; no backtracking; conflicts collapse into `no_compatible_version` without a rich trace despite the `pubgrub_glue` name.
- **No conflict detection on frozen path** — `lock.rs:resolve_transitive` keys `seen` by `name==version`; two different pins of one package both land in mamba.lock.
- **Host-flavored live lockfile** — `lock.rs:pick_artifact_url` uses `TagSelector::current_host`; `pkgmgr/platform_selector.rs` (cross-platform `--platform`) exists but is not wired into `cmd_lock`.
- **`/` means wheel path** — `add.rs:looks_like_wheel_path` treats any spec containing `/` as a local wheel; a slashed package spec never reaches the registry path.
- **Coarse version pick on add** — `add.rs:pick_pypi_latest`/`is_prerelease` substring filter (e.g. `.post1` filtered as prerelease) + `pep440_lite_cmp`; the real parser `pkgmgr/pep440.rs` is not used here.
- **Five PEP 503 normalize copies** — `add.rs:normalize_name`, `lock.rs:normalize_name`, `provider.rs:normalize_distribution_name`, `pkgmgr/name_normalize.rs:pep503_normalize`, `http.rs:normalize_name` (regex). Drift breaks index-dir lookups vs URL interpolation.
- **sync installs stubs** — `sync.rs:materialize_stub` writes a bare `__init__.py`; verified downloads land in the cache only. Import-probe fixtures pass; real code does not arrive in `.venv`.
- **Index-root inference** — `sync.rs:derive_index_url` reverse-engineers the index URL from an artifact URL to pick auth; mismatched artifact hosts get the wrong credential scope.
- **Plaintext credentials** — `auth.rs` stores tokens as plaintext JSON under `MAMBA_CREDENTIALS_DIR`; `pkgmgr/keyring_spec.rs` exists but is not wired in.
- **Dual lockfile writers** — `lock.rs:render_lockfile` (handwritten string emit) vs `pkgmgr/lockfile/serialize.rs`; the `lockfile/mod.rs` adapter must track both field vocabularies (`source_kind` legacy vs `source_ref` table).
- **Per-verb tokio runtimes** — `add.rs`, `lock.rs`, `sync.rs` each build their own runtime and `block_on`; calling these from an async context panics.
- **Source-policy regression risk** — the implicit-PyPI fallback was removed once; the fail-fast + no-mutation contract and its negative tests are pinned in `mamba-pkgmanage-remove-implicit-pypi-default-from-add-lock.md`.

## Extension points

- New CLI verb: `src/pkgmanage/<verb>.rs` + `mod.rs` entry + `src/main.rs` wiring; mirror test file `tests/pkgmgr/<verb>.rs` registered in `tests/pkgmgr/runner.rs`.
- New uv primitive: one tick module appended at the *bottom* of the `pkgmgr/mod.rs` `pub mod` block (blame rule in header) + classify it in the DDD map comment.
- New fetch backend (git / path / registry vendoring): `source/mod.rs` is the reserved, currently-empty landing zone (B1).
- New first-party provider package: `provider.rs:catalog` + a `provider_files` payload arm; sync/lock/add pick it up via `[tool.mamba.sources]`.
- New resolution policy: `Resolver::with_prerelease_policy` / `with_resolution_strategy` / `with_exclude_newer` builders (`pkgmgr/resolver/mod.rs:74-99`).
- New release-gated workflow family: `validation/profiles/package_manager.toml` `[families.<id>]` + fixture dir `tests/governance/gates/pkgmgr/<id>/` + schema gate in `tests/governance/schema_gates/`.

## EC surface

`tech-design/README.md` maps pkgmanage EC to "pkgmanage test suite"; `external-contracts/README.md` has no pkgmanage row (C1/C2 CPython gates do not cover this domain). What actually proves it:

| Gate | Artifact | Command |
|---|---|---|
| Verb integration umbrella | `tests/pkgmgr/runner.rs` (25 verb files, spawns `CARGO_BIN_EXE_mamba`) | `cargo test -p mamba --test pkgmgr` |
| Fixture families | `tests/governance/gates/pkgmgr/<family>/manifest.toml` — 19 dirs: add, lock, sync, run, init, index, hash, cache, remove, upgrade, downgrade, dev_dependency, dependency_group, direct_local_wheel, editable_local_project, env_marker, extras_resolution, json_summary, workspace_member | via umbrella + schema gates |
| Schema gates | `tests/governance/schema_gates/pkgmgr_*_fixture_*.rs` (20, incl. `uv_like_pkgmgr_offline_e2e_fixture_2532.rs`) | governance test binary |
| Release-blocking profile | `validation/profiles/package_manager.toml` — 9 required families, `network = "offline"`, `index_source = "frozen_local"`; live network opt-in only | `mamba pkgmgr-validate` (`validate.rs`) |
| In-crate unit gates | `src/pkgmanage/pkgmgr/tests/{installer,resolver,pypi_index_client,mvp_package_manager_umbrella_gate,venv_phase_gate}.rs` | `cargo test -p mamba pkgmgr` |
