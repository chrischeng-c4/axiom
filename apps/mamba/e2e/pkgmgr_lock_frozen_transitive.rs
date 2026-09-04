//! Black-box contract: a frozen index built by `mamba index build` must carry
//! enough metadata for `mamba add` and `mamba lock` to resolve a transitive
//! dependency, honour its version specifier, and record a verifiable digest.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over a throwaway directory tree
//! and reads only files the binary wrote. It never links the resolver, never
//! reads a product-internal type, and never hand-writes an index file: the
//! index is whatever `mamba index build` produces from three real wheels, so a
//! green run proves the dependency channel is the wheel's own `METADATA` and
//! not a fixture the case planted. `apps/mamba/tests/pkgmgr/fixtures.rs` writes
//! a `metadata.toml` by hand beside each wheel; that shortcut is exactly what
//! this case must not take, which is why it is self-contained rather than an
//! include of the legacy tree.
//!
//! # The fixture graph
//!
//! ```text
//!   app 1.0  --Requires-Dist: lib>=1-->  lib {1.0, 2.0}
//! ```
//!
//! Two `lib` versions exist in the index and the specifier admits both, so a
//! resolver that picks the first directory listed, or the lowest satisfying
//! version, records `lib==1.0` and fails the pin assertion. One version would
//! have made the selection unobservable.
//!
//! # Facets
//!
//! - **Behavior**: `lib` is reached at all (transitive closure), pinned at the
//!   highest satisfying version, listed under `app`'s `dependencies`, and the
//!   `add`-rendered lock and the `lock`-rendered lock agree byte for byte, so
//!   neither writer can drift from the other.
//! - **Security (supply-chain integrity)**: every entry carries a `sha256`
//!   equal to the digest of the artifact bytes on disk and a `path` that
//!   resolves inside the index directory. An empty digest is the fail-open
//!   state `apps/mamba/src/pkgmanage/sync.rs` reads as "nothing to verify", so
//!   the digest assertion is the boundary this work item reaches; the
//!   containment assertion refuses a lock that points a later install step at
//!   an artifact outside the index it was told to use.
//! - **Performance**: the work item names no budget, so this case asserts no
//!   timing. Resolver speed belongs to `resolver-speed-parity-with-uv`.
//!
//! # Hermeticity
//!
//! No network, ever: every wheel is built in-process and the only package
//! source handed to the binary is a local directory. `HOME`, `MAMBA_CACHE_DIR`,
//! `MAMBA_FROZEN_INDEX`, and `MAMBA_INDEX_URL` are pinned into the temp tree or
//! removed so an ambient index URL cannot silently supply an answer. Nothing is
//! skipped and nothing is `#[ignore]`d: a missing tool or a failed spawn
//! panics with the step that needed it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};
use sha2::{Digest, Sha256};

/// The binary under test. `CARGO_BIN_EXE_*` is resolved by cargo at compile
/// time, so a missing binary is a build failure rather than a skipped case.
fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

/// Run the binary in `cwd` with an environment that cannot reach a registry
/// or a user-global cache.
fn run(cwd: &Path, home: &Path, args: &[&str]) -> Output {
    Command::new(mamba_bin())
        .args(args)
        .current_dir(cwd)
        .env("HOME", home)
        .env("MAMBA_CACHE_DIR", home.join("cache"))
        .env_remove("MAMBA_FROZEN_INDEX")
        .env_remove("MAMBA_INDEX_URL")
        .env_remove("XDG_CACHE_HOME")
        .output()
        .unwrap_or_else(|e| panic!("spawn {} {args:?}: {e}", mamba_bin().display()))
}

