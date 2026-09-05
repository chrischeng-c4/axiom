//! Black-box contract: `mamba remove <name>` must **prune** `mamba.lock`, not
//! re-render it. What the removed package alone reached goes; everything the
//! remaining packages still reach stays, each keeping the `sha256` and `url`
//! the preceding `add` recorded — so the environment `mamba sync` builds from
//! the pruned lock is still the environment the lock described, and is still
//! built from bytes whose digests were verified.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over a throwaway directory tree
//! and reads only what the binary wrote. It never links the resolver, the lock
//! renderer, or `remove` itself, and it never hand-writes a `mamba.lock`: the
//! lock it judges is the one `mamba add --index-url` produced against a real
//! HTTP registry, and the three artifacts that registry serves are real wheels
//! built by the product's own `WheelBuilder`. A green run therefore means the
//! product kept a closure it had already discovered, not that the case agreed
//! with itself.
//!
//! # The observation point
//!
//! One `wiremock::MockServer` on loopback serves a three-node graph: `app`
//! depends on `lib`, `extra` depends on nothing, and `lib` is reachable from
//! nowhere else.
//!
//! | route | answer |
//! |---|---|
//! | `GET /pypi/app/json`, `/pypi/lib/json`, `/pypi/extra/json` | `404`, so the client's JSON-first probe falls through to the simple page |
//! | `GET /simple/app/`, `/simple/lib/`, `/simple/extra/` | `200 text/html`, one PEP 503 anchor with a `#sha256=` fragment |
//! | `GET /pypi/app/1.0/json` | `200`, `{"info":{"requires_dist":["lib"]}}` — the only place the `app` → `lib` edge exists |
//! | `GET /pypi/lib/1.0/json`, `/pypi/extra/1.0/json` | `404`, which the client reads as "declares no dependencies" |
//! | `GET /files/<dist>-1.0-py3-none-any.whl` | `200`, that wheel's own bytes |
//!
//! Every other path is unmounted, and an unmounted path is a `404`. The flow
//! is `init` → `add app --index-url <base>/simple` → `add extra --index-url
//! <base>/simple` → **lock A** → `remove extra` → **lock B**. Lock A is the
//! control: today's tree already records `app`, `lib`, and `extra` there with
//! paired digests, so a red in the lock A block means the fixture is wrong
//! (page shape, wheel build, or route) rather than the pruning this work item
//! names. Lock B is the contract. What the registry actually received is
//! folded into every panic message, together with the lock text, so a red says
//! what the product wrote rather than leaving it to be guessed.
//!
//! # Why today's tree cannot pass
//!
//! `cmd_remove` rewrites the lock with `render_lockfile_for_manifest`, which
//! emits one `[[package]]` per *manifest* dependency with `sha256 = ""`,
//! `url = ""`, `dependencies = []` and no `direct` key. So today `remove extra`
//! leaves `app` alone, blank: the transitive pin `lib` is gone, both digests
//! are gone, and `mamba sync` then refuses with ``cannot sync package `app`:
//! mamba.lock entry names neither `path` nor `url``. Removing one package
//! un-pins every other. The first contract assertion below — the surviving
//! `lib` entry — is that state.
//!
//! # Facets
//!
//! - **Behavior**: after `remove extra`, lock B holds `app` and `lib` and no
//!   `extra`; `app` stays `direct = true` with its `dependencies` still naming
//!   the pinned edge `lib==1.0`, and `lib` stays the transitive pin with
//!   `direct = false`. `mamba sync` converges the environment and exits 0, and
//!   `mamba run main.py` — a script importing both surviving packages — exits 0
//!   on the project's own `.venv` interpreter and prints the marker each
//!   installed wheel carries, so the pruned lock really is one a working
//!   environment can be built from. `mamba sync --check` then agrees. Removing
//!   `extra` a second time leaves the lock byte-identical (idempotent replay,
//!   the property `remove` already promises), and removing the last root drops
//!   the closure with it: no `[[package]]` survives, and `sync` and
//!   `sync --check` both still exit 0 over the emptied lock.
//! - **Security (supply-chain integrity across an edit the user thinks is a
//!   deletion)**: `remove` names one package, so every *other* pin it rewrites
//!   is collateral. A pin that loses its `sha256` is not a smaller lock, it is
//!   an unverified one — `mamba sync` reads an empty digest as nothing to
//!   verify. Each surviving entry is asserted to carry a non-empty `url` *and*
//!   a non-empty `sha256` before any comparison, so "keep the entry with blank
//!   hashes" cannot read as a fix; each value is then compared both against
//!   what lock A recorded and against the digest of the exact bytes the
//!   registry serves at that entry's own `url`, so one file's digest can never
//!   be paired with another file's URL. The dropped transitive pin is the same
//!   hazard one step removed: an entry silently deleted from the lock is an
//!   import `sync` can no longer vouch for. Verification is never weakened to
//!   reach the green — the `sync` step downloads through the same
//!   hash-verifying client the product ships. And `remove` is asserted to be
//!   offline: the registry receives no request at all while it runs, so a
//!   "fix" that re-resolves against an index — re-opening the pin to whatever
//!   the registry serves today — fails here.
//! - **Performance**: this work item names no budget, so the case asserts no
//!   timing, no request count as a budget, and no concurrency.
//!
//! # Hermeticity
//!
//! No network, ever: all three wheels are built in-process and the only
//! registry is an in-process `wiremock` server on loopback; every byte the case
//! sees comes from it. `HOME` and `MAMBA_CACHE_DIR` are pinned into this run's
//! own temp tree, so no artifact cached by another run can stand in for a
//! download; `MAMBA_FROZEN_INDEX`, `MAMBA_INDEX_URL`, `MAMBA_JOBS`,
//! `XDG_CACHE_HOME`, `VIRTUAL_ENV`, `PYTHONPATH`, and every proxy variable are
//! removed, so neither an ambient index nor a proxy can supply or divert an
//! answer, and no interpreter can import `app`, `lib`, or `extra` from anywhere
//! but the project's own `.venv`. The binary is only ever run inside the
//! temporary project directory. `python3` on `PATH` is a requirement of the
//! `sync` step, not an option: a host without one fails this case naming it.
//! Nothing is skipped, nothing is `#[ignore]`d, and nothing sleeps or retries.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};
use sha2::{Digest, Sha256};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The three-node graph under test: the root the user keeps, the transitive
/// pin only that root reaches, and the root the user removes.
const ROOT_DIST: &str = "app";
const LEAF_DIST: &str = "lib";
const OTHER_DIST: &str = "extra";
const VERSION: &str = "1.0";

