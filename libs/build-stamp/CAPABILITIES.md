# Build Stamp Capabilities

## Brief

`build-stamp` is the shared `build.rs` helper that stamps a crate's binary with
the three facts a support engineer needs to identify a running build: which
commit it came from, when it was compiled, and which target triple it was
compiled for. It emits those facts as `cargo:rustc-env` directives under a
caller-chosen prefix, so each crate's `env!`/`option_env!` consumers read
`<PREFIX>_GIT_SHA`, `<PREFIX>_BUILT_AT`, and `<PREFIX>_TARGET`.

It is not a version scheme, not a release manifest, and not a reproducibility
mechanism: it reports what the build environment says, and says `unknown` when
that environment cannot answer. Its defining constraint is that it is a build
script — it writes to a channel Cargo parses as commands, and it must never
fail the build it is stamping.

## Capabilities

Every capability belongs to exactly one of two feature roots:

- **Core Features** define what `build-stamp` fundamentally does: emit the three
  stamp directives, under the caller's prefix, on the channel Cargo reads.
- **Non-Core Features** keep that emission safe and total — every input it
  cannot obtain degrades to a stated value, and nothing it interpolates can turn
  into a second directive. Non-core does not mean optional.

This file contains stable product promises, claim IDs, and verification
surfaces. Delivery planning lives outside this contract and references these
IDs one way.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Version Stamp Emission | - | implemented | verified | smoke | ready | core; three prefixed `cargo:rustc-env` directives plus a rerun hint that is emitted only when the path it names exists |
| Best-Effort Degradation | - | implemented | verified | smoke | ready | non-core; every unobtainable input becomes the stable value `unknown`, and no stamp path can fail the build |
| Directive Channel Integrity | - | implemented | verified | smoke | ready | non-core; nothing interpolated into the stamp — prefix, sha, or target — can introduce a directive the caller did not ask for |

### Core Features

#### Version Stamp Emission

ID: version-stamp-emission
Root WI: -
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
A calling crate's `build.rs` gets exactly three environment stamps — commit, build time, and target triple — named under the prefix it chose, in a form Cargo turns into compile-time environment variables. The stamped commit stays current across rebuilds when the build ran inside a git checkout, and the mechanism that keeps it current is never emitted when it would point at a path that does not exist.
Surfaces:
- Rust API: `build_stamp::stamp` - the single entry point; takes the caller's prefix and writes every directive.
Rust internal: the emitted directive set is `<PREFIX>_GIT_SHA`, `<PREFIX>_BUILT_AT`, `<PREFIX>_TARGET`, and conditionally `cargo:rerun-if-changed=<head path>`.
EC Dimensions:
- behavior: `cargo test -p build-stamp --lib` - the directive set, its names under an arbitrary prefix, the epoch-seconds encoding of the build time, and the existence condition on the rerun hint are decided by stated rules.
- security: `cargo test -p build-stamp --lib` - the emitted set is exactly those directives and no others, so a stamp cannot silently acquire a link flag, a cfg, or a second rerun hint.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Prefixed directive set | change | - | implemented | verified | smoke | `cargo test -p build-stamp --lib`; one `stamp` call emits exactly three `cargo:rustc-env` directives whose names are the caller's prefix joined to `_GIT_SHA`, `_BUILT_AT`, and `_TARGET`, in that order |
| Build-time encoding | change | - | implemented | verified | smoke | `cargo test -p build-stamp --lib`; the build time is whole seconds since the Unix epoch with no unit suffix, no fractional part, and no locale-dependent formatting |
| Conditional rerun hint | change | - | implemented | verified | smoke | `cargo test -p build-stamp --lib`; the `cargo:rerun-if-changed` hint names the git HEAD path and is emitted when and only when that path exists, so a source tarball with no `.git` emits no hint rather than a hint pointing nowhere |

### Non-Core Features

#### Best-Effort Degradation

ID: best-effort-degradation
Root WI: -
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
A crate builds identically whether or not the build environment can answer the stamp's questions. Every input that cannot be obtained — no git checkout, no `git` binary, a git invocation that fails or answers with nothing, no target triple in the environment, an unreadable clock — becomes the same stable value `unknown`, and the stamp still emits its full directive set.
Surfaces:
- Rust API: `build_stamp::stamp` - the total function; it has no failure mode and no panic path.
Rust internal: the decode seams that decide the fallback — a failed or empty `git rev-parse`, an absent `TARGET`, and a clock before the epoch.
EC Dimensions:
- behavior: `cargo test -p build-stamp --lib` - each unobtainable input independently produces `unknown` while the other two stamps keep their real values, and the directive count never changes.
- security: `cargo test -p build-stamp --lib` - a degraded stamp reports only `unknown`, never an error message, exit status, path, or partial output, and no failure class escalates into a build failure.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Total fallback value | change | - | implemented | verified | smoke | `cargo test -p build-stamp --lib`; a failed git invocation, a successful one with empty output, and an absent target triple each yield exactly the string `unknown`, and the three fallbacks are the same value rather than three similar ones |
| Independent degradation | change | - | implemented | verified | smoke | `cargo test -p build-stamp --lib`; one unobtainable input degrades only its own directive, so a checkout without a `git` binary still stamps a real target triple and a real build time |
| No diagnostic leakage | change | - | implemented | verified | smoke | `cargo test -p build-stamp --lib`; the degraded value carries no stderr text, exit code, filesystem path, or environment-variable name, so a stamp cannot become a disclosure channel for the build host |

#### Directive Channel Integrity

ID: directive-channel-integrity
Root WI: -
Status: verified
Type: Security
Feature Class: non-core
Required Verification: smoke
Promise:
The stamp writes to a channel where a newline starts a new command Cargo will obey. Every value it interpolates — the caller's prefix, the sha read from a subprocess, the target triple read from the environment — is confined to the single directive it belongs to, so no input can cause Cargo to execute a directive the calling crate did not ask for.
Surfaces:
- Rust API: `build_stamp::stamp` - the interpolation point for the caller's prefix.
Rust internal: the decode seams that admit externally controlled bytes — `git rev-parse` stdout and the `TARGET` environment variable.
EC Dimensions:
- behavior: `cargo test -p build-stamp --lib` - trailing whitespace and a trailing newline are removed from the sha before it is interpolated, and the trim is defined on the decoded value rather than on the raw bytes.
- security: `cargo test -p build-stamp --lib` - a value containing a newline, a carriage return, or a `cargo:` sequence cannot produce a second directive line, and invalid UTF-8 from the subprocess is replaced rather than propagated or rejected.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Newline containment | change | - | implemented | verified | smoke | `cargo test -p build-stamp --lib`; a sha or target value carrying an embedded newline or carriage return does not yield a second parsable `cargo:` directive, and the number of directives emitted is a function of the code path taken rather than of the value's content |
| Sha normalization | change | - | implemented | verified | smoke | `cargo test -p build-stamp --lib`; surrounding whitespace is stripped from the decoded sha, an all-whitespace answer is treated as no answer rather than as an empty sha, and the normalization happens before any emission |
| Lossy decode rather than refusal | change | - | implemented | verified | smoke | `cargo test -p build-stamp --lib`; invalid UTF-8 in the subprocess answer is decoded with replacement so the stamp neither panics nor propagates raw bytes onto the directive channel |
