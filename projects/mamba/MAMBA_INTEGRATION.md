# Mamba Integration Guide

How to extend mamba with native Rust code — for both first-party (monorepo) crates
and true 3rd-party vendors.

This guide is the operational reference. The **contract** itself lives in
`crates/cclab-mamba-registry/` (the `MambaModule` trait, `MAMBA_MODULES` slice,
`rt_sym!` macro, `MbValue` type). `mamba` crate is the runtime that loads the
bindings — do not confuse the two.

## Architecture Overview

```
cclab-mamba-registry          (thin contract: MbValue, MambaModule, MAMBA_MODULES)
        ↑
        ├── mamba             (runtime — reads MAMBA_MODULES at startup)
        ├── <first-party>     (cclab-fetch-mamba, projects/mamba/mambalibs/httpkit/binding, ...)
        └── <3rd-party>       (any external Rust crate implementing MambaModule)
```

| Crate | Role | Depend on it when... |
|---|---|---|
| `cclab-mamba-registry` | ABI contract | Writing any binding crate |
| `mamba` | Runtime + compiler + stdlib | You want to reuse mamba's internal Rust stdlib (http, json, re) at build time |

Binding crates self-register via `#[distributed_slice(MAMBA_MODULES)]`. The
`linkme` crate collects all entries at **final-binary link time** — this means
every binding must be part of the same cargo build as the mamba binary.

## Integration Modes

### Mode 1: First-party monorepo crate (works today)

Your binding crate sits inside the mamba monorepo (e.g. `crates/acme-mamba/`).
`projects/mamba/src/main.rs` force-links it with `use acme_mamba as _;`.

Canonical example: `crates/cclab-fetch-mamba/` (async HTTP client binding).

Workflow:
1. Create `crates/acme-mamba/` inside the monorepo.
2. Implement `MambaModule` (see template below).
3. Add `use acme_mamba as _;` to `projects/mamba/src/main.rs`.
4. `cargo build -p mamba --release`.

Pros: zero fetch/lock machinery, fastest iteration.
Cons: requires monorepo commit; not available to external vendors.

### Mode 2: External 3rd-party crate (MVP — designed, not yet implemented)

Your crate lives in your own git repo. Consumers add it to their `mamba.toml`
and run `mamba build` — which fetches, synthesizes a cargo workspace, and
rebuilds a project-local mamba binary with your crate linked in.

Status: **design only**. This section is the spec; implementation tracked
separately.

## Writing a Binding Crate

### Cargo.toml

```toml
[package]
name = "acme-mamba"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib"]

[dependencies]
cclab-mamba-registry = "0.1"   # or path/git during MVP
linkme = "0.3"
```

Do not set `crate-type = ["cdylib"]` — current mode requires static linking,
not dynamic loading.

### src/lib.rs — minimal template

```rust
use cclab_mamba_registry::{
    rc::{MbObject, ObjData},
    MambaModule, ModuleRegistrar, MbValue, MAMBA_MODULES, rt_sym,
};
use linkme::distributed_slice;

pub struct AcmeModule;

impl MambaModule for AcmeModule {
    fn name(&self) -> &'static str { "acme" }
    fn doc(&self) -> &'static str { "Acme native extensions" }
    fn register(&self, r: &mut ModuleRegistrar) {
        r.add_symbols([
            rt_sym!("greet", mb_greet, "greet(name: str) -> str"),
        ]);
    }
}

#[distributed_slice(MAMBA_MODULES)]
static ACME_MODULE: &dyn MambaModule = &AcmeModule;

pub fn mb_greet(name: MbValue) -> MbValue {
    let n = unsafe { name.as_obj_str().unwrap_or("world") };
    let obj = MbObject::new_str(format!("Hello, {n}!"));
    MbValue::from_ptr(obj as usize)
}
```

### Mamba script usage

```python
from acme import greet
print(greet("alice"))   # "Hello, alice!"
```

## Function Signatures & ABI

Every symbol registered via `rt_sym!` must match one of these Rust signatures:

| Arity | Rust signature |
|---|---|
| 0 | `fn() -> MbValue` |
| 1..N | `fn(MbValue, ..., MbValue) -> MbValue` |
| Variadic | `unsafe extern "C" fn(*const MbValue, usize) -> MbValue` |

Variadic is used when Python-side call uses `*args` or arity is unknown. See
`projects/mamba/src/runtime/stdlib/itertools_mod.rs::dispatch_chain` for an
example.

**Memory ownership**:
- Arguments are borrowed; never `free` them.
- Returned `MbValue::from_ptr(...)` ownership transfers to the caller; the
  runtime manages refcount via `MbObject::header.rc`.
- Return `MbValue::none()` for Python `None`.

## `mamba.toml` Reference

Located at project root. `mamba` walks up from CWD to find it.

### Schema

```toml
[project]
name = "my-app"
version = "0.1.0"
entry_point = "src/main.py"

# Native Rust bindings — each one is linked into the mamba binary.
[crates.<key>]
# Exactly one of path / version / git must be set.
path = "../local-dir"             # local source (relative to mamba.toml)
version = "0.1.0"                 # registry version (not yet wired)
git = "https://github.com/..."    # git source (MVP — not yet implemented)
rev = "v1.0.0"                    #   pin via rev / branch / tag

expose = ["fn1", "fn2"]           # required — Python-visible name whitelist
module = "acme.payment"           # Python import path (defaults to <key>)
crate = "payment-mamba"           # Cargo crate name override (defaults to <key>)

[paths]
search = ["../shared-libs"]       # extra search paths for package resolution

[build]
target = "native"                 # native | wasm | llvm
opt_level = 2                     # 0-3
```