/// Require exit 0, reporting the whole invocation when it is not.
fn expect_ok(step: &str, args: &[&str], out: &Output) {
    if !out.status.success() {
        panic!(
            "step `{step}` failed: `mamba {}` exited {:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            args.join(" "),
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }
}

/// Build a real, importable wheel through the product's own wheel builder.
/// `requires` becomes the wheel's `Requires-Dist:` lines verbatim — the only
/// place this case states the dependency edge.
fn build_wheel(out_dir: &Path, name: &str, version: &str, requires: &[&str]) -> PathBuf {
    let filename = compose_filename(name, version, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-frozen-transitive");
    wheel_meta.tags.push("py3-none-any".into());
    let mut core_meta = CoreMetadata::new(name, version);
    core_meta.requires_dist = requires.iter().map(|r| (*r).to_string()).collect();
    let mut builder = WheelBuilder::new(filename, wheel_meta, core_meta);
    let module = name.replace(['-', '.'], "_").to_ascii_lowercase();
    builder.add_file(
        format!("{module}/__init__.py"),
        format!("__version__ = {version:?}\n"),
    );
    builder
        .build_to_dir(out_dir)
        .unwrap_or_else(|e| panic!("build wheel {name}-{version}: {e:?}"))
}

fn sha256_file(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// One `[[package]]` table of `mamba.lock`, read as data.
struct LockEntry {
    name: String,
    version: String,
    sha256: String,
    url: String,
    source_kind: String,
    path: String,
    dependencies: Vec<String>,
}

fn parse_lock(body: &str) -> Vec<LockEntry> {
    let doc: toml::Value = body
        .parse()
        .unwrap_or_else(|e| panic!("parse mamba.lock: {e}\n--- mamba.lock ---\n{body}"));
    let packages = doc
        .get("package")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| {
            panic!("mamba.lock has no [[package]] array\n--- mamba.lock ---\n{body}")
        });
    let string_at = |t: &toml::Value, key: &str| -> String {
        t.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    };
    packages
        .iter()
        .map(|t| LockEntry {
            name: string_at(t, "name"),
            version: string_at(t, "version"),
            sha256: string_at(t, "sha256"),
            url: string_at(t, "url"),
            source_kind: string_at(t, "source_kind"),
            path: string_at(t, "path"),
            dependencies: t
                .get("dependencies")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect()
}

fn entry<'a>(entries: &'a [LockEntry], name: &str, body: &str) -> &'a LockEntry {
    entries.iter().find(|e| e.name == name).unwrap_or_else(|| {
        panic!("mamba.lock has no [[package]] named `{name}`\n--- mamba.lock ---\n{body}")
    })
}

/// The distribution name of a dependency string, whatever specifier or marker
/// trails it (`lib`, `lib==2.0`, `lib>=1`, `lib>=1,<3; python_version>"3"`).
fn requirement_name(raw: &str) -> String {
    let cut = raw
        .find(|c: char| {
            c == '='
                || c == '<'
                || c == '>'
                || c == '!'
                || c == '~'
                || c == ';'
                || c == '['
                || c == '('
                || c == ' '
        })
        .unwrap_or(raw.len());
    raw[..cut].trim().to_ascii_lowercase()
}

fn names(entries: &[LockEntry]) -> Vec<String> {
    let mut out: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
    out.sort();
    out
}

#[test]
fn frozen_index_lock_pins_transitive_dependency_with_verifiable_digest() {
    let root = tempfile::tempdir().expect("create temp root");
    let root = root.path();
    let wheels = root.join("wheels");
    let index = root.join("index");
    let project = root.join("project");
    let home = root.join("home");
    for dir in [&wheels, &project, &home] {
        std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("create {}: {e}", dir.display()));
    }

    // ---- fixture: three real wheels, one dependency edge, stated only in
    // ---- app's own METADATA.
    let app_wheel = build_wheel(&wheels, "app", "1.0", &["lib>=1"]);
    let lib_1_wheel = build_wheel(&wheels, "lib", "1.0", &[]);
    let lib_2_wheel = build_wheel(&wheels, "lib", "2.0", &[]);

    // ---- step 1: freeze them into an index.
    let index_str = index.to_str().expect("index path is utf-8").to_string();
    let build_args = vec![
        "index",
        "build",
        "--out",
        index_str.as_str(),
        app_wheel.to_str().unwrap(),
        lib_1_wheel.to_str().unwrap(),
        lib_2_wheel.to_str().unwrap(),
    ];
    let out = run(root, &home, &build_args);
    expect_ok("index build", &build_args, &out);

    // Fixture self-check: a broken wheel builder or a changed index layout
    // must fail here, naming itself, rather than surfacing as a lock
    // assertion further down. `app` and `lib` are already PEP 503 normal
    // form, so the directory name is the package name.
    let indexed = |name: &str, version: &str, wheel: &Path| -> PathBuf {
        let want = index
            .join(name)
            .join(version)
            .join(wheel.file_name().unwrap());
        assert!(
            want.is_file(),
            "fixture: `mamba index build` did not stage {name}-{version} at {}",
            want.display()
        );
        want
    };
    let app_indexed = indexed("app", "1.0", &app_wheel);
    let lib_2_indexed = indexed("lib", "2.0", &lib_2_wheel);
    indexed("lib", "1.0", &lib_1_wheel);

    // ---- step 2: scaffold a project.
    let init_args = vec!["init"];
    let out = run(&project, &home, &init_args);
    expect_ok("init", &init_args, &out);
    assert!(
        project.join("mamba.toml").is_file(),
        "fixture: `mamba init` wrote no mamba.toml in {}",
        project.display()
    );

    // ---- step 3: add the direct dependency against the frozen index.
    let add_args = vec!["add", "app", "--index", index_str.as_str()];
    let out = run(&project, &home, &add_args);
    expect_ok("add app --index", &add_args, &out);

    let lock_path = project.join("mamba.lock");
    let add_lock = std::fs::read_to_string(&lock_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", lock_path.display()));
    let entries = parse_lock(&add_lock);

    // ---- step 4: the transitive closure is present and correctly selected.
    assert_eq!(
        names(&entries),
        vec!["app".to_string(), "lib".to_string()],
        "mamba.lock must pin `app` and its transitive `lib`\n--- mamba.lock ---\n{add_lock}"
    );

    let app = entry(&entries, "app", &add_lock);
    let lib = entry(&entries, "lib", &add_lock);
    assert_eq!(
        app.version, "1.0",
        "`app` must pin the only indexed version\n--- mamba.lock ---\n{add_lock}"
    );
    assert_eq!(
        lib.version, "2.0",
        "`lib>=1` must select the highest satisfying indexed version, not `1.0`\
         \n--- mamba.lock ---\n{add_lock}"
    );

    // ---- step 5: the edge is recorded, and no edge is invented.
    let app_deps: BTreeSet<String> = app.dependencies.iter().map(|d| requirement_name(d)).collect();
    assert_eq!(
        app_deps,
        BTreeSet::from(["lib".to_string()]),
        "`app`'s dependencies must name `lib` (raw: {:?})\n--- mamba.lock ---\n{add_lock}",
        app.dependencies
    );
    assert!(
        lib.dependencies.is_empty(),
        "`lib` declares no Requires-Dist, so its dependencies must be empty (raw: {:?})\
         \n--- mamba.lock ---\n{add_lock}",
        lib.dependencies
    );

    // ---- step 6: every entry carries the digest of the artifact on disk.
    for (locked, wheel) in [(app, app_indexed.as_path()), (lib, lib_2_indexed.as_path())] {
        let want = sha256_file(wheel);
        assert_eq!(
            locked.sha256,
            want,
            "`{}` must lock the sha256 of {} (an empty digest is the fail-open state \
             `mamba sync` reads as nothing-to-verify)\n--- mamba.lock ---\n{add_lock}",
            locked.name,
            wheel.display()
        );
    }

    // ---- step 7: every entry points at the artifact, inside the index.
    let index_real = std::fs::canonicalize(&index)
        .unwrap_or_else(|e| panic!("canonicalize {}: {e}", index.display()));
    for (locked, wheel) in [(app, app_indexed.as_path()), (lib, lib_2_indexed.as_path())] {
        assert!(
            !locked.path.is_empty(),
            "`{}` must lock a path to its wheel\n--- mamba.lock ---\n{add_lock}",
            locked.name
        );
        let locked_path = Path::new(&locked.path);
        assert!(
            locked_path.is_file(),
            "`{}` locks path `{}`, which is not a file\n--- mamba.lock ---\n{add_lock}",
            locked.name,
            locked.path
        );
        let locked_real = std::fs::canonicalize(locked_path)
            .unwrap_or_else(|e| panic!("canonicalize {}: {e}", locked.path));
        assert!(
            locked_real.starts_with(&index_real),
            "`{}` locks path `{}`, which resolves outside the index {}\
             \n--- mamba.lock ---\n{add_lock}",
            locked.name,
            locked_real.display(),
            index_real.display()
        );
        assert_eq!(
            locked_real,
            std::fs::canonicalize(wheel).unwrap(),
            "`{}` must lock the wheel `mamba index build` staged\n--- mamba.lock ---\n{add_lock}",
            locked.name
        );
    }

    // ---- step 8: the source is declared as the frozen index, with no URL.
    for locked in [app, lib] {
        assert_eq!(
            locked.source_kind, "index",
            "`{}` came from a frozen index and must say so\n--- mamba.lock ---\n{add_lock}",
            locked.name
        );
        assert_eq!(
            locked.url, "",
            "`{}` came from a frozen index, so it has no artifact URL\
             \n--- mamba.lock ---\n{add_lock}",
            locked.name
        );
    }

    // ---- step 9: `lock` reproduces what `add` wrote, byte for byte.
    let lock_args = vec!["lock", "--index", index_str.as_str()];
    let out = run(&project, &home, &lock_args);
    expect_ok("lock --index", &lock_args, &out);
    let relocked = std::fs::read_to_string(&lock_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", lock_path.display()));
    assert_eq!(
        relocked, add_lock,
        "`mamba lock --index` must reproduce the lock `mamba add --index` wrote, byte for byte\
         \n--- after add ---\n{add_lock}\n--- after lock ---\n{relocked}"
    );
}
