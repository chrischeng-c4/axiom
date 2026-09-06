//! Black-box contract: `mamba add` must read its argument as the PEP 508
//! requirement a `uv` user types — a range, a wildcard, a compatible release —
//! record that requirement in `mamba.toml` verbatim, resolve it against the
//! index into the same `mamba.lock` body `mamba lock` renders for that same
//! manifest, and keep **one identity per dependency** so a second spelling of
//! the same package replaces the first instead of standing beside it.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over throwaway directory trees and
//! reads only what the binary wrote: the bytes of `mamba.toml`, the bytes of
//! `mamba.lock`, the exit code, and stderr. It never links the spec parser,
//! the resolver, or the lock renderer — the three things under judgement here
//! — and it never hand-writes a `mamba.lock`. The one piece of product code it
//! does link is `wheel_build`, which is fixture material: it builds the real
//! wheels the registry serves, so the artifacts and their digests are a
//! package manager's own, not a string this file invented.
//!
//! # The observation point
//!
//! One `wiremock::MockServer` bound to loopback serves a two-name graph with
//! four releases of the package the user asks for, so the version a specifier
//! selects is a choice among alternatives rather than the only thing on offer:
//!
//! ```text
//!   addapp 1.0  --requires_dist: addlib>=1-->  addlib {1.5, 2.5}
//!   addapp 2.0  --requires_dist: addlib>=1-->
//!   addapp 2.5  --requires_dist: addlib>=1-->
//!   addapp 3.0  --requires_dist: addlib>=2-->
//! ```
//!
//! | route | answer |
//! |---|---|
//! | `GET /pypi/<dist>/json` | `404`, so the PEP 503 simple page is the only release list |
//! | `GET /simple/<dist>/` | `200 text/html`, one anchor per release with a `#sha256=` fragment |
//! | `GET /pypi/<dist>/<version>/json` | `200`, `{"info":{"requires_dist":[…]}}` — the only place an edge exists |
//! | `GET /files/<wheel>` | `200`, that wheel's own bytes |
//!
//! Every other path is unmounted, and an unmounted path is a `404`. The
//! registry wheels declare no `Requires-Dist` of their own, so `addlib` can
//! only reach a lock if the binary read the per-version JSON and believed it.
//! Every panic message folds in the requests the registry received, so a red
//! says what the client actually asked for.
//!
//! `--index-url` is passed as `http://127.0.0.1:<port>/simple`, the PEP 503
//! spelling a `uv` user has in their fingers.
//!
//! Because the four releases of `addapp` are distinguishable, each specifier
//! shape names a different answer and no two of them can be satisfied by the
//! same accident:
//!
//! | spec | the release it admits | why that release |
//! |---|---|---|
//! | `addapp>=2,<3` | `2.5` | the newest below `3.0` |
//! | `addapp==2.*` | `2.5` | the newest whose version begins `2.` |
//! | `addapp~=2.0` | `2.5` | `>=2.0` and `==2.*` |
//! | `addapp<3` | `2.5` | the newest below `3.0` |
//! | `addapp>=9` | none | the refusal below |
//! | `addapp` | `3.0` | the bare-name grammar, unchanged |
//! | `addapp==2.0` | `2.0` | the exact-pin grammar, unchanged |
//!
//! `3.0` is the only release requiring `addlib>=2`, so the `addlib` edge also
//! says which `addapp` was chosen: a lock naming `addapp==3.0` for a spec that
//! excludes it is a lock that ignored the bound.
//!
//! # Why today's tree cannot pass
//!
//! `mamba add` splits its argument on `==` and takes anything else whole as
//! the package name, so `addapp>=2,<3` is looked up as a *package called*
//! `addapp>=2,<3` and refused with `not found on index`, while `addapp==2.*`
//! is looked up as version `2.*` of `addapp` and refused with `version 2.*
//! not on index`. The manifest line it writes is always `name==version`, and
//! the identity two manifest entries are compared by is the raw text before
//! `==` — so `AddApp==3.0` and `addapp==3.0` are two dependencies, and
//! `mamba remove addapp` cannot see an entry spelled `addapp>=2,<3` at all
//! and reports it as a no-op.
//!
//! # Facets
//!
//! - **Behavior**: the grammar `add` accepts (`add_records_the_range_spec…`,
//!   `add_accepts_a_wildcard_and_a_compatible_release_spec`), the agreement
//!   between `add` and `lock` on the same manifest
//!   (`add_writes_the_lock_that_lock_writes…`), the identity rule
//!   (`one_dependency_keeps_one_identity_across_its_spellings`), and the two
//!   grammars that must survive the change
//!   (`add_keeps_the_bare_name_and_exact_pin_grammars_it_already_had`).
//! - **Security**: two boundaries. *Fail closed*:
//!   `add_refuses_an_unsatisfiable_range_and_writes_nothing` pins that a
//!   refusal leaves the project exactly as it was — the manifest byte for
//!   byte, no `mamba.lock`, and no `mamba.lock.tmp`/`mamba.toml.tmp` half-write
//!   left behind for a later command to read as state — and that the refusal
//!   is a diagnosis naming the package and the bound, not a panic. The same
//!   assertion is made on the frozen `--index DIR` path, whose narrower
//!   grammar must refuse *before* writing rather than resolve loosely.
//!   *Supply chain*: `addlib` is a package the user never named, so its pin is
//!   all that stands between them and whatever the registry serves later —
//!   every entry is asserted to carry a non-empty `url` **and** a non-empty
//!   `sha256` before the pairing is compared, so an empty one (the fail-open
//!   state `mamba sync` reads as nothing-to-verify) cannot satisfy the pairing
//!   vacuously, and each digest is compared against the bytes the registry
//!   actually serves at that entry's own `url`. *Input handling*: the user's
//!   requirement text is recorded verbatim and never rewritten — a range must
//!   not silently become an exact pin, which would narrow the user's stated
//!   intent behind their back — while identity comparison is normalised, so
//!   case is not a way to smuggle a second copy of one package into a project.
//! - **Performance**: this work item names no budget, so this case asserts no
//!   timing, no request count, and no concurrency.
//!
//! # Hermeticity
//!
//! No network, ever: every wheel is built in-process and the only package
//! sources handed to the binary are a loopback `wiremock` server and a local
//! directory. `HOME` and `MAMBA_CACHE_DIR` are pinned into the temp tree;
//! `MAMBA_FROZEN_INDEX`, `MAMBA_JOBS`, `XDG_CACHE_HOME`, `VIRTUAL_ENV`,
//! `PYTHONPATH` and every proxy variable are removed, so no ambient index and
//! no proxy can supply or divert an answer. `MAMBA_INDEX_URL` is removed
//! everywhere except the two `mamba remove` runs that this work item
//! specifies with it set — `remove` is an offline command, and setting it
//! there proves a no-op removal is not merely a missing package source. The
//! binary is only ever run inside a temporary project directory. No
//! interpreter is needed: nothing here syncs or runs a script. Nothing is
//! skipped, nothing is `#[ignore]`d, and nothing sleeps or retries.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};
use sha2::{Digest, Sha256};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// --------------------------------------------------------------------------
// the graph the registry serves
// --------------------------------------------------------------------------
const APP: &str = "addapp";
const LIB: &str = "addlib";