const ROOT_WHEEL_FILE: &str = "app-1.0-py3-none-any.whl";
const LEAF_WHEEL_FILE: &str = "lib-1.0-py3-none-any.whl";
const OTHER_WHEEL_FILE: &str = "extra-1.0-py3-none-any.whl";

/// The mounted routes. Everything else the server is asked for is a `404`.
/// The project-level JSON endpoints and the two dependency-free per-version
/// endpoints are mounted explicitly as `404` so each fallback is a served
/// answer rather than an accident of the harness.
const ROOT_JSON_ROUTE: &str = "/pypi/app/json";
const LEAF_JSON_ROUTE: &str = "/pypi/lib/json";
const OTHER_JSON_ROUTE: &str = "/pypi/extra/json";
const ROOT_VERSION_JSON_ROUTE: &str = "/pypi/app/1.0/json";
const LEAF_VERSION_JSON_ROUTE: &str = "/pypi/lib/1.0/json";
const OTHER_VERSION_JSON_ROUTE: &str = "/pypi/extra/1.0/json";
const ROOT_SIMPLE_ROUTE: &str = "/simple/app/";
const LEAF_SIMPLE_ROUTE: &str = "/simple/lib/";
const OTHER_SIMPLE_ROUTE: &str = "/simple/extra/";
const ROOT_WHEEL_ROUTE: &str = "/files/app-1.0-py3-none-any.whl";
const LEAF_WHEEL_ROUTE: &str = "/files/lib-1.0-py3-none-any.whl";
const OTHER_WHEEL_ROUTE: &str = "/files/extra-1.0-py3-none-any.whl";

/// The one place the dependency edge exists: the registry's per-version JSON
/// for `app`. Nothing in any wheel, simple page, or manifest mentions `lib`.
const ROOT_VERSION_JSON_BODY: &str = "{\"info\":{\"requires_dist\":[\"lib\"]}}";

/// The pinned edge as the lock renderer writes it: `name==version`.
const EXPECTED_EDGE: &str = "lib==1.0";

/// The module body each wheel carries. The marker is per-distribution, so the
/// script the case runs proves *which* wheel each import resolved to, not just
/// that some module of that name was importable.
const ROOT_ORIGIN: &str = "app-1.0-from-the-registry";
const LEAF_ORIGIN: &str = "lib-1.0-from-the-registry";
const OTHER_ORIGIN: &str = "extra-1.0-from-the-registry";

/// The script `mamba run` executes after the removal: the interpreter it was
/// given, then a marker from each package that must have survived.
const MAIN_PY: &str = "import sys\n\
                       import app\n\
                       import lib\n\
                       print(sys.executable)\n\
                       print(app.ORIGIN)\n\
                       print(lib.ORIGIN)\n";

/// The binary under test. `CARGO_BIN_EXE_*` is resolved by cargo at compile
/// time, so a missing binary is a build failure rather than a skipped case.
fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

