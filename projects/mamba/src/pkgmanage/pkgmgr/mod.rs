// pkgmgr — uv-compatible package manager for mamba.
//
// Original scaffold (Shards 1–3):
//   * Shard 1:  typed API surface + JSON parsing (data layer, no I/O).
//   * Shard 2a: HTTP fetch path — fetch_metadata_json with retry + backoff.
//   * Shard 2b: Simple API (PEP 503 HTML + PEP 691 JSON negotiation).
//   * Shard 3:  concurrency model (tokio semaphore, batch fetch, FS cache).
//
// Tick 40+ expansion (uv replication, atomic ticks). Each tick adds one
// focused module here. Themes covered so far:
//   * Resolution / install / lockfile core: resolver, installer, lockfile.
//   * Wheel / sdist build: pep517, sdist, sdist_build, wheel_build, editable.
//   * Workspaces + groups + tools: workspace, groups, tools.
//   * Toolchain + interpreter selection: toolchain, interpreter_request,
//     python_version_file, pbs_url, pbs_host, platforms.
//   * Config files (uv-compatible): uv_config, uv_toml, uv_sources, uv_dirs,
//     pyproject_deps, constraints, overrides.
//   * Metadata file readers/writers (PEP 376/491/610/621/625/753/etc.):
//     record_reader, record_writer, wheel_metadata, direct_url, direct_url_json,
//     entry_points, installer_file.
//   * Lockfile / requirements exchange: requirements_parse, requirements_loader,
//     requirements_export, pylock_export, pylock_import.
//   * Operational helpers: tree, freshness, upgrade, version_bump, publish,
//     parity, benchmark, indexes, cache_prune, bytecode, init_scaffold,
//     pip_check, pip_inventory, pip_tree, pep723, name_normalize, netrc.
//   * HTTP transport details: retry_after (Tick 75), cache_control (76),
//     etag (79), link_header (86), media_type (89),
//     content_disposition (91), range (94), accept_encoding (97) — covers
//     conditional-GET, back-off, paginated indexes, PEP 691 content
//     negotiation, opaque-URL filename recovery, resumable downloads
//     and central-directory tail probes, plus compressed-response codec
//     negotiation.
//   * Standards parsers: pep639 (Tick 72, SPDX license expression),
//     manifest_in (73, sdist template), build_system (77, PEP 517 table),
//     simple_url (78, PEP 503 URL canonicalization), wheel_filename (88,
//     PEP 427 filename decomposition), pep660 (96, editable-wheel
//     classifier), plus the parse_core_metadata reader bolted onto
//     wheel_build.rs in Tick 74.
//   * Site / path-config files (Ticks 81, 83): top_level (.dist-info/
//     top_level.txt), pth_file (site-packages .pth path-configuration).
//   * Security / identity primitives (Ticks 82, 84, 85): hash_spec
//     (pip --hash= flag validation), externally_managed (PEP 668 marker),
//     url_redact (credential redaction / strip for logs and cache keys).
//   * Requirements-format extensions (Ticks 82, 87): hash_spec drives the
//     typed-hash layer on per-line entries; requirements_options parses the
//     file-prelude flags (`-i`, `--extra-index-url`, `--find-links`,
//     `--no-index`, `--trusted-host`, `--pre`, `--require-hashes`).
//   * Credential discovery pipeline (Ticks 92, 99 — pairs with the
//     Tick-80 `netrc` module): keyring_spec resolves an index URL into
//     a service key + Disabled/Auto/Subprocess provider + Basic/Creds
//     mode; auth_header turns the resulting (user, pass) into RFC 7617
//     Basic or RFC 6750 Bearer Authorization values.
//   * pip-compat configuration (Tick 93): pip_conf reads pip's INI
//     config (configparser-flavoured) so pip-to-uv migrations can
//     reuse /etc/pip.conf, ~/.config/pip/pip.conf, etc.
//   * Wheel signing + upload (Ticks 95, 98): wheel_jws decodes
//     RECORD.jws JSON-serialized JWS files; pep694_session models the
//     staged upload-session lifecycle (Pending → Initiated → Completed
//     / Cancelled / Errored) that replaces the legacy /upload endpoint.
//   * PEP 658 sidecar fallback chain (Ticks 94, 101–107): the
//     METADATA-without-full-download path uv pioneered. range (94)
//     issues the byte-range fetches; pep658_url (101) detects the
//     advertised `.metadata` sidecar from PEP 691 + PEP 503 hints;
//     when none is offered, pep658_fallback (107) drives the chain
//     zip_tail (102) → zip_cdr (103) → zip_lfh (104) →
//     deflate_inflate (105) → crc32_verify (106), parsing the PKZIP
//     central-directory + local-file-header + raw RFC 1951 deflate
//     stream + ITU-T V.42 CRC right inside the resolver.
//   * Index API extensions (Ticks 108, 109): pep700 reads the
//     PEP 700 file-level fields (`requires-python`, `size`,
//     `upload-time`) that uv uses to short-circuit incompatible
//     wheels before METADATA fetch; pep740 decodes PEP 740 attestation
//     envelopes (Trusted Publisher DSSE bundles, X.509 cert + Sigstore
//     transparency entries) at the structural level — cryptographic
//     verification is left to a Sigstore/TUF dependency tree we
//     intentionally don't pull in.
//   * PEP 508 / 440 / 621 / 723 typed parsers (Ticks 111–117): the
//     core grammar layer the resolver and installer dispatch against.
//     yanked (111) handles PEP 592 file-level yank gating;
//     extras_spec (112) decodes the `[a,b]` qualifier with PEP 685
//     normalization; requirement_string (113) splits the full PEP 508
//     `name[extras] specifier ; marker` head; specifier_set (114)
//     parses comma-separated PEP 440 version-specifier clauses with
//     ==X.* wildcards and ~= compatible-release semantics; pep621 (115)
//     reads the pyproject.toml [project] table; pep723_typed (116)
//     layers structured Requirement parsing on the existing pep723
//     inline-script reader; local_version (117) parses the `+local`
//     suffix that direct-URL pins (PyTorch CUDA wheels at
//     download.pytorch.org/whl/cu118/, /cu121/) actually use as a
//     sort key.
//   * Wheel-selection layer (Ticks 118, 119): wheel_picker scores a
//     candidate list against any TagSelector and picks the best wheel
//     with PEP 491 build-tag tie-breaks; platform_selector builds an
//     explicitly-named TagSelector from a SupportedEnvironment so
//     `mamba lock --platform linux_x86_64` running on a macOS host
//     pins Linux wheels (cross-platform lockfile resolution).
//   * Direct-reference + project-URL layer (Ticks 121, 122): vcs_url
//     parses PEP 440 direct-reference VCS schemes (`git+`, `hg+`,
//     `svn+`, `bzr+`) including scp-form ssh canonicalization and
//     bzr launchpad shorthand; well_known_urls implements the
//     PEP 753 label normalization and classifier (`Homepage`,
//     `Source`, `Documentation`, etc.) used by metadata writers.
//   * Resolver glue + index navigation (Ticks 123, 124, 125):
//     requirement_filter bridges the typed PEP 508 parser to
//     `markers::evaluate` with extras-activation; find_links parses
//     PEP 503-flat HTML index entries (`.whl`, `.tar.gz`, `.zip`)
//     into typed FindLinksEntry records; egg_info_requires reads
//     legacy setuptools `.egg-info/requires.txt` bracket sections
//     and normalizes them to PEP 508 strings.
//   * Environment + auth surface (Ticks 126, 127, 128): shell
//     classifies the user's interactive shell and emits per-shell
//     PATH-prepend snippets with managed-block markers;
//     proxy_config decodes `HTTP_PROXY`/`HTTPS_PROXY`/`ALL_PROXY` URLs
//     and matches `NO_PROXY` host suffixes with port-specific rules;
//     cert_bundle resolves the TLS-CA env-var precedence chain
//     (`PIP_CERT` > `REQUESTS_CA_BUNDLE` > `CURL_CA_BUNDLE` >
//     `SSL_CERT_FILE`) plus the orthogonal `SSL_CERT_DIR` directory.
//   * Build-environment configuration (Ticks 129, 130):
//     build_constraints reads `UV_BUILD_CONSTRAINT` /
//     `PIP_BUILD_CONSTRAINT` files (PEP 508 lines + `-r` includes,
//     extras + direct-URL refs rejected); maturin_compat reads
//     the `[tool.maturin]` table from pyproject.toml so the
//     resolver can identify PyO3 / cffi / uniffi extension builds
//     without delegating to maturin itself.
//   * Install-time layout (Tick 131): scheme_paths renders the
//     sysconfig install-scheme path templates (`{base}`,
//     `{python_version}`, `{python_version_nodot}`, `{abiflags}`,
//     `{dist_name}`) into concrete paths for posix-prefix /
//     posix-user / posix-venv / nt-prefix / nt-user / nt-venv —
//     the invariant `purelib == platlib` is preserved on every
//     shipped scheme.
//   * Resolution policy ADTs (Ticks 132–136): exclude_newer parses
//     the PEP 700 `upload-time` cutoff (date / RFC 3339 Zulu / RFC
//     3339 with offset) without pulling in `chrono`/`time`;
//     prerelease_policy models uv's five-state `--prerelease` flag
//     (Disallow / Allow / IfNecessary / Explicit /
//     IfNecessaryOrExplicit, last is default); resolution_strategy
//     picks Highest / Lowest / LowestDirect with a `pick_candidate`
//     helper on any ascending slice; fork_strategy chooses Fewest
//     (universal) vs RequiresPython (per-Python forks); and
//     index_strategy is the dependency-confusion gate
//     (FirstIndex default vs UnsafeFirstMatch / UnsafeBestMatch).
//   * Install-time scope policies (Ticks 137–139): reinstall_scope
//     and refresh_scope share the None / All / Selective ADT shape
//     to drive `--reinstall`/`--reinstall-package` (force-redo on
//     disk) and `--refresh`/`--refresh-package` (HTTP-cache bypass)
//     respectively, orthogonal to upgrade.rs; build_isolation
//     models `--no-build-isolation` and `--no-build-isolation-
//     package` with INVERTED predicate semantics
//     (`should_isolate` defaults true — the PEP 517-safe choice)
//     and exposes `has_non_isolated_builds` so the build cache
//     can refuse to reuse a non-isolated artifact.
//   * Source-selection policy (Tick 140): source_strategy unifies
//     uv's `--no-binary` and `--only-binary` flags into a
//     SourceFilterSet × SourceFilterSet ADT (Empty / All / Named
//     per side) with `allows_wheel(pkg)` + `allows_sdist(pkg)`
//     predicates and `is_contradictory(pkg)` to flag the
//     pathological `:all:` + same-name conflict (e.g. torch).
//
// -----------------------------------------------------------------
// Domain-Driven Design bounded-context map (Tick 140 organization
// sweep). The crate is still a flat `mod` tree on disk — physical
// directory moves would invalidate ~110 git-blame trails on a
// single sweep, which we want to do as a deliberate one-shot
// later, not as part of the per-tick cadence. The map below is
// the logical DDD layout each module belongs to. Keep this list
// in sync as new ticks land:
//
//   1. Index Access (transport + index-API)
//      json_api, http, simple_api, retry_after, cache_control,
//      etag, link_header, media_type, content_disposition, range,
//      accept_encoding, simple_url, indexes, find_links,
//      pep700, pep740, pep694_session, publish.
//
//   2. Distribution Metadata (artifact + sidecar parsing)
//      wheel_metadata, wheel_filename, wheel_jws, record_reader,
//      record_writer, direct_url, direct_url_json,
//      installer_file, entry_points, top_level, pth_file,
//      manifest_in, egg_info_requires, parse_core_metadata
//      (lives on wheel_build), pep658_url, pep658_fallback,
//      zip_tail, zip_cdr, zip_lfh, deflate_inflate, crc32_verify,
//      content_disposition (also Index Access).
//
//   3. Versioning & Requirements (grammar + name canon)
//      pep440, pep639, pep660, name_normalize, yanked,
//      extras_spec, requirement_string, specifier_set,
//      local_version, hash_spec, requirements_parse,
//      requirements_loader, requirements_export,
//      requirements_options, requirement_filter, markers,
//      vcs_url, well_known_urls.
//
//   4. Resolution Policy (graph-level decisions)
//      resolver, lockfile, pylock_export, pylock_import,
//      constraints, overrides, exclude_newer, prerelease_policy,
//      resolution_strategy, fork_strategy, index_strategy,
//      upgrade, refresh_scope, freshness, tree, parity,
//      version_bump, pip_check, pip_inventory, wheel_picker,
//      platform_selector, source_strategy.
//
//   5. Installation & Layout (write-side execution)
//      installer, reinstall_scope, build_isolation, pep517,
//      sdist, sdist_build, wheel_build, editable, scheme_paths,
//      venv, bytecode, cache, cache_prune, init_scaffold.
//
//   6. Workspace & Tools (project / cross-project surfaces)
//      workspace, groups, tools, pyproject_deps, pep621,
//      pep723, pep723_typed.
//
//   7. Configuration (uv + pip on-disk config)
//      uv_config, uv_toml, uv_sources, uv_dirs, pip_conf,
//      build_constraints, maturin_compat, build_system.
//
//   8. Toolchain (interpreter discovery + download)
//      toolchain, interpreter_request, python_version_file,
//      pbs_url, pbs_host, platforms, tags.
//
//   9. Security & Identity (credentials, hashes, attestations)
//      netrc, keyring_spec, auth_header, url_redact,
//      externally_managed, hash_spec (also Versioning),
//      pep740 (also Index Access), wheel_jws (also Distribution
//      Metadata).
//
//   10. Environment Surface (OS-level integration)
//       shell, proxy_config, cert_bundle, benchmark, types.
//
// A module appearing in two contexts is intentional — it's a
// boundary object between aggregates and shouldn't be "fixed" by
// duplicating logic. When in doubt, the *primary* responsibility
// is the first context listed.
//
// New modules should be appended at the bottom of the `pub mod` block to
// preserve git-blame for the registration line (the module file itself is
// the source of truth).