const APP_1_0: &str = "1.0";
const APP_2_0: &str = "2.0";
/// The release every range in this case selects: the newest `addapp` below
/// `3.0`.
const APP_2_5: &str = "2.5";
/// The newest release, which the bare-name grammar selects and which every
/// range here excludes.
const APP_3_0: &str = "3.0";

const LIB_1_5: &str = "1.5";
/// The newest `addlib`, which satisfies both edges, so the `addlib` pin never
/// varies and the `addapp` pin is the only thing a specifier moves.
const LIB_2_5: &str = "2.5";

/// The edge `1.0`, `2.0` and `2.5` declare.
const REQUIRES_LIB_1: &str = "addlib>=1";
/// The edge only `3.0` declares, so the lock's `addlib` edge also identifies
/// which `addapp` was picked.
const REQUIRES_LIB_2: &str = "addlib>=2";

// --------------------------------------------------------------------------
// the specs under test
// --------------------------------------------------------------------------
/// The Goal's spec.
const RANGE: &str = "addapp>=2,<3";
/// A second range over the same name, used to prove one name keeps one entry.
const RANGE_OPEN: &str = "addapp<3";
const WILDCARD: &str = "addapp==2.*";
const COMPATIBLE: &str = "addapp~=2.0";
/// A range no release satisfies. The bound is spelled out separately because
/// the refusal has to name it: "no such package" is a different diagnosis
/// from "no release satisfies this bound", and only the second is true.
const UNSATISFIABLE: &str = "addapp>=9";
const UNSATISFIABLE_BOUND: &str = ">=9";
/// The same package, spelled the way a human types it in prose.
const MIXED_CASE: &str = "AddApp";
/// The exact pin grammar `add` already accepts.
const EXACT_PIN_2_0: &str = "addapp==2.0";
/// A range on the frozen path, whose narrower grammar refuses it.
const FROZEN_RANGE: &str = "addapp>=2";

/// The no-op line `mamba remove` prints when it could not find the entry. A
/// removal that reports this has left the dependency in the manifest.
const NOT_RECORDED: &str = "was not recorded";
/// The substring a crash leaves on stderr. A refusal is a diagnosis, not a
/// panic.
const PANIC_MARKER: &str = "panicked at";

/// The binary under test. `CARGO_BIN_EXE_*` is resolved by cargo at compile
/// time, so a missing binary is a build failure rather than a skipped case.
fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

/// Run the binary in `cwd` with an environment that can reach neither an
/// ambient index, nor a user-global cache, nor a proxy. `ambient_index_url`
/// is the one deliberate exception: `Some` sets `MAMBA_INDEX_URL`, which the
/// two `mamba remove` runs need so that a no-op removal cannot be blamed on a
/// missing package source.
fn run_in(cwd: &Path, home: &Path, args: &[&str], ambient_index_url: Option<&str>) -> Output {
    let mut cmd = Command::new(mamba_bin());
    cmd.args(args)
        .current_dir(cwd)
        .env("HOME", home)
        .env("MAMBA_CACHE_DIR", home.join("cache"))
        .env_remove("MAMBA_FROZEN_INDEX")
        .env_remove("MAMBA_JOBS")
        .env_remove("XDG_CACHE_HOME")
        .env_remove("VIRTUAL_ENV")
        .env_remove("PYTHONPATH")
        .env_remove("HTTP_PROXY")
        .env_remove("HTTPS_PROXY")
        .env_remove("ALL_PROXY")
        .env_remove("http_proxy")
        .env_remove("https_proxy")
        .env_remove("all_proxy");
    match ambient_index_url {
        Some(url) => cmd.env("MAMBA_INDEX_URL", url),
        None => cmd.env_remove("MAMBA_INDEX_URL"),
    };
    cmd.output()
        .unwrap_or_else(|e| panic!("spawn {} {args:?}: {e}", mamba_bin().display()))
}

fn render(args: &[&str], out: &Output) -> String {
    format!(
        "`mamba {}` exited {:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        args.join(" "),
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    )
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

// --------------------------------------------------------------------------
// reading what the binary wrote
// --------------------------------------------------------------------------

/// One `[[package]]` table of `mamba.lock`, read as data. Kept as a `Vec`
/// rather than a map keyed by name, because "the same name twice" is a state
/// this file exists to observe.
struct LockEntry {
    name: String,
    version: String,
    sha256: String,
    url: String,
    direct: Option<bool>,
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
            direct: t.get("direct").and_then(|v| v.as_bool()),
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

/// Every `name==version` the lock pins, in file order.
fn pins_of(entries: &[LockEntry]) -> Vec<String> {
    entries
        .iter()
        .map(|e| format!("{}=={}", e.name, e.version))
        .collect()
}

/// The one entry named `name`, refusing both zero and two.
fn only_entry<'a>(entries: &'a [LockEntry], name: &str, body: &str) -> &'a LockEntry {
    let matching: Vec<&LockEntry> = entries.iter().filter(|e| e.name == name).collect();
    assert_eq!(
        matching.len(),
        1,
        "mamba.lock must hold exactly one [[package]] named `{name}`; it holds \
         {}, and the lock pins {:?}\n--- mamba.lock ---\n{body}",
        matching.len(),
        pins_of(entries)
    );
    matching[0]
}

// --------------------------------------------------------------------------
// the project the binary runs in
// --------------------------------------------------------------------------

/// One scaffolded project: a throwaway `HOME` with its own cache, and a
/// `mamba init`ed directory. The binary is never run anywhere else.
struct Project {
    _root: tempfile::TempDir,
    dir: PathBuf,
    home: PathBuf,
}

impl Project {
    fn start() -> Project {
        let root_dir = tempfile::tempdir().expect("fixture: create temp root");
        let root = root_dir.path().to_path_buf();
        let dir = root.join("project");
        let home = root.join("home");
        for target in [&dir, &home] {
            std::fs::create_dir_all(target)
                .unwrap_or_else(|e| panic!("fixture: create {}: {e}", target.display()));
        }
        let init_args = ["init"];
        let out = run_in(&dir, &home, &init_args, None);
        assert!(
            out.status.success(),
            "fixture: `mamba init` must succeed: {}",
            render(&init_args, &out)
        );
        let project = Project {
            _root: root_dir,
            dir,
            home,
        };
        assert_eq!(
            project.dependencies_line(),
            "dependencies = []",
            "fixture: `mamba init` must scaffold an empty dependency list, \
             which is the state every assertion below is measured against\n\
             --- mamba.toml ---\n{}",
            project.manifest_text()
        );
        assert!(
            !project.lock_path().exists(),
            "fixture: `mamba init` must not write a mamba.lock; this case reads \
             the absence of one as evidence that a refusal wrote nothing"
        );
        project
    }

