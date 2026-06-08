// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
// CODEGEN-BEGIN
//! `jet build --wasm` — frontend source graph → Rust/WASM build pipeline.
//!
//! Takes an app directory containing a `jet.config.toml` `[wasm]`
//! section, a TS/TSX entry file, an HTML shell, and CSS side-effect
//! imports, then produces a ready-to-open `dist/index.html` with
//! `dist/app_bg.wasm` + `dist/boot.js`.
//!
//! Pipeline:
//!
//! 1. Read `jet.config.toml` → entry file + root component name + root props.
//! 2. Read the shared frontend source set (`TS/TSX`, `HTML`, `CSS`) through
//!    `frontend::FrontendSources`.
//! 3. Run `jet::tsx_to_rust::transpile` on the typed entry source → Rust source.
//! 4. Scaffold a temporary Cargo project (`.jet/wasm-build/`) that:
//!    - depends on `jet-wasm` (path dep) + `wasm-bindgen` + `web-sys`
//!    - contains the transpiled Rust module + a small `lib.rs` that
//!      mounts the root component, constructs a Renderer with the
//!      CanvasBackend, and schedules a rAF render loop.
//!    - exports `#[wasm_bindgen(start)]` so the browser kicks it.
//! 5. Invoke `cargo build --target wasm32-unknown-unknown --release`.
//! 6. Post-process with `wasm-bindgen` to emit the JS glue + cleaned
//!    WASM binary.
//! 7. Copy outputs + thin `jet-host.js` + `boot.js`, and rewrite the source
//!    HTML shell into a canvas-mounted WASM shell.
//!
//! v0 scope: single-file TSX entry plus CSS side-effect imports. The
//! front half now matches regular `jet build`; backend lowering remains
//! a subset compiler until the typed IR covers full React/MUI graphs.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod config;
pub mod lint;
pub mod manifest;
pub mod schema;

/// Build profile — drives `wasm-pack build --release` vs `--dev`, and
/// whether the `jet-wasm` `debug` feature (runtime introspection +
/// `window.__jet_debug`) is on.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// Production build. Opt-level=s, LTO, no DWARF, no debug feature.
    Release,
    /// Development build. `wasm-pack --dev` keeps DWARF (so Chromium's
    /// C/C++ DevTools for WebAssembly can step Rust source), and
    /// enables the `debug` feature so `window.__jet_debug` is live
    /// for `jet browser` to attach to.
    Dev,
}

/// Top-level entry — called from the CLI when `--wasm` is set.
/// Defaults to the web target; multi-target callers should use
/// [`build_with_profile`] directly.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
pub fn build(root_dir: &Path, output_dir: &Path) -> Result<()> {
    build_with_profile(
        root_dir,
        output_dir,
        Profile::Release,
        crate::build_target::BuildTarget::Web,
    )
}