pub mod accept_encoding;
pub mod auth_header;
pub mod benchmark;
pub mod build_constraints;
pub mod build_isolation;
pub mod build_system;
pub mod bytecode;
pub mod cache;
pub mod cache_control;
pub mod cache_prune;
pub mod cert_bundle;
pub mod constraints;
pub mod content_disposition;
pub mod crc32_verify;
pub mod deflate_inflate;
pub mod direct_url;
pub mod direct_url_json;
pub mod editable;
pub mod egg_info_requires;
pub mod entry_points;
pub mod etag;
pub mod exclude_newer;
pub mod externally_managed;
pub mod extras_spec;
pub mod find_links;
pub mod fork_strategy;
pub mod freshness;
pub mod groups;
pub mod hash_spec;
pub mod http;
pub mod index_strategy;
pub mod indexes;
pub mod init_scaffold;
pub mod installer;
pub mod installer_file;
pub mod interpreter_request;
pub mod json_api;
pub mod keyring_spec;
pub mod link_header;
pub mod local_version;
pub mod lockfile;
pub mod manifest_in;
pub mod markers;
pub mod maturin_compat;
pub mod media_type;
pub mod name_normalize;
pub mod netrc;
pub mod overrides;
pub mod parity;
pub mod pbs_host;
pub mod pbs_url;
pub(crate) mod pep440;
pub mod pep517;
pub mod pep621;
pub mod pep639;
pub mod pep658_fallback;
pub mod pep658_url;
pub mod pep660;
pub mod pep694_session;
pub mod pep700;
pub mod pep723;
pub mod pep723_typed;
pub mod pep740;
pub mod pip_check;
pub mod pip_conf;
pub mod pip_inventory;
pub mod pip_tree;
pub mod platform_selector;
pub mod platforms;
pub mod prerelease_policy;
pub mod proxy_config;
pub mod pth_file;
pub mod publish;
pub mod pylock_export;
pub mod pylock_import;
pub mod pyproject_deps;
pub mod python_version_file;
pub mod range;
pub mod record_reader;
pub mod record_writer;
pub mod refresh_scope;
pub mod reinstall_scope;
pub mod requirement_filter;
pub mod requirement_string;
pub mod requirements_export;
pub mod requirements_loader;
pub mod requirements_options;
pub mod requirements_parse;
pub mod resolution_strategy;
pub mod resolver;
pub mod retry_after;
pub mod scheme_paths;
pub mod sdist;
pub mod sdist_build;
pub mod shell;
pub mod simple_api;
pub mod simple_url;
pub mod source_strategy;
pub mod specifier_set;
pub mod tags;
pub mod toolchain;
pub mod tools;
pub mod top_level;
pub mod tree;
pub mod types;
pub mod upgrade;
pub mod url_redact;
pub mod uv_config;
pub mod uv_dirs;
pub mod uv_sources;
pub mod uv_toml;
pub mod vcs_url;
pub mod venv;
pub mod version_bump;
pub mod well_known_urls;
pub mod wheel_build;
pub mod wheel_filename;
pub mod wheel_jws;
pub mod wheel_metadata;
pub mod wheel_picker;
pub mod workspace;
pub mod yanked;
pub mod zip_cdr;
pub mod zip_lfh;
pub mod zip_tail;

pub use installer::{
    InstallKind, InstallMode, InstallRequest, InstallResult, Installer, InstallerError,
};
pub use json_api::parse_json_metadata;
pub use lockfile::{
    Lockfile, LockfileDiff, LockfileError, MAX_SUPPORTED_FORMAT_VERSION, Package, PackageChange,
    SourceRef, SourceRefKind,
};
pub use resolver::{ResolutionError, ResolutionErrorKind, ResolvedGraph, ResolvedNode, Resolver};
pub use types::{FileHash, IndexClient, IndexError, PackageMetadata, ReleaseFile};

#[cfg(test)]
mod tests {
    mod installer;
    mod mvp_package_manager_umbrella_gate;
    mod pypi_index_client;
    mod resolver;
    mod venv_phase_gate;
}