    fn run(&self, args: &[&str]) -> Output {
        run_in(&self.dir, &self.home, args, None)
    }

    /// Run with `MAMBA_INDEX_URL` set. Used only for `mamba remove`, so that a
    /// no-op removal cannot be explained away as a missing package source.
    fn run_with_ambient_index(&self, args: &[&str], index_url: &str) -> Output {
        run_in(&self.dir, &self.home, args, Some(index_url))
    }

    fn manifest_path(&self) -> PathBuf {
        self.dir.join("mamba.toml")
    }

    fn lock_path(&self) -> PathBuf {
        self.dir.join("mamba.lock")
    }

    fn manifest_text(&self) -> String {
        let target = self.manifest_path();
        std::fs::read_to_string(&target)
            .unwrap_or_else(|e| panic!("read {}: {e}", target.display()))
    }

    fn manifest_bytes(&self) -> Vec<u8> {
        let target = self.manifest_path();
        std::fs::read(&target).unwrap_or_else(|e| panic!("read {}: {e}", target.display()))
    }

    /// The manifest's dependency list, re-read from disk and rendered back as
    /// one canonical `dependencies = [...]` line. Rendering from the parsed
    /// value rather than matching the raw text means the assertion is on the
    /// list the product recorded — its contents, its order and its exact
    /// spelling — and not on how many spaces the writer indents with.
    fn dependencies_line(&self) -> String {
        let body = self.manifest_text();
        let doc: toml::Value = body
            .parse()
            .unwrap_or_else(|e| panic!("parse mamba.toml: {e}\n--- mamba.toml ---\n{body}"));
        let items = doc
            .get("project")
            .and_then(|p| p.get("dependencies"))
            .and_then(|d| d.as_array())
            .unwrap_or_else(|| {
                panic!(
                    "mamba.toml has no `[project] dependencies` array\n\
                     --- mamba.toml ---\n{body}"
                )
            });
        let rendered: Vec<String> = items
            .iter()
            .map(|v| {
                let s = v.as_str().unwrap_or_else(|| {
                    panic!(
                        "mamba.toml records a non-string dependency {v:?}\n\
                         --- mamba.toml ---\n{body}"
                    )
                });
                format!("{s:?}")
            })
            .collect();
        format!("dependencies = [{}]", rendered.join(", "))
    }