/// Like `build` but caller-chooses the profile and target.
///
/// @spec .aw/tech-design/projects/jet/logic/multi-target/build-targets.md
/// (Slice 2: target → cargo features pass-through).
///
/// `target = BuildTarget::Tui` is rejected here — TUI builds emit
/// native binaries via #1241's pipeline, not wasm. Slice 1's CLI
/// validation already catches `--target tui --wasm`; this is the
/// belt-and-braces guard for callers that bypass the CLI parser
/// (tests, future programmatic use).
pub fn build_with_profile(
    root_dir: &Path,
    output_dir: &Path,
    profile: Profile,
    target: crate::build_target::BuildTarget,
) -> Result<()> {
    if !target.produces_wasm() {
        bail!(
            "wasm_build::build_with_profile rejected --target {}: \
             TUI builds emit a native binary, not wasm. Track #1241.",
            target
        );
    }
    let cfg =
        config::WasmConfig::load(root_dir).context("failed to read [wasm] from jet.config.toml")?;

    let frontend = crate::frontend::FrontendSources::load(root_dir, PathBuf::from(&cfg.entry))
        .context("failed to read frontend sources")?;

    let component_source = resolve_component_source(&frontend, &cfg.root_component)
        .context("resolving WASM root component source")?;

    eprintln!(
        "[jet build --wasm] transpiling {}",
        component_source.source_label
    );
    // Annotate only in Dev profile. Release keeps the generated Rust
    // free of `// @tsx` comments so snapshot comparisons stay tight.
    let annotate_source = match profile {
        Profile::Dev => cfg.entry.clone(),
        Profile::Release => String::new(),
    };
    let transpiled = crate::tsx_to_rust::transpile_with_source(
        &component_source.source,
        if matches!(profile, Profile::Dev) {
            &component_source.source_label
        } else {
            &annotate_source
        },
    )
    .or_else(|strict_err| {
        eprintln!(
            "[jet build --wasm] strict TSX lowering is incomplete for {}; using Rust/WASM compatibility lowering: {strict_err:#}",
            component_source.source_label
        );
        crate::tsx_to_rust::transpile_compat_with_source(
            &component_source.source,
            if matches!(profile, Profile::Dev) {
                &component_source.source_label
            } else {
                &annotate_source
            },
            &cfg.root_component,
        )
    })
    .with_context(|| format!("transpiling {}", component_source.path.display()))?;
    let mut component_style_imports = component_source.css_side_effect_imports.clone();
    for import in &transpiled.style_imports {
        if !component_style_imports.contains(import) {
            component_style_imports.push(import.clone());
        }
    }
    let mut generated_rust = transpiled.rust_source;

    // Append the wasm-bindgen entry point that the boot loader calls.
    generated_rust.push_str(&emit_wasm_entry(&cfg));

    let work_dir = root_dir.join(".jet").join("wasm-build");
    scaffold_cargo_project(&work_dir, &generated_rust, profile, target, cfg.renderer)
        .context("scaffolding cargo workspace for wasm build")?;

    eprintln!(
        "[jet build --wasm] wasm-pack build ({}) — cargo build wasm32 + wasm-bindgen",
        match profile {
            Profile::Release => "release",
            Profile::Dev => "dev + DWARF",
        }
    );
    run_wasm_pack(&work_dir, profile).context("wasm-pack build failed")?;

    eprintln!("[jet build --wasm] writing dist/");
    let dist = root_dir.join(output_dir);
    fs::create_dir_all(&dist).context("creating dist/")?;
    copy_pkg_outputs(&work_dir.join("pkg"), &dist).context("copying wasm-pack outputs")?;
    let style_groups = [
        (
            frontend.entry_path.as_path(),
            frontend.css_side_effect_imports.as_slice(),
        ),
        (
            component_source.path.as_path(),
            component_style_imports.as_slice(),
        ),
    ];
    let stylesheet = copy_style_import_groups(root_dir, &style_groups, &dist)
        .context("copying WASM CSS side-effect imports")?;
    fs::write(dist.join("jet-host.js"), host_bridge_js())?;
    fs::write(dist.join("boot.js"), boot_js(&cfg))?;
    fs::write(
        dist.join("index.html"),
        index_html(&frontend.html_template, &cfg, stylesheet.as_deref()),
    )?;

    // TSX source map side-car. Small JSON that `jet browser tsx`
    // reads to tell the user where a component was declared. Emitted
    // on every build (release + dev) so CLI tooling doesn't care
    // which profile produced the bundle.
    let map_json = serde_json::to_string_pretty(&transpiled.position_map)
        .context("serializing tsx-source-map")?;
    fs::write(dist.join("tsx-source-map.json"), map_json)?;

    // jet-target.json — Slice 3 of #1239. Downstream packagers and CI
    // verifiers consume this to confirm the build target + cargo
    // feature set the bundle was produced with.
    let cargo_features = manifest_cargo_features(target, profile, cfg.renderer);
    write_target_manifest(root_dir, &dist, &cfg, target, profile, cargo_features)?;

    eprintln!(
        "[jet build --wasm] done → open {} in a static file server",
        dist.join("index.html").display()
    );
    Ok(())
}

struct ComponentSource {
    path: PathBuf,
    source_label: String,
    source: String,
    css_side_effect_imports: Vec<String>,
}