### Current status of each source type

| Source | Parsing | Actual fetch / link |
|---|---|---|
| `path` | Works | Works (via monorepo force-link in `main.rs`) |
| `version` | Works | Not wired — needs registry |
| `git` | Not supported | Not implemented — MVP target |

## `mamba build` — Build Flow (MVP spec)

```
$ mamba build
```

Steps:
1. Read `mamba.toml` (walks up from CWD).
2. For each `[crates.<key>]`:
   - `path`: use in place
   - `git`: `git clone` into `.mamba/build/<key>/`, checkout `rev`/`branch`/`tag`
   - `version`: MVP errors out (deferred)
3. Synthesize a cargo workspace at `.mamba/cargo/Cargo.toml`:
   ```toml
   [workspace]
   members = [
     "/abs/path/to/mamba",
     ".mamba/build/payment",
     "mamba-stub",
   ]
   ```
4. Generate `.mamba/cargo/mamba-stub/src/main.rs` with force-link directives:
   ```rust
   use payment_mamba as _;
   fn main() { mamba::driver::run_cli() }
   ```
5. `cargo build --release --manifest-path .mamba/cargo/Cargo.toml -p mamba-stub`.
6. Copy/symlink output to `.mamba/bin/mamba`.
7. Write `mamba.lock` with pinned git SHAs + source checksums.

Result: a project-local mamba binary at `.mamba/bin/mamba`.

### Execution

```
$ .mamba/bin/mamba run src/main.py
```

Or symlink at project root:
```
$ ln -sf .mamba/bin/mamba ./mamba
$ ./mamba run src/main.py
```

### MVP scope boundary

**In scope**: `path` + `git` sources, per-project binary, manual `mamba build`.

**Out of scope (deferred)**:
- Registry (`version` source) fetch
- `mamba run` auto-rebuild on stale `mamba.toml`
- Dynamic plugin loading via `dlopen` / `libloading`
- WASM plugins
- Cross-version ABI stability guarantees

### Prerequisites

Users must have a stable Rust toolchain (`cargo`, `rustc`). First build is
slow (~5 min for mamba + typical deps); subsequent builds are incremental.

## Do / Don't

**Do**:
- Depend only on `cclab-mamba-registry` unless you genuinely need `mamba` internals.
- Use `rt_sym!` for all exports — it generates the correct `RuntimeSymbol` shape.
- Keep `MambaModule::name()` namespaced (`"acme.payment"`, not `"payment"`).
- Use the `MbValue` accessors (`as_int`, `as_ptr`, `as_obj_str`) — don't assume layout.

**Don't**:
- Don't `panic!` from a symbol function — raise via `mb_raise` instead.
- Don't set `crate-type = ["cdylib"]` — mode is static link, not dylib.
- Don't block a worker thread with long sync I/O — use tokio in the binding.
- Don't depend on `mamba` (the full crate) unless needed — it adds 20-60s compile time.

## Reusing Mamba's Stdlib in Rust

If your binding needs to call mamba's built-in `urllib.parse.quote`,
`json.dumps`, etc. from **inside your Rust code** (not from a Mamba script),
add `mamba` as a direct dep:

```toml
[dependencies]
cclab-mamba-registry = "0.1"
mamba = { path = "../mamba" }   # +20-60s compile time
```

Then:

```rust
use mamba::runtime::stdlib::http_mod::mb_urllib_quote;

pub fn my_fn(url: MbValue) -> MbValue {
    mb_urllib_quote(url, MbValue::none())
}
```

Tradeoff: build-time type safety + zero dispatch cost vs. heavy compile.
If this becomes painful, we plan to extract `cclab-mamba-stdlib` (pure Rust,
no `MbValue` dep) so both the runtime and bindings can share implementations
without pulling the compiler.

## Status Matrix

| Feature | Status |
|---|---|
| First-party monorepo binding via `linkme` | Stable |
| `mamba.toml` `[crates.*]` parsing | Implemented |
| `mamba.toml` `path` source | Implemented (monorepo force-link) |
| `mamba.toml` `version` source | Field parsed, fetch not wired |
| `mamba.toml` `git` source | Not implemented — MVP target |
| `mamba build` native-dep resolution | Not implemented — current `mamba build` only compiles `.py` |
| `mamba.lock` | Not implemented |
| Project-local `.mamba/bin/mamba` | Not implemented |
| `cclab-mamba-stdlib` (pure-Rust stdlib lib crate) | Not implemented |
| Dynamic plugin loading (dlopen / libloading) | Out of scope |
| WASM plugins | Out of scope |
| Cross-version ABI stability guarantees | Unplanned |

## References

- Contract crate: `crates/cclab-mamba-registry/src/lib.rs`
- Canonical binding: `crates/cclab-fetch-mamba/`
- Higher-level binding: `projects/mamba/mambalibs/httpkit/binding/` (FastAPI-style router)
- Config schema: `projects/mamba/src/config/schema.rs`
- Current `mamba build` entry: `projects/mamba/src/main.rs:79`
- Ecosystem overview: `ECOSYSTEM.md`