    /// Rewrite the manifest's one `dependencies` line, leaving every other
    /// byte `mamba init` wrote in place. Used only where this work item's own
    /// premise was measured that way — a requirement string a user hand-wrote
    /// into `mamba.toml`, which `mamba remove` must still recognise.
    fn set_dependencies(&self, deps: &[&str]) {
        let manifest = self.manifest_path();
        let body = self.manifest_text();
        let rendered = format!(
            "dependencies = [{}]",
            deps.iter()
                .map(|d| format!("{d:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let mut replaced = 0usize;
        let mut out = String::with_capacity(body.len() + rendered.len());
        for line in body.lines() {
            if line.starts_with("dependencies = ") {
                out.push_str(&rendered);
                replaced += 1;
            } else {
                out.push_str(line);
            }
            out.push('\n');
        }
        assert_eq!(
            replaced,
            1,
            "fixture: the manifest must carry exactly one `dependencies = …` \
             line to rewrite\n--- {} ---\n{body}",
            manifest.display()
        );
        std::fs::write(&manifest, out)
            .unwrap_or_else(|e| panic!("fixture: write {}: {e}", manifest.display()));
    }

    fn lock_body(&self) -> String {
        let target = self.lock_path();
        std::fs::read_to_string(&target)
            .unwrap_or_else(|e| panic!("read {}: {e}", target.display()))
    }

    /// The lock, parsed, together with its bytes for panic messages.
    fn lock(&self) -> (String, Vec<LockEntry>) {
        let body = self.lock_body();
        let entries = parse_lock(&body);
        (body, entries)
    }
}

// --------------------------------------------------------------------------
// the registry
// --------------------------------------------------------------------------

/// One artifact the registry serves: its filename, where it lives, and what
/// its bytes hash to. The digest is taken over the exact bytes mounted at
/// `url`, so a lock entry is always compared against one real file.
struct Artifact {
    file: String,
    /// The path the artifact is mounted at, relative to the registry base.
    url: String,
    digest: String,
    bytes: Vec<u8>,
}

/// One release the registry publishes: a distribution at a version, with the
/// `requires_dist` lines the registry declares for it. The registry's wheels
/// declare nothing themselves, so this list is the only place the edge exists.
struct Release {
    dist: &'static str,
    version: &'static str,
    requires: &'static [&'static str],
}

/// The graph every registry-backed test in this file serves.
fn graph() -> Vec<Release> {
    vec![
        Release {
            dist: APP,
            version: APP_1_0,
            requires: &[REQUIRES_LIB_1],
        },
        Release {
            dist: APP,
            version: APP_2_0,
            requires: &[REQUIRES_LIB_1],
        },
        Release {
            dist: APP,
            version: APP_2_5,
            requires: &[REQUIRES_LIB_1],
        },
        Release {
            dist: APP,
            version: APP_3_0,
            requires: &[REQUIRES_LIB_2],
        },
        Release {
            dist: LIB,
            version: LIB_1_5,
            requires: &[],
        },
        Release {
            dist: LIB,
            version: LIB_2_5,
            requires: &[],
        },
    ]
}

/// Build one real wheel through the product's own wheel builder. `requires`
/// becomes the wheel's own `Requires-Dist:` lines; the registry passes an
/// empty list so that the only channel for an edge there is the per-version
/// JSON, while the frozen index needs the edge in the wheel itself.
fn build_wheel_file(out_dir: &Path, dist: &str, version: &str, requires: &[&str]) -> PathBuf {
    let filename = compose_filename(dist, version, "py3", "none", "any");
    let rendered = filename.to_filename();
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-add-requirement-specifier");
    wheel_meta.tags.push("py3-none-any".into());
    let mut core_meta = CoreMetadata::new(dist, version);
    core_meta.requires_dist = requires.iter().map(|r| (*r).to_string()).collect();
    let mut builder = WheelBuilder::new(filename, wheel_meta, core_meta);
    let module = dist.replace(['-', '.'], "_").to_ascii_lowercase();
    builder.add_file(
        format!("{module}/__init__.py"),
        format!("__version__ = {version:?}\n"),
    );
    let built = builder
        .build_to_dir(out_dir)
        .unwrap_or_else(|e| panic!("fixture: build wheel {dist}-{version}: {e:?}"));
    assert_eq!(
        built.file_name().and_then(|n| n.to_str()),
        Some(rendered.as_str()),
        "fixture: the wheel builder named the artifact {}",
        built.display()
    );
    built
}

/// Build one wheel and describe it as the artifact a registry advertises.
fn build_artifact(dist: &str, version: &str) -> Artifact {
    let dir = tempfile::tempdir().expect("fixture: create a temp dir for the wheel");
    let built = build_wheel_file(dir.path(), dist, version, &[]);
    let file = built
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_else(|| {
            panic!(
                "fixture: the wheel builder named the artifact {}",
                built.display()
            )
        })
        .to_string();
    let bytes =
        std::fs::read(&built).unwrap_or_else(|e| panic!("fixture: read {}: {e}", built.display()));
    let digest = sha256_bytes(&bytes);
    Artifact {
        url: format!("/files/{file}"),
        file,
        digest,
        bytes,
    }
}

/// The PEP 503 page the registry serves for one distribution: one anchor per
/// release, each carrying the artifact URL and its `#sha256=` fragment,
/// exactly as a real simple index writes it.
fn simple_page(dist: &str, artifacts: &[&Artifact], base: &str) -> String {
    let anchors: String = artifacts
        .iter()
        .map(|a| {
            format!(
                "<a href=\"{base}{}#sha256={}\" data-requires-python=\"&gt;=3.8\">{}</a><br/>\n",
                a.url, a.digest, a.file
            )
        })
        .collect();
    format!(
        "<!DOCTYPE html>\n\
         <html>\n\
         <head><meta name=\"pypi:repository-version\" content=\"1.0\"><title>Links for {dist}</title></head>\n\
         <body>\n\
         <h1>Links for {dist}</h1>\n\
         {anchors}\
         </body>\n\
         </html>\n"
    )
}

/// A loopback registry serving the graph, up for the whole test.
///
/// `server` is declared before `rt` so the server is shut down while the
/// runtime that started it is still alive. Neither field is leaked: a
/// `mem::forget`-ed server outlives the observation it is meant to bound, and
/// the request log a failure reads would go with it.
struct Registry {
    server: MockServer,
    rt: tokio::runtime::Runtime,
    /// `http://127.0.0.1:<port>` — the registry base, with no trailing slash.
    base: String,
    /// Every release served, paired with the artifact built for it.
    artifacts: Vec<(&'static str, &'static str, Artifact)>,
}

impl Registry {
    /// Serve exactly `releases`: one PEP 503 page per distribution listing its
    /// releases, one per-version JSON document per release carrying its edges,
    /// and the wheel bytes each anchor points at.
    fn start(releases: &[Release]) -> Registry {
        let artifacts: Vec<(&'static str, &'static str, Artifact)> = releases
            .iter()
            .map(|r| (r.dist, r.version, build_artifact(r.dist, r.version)))
            .collect();
        for (i, (left_dist, left_version, left)) in artifacts.iter().enumerate() {
            for (right_dist, right_version, right) in artifacts.iter().skip(i + 1) {
                assert_ne!(
                    left.digest, right.digest,
                    "fixture: {left_dist}=={left_version} and \
                     {right_dist}=={right_version} hash to the same bytes, so \
                     this case could not tell which artifact a lock entry \
                     described"
                );
            }
        }
        let mut dists: Vec<&str> = releases.iter().map(|r| r.dist).collect();
        dists.sort_unstable();
        dists.dedup();

        let rt = tokio::runtime::Runtime::new().expect("fixture: build a runtime for the registry");
        let (server, base) = rt.block_on(async {
            let server = MockServer::start().await;
            let base = server.uri();

            for dist in &dists {
                // The project-level JSON endpoint says "no such project", so
                // the PEP 503 simple page is the only route to a release list.
                // Mounted explicitly rather than left to the unmatched-request
                // default, so the fallback is a served answer.
                Mock::given(method("GET"))
                    .and(path(format!("/pypi/{dist}/json")))
                    .respond_with(ResponseTemplate::new(404))
                    .mount(&server)
                    .await;

                let page_artifacts: Vec<&Artifact> = artifacts
                    .iter()
                    .filter(|(d, _, _)| d == dist)
                    .map(|(_, _, a)| a)
                    .collect();
                let page = simple_page(dist, &page_artifacts, &base);
                Mock::given(method("GET"))
                    .and(path(format!("/simple/{dist}/")))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .set_body_raw(page.into_bytes(), "text/html; charset=utf-8"),
                    )
                    .mount(&server)
                    .await;
            }

            // The only place a dependency edge exists.
            for release in releases {
                let body = format!(
                    "{{\"info\":{{\"requires_dist\":[{}]}}}}",
                    release
                        .requires
                        .iter()
                        .map(|r| format!("{r:?}"))
                        .collect::<Vec<_>>()
                        .join(",")
                );
                Mock::given(method("GET"))
                    .and(path(format!(
                        "/pypi/{}/{}/json",
                        release.dist, release.version
                    )))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .set_body_raw(body.into_bytes(), "application/json"),
                    )
                    .mount(&server)
                    .await;
            }

            for (_, _, artifact) in &artifacts {
                Mock::given(method("GET"))
                    .and(path(artifact.url.clone()))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .set_body_raw(artifact.bytes.clone(), "application/octet-stream"),
                    )
                    .mount(&server)
                    .await;
            }

            (server, base)
        });

        Registry {
            server,
            rt,
            base,
            artifacts,
        }
    }

    /// The `--index-url` every test passes: the PEP 503 simple URL, which the
    /// client normalises back to the registry base before building its own
    /// paths.
    fn index_url(&self) -> String {
        format!("{}/simple", self.base)
    }

    /// The artifact advertised for one release.
    fn artifact(&self, dist: &str, version: &str) -> &Artifact {
        self.artifacts
            .iter()
            .find(|(d, v, _)| *d == dist && *v == version)
            .map(|(_, _, a)| a)
            .unwrap_or_else(|| panic!("fixture: nothing was published for {dist}=={version}"))
    }

    /// The absolute URL an artifact is advertised at.
    fn absolute(&self, artifact: &Artifact) -> String {
        format!("{}{}", self.base, artifact.url)
    }

    /// Every request the registry saw, as `METHOD /path`, folded into each
    /// panic message so a red names what the client actually asked for — in
    /// particular whether it looked up a *package* whose name is the whole
    /// requirement string.
    fn seen(&self) -> String {
        let lines: Vec<String> = self
            .rt
            .block_on(self.server.received_requests())
            .unwrap_or_default()
            .iter()
            .map(|r| format!("{} {}", r.method, r.url.path()))
            .collect();
        format!(
            "--- requests the registry received ---\n{}",
            if lines.is_empty() {
                "(none)".to_string()
            } else {
                lines.join("\n")
            }
        )
    }

    /// Which advertised artifact a digest belongs to, in words, so a failure
    /// says *what* a lock entry described rather than only that it was wrong.
    fn describe_digest(&self, digest: &str) -> String {
        if digest.is_empty() {
            return "nothing — the digest is empty, which `mamba sync` reads as \
                    nothing to verify"
                .to_string();
        }
        for (dist, version, artifact) in &self.artifacts {
            if artifact.digest == digest {
                return format!("the wheel {} ({dist}=={version})", artifact.file);
            }
        }
        "no artifact this registry advertised".to_string()
    }
}