fn resolve_component_source(
    frontend: &crate::frontend::FrontendSources,
    root_component: &str,
) -> Result<ComponentSource> {
    if crate::frontend::contains_function_component(&frontend.entry_source, root_component)? {
        return Ok(ComponentSource {
            path: frontend.entry_path.clone(),
            source_label: frontend.entry.to_string_lossy().to_string(),
            source: frontend.entry_source.clone(),
            css_side_effect_imports: frontend.css_side_effect_imports.clone(),
        });
    }

    let imports = crate::frontend::extract_local_component_imports(&frontend.entry_source)
        .context("reading local component imports")?;
    let Some(component_import) = imports
        .iter()
        .find(|import| import.imported_name == root_component)
    else {
        bail!(
            "WASM root component `{root_component}` was not declared in {} and no matching local import was found",
            frontend.entry.display()
        );
    };

    let path = crate::frontend::resolve_local_import_path(
        &frontend.entry_path,
        &component_import.specifier,
    )?;
    let path = path.canonicalize().unwrap_or(path);
    let source = fs::read_to_string(&path)
        .with_context(|| format!("reading WASM component source: {}", path.display()))?;
    if !crate::frontend::contains_function_component(&source, root_component)? {
        bail!(
            "local import `{}` resolved to {}, but it does not declare function component `{root_component}`",
            component_import.specifier,
            path.display()
        );
    }
    let css_side_effect_imports = crate::frontend::extract_css_side_effect_imports(&source)
        .with_context(|| {
            format!(
                "reading CSS imports from WASM component source: {}",
                path.display()
            )
        })?;
    let root_dir = frontend
        .root_dir
        .canonicalize()
        .unwrap_or_else(|_| frontend.root_dir.clone());
    let source_label = path
        .strip_prefix(&root_dir)
        .unwrap_or(&path)
        .to_string_lossy()
        .to_string();
    Ok(ComponentSource {
        path,
        source_label,
        source,
        css_side_effect_imports,
    })
}

fn copy_style_import_groups(
    root_dir: &Path,
    style_groups: &[(&Path, &[String])],
    dist: &Path,
) -> Result<Option<String>> {
    if style_groups.iter().all(|(_, imports)| imports.is_empty()) {
        return Ok(None);
    }

    let root = root_dir
        .canonicalize()
        .with_context(|| format!("canonicalizing project root: {}", root_dir.display()))?;
    let mut bundle = String::new();
    let mut seen = std::collections::BTreeSet::new();

    for (importer_path, style_imports) in style_groups {
        let importer_dir = importer_path.parent().unwrap_or(root_dir);
        for import in *style_imports {
            if !(import.starts_with("./") || import.starts_with("../")) {
                bail!(
                    "CSS import `{import}` in WASM source {} must be relative so Jet can bundle it",
                    importer_path.display()
                );
            }
            let path = importer_dir.join(import);
            let canonical = path
                .canonicalize()
                .with_context(|| format!("resolving CSS import `{import}`"))?;
            if !canonical.starts_with(&root) {
                bail!(
                    "CSS import `{import}` resolves outside project root: {}",
                    canonical.display()
                );
            }
            if !seen.insert(canonical.clone()) {
                continue;
            }
            let css = fs::read_to_string(&canonical)
                .with_context(|| format!("reading CSS import `{import}`"))?;
            bundle.push_str("/* ");
            bundle.push_str(import);
            bundle.push_str(" */\n");
            bundle.push_str(&css);
            if !bundle.ends_with('\n') {
                bundle.push('\n');
            }
        }
    }

    fs::write(dist.join("style.css"), bundle).context("writing dist/style.css")?;
    Ok(Some("style.css".to_string()))
}

fn write_target_manifest(
    root_dir: &Path,
    dist: &Path,
    cfg: &config::WasmConfig,
    target: crate::build_target::BuildTarget,
    profile: Profile,
    cargo_features: Vec<String>,
) -> Result<()> {
    let jet_config_path = root_dir.join("jet.config.toml");
    let m = manifest::Manifest::build(manifest::ManifestInputs {
        target,
        profile_mode: match profile {
            Profile::Release => "release",
            Profile::Dev => "dev",
        },
        entry: &cfg.entry,
        root_component: &cfg.root_component,
        jet_config_path: &jet_config_path,
        cargo_features,
    })?;
    manifest::write(dist, &m)
}

/// The exact feature set passed to cargo for this `(target, profile)`
/// combo. The current flattened Jet layout builds the WASM app from
/// `jet-wasm` directly; the removed `jet-multi-target` crate is no
/// longer part of the scaffold. Recorded in `jet-target.json` so a CI
/// verifier can reproduce the build.
fn manifest_cargo_features(
    _target: crate::build_target::BuildTarget,
    profile: Profile,
    renderer: config::WasmRenderer,
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut jw: Vec<String> = match renderer {
        config::WasmRenderer::Canvas => vec![
            "jet-wasm/react".into(),
            "jet-wasm/canvas".into(),
            "jet-wasm/canvas-app".into(),
        ],
        config::WasmRenderer::Dom => vec!["jet-wasm/react".into(), "jet-wasm/dom-app".into()],
        config::WasmRenderer::WebGpu => vec![
            "jet-wasm/react".into(),
            "jet-wasm/webgpu".into(),
            "jet-wasm/webgpu-app".into(),
        ],
    };
    if matches!(
        (profile, renderer),
        (Profile::Dev, config::WasmRenderer::Canvas)
    ) {
        jw.push("jet-wasm/debug".into());
    }
    out.extend(jw);
    out
}