/// Run the binary in `cwd` with an environment that can reach neither an
/// ambient registry, nor a user-global cache, nor a proxy — and whose child
/// interpreter can import nothing but what the project's own `.venv` holds.
fn run(cwd: &Path, home: &Path, args: &[&str]) -> Output {
    Command::new(mamba_bin())
        .args(args)
        .current_dir(cwd)
        .env("HOME", home)
        .env("MAMBA_CACHE_DIR", home.join("cache"))
        .env_remove("MAMBA_FROZEN_INDEX")
        .env_remove("MAMBA_INDEX_URL")
        .env_remove("MAMBA_JOBS")
        .env_remove("XDG_CACHE_HOME")
        .env_remove("VIRTUAL_ENV")
        .env_remove("PYTHONPATH")
        .env_remove("HTTP_PROXY")
        .env_remove("HTTPS_PROXY")
        .env_remove("ALL_PROXY")
        .env_remove("http_proxy")
        .env_remove("https_proxy")
        .env_remove("all_proxy")
        .output()
        .unwrap_or_else(|e| panic!("spawn {} {args:?}: {e}", mamba_bin().display()))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn canon(path: &Path) -> PathBuf {
    path.canonicalize()
        .unwrap_or_else(|e| panic!("canonicalize {}: {e}", path.display()))
}

/// Build one real wheel through the product's own wheel builder and hand back
/// its bytes. The module inside is importable and carries `ORIGIN`, so an
/// interpreter can say which artifact it loaded.
fn build_wheel(dist: &str, origin: &str, expected_file: &str) -> Vec<u8> {
    let dir = tempfile::tempdir().expect("fixture: create a temp dir for the wheel");
    let filename = compose_filename(dist, VERSION, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-remove-keeps-remaining-closure");
    wheel_meta.tags.push("py3-none-any".into());
    let core_meta = CoreMetadata::new(dist, VERSION);
    let mut builder = WheelBuilder::new(filename, wheel_meta, core_meta);
    builder.add_file(
        format!("{dist}/__init__.py"),
        format!("ORIGIN = \"{origin}\"\n"),
    );
    let wheel = builder
        .build_to_dir(dir.path())
        .unwrap_or_else(|e| panic!("fixture: build wheel {dist}-{VERSION}: {e:?}"));
    assert_eq!(
        wheel.file_name().and_then(|n| n.to_str()),
        Some(expected_file),
        "fixture: the wheel builder named the artifact {}",
        wheel.display()
    );
    std::fs::read(&wheel).unwrap_or_else(|e| panic!("fixture: read {}: {e}", wheel.display()))
}

/// The PEP 503 page the registry serves for one distribution: a single anchor
/// whose text is the filename and whose `href` carries the artifact URL plus
/// the `#sha256=` fragment, exactly as a real simple index writes it.
fn simple_page(dist: &str, wheel_file: &str, wheel_url: &str, digest: &str) -> String {
    format!(
        "<!DOCTYPE html>\n\
         <html>\n\
         <head><meta name=\"pypi:repository-version\" content=\"1.0\"><title>Links for {dist}</title></head>\n\
         <body>\n\
         <h1>Links for {dist}</h1>\n\
         <a href=\"{wheel_url}#sha256={digest}\" data-requires-python=\"&gt;=3.8\">{wheel_file}</a><br/>\n\
         </body>\n\
         </html>\n"
    )
}

/// One artifact the registry serves: where it lives and what its bytes hash
/// to. The digest is taken over the exact bytes mounted at `url`, so every
/// pairing assertion below compares a lock entry against one file, never
/// against a value the case invented.
struct Artifact {
    dist: &'static str,
    wheel_file: &'static str,
    url: String,
    digest: String,
}

/// A loopback registry that stays up for the whole case.
///
/// `server` is declared before `rt` so the server is shut down while the
/// runtime that started it is still alive. Neither field is leaked: a
/// `mem::forget`-ed server outlives the observation it is supposed to bound,
/// and the request log this case reads on failure would go with it.
struct Registry {
    server: MockServer,
    rt: tokio::runtime::Runtime,
    /// `http://127.0.0.1:<port>` — the registry base, with no trailing slash.
    base: String,
    /// The root the user keeps, the pin only that root reaches, and the root
    /// the user removes.
    root: Artifact,
    leaf: Artifact,
    other: Artifact,
}

impl Registry {
    fn start() -> Registry {
        let root_bytes = build_wheel(ROOT_DIST, ROOT_ORIGIN, ROOT_WHEEL_FILE);
        let leaf_bytes = build_wheel(LEAF_DIST, LEAF_ORIGIN, LEAF_WHEEL_FILE);
        let other_bytes = build_wheel(OTHER_DIST, OTHER_ORIGIN, OTHER_WHEEL_FILE);
        let root_digest = sha256_bytes(&root_bytes);
        let leaf_digest = sha256_bytes(&leaf_bytes);
        let other_digest = sha256_bytes(&other_bytes);
        for (a, b) in [
            (&root_digest, &leaf_digest),
            (&root_digest, &other_digest),
            (&leaf_digest, &other_digest),
        ] {
            assert_ne!(
                a, b,
                "fixture: the three served wheels must have pairwise different \
                 digests, or this case cannot tell which artifact a lock entry \
                 described"
            );
        }

        let rt = tokio::runtime::Runtime::new().expect("fixture: build a runtime for the registry");
        let (server, base, root_url, leaf_url, other_url) = rt.block_on(async {
            let server = MockServer::start().await;
            let base = server.uri();
            let root_url = format!("{base}{ROOT_WHEEL_ROUTE}");
            let leaf_url = format!("{base}{LEAF_WHEEL_ROUTE}");
            let other_url = format!("{base}{OTHER_WHEEL_ROUTE}");

            // Every project-level JSON endpoint says "no such project", so the
            // PEP 503 simple page is the only route to each release list; the
            // two dependency-free per-version endpoints say the same, which the
            // client reads as "declares no dependencies". Mounted explicitly
            // rather than left to the unmatched-request default, so each
            // fallback is a served answer.
            for route in [
                ROOT_JSON_ROUTE,
                LEAF_JSON_ROUTE,
                OTHER_JSON_ROUTE,
                LEAF_VERSION_JSON_ROUTE,
                OTHER_VERSION_JSON_ROUTE,
            ] {
                Mock::given(method("GET"))
                    .and(path(route))
                    .respond_with(ResponseTemplate::new(404))
                    .mount(&server)
                    .await;
            }

            // The one place the dependency edge exists.
            Mock::given(method("GET"))
                .and(path(ROOT_VERSION_JSON_ROUTE))
                .respond_with(ResponseTemplate::new(200).set_body_raw(
                    ROOT_VERSION_JSON_BODY.as_bytes().to_vec(),
                    "application/json",
                ))
                .mount(&server)
                .await;

            for (route, page) in [
                (
                    ROOT_SIMPLE_ROUTE,
                    simple_page(ROOT_DIST, ROOT_WHEEL_FILE, &root_url, &root_digest),
                ),
                (
                    LEAF_SIMPLE_ROUTE,
                    simple_page(LEAF_DIST, LEAF_WHEEL_FILE, &leaf_url, &leaf_digest),
                ),
                (
                    OTHER_SIMPLE_ROUTE,
                    simple_page(OTHER_DIST, OTHER_WHEEL_FILE, &other_url, &other_digest),
                ),
            ] {
                Mock::given(method("GET"))
                    .and(path(route))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .set_body_raw(page.into_bytes(), "text/html; charset=utf-8"),
                    )
                    .mount(&server)
                    .await;
            }

            for (route, bytes) in [
                (ROOT_WHEEL_ROUTE, root_bytes.clone()),
                (LEAF_WHEEL_ROUTE, leaf_bytes.clone()),
                (OTHER_WHEEL_ROUTE, other_bytes.clone()),
            ] {
                Mock::given(method("GET"))
                    .and(path(route))
                    .respond_with(
                        ResponseTemplate::new(200).set_body_raw(bytes, "application/octet-stream"),
                    )
                    .mount(&server)
                    .await;
            }

            (server, base, root_url, leaf_url, other_url)
        });

        Registry {
            server,
            rt,
            base,
            root: Artifact {
                dist: ROOT_DIST,
                wheel_file: ROOT_WHEEL_FILE,
                url: root_url,
                digest: root_digest,
            },
            leaf: Artifact {
                dist: LEAF_DIST,
                wheel_file: LEAF_WHEEL_FILE,
                url: leaf_url,
                digest: leaf_digest,
            },
            other: Artifact {
                dist: OTHER_DIST,
                wheel_file: OTHER_WHEEL_FILE,
                url: other_url,
                digest: other_digest,
            },
        }
    }

    /// The `--index-url` this case passes: the PEP 503 simple URL.
    fn index_url(&self) -> String {
        format!("{}/simple", self.base)
    }

    /// Every request the registry saw, as `METHOD /path`. Folded into each
    /// panic message so a red names what the client actually asked for — and
    /// counted around `remove`, which must ask for nothing at all.
    fn seen(&self) -> Vec<String> {
        self.rt
            .block_on(self.server.received_requests())
            .unwrap_or_default()
            .iter()
            .map(|r| format!("{} {}", r.method, r.url.path()))
            .collect()
    }

    /// Which advertised artifact a digest belongs to, in words, so a failure
    /// says *what* the lock described rather than only that it was wrong.
    fn describe_digest(&self, digest: &str) -> String {
        if digest == self.root.digest {
            format!("the wheel {ROOT_WHEEL_FILE}")
        } else if digest == self.leaf.digest {
            format!("the wheel {LEAF_WHEEL_FILE}")
        } else if digest == self.other.digest {
            format!("the wheel {OTHER_WHEEL_FILE}")
        } else if digest.is_empty() {
            "nothing — the digest is empty, which `mamba sync` reads as nothing to verify"
                .to_string()
        } else {
            "no artifact this registry advertised".to_string()
        }
    }
}

/// One scaffolded project: a throwaway `HOME` and a `mamba init`ed directory.
/// The binary is never run anywhere else.
struct Project {
    _root: tempfile::TempDir,
    project: PathBuf,
    home: PathBuf,
}

impl Project {
    fn start() -> Project {
        let root_dir = tempfile::tempdir().expect("fixture: create temp root");
        let root = root_dir.path().to_path_buf();
        let project = root.join("project");
        let home = root.join("home");
        for dir in [&project, &home] {
            std::fs::create_dir_all(dir)
                .unwrap_or_else(|e| panic!("fixture: create {}: {e}", dir.display()));
        }
        let init_args = ["init"];
        let out = run(&project, &home, &init_args);
        assert!(
            out.status.success(),
            "fixture: `mamba init` must succeed: {}",
            render(&init_args, &out)
        );
        assert!(
            project.join("mamba.toml").is_file(),
            "fixture: `mamba init` wrote no mamba.toml in {}",
            project.display()
        );
        Project {
            _root: root_dir,
            project,
            home,
        }
    }

    fn run(&self, args: &[&str]) -> Output {
        run(&self.project, &self.home, args)
    }

    fn lock_body(&self) -> String {
        let path = self.project.join("mamba.lock");
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    fn write(&self, name: &str, body: &str) {
        let path = self.project.join(name);
        std::fs::write(&path, body).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    }

    /// `canonicalize(<project>/.venv)` — the one environment a file run inside
    /// this project may execute on.
    fn venv_dir(&self) -> PathBuf {
        canon(&self.project.join(".venv"))
    }
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

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// Everything a reader needs to act on a red: the step, the whole invocation
/// with its exit code and both streams, what the registry was asked for, and
/// the lock the product wrote.
fn report(step: &str, args: &[&str], out: &Output, seen: &[String], lock: &str) -> String {
    format!(
        "step: {step}\n{}\n--- requests the registry received ---\n{}\n--- mamba.lock ---\n{lock}",
        render(args, out),
        if seen.is_empty() {
            "(none)".to_string()
        } else {
            seen.join("\n")
        }
    )
}

/// One `[[package]]` table of `mamba.lock`, read as data. `direct` is an
/// `Option` because the manifest-shaped renderer omits the key entirely, and
/// "absent" is a different observation from "false".
struct LockEntry {
    name: String,
    version: String,
    sha256: String,
    url: String,
    direct: Option<bool>,
    dependencies: Vec<String>,
}

/// Read the `[[package]]` array. A lock with no `package` key is a legal
/// observation here — it is the state the last `remove` must reach — so it
/// reads as zero entries rather than a panic; every step that requires an
/// entry says so itself, and prints the pins it did find.
fn parse_lock(body: &str) -> Vec<LockEntry> {
    let doc: toml::Value = body
        .parse()
        .unwrap_or_else(|e| panic!("parse mamba.lock: {e}\n--- mamba.lock ---\n{body}"));
    let Some(packages) = doc.get("package") else {
        return Vec::new();
    };
    let packages = packages.as_array().unwrap_or_else(|| {
        panic!("mamba.lock `package` is not an array\n--- mamba.lock ---\n{body}")
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

/// The `name==version` pins in the lock, in file order, for panic messages.
fn pins_of(entries: &[LockEntry]) -> String {
    if entries.is_empty() {
        return "(no [[package]] entries)".to_string();
    }
    entries
        .iter()
        .map(|e| format!("{}=={}", e.name, e.version))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The entry names, sorted — the set comparison that catches both a pin that
/// should have been dropped and one that should have been kept.
fn names_of(entries: &[LockEntry]) -> Vec<String> {
    let mut names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
    names.sort();
    names
}

fn entry<'a>(entries: &'a [LockEntry], name: &str) -> Option<&'a LockEntry> {
    entries.iter().find(|e| e.name == name)
}

/// The directory two levels above `executable` — `<venv>` for
/// `<venv>/bin/python` and for `<venv>\Scripts\python.exe` alike.
fn environment_root_of(executable: &str) -> PathBuf {
    Path::new(executable)
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| {
            panic!(
                "`mamba run` reported the interpreter `{executable}`, which has no \
                 enclosing environment directory (expected `<env>/bin/python` or \
                 `<env>\\Scripts\\python.exe`)"
            )
        })
        .to_path_buf()
}

/// The `python3` `mamba sync` will create the project environment from. It is
/// a requirement of this case, not an option: a host without one fails here,
/// naming it, and the case never skips.
fn require_python3() {
    let path_var = std::env::var_os("PATH")
        .unwrap_or_else(|| panic!("this case needs `python3`, and PATH is unset"));
    let mut candidate: Option<PathBuf> = None;
    for dir in std::env::split_paths(&path_var) {
        for name in ["python3", "python3.exe"] {
            let probe = dir.join(name);
            if probe.is_file() {
                candidate = Some(probe);
                break;
            }
        }
        if candidate.is_some() {
            break;
        }
    }
    let candidate = candidate.unwrap_or_else(|| {
        panic!(
            "this case needs `python3` on PATH — `mamba sync` builds the project \
             environment from it — and found none in {path_var:?}"
        )
    });
    let out = Command::new(&candidate)
        .args(["-c", "import sys; print(sys.executable)"])
        .output()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", candidate.display()));
    assert!(
        out.status.success(),
        "`{} -c 'print(sys.executable)'` failed, so `mamba sync` has no interpreter \
         to build the project environment from: {}",
        candidate.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    let real = PathBuf::from(stdout_of(&out).trim().to_string());
    assert!(
        real.is_file(),
        "`{}` reported the interpreter {}, which is not a file",
        candidate.display(),
        real.display()
    );
}

/// Assert one lock entry describes exactly one artifact: a non-empty `url` and
/// `sha256`, the URL the registry advertised, and the digest of the bytes it
/// actually serves there.
#[allow(clippy::too_many_arguments)]
fn assert_pins_artifact(
    locked: &LockEntry,
    artifact: &Artifact,
    registry: &Registry,
    step: &str,
    args: &[&str],
    out: &Output,
    seen: &[String],
    lock: &str,
) {
    // A blanked pin is not a smaller lock, it is an unverified one: an empty
    // `url` or an empty `sha256` is the fail-open state `mamba sync` reads as
    // nothing to verify, and it would otherwise satisfy the pairing
    // comparisons below vacuously.
    assert!(
        !locked.url.is_empty() && !locked.sha256.is_empty(),
        "the locked entry for `{}` must carry both a url and a sha256 — an empty \
         one is the fail-open state `mamba sync` reads as nothing to verify; got \
         url {:?} and sha256 {:?}\n{}",
        artifact.dist,
        locked.url,
        locked.sha256,
        report(step, args, out, seen, lock)
    );
    assert_eq!(
        locked.url, artifact.url,
        "`{}` must be locked at the artifact URL this registry advertised for it \
         ({})\n{}",
        artifact.dist,
        artifact.wheel_file,
        report(step, args, out, seen, lock)
    );
    assert_eq!(
        locked.sha256, artifact.digest,
        "the sha256 locked for `{}` must be the digest of the bytes the registry \
         serves at the locked url {}\n  expected ({}): {}\n  actual: {} — {}\n{}",
        artifact.dist,
        locked.url,
        artifact.wheel_file,
        artifact.digest,
        locked.sha256,
        registry.describe_digest(&locked.sha256),
        report(step, args, out, seen, lock)
    );
}

/// `init` → `add app` → `add extra` (lock A, the control) → `remove extra`
/// (lock B, the contract) → `sync` → `run main.py` → `sync --check` →
/// `remove extra` again → `remove app` → the emptied lock still syncs.
#[test]
fn remove_keeps_the_remaining_packages_closure_and_its_paired_digests() {
    require_python3();
    let registry = Registry::start();
    let index_url = registry.index_url();
    let project = Project::start();

    // ---- lock A: the two roots and the closure `add` discovered -----------
    // This whole block is the control. Today's tree already records `app`,
    // `lib`, and `extra` here against this same registry, these same pages and
    // these same wheels, so a red before the `remove` step below means the
    // fixture is wrong rather than the pruning this case judges.
    let add_root_args = ["add", ROOT_DIST, "--index-url", index_url.as_str()];
    let out = project.run(&add_root_args);
    let seen = registry.seen();
    assert!(
        out.status.success(),
        "fixture self-check: `mamba add {ROOT_DIST} --index-url <base>/simple` must \
         succeed: the registry serves this package at {ROOT_SIMPLE_ROUTE}\n{}",
        report("add app", &add_root_args, &out, &seen, "(not written yet)")
    );

    let add_other_args = ["add", OTHER_DIST, "--index-url", index_url.as_str()];
    let out = project.run(&add_other_args);
    let seen = registry.seen();
    let lock_a = project.lock_body();
    assert!(
        out.status.success(),
        "fixture self-check: `mamba add {OTHER_DIST} --index-url <base>/simple` must \
         succeed: the registry serves this package at {OTHER_SIMPLE_ROUTE}\n{}",
        report("add extra", &add_other_args, &out, &seen, &lock_a)
    );

    let entries_a = parse_lock(&lock_a);
    let mut expected_a = vec![
        ROOT_DIST.to_string(),
        LEAF_DIST.to_string(),
        OTHER_DIST.to_string(),
    ];
    expected_a.sort();
    assert_eq!(
        names_of(&entries_a),
        expected_a,
        "fixture self-check: after adding both roots the lock must pin both of them \
         plus the closure the registry declared at {ROOT_VERSION_JSON_ROUTE}; it \
         pins {}\n{}",
        pins_of(&entries_a),
        report("add extra", &add_other_args, &out, &seen, &lock_a)
    );

    let root_a = entry(&entries_a, ROOT_DIST).expect("checked by the name-set assertion above");
    let leaf_a = entry(&entries_a, LEAF_DIST).expect("checked by the name-set assertion above");
    let other_a = entry(&entries_a, OTHER_DIST).expect("checked by the name-set assertion above");

    for (locked, artifact, direct, deps) in [
        (
            root_a,
            &registry.root,
            Some(true),
            vec![EXPECTED_EDGE.to_string()],
        ),
        (leaf_a, &registry.leaf, Some(false), Vec::new()),
        (other_a, &registry.other, Some(true), Vec::new()),
    ] {
        assert_eq!(
            locked.version, VERSION,
            "fixture self-check: `{}` must be pinned at the only version its simple \
             page advertises\n{}",
            artifact.dist,
            report("add extra", &add_other_args, &out, &seen, &lock_a)
        );
        assert_eq!(
            locked.direct, direct,
            "fixture self-check: `{}` must be recorded with direct = {:?} by `mamba \
             add --index-url`; a red here means the fixture is wrong rather than the \
             pruning this case judges\n{}",
            artifact.dist,
            direct,
            report("add extra", &add_other_args, &out, &seen, &lock_a)
        );
        assert_eq!(
            locked.dependencies, deps,
            "fixture self-check: `{}` must record the edges the registry declared, in \
             the `name==version` form the lock renderer writes\n{}",
            artifact.dist,
            report("add extra", &add_other_args, &out, &seen, &lock_a)
        );
        assert_pins_artifact(
            locked,
            artifact,
            &registry,
            "add extra",
            &add_other_args,
            &out,
            &seen,
            &lock_a,
        );
    }

    // ---- `mamba remove extra`: the contract -------------------------------
    let requests_before_remove = registry.seen().len();
    let remove_other_args = ["remove", OTHER_DIST];
    let out = project.run(&remove_other_args);
    let seen = registry.seen();
    let lock_b = project.lock_body();
    assert!(
        out.status.success(),
        "`mamba remove {OTHER_DIST}` must succeed\n{}",
        report("remove extra", &remove_other_args, &out, &seen, &lock_b)
    );

    // `remove` is offline: it works from the lock already on disk, so a "fix"
    // that re-resolves against the index — re-opening every surviving pin to
    // whatever the registry serves today — is refused here.
    assert_eq!(
        seen.len(),
        requests_before_remove,
        "`mamba remove` must not consult any index: it takes the lock already on \
         disk and prunes it. The registry received {} request(s) while `remove` \
         ran.\n{}",
        seen.len().saturating_sub(requests_before_remove),
        report("remove extra", &remove_other_args, &out, &seen, &lock_b)
    );

    let entries_b = parse_lock(&lock_b);
    assert!(
        entry(&entries_b, OTHER_DIST).is_none(),
        "`mamba remove {OTHER_DIST}` must drop the removed package's own pin; the \
         lock still pins {}\n{}",
        pins_of(&entries_b),
        report("remove extra", &remove_other_args, &out, &seen, &lock_b)
    );
    let root_b = entry(&entries_b, ROOT_DIST).unwrap_or_else(|| {
        panic!(
            "`mamba remove {OTHER_DIST}` must keep the package it did not remove; the \
             lock pins {}\n{}",
            pins_of(&entries_b),
            report("remove extra", &remove_other_args, &out, &seen, &lock_b)
        )
    });
    // `lib` is reachable only through `app`, which stays. Re-rendering the
    // lock from the manifest — which never named `lib` — is exactly the state
    // this assertion refuses.
    let leaf_b = entry(&entries_b, LEAF_DIST).unwrap_or_else(|| {
        panic!(
            "`mamba remove {OTHER_DIST}` must keep the transitive closure of the \
             packages that remain: `{ROOT_DIST}` is still a dependency and the \
             registry declares `requires_dist: [\"{LEAF_DIST}\"]` for \
             {ROOT_DIST}=={VERSION}, so `{LEAF_DIST}` must still be pinned. The lock \
             pins {}\n{}",
            pins_of(&entries_b),
            report("remove extra", &remove_other_args, &out, &seen, &lock_b)
        )
    });
    let mut expected_b = vec![ROOT_DIST.to_string(), LEAF_DIST.to_string()];
    expected_b.sort();
    assert_eq!(
        names_of(&entries_b),
        expected_b,
        "after `mamba remove {OTHER_DIST}` the lock must pin exactly what the \
         remaining root reaches — no more, no less; it pins {}\n{}",
        pins_of(&entries_b),
        report("remove extra", &remove_other_args, &out, &seen, &lock_b)
    );

    assert_eq!(
        root_b.version, root_a.version,
        "`{ROOT_DIST}` must keep the version `mamba add` pinned\n{}",
        report("remove extra", &remove_other_args, &out, &seen, &lock_b)
    );
    assert_eq!(
        leaf_b.version, leaf_a.version,
        "`{LEAF_DIST}` must keep the version `mamba add` pinned\n{}",
        report("remove extra", &remove_other_args, &out, &seen, &lock_b)
    );
    assert_eq!(
        root_b.direct,
        Some(true),
        "`{ROOT_DIST}` is still a dependency the user named, so it must stay \
         `direct = true` (an absent key reads as None here)\n{}",
        report("remove extra", &remove_other_args, &out, &seen, &lock_b)
    );
    assert_eq!(
        leaf_b.direct,
        Some(false),
        "`{LEAF_DIST}` is still only a transitive pin, so it must stay \
         `direct = false` — promoting it would leave it stranded after a later \
         `remove {ROOT_DIST}` (an absent key reads as None here)\n{}",
        report("remove extra", &remove_other_args, &out, &seen, &lock_b)
    );
    assert_eq!(
        root_b.dependencies,
        vec![EXPECTED_EDGE.to_string()],
        "`{ROOT_DIST}`'s `dependencies` must still name the pinned edge, in the \
         `name==version` form lock A recorded ({:?}) — the edge is what makes \
         `{LEAF_DIST}` reachable, so dropping it un-pins the closure on the next \
         removal\n{}",
        root_a.dependencies,
        report("remove extra", &remove_other_args, &out, &seen, &lock_b)
    );
    assert_eq!(
        leaf_b.dependencies,
        Vec::<String>::new(),
        "`{LEAF_DIST}` declares no dependencies ({LEAF_VERSION_JSON_ROUTE} is a 404, \
         which the client reads as none), so its `dependencies` must stay empty\n{}",
        report("remove extra", &remove_other_args, &out, &seen, &lock_b)
    );

    // Each surviving pin must still describe exactly one artifact — the same
    // one lock A described, and the bytes the registry actually serves there.
    for (before, after, artifact) in [
        (root_a, root_b, &registry.root),
        (leaf_a, leaf_b, &registry.leaf),
    ] {
        assert_pins_artifact(
            after,
            artifact,
            &registry,
            "remove extra",
            &remove_other_args,
            &out,
            &seen,
            &lock_b,
        );
        assert_eq!(
            after.sha256, before.sha256,
            "the sha256 `mamba add` recorded for `{}` must survive `mamba remove \
             {OTHER_DIST}` byte for byte: removing one package must not re-open \
             another's pin\n  lock A: {}\n  lock B: {} — {}\n{}",
            artifact.dist,
            before.sha256,
            after.sha256,
            registry.describe_digest(&after.sha256),
            report("remove extra", &remove_other_args, &out, &seen, &lock_b)
        );
        assert_eq!(
            after.url, before.url,
            "the url `mamba add` recorded for `{}` must survive `mamba remove \
             {OTHER_DIST}` byte for byte\n{}",
            artifact.dist,
            report("remove extra", &remove_other_args, &out, &seen, &lock_b)
        );
    }

    // ---- `mamba sync`: the product's own verdict on the pruned lock -------
    // Both surviving urls are fetched through the hash-verifying downloader,
    // so an entry whose digest belongs to another file aborts here.
    let sync_args = ["sync"];
    let out = project.run(&sync_args);
    let seen = registry.seen();
    assert!(
        out.status.success(),
        "`mamba sync` must install every package the pruned lock still pins: the \
         registry serves {ROOT_WHEEL_FILE} at {ROOT_WHEEL_ROUTE} and \
         {LEAF_WHEEL_FILE} at {LEAF_WHEEL_ROUTE}. An entry `mamba remove` blanked \
         cannot be installed at all.\n{}",
        report("sync", &sync_args, &out, &seen, &lock_b)
    );

    // ---- `mamba run`: the environment that pruned lock describes -----------
    project.write("main.py", MAIN_PY);
    let run_args = ["run", "main.py"];
    let out = project.run(&run_args);
    let seen = registry.seen();
    assert!(
        out.status.success(),
        "`mamba run main.py` must execute a script importing both surviving \
         packages; a closure `mamba remove` dropped is a `ModuleNotFoundError` \
         here\n{}",
        report("run", &run_args, &out, &seen, &lock_b)
    );

    let stdout = stdout_of(&out);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "`mamba run main.py` must pass the script's stdout through unchanged — \
         three lines, nothing added\n{}",
        report("run", &run_args, &out, &seen, &lock_b)
    );

    // The interpreter is the project's own, compared as canonical directories:
    // `.venv/bin/python` is a symlink to the base interpreter, so the reported
    // path itself must not be resolved.
    let reported = lines[0];
    let environment = canon(&environment_root_of(reported));
    let expected_env = project.venv_dir();
    assert_eq!(
        environment,
        expected_env,
        "`mamba run main.py` must execute the file on the project's own \
         interpreter: it reported `{reported}`, whose environment directory is {}, \
         not {}\n{}",
        environment.display(),
        expected_env.display(),
        report("run", &run_args, &out, &seen, &lock_b)
    );
    assert_eq!(
        lines[1], ROOT_ORIGIN,
        "the interpreter must import `{ROOT_DIST}` from the wheel this registry \
         served — the pruned lock's own pin\n{}",
        report("run", &run_args, &out, &seen, &lock_b)
    );
    assert_eq!(
        lines[2], LEAF_ORIGIN,
        "the interpreter must import `{LEAF_DIST}` from the wheel this registry \
         served — the transitive package is only in the environment because the \
         pruned lock still pinned it\n{}",
        report("run", &run_args, &out, &seen, &lock_b)
    );

    let check_args = ["sync", "--check"];
    let out = project.run(&check_args);
    let seen = registry.seen();
    assert!(
        out.status.success(),
        "`mamba sync --check` must agree that the environment matches the pruned \
         lock\n{}",
        report("sync --check", &check_args, &out, &seen, &lock_b)
    );

    // ---- removing the same package again: deterministic replay ------------
    let out = project.run(&remove_other_args);
    let seen = registry.seen();
    let lock_b_again = project.lock_body();
    assert!(
        out.status.success(),
        "removing a package that is no longer recorded is a soft no-op success\n{}",
        report(
            "remove extra (replay)",
            &remove_other_args,
            &out,
            &seen,
            &lock_b_again
        )
    );
    assert_eq!(
        lock_b_again, lock_b,
        "`mamba remove` must be deterministic: removing `{OTHER_DIST}` a second time \
         rewrote mamba.lock\n{}\n--- lock B, the first removal ---\n{lock_b}",
        report(
            "remove extra (replay)",
            &remove_other_args,
            &out,
            &seen,
            &lock_b_again
        )
    );

    // ---- removing the last root takes its closure with it -----------------
    let remove_root_args = ["remove", ROOT_DIST];
    let out = project.run(&remove_root_args);
    let seen = registry.seen();
    let lock_c = project.lock_body();
    assert!(
        out.status.success(),
        "`mamba remove {ROOT_DIST}` must succeed\n{}",
        report("remove app", &remove_root_args, &out, &seen, &lock_c)
    );
    let entries_c = parse_lock(&lock_c);
    assert!(
        entries_c.is_empty(),
        "with no dependency left, nothing is reachable: the lock must hold no \
         [[package]] entry, and in particular must not strand `{LEAF_DIST}`, which \
         only `{ROOT_DIST}` reached. It pins {}\n{}",
        pins_of(&entries_c),
        report("remove app", &remove_root_args, &out, &seen, &lock_c)
    );

    let out = project.run(&sync_args);
    let seen = registry.seen();
    assert!(
        out.status.success(),
        "`mamba sync` must converge the environment on an emptied lock\n{}",
        report("sync (emptied)", &sync_args, &out, &seen, &lock_c)
    );
    let out = project.run(&check_args);
    let seen = registry.seen();
    assert!(
        out.status.success(),
        "`mamba sync --check` must agree that the environment matches the emptied \
         lock — every package the removed root reached is gone from it\n{}",
        report("sync --check (emptied)", &check_args, &out, &seen, &lock_c)
    );
}