// --------------------------------------------------------------------------
// shared assertions
// --------------------------------------------------------------------------

/// The entry for `name` pins `version`, and the file it names is the one the
/// registry advertised for that very version — with a `url` and a `sha256`
/// that are both present before either is compared, because an empty one is
/// the fail-open state `mamba sync` reads as nothing to verify and would
/// otherwise satisfy the pairing vacuously.
fn assert_pinned_at(
    context: &str,
    registry: &Registry,
    entries: &[LockEntry],
    body: &str,
    name: &str,
    version: &str,
) {
    let entry = only_entry(entries, name, body);
    assert_eq!(
        entry.version,
        version,
        "{context}: `{name}` must be pinned at {version}, the release the \
         requirement admits\n--- mamba.lock ---\n{body}\n{}",
        registry.seen()
    );
    let artifact = registry.artifact(name, version);
    assert!(
        !entry.url.is_empty() && !entry.sha256.is_empty(),
        "{context}: the entry for `{name}` must carry both a url and a sha256, \
         or `mamba sync` has nothing to fetch and nothing to verify; got url \
         {:?} and sha256 {:?}\n--- mamba.lock ---\n{body}",
        entry.url,
        entry.sha256
    );
    assert_eq!(
        entry.url,
        registry.absolute(artifact),
        "{context}: `{name}` must be locked at the artifact URL this registry \
         advertised for {name}=={version}\n--- mamba.lock ---\n{body}"
    );
    assert_eq!(
        entry.sha256,
        artifact.digest,
        "{context}: the sha256 locked for `{name}` must be the digest of the \
         bytes served for {name}=={version}; it recorded {}\n\
         --- mamba.lock ---\n{body}",
        registry.describe_digest(&entry.sha256)
    );
}

/// The entry for `name` is recorded as direct or transitive as the manifest
/// says, and its edges are rendered against the versions actually pinned.
fn assert_edges(
    context: &str,
    entries: &[LockEntry],
    body: &str,
    name: &str,
    direct: bool,
    edges: &[String],
) {
    let entry = only_entry(entries, name, body);
    assert_eq!(
        entry.direct,
        Some(direct),
        "{context}: `{name}` must be recorded with `direct = {direct}` — the \
         package the user named is the direct one, and the closure is recorded \
         around it\n--- mamba.lock ---\n{body}"
    );
    assert_eq!(
        entry.dependencies,
        edges.to_vec(),
        "{context}: `{name}`'s edges must be rendered against the versions the \
         lock pinned — an edge naming a version no [[package]] holds, or an \
         edge that vanished between the registry and the lock, is a lock that \
         disagrees with itself\n--- mamba.lock ---\n{body}"
    );
}

/// The whole lock this graph must produce for a requirement that admits
/// `addapp` at `app_version`: two entries and nothing else, each paired with
/// the bytes the registry serves, `addapp` direct and carrying the pinned
/// edge, `addlib` transitive with no edges of its own.
fn assert_closure(
    context: &str,
    registry: &Registry,
    body: &str,
    entries: &[LockEntry],
    app_version: &str,
    lib_version: &str,
) {
    assert_eq!(
        pins_of(entries),
        vec![
            format!("{APP}=={app_version}"),
            format!("{LIB}=={lib_version}"),
        ],
        "{context}: the lock must hold exactly the closure of the requirement — \
         `{APP}` at {app_version} and the `{LIB}` its own metadata pulls in at \
         {lib_version}, and nothing else\n--- mamba.lock ---\n{body}\n{}",
        registry.seen()
    );
    assert_pinned_at(context, registry, entries, body, APP, app_version);
    assert_pinned_at(context, registry, entries, body, LIB, lib_version);
    assert_edges(
        context,
        entries,
        body,
        APP,
        true,
        &[format!("{LIB}=={lib_version}")],
    );
    assert_edges(context, entries, body, LIB, false, &[]);
}

/// Add one spec against the registry and require the binary to have stood
/// behind the answer: exit 0 and a silent stderr.
fn add_ok(project: &Project, registry: &Registry, spec: &str, why: &str) {
    let index_url = registry.index_url();
    let args = ["add", spec, "--index-url", index_url.as_str()];
    let out = project.run(&args);
    assert!(
        out.status.success(),
        "`mamba add {spec} --index-url <base>/simple` must succeed: {why}. \
         Today the argument is split on `==` and anything else is taken whole \
         as a package name, so the registry is asked for a project literally \
         called `{spec}`\n{}\n{}",
        render(&args, &out),
        registry.seen()
    );
    assert!(
        stderr_of(&out).trim().is_empty(),
        "`mamba add {spec} --index-url` succeeded but wrote to stderr; a \
         successful add is silent, so a caller can read any stderr as the \
         reason something did not happen\n{}",
        render(&args, &out)
    );
}

// --------------------------------------------------------------------------
// one — the Goal
// --------------------------------------------------------------------------