fn emit_wasm_entry(cfg: &config::WasmConfig) -> String {
    // The transpiled module exposes `pub fn <snake>(args) -> Component`.
    // All renderer + event-loop wiring lives inside a jet-wasm app
    // helper, so the generated entry remains one line.
    //
    // snake-case name matches `to_snake` in tsx_to_rust/emit.rs.
    let factory = snake(&cfg.root_component);
    let args = cfg
        .root_props
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let run_expr = match cfg.renderer {
        config::WasmRenderer::Canvas => {
            format!(r#"jet_wasm::react::canvas_app::run("jet-canvas", {factory}({args}))"#)
        }
        config::WasmRenderer::Dom => {
            format!(r#"jet_wasm::react::dom_app::run("jet-root", {factory}({args}))"#)
        }
        config::WasmRenderer::WebGpu => {
            format!(
                r#"jet_wasm::react::webgpu_app::run("jet-canvas", {factory}({args})).map(|_| ())"#
            )
        }
    };

    format!(
        r#"

// ── WASM entry ──────────────────────────────────────────────────────
// Emitted by `jet build --wasm` after the transpiled component above.
// Mounts the root component and installs the browser event loop.

use wasm_bindgen::prelude::*;

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
#[wasm_bindgen(start)]
pub fn __jet_start() -> Result<(), JsValue> {{
    console_error_panic_hook::set_once();
    {run_expr}
}}
"#,
        run_expr = run_expr,
    )
}

fn snake(camel: &str) -> String {
    let mut out = String::new();
    for (i, ch) in camel.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn scaffold_cargo_project(
    work_dir: &Path,
    generated_rust: &str,
    profile: Profile,
    target: crate::build_target::BuildTarget,
    renderer: config::WasmRenderer,
) -> Result<()> {
    fs::create_dir_all(work_dir.join("src")).context("create work_dir/src")?;
    // Pin jet-wasm via path so this always builds against the in-repo
    // workspace version. A future "published" mode would version-pin.
    let workspace = workspace_root()?;
    let jet_wasm_abs = workspace
        .join("projects")
        .join("jet")
        .join("wasm")
        .canonicalize()
        .context("resolving jet-wasm path")?;

    // Feature list the scaffolded crate asks jet-wasm for. Canvas dev
    // keeps the JetDebug bridge + window.__jet_debug. WebGPU uses the
    // grid wasm renderer bridge, which does not expose the canvas debug
    // bridge yet.
    let features = match (profile, renderer) {
        (Profile::Release, config::WasmRenderer::Canvas) => r#""react", "canvas", "canvas-app""#,
        (Profile::Dev, config::WasmRenderer::Canvas) => {
            r#""react", "canvas", "canvas-app", "debug""#
        }
        (_, config::WasmRenderer::Dom) => r#""react", "dom-app""#,
        (_, config::WasmRenderer::WebGpu) => r#""react", "webgpu", "webgpu-app""#,
    };

    if matches!(target, crate::build_target::BuildTarget::Tui) {
        unreachable!("TUI target rejected at build_with_profile entry");
    }

    let cargo_toml = format!(
        r#"[package]
name = "jet-wasm-app"
version = "0.0.0"
edition = "2021"
publish = false

[lib]
crate-type = ["cdylib"]

[dependencies]
jet-wasm = {{ path = {jet_wasm_path:?}, features = [{features}] }}
wasm-bindgen = "=0.2.121"
wasm-bindgen-futures = "=0.4.71"
js-sys = "=0.3.98"
web-sys = "=0.3.98"
console_error_panic_hook = "0.1"

[profile.release]
opt-level = "s"
lto = true
codegen-units = 1

[package.metadata.wasm-pack.profile.release]
wasm-opt = false

[package.metadata.wasm-pack.profile.dev]
wasm-opt = false
"#,
        jet_wasm_path = jet_wasm_abs.to_string_lossy().to_string(),
        features = features,
    );
    // Prepend a bare `[workspace]` so this scaffolded crate is NOT
    // pulled into the outer workspace. It has its own Cargo.lock and
    // compiles in isolation.
    let cargo_toml = format!("[workspace]\n\n{cargo_toml}");
    fs::write(work_dir.join("Cargo.toml"), cargo_toml)?;
    fs::write(work_dir.join("src").join("lib.rs"), generated_rust)?;
    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    // Find the workspace root by walking up until Cargo.toml with
    // [workspace]. For this first cut, assume CARGO_MANIFEST_DIR is set
    // when invoked from the jet binary's `cargo run` / tests, otherwise
    // fall back to current_dir.
    let start = std::env::current_dir()?;
    workspace_root_from(&start)
}

/// Walk up from `start` looking for a `Cargo.toml` that contains a
/// `[workspace]` table.
///
/// GH #3123 — the previous implementation used
/// `fs::read_to_string(&manifest).unwrap_or_default()`, which silently
/// turned IO failures (permission denied, transient error, EISDIR)
/// into an empty body and caused the walk to skip a real workspace
/// root. When the walk then fell off the top of the filesystem the
/// user got the misleading "could not locate workspace root" message
/// even though the manifest existed — it just couldn't be read.
///
/// We now distinguish the two cases: a missing `[workspace]` table is
/// silent (an inner crate manifest is a normal thing to walk past);
/// an IO failure is logged to stderr with the manifest path so the
/// user has a breadcrumb if the walk ultimately fails.
fn workspace_root_from(start: &Path) -> Result<PathBuf> {
    let mut cur: &Path = start;
    loop {
        let manifest = cur.join("Cargo.toml");
        if manifest.exists() {
            match fs::read_to_string(&manifest) {
                Ok(body) => {
                    if body.contains("[workspace]") {
                        return Ok(cur.to_path_buf());
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[jet wasm] Cargo.toml at {} is unreadable: {e}; skipping during workspace-root walk (GH #3123)",
                        manifest.display()
                    );
                }
            }
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => break,
        }
    }
    bail!("could not locate workspace root from {}", start.display());
}

/// GH #3594 — resolve the rustup-toolchain bin path from $HOME without
/// collapsing the two `std::env::VarError` discriminants into a single
/// silent empty string.
///
/// Cases:
/// - `Ok(home)` → `(<home>/.rustup/toolchains/stable-aarch64-apple-darwin/bin, None)`.
/// - `Err(NotPresent)` → `(String::new(), Some(warn_msg))` — HOME is genuinely
///   unset (daemon contexts, fresh shells). Caller falls back to bare PATH.
/// - `Err(NotUnicode(_))` → `(String::new(), Some(warn_msg))` — HOME was
///   explicitly set to non-UTF-8 bytes (misconfiguration). The warn
///   names the distinction so log readers do not conflate the two.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
pub(crate) fn compute_rustup_bin(
    current: Result<String, std::env::VarError>,
) -> (String, Option<String>) {
    match current {
        Ok(home) => (
            format!("{home}/.rustup/toolchains/stable-aarch64-apple-darwin/bin"),
            None,
        ),
        Err(std::env::VarError::NotPresent) => (
            String::new(),
            Some(format_wasm_pack_env_warn("HOME", "not-present")),
        ),
        Err(std::env::VarError::NotUnicode(_)) => (
            String::new(),
            Some(format_wasm_pack_env_warn("HOME", "not-unicode")),
        ),
    }
}

/// GH #3594 — compose the PATH passed to `wasm-pack`. The prior code
/// produced `format!("{rustup_bin}:{path}")` even when `path == ""`,
/// leaving a trailing colon (POSIX = CWD lookup). When BOTH inputs
/// resolved to empty strings the child PATH was `""` — wasm-pack
/// lookup then fell back to libc defaults silently.
///
/// Cases for `current`:
/// - `Ok(path)` → prepend `rustup_bin` (if non-empty) with `:`. No warn.
/// - `Err(NotPresent)` → use `rustup_bin` only — no trailing colon.
///   No warn (canonical "no PATH" case).
/// - `Err(NotUnicode(_))` → use `rustup_bin` only — no trailing colon.
///   Warn naming the discarded inherited PATH.
///
/// When `rustup_bin` is empty AND the current path is empty, the
/// returned PATH is `""` — there is no PATH to compose against. The
/// caller's downstream wasm-pack spawn will fail with a meaningful
/// error rather than silently picking up CWD via a trailing colon.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
pub(crate) fn compose_wasm_pack_path(
    rustup_bin: &str,
    current: Result<String, std::env::VarError>,
) -> (String, Option<String>) {
    match current {
        Ok(path) => {
            if rustup_bin.is_empty() {
                (path, None)
            } else if path.is_empty() {
                (rustup_bin.to_string(), None)
            } else {
                (format!("{rustup_bin}:{path}"), None)
            }
        }
        Err(std::env::VarError::NotPresent) => (rustup_bin.to_string(), None),
        Err(std::env::VarError::NotUnicode(_)) => (
            rustup_bin.to_string(),
            Some(format_wasm_pack_env_warn("PATH", "not-unicode")),
        ),
    }
}

/// GH #3594 — build the warn message for a HOME/PATH lookup that
/// resolved to `NotPresent` or `NotUnicode`. Extracted so the wording
/// (tag + var + observed kind + consequence) is unit-testable without
/// provoking the actual platform case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
pub(crate) fn format_wasm_pack_env_warn(var: &str, observed_kind: &str) -> String {
    format!(
        "GH #3594 wasm-pack env lookup: {var} observed as {observed_kind}. \
         Falling back to a PATH that omits this contribution — if wasm-pack \
         fails with `command not found`, fix the {var} value. (POSIX trailing-\
         colon CWD-hijack is refused either way.)"
    )
}

fn run_wasm_pack(work_dir: &Path, profile: Profile) -> Result<()> {
    // Use the rustup toolchain explicitly — same workaround the POC uses
    // because Homebrew's rustc can't target wasm32-unknown-unknown on
    // this machine.
    //
    // GH #3594 — match HOME and PATH `VarError` explicitly. The prior
    // `unwrap_or_default()` chain collapsed NotPresent and NotUnicode
    // to `""`, which could combine to produce an empty PATH (POSIX
    // semantics: an empty PATH element resolves to CWD, letting CWD-
    // dropped binaries hijack wasm-pack resolution). Same family as
    // #3582 (PATH in runner/env.rs).
    let (rustup_bin, home_warn) = compute_rustup_bin(std::env::var("HOME"));
    if let Some(msg) = home_warn {
        tracing::warn!(target: "jet::wasm_build", "{}", msg);
    }
    let (path, path_warn) = compose_wasm_pack_path(&rustup_bin, std::env::var("PATH"));
    if let Some(msg) = path_warn {
        tracing::warn!(target: "jet::wasm_build", "{}", msg);
    }

    let profile_flag = match profile {
        Profile::Release => "--release",
        Profile::Dev => "--dev",
    };

    let status = Command::new("wasm-pack")
        .args(["build", "--target", "web", profile_flag, "--no-typescript"])
        .current_dir(work_dir)
        .env("PATH", path)
        .status()
        .context("spawning wasm-pack — install with `cargo install wasm-pack` if missing")?;
    if !status.success() {
        bail!("wasm-pack exited {}", status);
    }
    Ok(())
}

fn copy_pkg_outputs(from: &Path, to: &Path) -> Result<()> {
    // wasm-bindgen emits: <name>.js + <name>_bg.wasm (+ .d.ts etc).
    // We only need the first two. Rename to app.js / app.wasm for a
    // predictable boot.js import path.
    let js_src = from.join("jet_wasm_app.js");
    let wasm_src = from.join("jet_wasm_app_bg.wasm");
    if !js_src.exists() || !wasm_src.exists() {
        bail!(
            "wasm-bindgen output missing: {} / {}",
            js_src.display(),
            wasm_src.display()
        );
    }
    fs::copy(&js_src, to.join("app.js"))?;
    fs::copy(&wasm_src, to.join("app_bg.wasm"))?;
    Ok(())
}

fn boot_js(_cfg: &config::WasmConfig) -> String {
    // wasm-bindgen's generated app.js is an ES module exporting a
    // default `init` function. boot.js installs the thin browser
    // host adapter, then imports + runs the wasm module.
    r#"// boot.js — emitted by `jet build --wasm`.
// Instantiates the WASM module; the Rust `__jet_start` fn (tagged
// #[wasm_bindgen(start)]) runs automatically during init and mounts
// the app on #jet-canvas.

import init from './app.js';
import { installJetHost } from './jet-host.js';

installJetHost();

init('./app_bg.wasm').catch((err) => {
  console.error('jet-wasm init failed:', err);
});
"#
    .to_string()
}

fn host_bridge_js() -> String {
    r#"// jet-host.js — thin browser capability adapter for Jet WASM.
// App/domain/render logic belongs in app_bg.wasm. This file only
// exposes browser host capabilities that WASM cannot call directly.

function ensureJetHost() {
  if (globalThis.__JET_HOST__) return globalThis.__JET_HOST__;

  const adapter = Object.freeze({
    fetch(input, init) {
      return globalThis.fetch(input, init);
    },
    console: Object.freeze({
      log(...args) {
        globalThis.console?.log?.(...args);
      },
      warn(...args) {
        globalThis.console?.warn?.(...args);
      },
      error(...args) {
        globalThis.console?.error?.(...args);
      },
      info(...args) {
        globalThis.console?.info?.(...args);
      },
    }),
    localStorage: Object.freeze({
      getItem(key) {
        return globalThis.localStorage.getItem(key);
      },
      setItem(key, value) {
        globalThis.localStorage.setItem(key, value);
      },
      removeItem(key) {
        globalThis.localStorage.removeItem(key);
      },
      clear() {
        globalThis.localStorage.clear();
      },
    }),
  });

  Object.defineProperty(globalThis, '__JET_HOST__', {
    value: adapter,
    configurable: true,
  });
  return adapter;
}

export function installJetHost() {
  return ensureJetHost();
}

export function jet_bridge_fetch(input, init) {
  return ensureJetHost().fetch(input, init);
}

export function jet_bridge_console_log(...args) {
  return ensureJetHost().console.log(...args);
}

export function jet_bridge_console_warn(...args) {
  return ensureJetHost().console.warn(...args);
}

export function jet_bridge_console_error(...args) {
  return ensureJetHost().console.error(...args);
}

export function jet_bridge_console_info(...args) {
  return ensureJetHost().console.info(...args);
}

export const jet_bridge_console = Object.freeze({
  log: jet_bridge_console_log,
  warn: jet_bridge_console_warn,
  error: jet_bridge_console_error,
  info: jet_bridge_console_info,
});

export function jet_bridge_local_storage_get_item(key) {
  return ensureJetHost().localStorage.getItem(key);
}

export function jet_bridge_local_storage_set_item(key, value) {
  return ensureJetHost().localStorage.setItem(key, value);
}

export function jet_bridge_local_storage_remove_item(key) {
  return ensureJetHost().localStorage.removeItem(key);
}

export function jet_bridge_local_storage_clear() {
  return ensureJetHost().localStorage.clear();
}

export const jet_bridge_local_storage = Object.freeze({
  getItem: jet_bridge_local_storage_get_item,
  setItem: jet_bridge_local_storage_set_item,
  removeItem: jet_bridge_local_storage_remove_item,
  clear: jet_bridge_local_storage_clear,
});
"#
    .to_string()
}

fn index_html(template: &str, cfg: &config::WasmConfig, stylesheet: Option<&str>) -> String {
    crate::frontend::render_wasm_index_html(
        template,
        &cfg.root_component,
        stylesheet,
        matches!(cfg.renderer, config::WasmRenderer::Dom),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wasm_config(src: &str) -> config::WasmConfig {
        config::WasmConfig::parse_str(src, Path::new("/tmp/jet.config.toml")).unwrap()
    }

    #[test]
    fn manifest_features_default_to_canvas_app() {
        let features = manifest_cargo_features(
            crate::build_target::BuildTarget::Web,
            Profile::Dev,
            config::WasmRenderer::Canvas,
        );

        assert!(features.contains(&"jet-wasm/canvas".to_string()));
        assert!(features.contains(&"jet-wasm/canvas-app".to_string()));
        assert!(features.contains(&"jet-wasm/debug".to_string()));
        assert!(!features.contains(&"jet-wasm/webgpu-app".to_string()));
    }

    #[test]
    fn manifest_features_select_webgpu_app() {
        let features = manifest_cargo_features(
            crate::build_target::BuildTarget::Web,
            Profile::Dev,
            config::WasmRenderer::WebGpu,
        );

        assert!(features.contains(&"jet-wasm/webgpu".to_string()));
        assert!(features.contains(&"jet-wasm/webgpu-app".to_string()));
        assert!(!features.contains(&"jet-wasm/canvas-app".to_string()));
        assert!(!features.contains(&"jet-wasm/debug".to_string()));
    }

    #[test]
    fn emitted_entry_defaults_to_canvas_app() {
        let cfg = wasm_config(
            r#"
            [wasm]
            entry = "src/App.tsx"
            root_component = "App"
            "#,
        );

        let entry = emit_wasm_entry(&cfg);

        assert!(entry.contains("jet_wasm::react::canvas_app::run"));
        assert!(!entry.contains("jet_wasm::react::webgpu_app::run"));
    }

    #[test]
    fn emitted_entry_selects_webgpu_app() {
        let cfg = wasm_config(
            r#"
            [wasm]
            entry = "src/App.tsx"
            root_component = "App"
            renderer = "web-gpu"
            "#,
        );

        let entry = emit_wasm_entry(&cfg);

        assert!(entry.contains("jet_wasm::react::webgpu_app::run"));
        assert!(entry.contains(".map(|_| ())"));
        assert!(!entry.contains("jet_wasm::react::canvas_app::run"));
    }

    #[test]
    fn boot_installs_host_adapter_before_wasm_init() {
        let boot = boot_js(&wasm_config(
            r#"
            [wasm]
            entry = "src/App.tsx"
            root_component = "App"
            "#,
        ));

        let install_pos = boot.find("installJetHost();").unwrap();
        let init_pos = boot.find("init('./app_bg.wasm')").unwrap();
        assert!(boot.contains("import { installJetHost } from './jet-host.js';"));
        assert!(install_pos < init_pos);
    }

    #[test]
    fn host_bridge_exports_thin_browser_capabilities() {
        let host = host_bridge_js();

        for symbol in [
            "installJetHost",
            "jet_bridge_fetch",
            "jet_bridge_console",
            "jet_bridge_local_storage",
        ] {
            assert!(host.contains(symbol), "missing {symbol}");
        }
        assert!(host.contains("globalThis.fetch(input, init)"));
        assert!(!host.contains("ReactDOM"));
        assert!(!host.contains("createRoot"));
        assert!(!host.contains("document.createElement('div')"));
    }

    #[test]
    fn index_html_links_bundled_stylesheet_when_present() {
        let cfg = wasm_config(
            r#"
            [wasm]
            entry = "src/App.tsx"
            root_component = "App"
            "#,
        );

        let html = index_html(
            &crate::frontend::default_index_html(),
            &cfg,
            Some("style.css"),
        );

        assert!(html.contains(r#"<link rel="stylesheet" href="./style.css" />"#));
        assert!(html.contains(r#"<script type="module" src="./boot.js"></script>"#));
    }

    #[test]
    fn copy_style_imports_concatenates_relative_css() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("dist")).unwrap();
        fs::write(root.join("src/base.css"), "body { color: black; }").unwrap();
        fs::write(root.join("src/theme.css"), "#jet-canvas { width: 100vw; }").unwrap();

        let imports = ["./base.css".to_string(), "./theme.css".to_string()];
        let out = copy_style_import_groups(
            root,
            &[(&root.join("src/App.tsx"), imports.as_slice())],
            &root.join("dist"),
        )
        .unwrap();

        assert_eq!(out.as_deref(), Some("style.css"));
        let css = fs::read_to_string(root.join("dist/style.css")).unwrap();
        assert!(css.contains("/* ./base.css */"));
        assert!(css.contains("body { color: black; }"));
        assert!(css.contains("/* ./theme.css */"));
        assert!(css.contains("#jet-canvas { width: 100vw; }"));
    }

    #[test]
    fn resolve_component_source_follows_local_reactdom_entry_import() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/main.tsx"),
            r#"
import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './App.tsx';
import './styles.css';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(<App />);
"#,
        )
        .unwrap();
        fs::write(
            root.join("src/App.tsx"),
            r#"
interface AppProps {}

export function App({}: AppProps) {
  return <div id="root">hello</div>;
}
"#,
        )
        .unwrap();
        fs::write(root.join("src/styles.css"), "body { color: black; }").unwrap();

        let frontend =
            crate::frontend::FrontendSources::load(root, PathBuf::from("src/main.tsx")).unwrap();
        let component = resolve_component_source(&frontend, "App").unwrap();

        assert_eq!(component.source_label, "src/App.tsx");
        assert!(component.source.contains("export function App"));
        assert_eq!(
            frontend.css_side_effect_imports,
            vec!["./styles.css".to_string()]
        );
    }
}
// CODEGEN-END