/// `mamba add "addapp>=2,<3" --index-url` records the requirement the user
/// wrote and locks the release it admits, with the closure that release pulls
/// in, paired to the bytes the registry serves.
#[test]
fn add_records_the_range_spec_and_locks_the_release_it_admits() {
    let registry = Registry::start(&graph());
    let project = Project::start();

    add_ok(
        &project,
        &registry,
        RANGE,
        "the registry publishes four releases of `addapp` and two of them \
         satisfy this range",
    );

    // The manifest keeps what the user wrote. Rewriting `addapp>=2,<3` into
    // `addapp==2.5` would narrow a stated intent into a pin the user never
    // asked for, and the next `mamba lock` would have nothing left to widen.
    assert_eq!(
        project.dependencies_line(),
        format!("dependencies = [{RANGE:?}]"),
        "`mamba add {RANGE}` must record the requirement verbatim in \
         mamba.toml: the range is the user's stated intent and belongs in the \
         manifest, while the pin it currently resolves to belongs in \
         mamba.lock\n--- mamba.toml ---\n{}",
        project.manifest_text()
    );

    let (body, entries) = project.lock();
    assert_closure("the Goal", &registry, &body, &entries, APP_2_5, LIB_2_5);

    // Spelled out once, in the Goal's own words: the `2.5` wheel's url and
    // sha256 are the pair the index advertises, so a lock that recorded the
    // right version against the wrong artifact still fails here.
    let wheel = registry.artifact(APP, APP_2_5);
    let locked = only_entry(&entries, APP, &body);
    assert_eq!(
        locked.url,
        registry.absolute(wheel),
        "the locked url for `{APP}` must be the one /simple/{APP}/ advertises \
         for {APP}=={APP_2_5} ({})\n--- mamba.lock ---\n{body}",
        wheel.file
    );
    assert_eq!(
        locked.sha256,
        wheel.digest,
        "the locked sha256 for `{APP}` must be the digest of the bytes served \
         at that url; it recorded {}\n--- mamba.lock ---\n{body}",
        registry.describe_digest(&locked.sha256)
    );
}

// --------------------------------------------------------------------------
// two — the other two specifier shapes a uv user types
// --------------------------------------------------------------------------

/// A wildcard and a compatible-release specifier are requirements like any
/// other: recorded verbatim, resolved to the newest release they admit.
#[test]
fn add_accepts_a_wildcard_and_a_compatible_release_spec() {
    let registry = Registry::start(&graph());

    for (spec, why) in [
        (
            WILDCARD,
            "`==2.*` admits every release whose version begins `2.`, of which \
             the registry publishes two",
        ),
        (
            COMPATIBLE,
            "`~=2.0` is `>=2.0` and `==2.*` together, which the registry's \
             `2.0` and `2.5` both satisfy",
        ),
    ] {
        let project = Project::start();
        add_ok(&project, &registry, spec, why);

        assert_eq!(
            project.dependencies_line(),
            format!("dependencies = [{spec:?}]"),
            "`mamba add {spec}` must record the requirement verbatim; today the \
             argument is split on `==`, so `{spec}` is looked up as a version \
             string rather than as a specifier\n--- mamba.toml ---\n{}",
            project.manifest_text()
        );

        let (body, entries) = project.lock();
        assert_closure(spec, &registry, &body, &entries, APP_2_5, LIB_2_5);
        // `3.0` is the only release requiring `addlib>=2`, so an `addlib` edge
        // reading `addlib==2.5` under a `3.0` pin would still be caught here:
        // the version the specifier excludes is named outright.
        assert_ne!(
            only_entry(&entries, APP, &body).version,
            APP_3_0,
            "`{spec}` must not select `{APP}=={APP_3_0}`, the release it \
             excludes\n--- mamba.lock ---\n{body}\n{}",
            registry.seen()
        );
    }
}

// --------------------------------------------------------------------------
// three — the fail-closed refusal
// --------------------------------------------------------------------------

/// A requirement no release satisfies is refused, and the refusal leaves the
/// project exactly as it was: the manifest byte for byte, no lock, and no
/// half-written temporary for a later command to read as state.
#[test]
fn add_refuses_an_unsatisfiable_range_and_writes_nothing() {
    let registry = Registry::start(&graph());

    // Fixture self-check: this registry does serve `addapp`, so the refusal
    // below is about the bound and not about a package the fixture forgot to
    // publish. A red here means the fixture is wrong rather than the refusal.
    let reachable = Project::start();
    add_ok(
        &reachable,
        &registry,
        EXACT_PIN_2_0,
        "fixture self-check: /simple/addapp/ advertises this release",
    );

    let project = Project::start();
    let before = project.manifest_bytes();
    let index_url = registry.index_url();
    let args = ["add", UNSATISFIABLE, "--index-url", index_url.as_str()];
    let out = project.run(&args);

    assert!(
        !out.status.success(),
        "`mamba add {UNSATISFIABLE}` must be refused: this registry publishes \
         `{APP}` at {APP_1_0}, {APP_2_0}, {APP_2_5} and {APP_3_0}, and none of \
         them satisfies `{UNSATISFIABLE_BOUND}`. Exiting 0 would leave the \
         caller with a manifest naming a requirement nothing can \
         satisfy\n{}\n{}",
        render(&args, &out),
        registry.seen()
    );
    let stderr = stderr_of(&out);
    assert!(
        stderr.contains(APP),
        "the refusal must name the package it could not satisfy, or the caller \
         cannot tell which requirement in the command was the problem\n{}",
        render(&args, &out)
    );
    assert!(
        stderr.contains(UNSATISFIABLE_BOUND),
        "the refusal must name the bound `{UNSATISFIABLE_BOUND}` it could not \
         satisfy; a message that drops it says only that something went \
         wrong\n{}",
        render(&args, &out)
    );
    assert!(
        !stderr.contains(PANIC_MARKER),
        "an unsatisfiable requirement is a diagnosis, not a crash: a panic \
         leaves no message a caller can act on and no promise about what was \
         written first\n{}",
        render(&args, &out)
    );

    // Fail closed. `mamba add` writes the manifest and the lock last, after
    // the lock body has been rendered, so a refusal anywhere before that must
    // leave both untouched — including the `.tmp` files the atomic writer
    // renames through, which a later command would otherwise find on disk.
    assert_eq!(
        project.manifest_bytes(),
        before,
        "a refused `mamba add` must leave mamba.toml byte for byte as it was; \
         a manifest carrying a requirement the command refused is a project \
         whose next `mamba lock` fails for a reason nobody recorded\n\
         --- mamba.toml now ---\n{}",
        project.manifest_text()
    );
    for leftover in ["mamba.lock", "mamba.lock.tmp", "mamba.toml.tmp"] {
        let stray = project.dir.join(leftover);
        assert!(
            !stray.exists(),
            "a refused `mamba add` must leave no `{leftover}` behind: this \
             project had none before the command, and a file written on the \
             way to a refusal is state no one decided to keep\n{}",
            render(&args, &out)
        );
    }
}

// --------------------------------------------------------------------------
// four — `add` and `lock` write the same bytes
// --------------------------------------------------------------------------

/// The lock `add` renders for a requirement is the lock `lock` renders for the
/// manifest it just wrote. The two commands share one renderer, and this is
/// the observation that keeps them sharing it.
#[test]
fn add_writes_the_lock_that_lock_writes_for_the_same_range_manifest() {
    let registry = Registry::start(&graph());
    let project = Project::start();

    add_ok(
        &project,
        &registry,
        RANGE,
        "this test's subject is the agreement between the two writers, which \
         cannot be observed until `add` has written a lock at all — that the \
         range is accepted is \
         `add_records_the_range_spec_and_locks_the_release_it_admits`'s claim, \
         and a red here alongside a red there is one defect, not two",
    );
    let after_add = project.lock_body();

    let index_url = registry.index_url();
    let lock_args = ["lock", "--index-url", index_url.as_str()];
    let out = project.run(&lock_args);
    assert!(
        out.status.success(),
        "`mamba lock --index-url` must resolve the manifest `mamba add` just \
         wrote: `{RANGE}` is a requirement `lock` already understands\n{}\n{}",
        render(&lock_args, &out),
        registry.seen()
    );

    let after_lock = project.lock_body();
    assert_eq!(
        after_lock,
        after_add,
        "`mamba add {RANGE} --index-url` must write the lock `mamba lock \
         --index-url` writes for the same manifest, byte for byte — otherwise \
         adding a dependency and re-locking are two different answers to one \
         question, and a project's lock depends on which command last touched \
         it\n--- after `mamba add` ---\n{after_add}\n--- after `mamba lock` ---\
         \n{after_lock}\n{}",
        registry.seen()
    );
    assert_eq!(
        project.dependencies_line(),
        format!("dependencies = [{RANGE:?}]"),
        "`mamba lock` must not rewrite the requirement in mamba.toml; the \
         manifest is the user's input to the lock, not the lock's output\n\
         --- mamba.toml ---\n{}",
        project.manifest_text()
    );
}

// --------------------------------------------------------------------------
// five — one dependency, one identity
// --------------------------------------------------------------------------

/// Record one violation of the identity rule instead of raising it, so a
/// single run reports every spelling that lost its identity rather than the
/// first. Nothing is skipped: every sub-case below runs unconditionally, and
/// any recorded violation fails the test at the end.
fn note(violations: &mut Vec<String>, ok: bool, message: String) {
    if !ok {
        violations.push(message);
    }
}

/// A package is one dependency however it is spelled. A second `add` of the
/// same name replaces the entry the first wrote — whatever specifier or
/// capitalisation either used — and `mamba remove <name>` finds the entry
/// whatever specifier trails it.
#[test]
fn one_dependency_keeps_one_identity_across_its_spellings() {
    let registry = Registry::start(&graph());
    let index_url = registry.index_url();
    let mut violations: Vec<String> = Vec::new();

    // ---- capitalisation is not a second package ------------------------
    // Both spellings resolve through the bare-name grammar that already
    // works, so this sub-case observes the identity rule and nothing else.
    let folded = Project::start();
    for spec in [MIXED_CASE, APP] {
        let args = ["add", spec, "--index-url", index_url.as_str()];
        let out = folded.run(&args);
        note(
            &mut violations,
            out.status.success(),
            format!(
                "`mamba add {spec} --index-url` must succeed — a bare name is \
                 the grammar `add` has always accepted\n{}",
                render(&args, &out)
            ),
        );
    }
    let folded_expected = format!("dependencies = [{:?}]", format!("{APP}=={APP_3_0}"));
    let folded_line = folded.dependencies_line();
    note(
        &mut violations,
        folded_line == folded_expected,
        format!(
            "`mamba add {MIXED_CASE}` then `mamba add {APP}` must leave one \
             dependency: `{APP}` and `{MIXED_CASE}` are the same package under \
             PEP 503 normalisation, so the second add replaces the first \
             entry. Two entries here are two pins of one package that a later \
             resolve has to reconcile, and whichever spelling was written last \
             silently wins.\n  expected: {folded_expected}\n  actual:   \
             {folded_line}\n--- mamba.toml ---\n{}",
            folded.manifest_text()
        ),
    );

    // ---- `remove` finds an entry that carries a specifier ---------------
    // The manifest is hand-written here, exactly as this work item's premise
    // measured it: `mamba.toml` already stores PEP 508 strings, so a user who
    // typed one into the file must be able to remove it by name. This is the
    // one sub-case that needs no working `add`, so the no-op regression is
    // observable on its own.
    let handwritten = Project::start();
    handwritten.set_dependencies(&[RANGE]);
    let remove_args = ["remove", APP];
    let out = handwritten.run_with_ambient_index(&remove_args, index_url.as_str());
    note(
        &mut violations,
        out.status.success(),
        format!(
            "`mamba remove {APP}` must succeed against a manifest that records \
             `{RANGE}`\n{}",
            render(&remove_args, &out)
        ),
    );
    note(
        &mut violations,
        !stderr_of(&out).contains(NOT_RECORDED),
        format!(
            "`mamba remove {APP}` must not report a hand-written `{RANGE}` as \
             unrecorded: a dependency's identity is the name at the head of the \
             requirement, and reporting a no-op here tells the user the entry \
             was never there while leaving it in the file\n{}",
            render(&remove_args, &out)
        ),
    );
    let handwritten_line = handwritten.dependencies_line();
    note(
        &mut violations,
        handwritten_line == "dependencies = []",
        format!(
            "`mamba remove {APP}` must remove the entry spelled `{RANGE}`\n  \
             expected: dependencies = []\n  actual:   {handwritten_line}\n\
             --- mamba.toml ---\n{}",
            handwritten.manifest_text()
        ),
    );

    // ---- one range replaces another over the same name ------------------
    let ranged = Project::start();
    for spec in [RANGE, RANGE_OPEN] {
        let args = ["add", spec, "--index-url", index_url.as_str()];
        let out = ranged.run(&args);
        note(
            &mut violations,
            out.status.success(),
            format!(
                "`mamba add {spec} --index-url` must succeed before this \
                 sub-case can observe the identity rule; that the specifier is \
                 accepted at all is \
                 `add_records_the_range_spec_and_locks_the_release_it_admits`'s \
                 claim, and a red here alongside a red there is one defect, not \
                 two\n{}\n{}",
                render(&args, &out),
                registry.seen()
            ),
        );
    }
    let ranged_line = ranged.dependencies_line();
    note(
        &mut violations,
        ranged_line == format!("dependencies = [{RANGE_OPEN:?}]"),
        format!(
            "`mamba add {RANGE}` then `mamba add {RANGE_OPEN}` must leave one \
             dependency carrying the requirement written last: two \
             requirements on one name in a manifest is a project whose lock \
             has to satisfy both without anyone having said so.\n  expected: \
             dependencies = [{RANGE_OPEN:?}]\n  actual:   {ranged_line}\n\
             --- mamba.toml ---\n{}",
            ranged.manifest_text()
        ),
    );

    // ---- `remove` finds what `add` wrote --------------------------------
    let added = Project::start();
    let add_args = ["add", RANGE, "--index-url", index_url.as_str()];
    let out = added.run(&add_args);
    note(
        &mut violations,
        out.status.success(),
        format!(
            "`mamba add {RANGE} --index-url` must succeed before this sub-case \
             can observe that `remove` finds what `add` wrote; that the \
             specifier is accepted at all is \
             `add_records_the_range_spec_and_locks_the_release_it_admits`'s \
             claim\n{}\n{}",
            render(&add_args, &out),
            registry.seen()
        ),
    );
    let out = added.run_with_ambient_index(&remove_args, index_url.as_str());
    note(
        &mut violations,
        out.status.success(),
        format!(
            "`mamba remove {APP}` must succeed after `mamba add {RANGE}`\n{}",
            render(&remove_args, &out)
        ),
    );
    note(
        &mut violations,
        !stderr_of(&out).contains(NOT_RECORDED),
        format!(
            "`mamba remove {APP}` must not report the entry `mamba add {RANGE}` \
             just wrote as unrecorded — the two commands would then disagree \
             about what a dependency is called\n{}",
            render(&remove_args, &out)
        ),
    );
    let added_line = added.dependencies_line();
    note(
        &mut violations,
        added_line == "dependencies = []",
        format!(
            "`mamba remove {APP}` must remove the entry `mamba add {RANGE}` \
             wrote\n  expected: dependencies = []\n  actual:   {added_line}\n\
             --- mamba.toml ---\n{}",
            added.manifest_text()
        ),
    );

    assert!(
        violations.is_empty(),
        "one package must be one dependency however it is spelled; {} of the \
         observations below disagree\n\n{}\n{}",
        violations.len(),
        violations.join("\n\n"),
        registry.seen()
    );
}

// --------------------------------------------------------------------------
// six — the grammars that must survive the change
// --------------------------------------------------------------------------

/// The two spellings `add` accepts today keep their exact meaning, and the
/// frozen `--index DIR` path keeps refusing what it cannot resolve rather
/// than resolving it loosely.
#[test]
fn add_keeps_the_bare_name_and_exact_pin_grammars_it_already_had() {
    let registry = Registry::start(&graph());

    // ---- a bare name still pins the newest release ----------------------
    let bare = Project::start();
    add_ok(
        &bare,
        &registry,
        APP,
        "a bare name is the grammar `add` has always accepted",
    );
    assert_eq!(
        bare.dependencies_line(),
        format!("dependencies = [{:?}]", format!("{APP}=={APP_3_0}")),
        "`mamba add {APP}` must keep writing an exact pin at the newest \
         release; teaching `add` to read specifiers must not turn a bare name \
         into an unpinned entry\n--- mamba.toml ---\n{}",
        bare.manifest_text()
    );
    let (body, entries) = bare.lock();
    // `3.0` is the release requiring `addlib>=2`, so this closure is also the
    // evidence that the bare name resolved to `3.0` rather than to a lower
    // release that happens to share a version string.
    assert_closure("a bare name", &registry, &body, &entries, APP_3_0, LIB_2_5);

    // ---- an exact pin still means that exact release --------------------
    let pinned = Project::start();
    add_ok(
        &pinned,
        &registry,
        EXACT_PIN_2_0,
        "`name==version` is the grammar `add` has always accepted",
    );
    assert_eq!(
        pinned.dependencies_line(),
        format!("dependencies = [{EXACT_PIN_2_0:?}]"),
        "`mamba add {EXACT_PIN_2_0}` must record that exact pin; an `==` \
         requirement read as a range would let a later resolve move off the \
         version the user named\n--- mamba.toml ---\n{}",
        pinned.manifest_text()
    );
    let (body, entries) = pinned.lock();
    assert_closure(EXACT_PIN_2_0, &registry, &body, &entries, APP_2_0, LIB_2_5);

    // ---- the frozen index keeps its narrower grammar --------------------
    // `mamba index build` output carries no release list a range could be
    // evaluated against, so a range there must be refused before anything is
    // written — never resolved to whatever directory happens to be newest.
    let staging = tempfile::tempdir().expect("fixture: create temp root for the frozen index");
    let wheels = staging.path().join("wheels");
    let index = staging.path().join("index");
    std::fs::create_dir_all(&wheels)
        .unwrap_or_else(|e| panic!("fixture: create {}: {e}", wheels.display()));
    let built = [
        build_wheel_file(&wheels, APP, APP_2_0, &[REQUIRES_LIB_1]),
        build_wheel_file(&wheels, APP, APP_3_0, &[REQUIRES_LIB_2]),
        build_wheel_file(&wheels, LIB, LIB_2_5, &[]),
    ];
    let index_str = index
        .to_str()
        .expect("fixture: the index path is utf-8")
        .to_string();
    let mut build_args = vec!["index", "build", "--out", index_str.as_str()];
    for wheel in &built {
        build_args.push(wheel.to_str().expect("fixture: the wheel path is utf-8"));
    }
    let host = Project::start();
    let out = run_in(staging.path(), &host.home, &build_args, None);
    assert!(
        out.status.success(),
        "fixture: `mamba index build` must stage the three wheels this \
         sub-case resolves against: {}",
        render(&build_args, &out)
    );

    // Fixture self-check: the frozen index really does serve `addapp`, so the
    // refusal below is about the grammar and not about an index the fixture
    // failed to build.
    let frozen_ok = Project::start();
    let frozen_args = ["add", APP, "--index", index_str.as_str()];
    let out = frozen_ok.run(&frozen_args);
    assert!(
        out.status.success(),
        "fixture self-check: `mamba add {APP} --index <DIR>` must resolve \
         against the frozen index this test just built; a red here means the \
         index is wrong rather than the grammar this sub-case judges\n{}",
        render(&frozen_args, &out)
    );

    let frozen_refused = Project::start();
    let before = frozen_refused.manifest_bytes();
    let refused_args = ["add", FROZEN_RANGE, "--index", index_str.as_str()];
    let out = frozen_refused.run(&refused_args);
    assert!(
        !out.status.success(),
        "`mamba add {FROZEN_RANGE} --index <DIR>` must be refused: the frozen \
         index keeps the `NAME` / `NAME==VERSION` grammar, and silently \
         resolving a range there would answer a bound the index cannot \
         evaluate\n{}",
        render(&refused_args, &out)
    );
    assert!(
        !stderr_of(&out).contains(PANIC_MARKER),
        "the frozen-path refusal must be a diagnosis, not a crash\n{}",
        render(&refused_args, &out)
    );
    assert_eq!(
        frozen_refused.manifest_bytes(),
        before,
        "a refused `mamba add --index <DIR>` must leave mamba.toml byte for \
         byte as it was\n--- mamba.toml now ---\n{}",
        frozen_refused.manifest_text()
    );
    assert!(
        !frozen_refused.lock_path().exists(),
        "a refused `mamba add --index <DIR>` must write no mamba.lock\n{}",
        render(&refused_args, &out)
    );
}
