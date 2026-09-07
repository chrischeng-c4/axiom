//! Black-box contract: `mamba add <name> --index-url <registry>` must record
//! the **transitive closure** of what it added — every package the registry's
//! own metadata says the request pulls in, each pinned with the `sha256` of
//! the artifact at the `url` written beside it — so the environment
//! `mamba sync` builds from that lock can import what was added.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over a throwaway directory tree
//! and reads only what the binary wrote. It never links the index client, the
//! resolver, or the lock renderer, and it never hand-writes a `mamba.lock`:
//! both artifacts the registry serves are real wheels built by the product's
//! own `WheelBuilder`, the pages that advertise them are PEP 503 anchor lists,
//! and the dependency edge is a `requires_dist` field on the registry's own
//! per-version JSON. A green run therefore means `mamba add` reached the
//! registry over HTTP, discovered the edge there, and recorded both ends of it
//! — not that the case agreed with itself.
//!
//! # The observation point
//!
//! The registry is one `wiremock::MockServer` bound to loopback, serving a
//! two-node graph: `app` depends on `lib`, and nothing else exists.
//!
//! | route | answer |
//! |---|---|
//! | `GET /pypi/app/json` | `404`, so the client's JSON-first probe falls through |
//! | `GET /pypi/lib/json` | `404`, likewise |
//! | `GET /simple/app/` | `200 text/html`, one PEP 503 anchor with a `#sha256=` fragment |
//! | `GET /simple/lib/` | `200 text/html`, likewise |
//! | `GET /pypi/app/1.0/json` | `200`, `{"info":{"requires_dist":["lib"]}}` — the only place the edge exists |
//! | `GET /pypi/lib/1.0/json` | `404`, which the client reads as "declares no dependencies" |
//! | `GET /files/app-1.0-py3-none-any.whl` | `200`, that wheel's own bytes |
//! | `GET /files/lib-1.0-py3-none-any.whl` | `200`, that wheel's own bytes |
//!
//! Every other path is unmounted, and an unmounted path is a `404`. The edge
//! is therefore not something the case asserts as a string: `lib` can only
//! appear in the lock if the binary asked `/pypi/app/1.0/json` and believed
//! the answer. What the registry actually received is folded into every panic
//! message, so a red says which routes were requested rather than leaving it
//! to be guessed.
//!
//! `--index-url` is passed as `http://127.0.0.1:<port>/simple` — the PEP 503
//! spelling a `uv` user has in their fingers, which the client normalises back
//! to the registry base before building its own paths (#4209).
//!
//! # Why today's tree cannot pass
//!
//! `mamba add` renders the transitive closure only for a local frozen index;
//! every other source, the registry included, hands the single `ResolvedDep`
//! built from one metadata fetch to the single-package lock renderer. So today
//! `mamba add app --index-url …` exits 0 and writes a `mamba.lock` holding
//! `app` alone with `dependencies = []`; `/pypi/app/1.0/json` is never
//! requested, `lib` is never pinned, `mamba sync` installs one wheel, and
//! `mamba run main.py` dies with `ModuleNotFoundError: No module named 'lib'`.
//! The first contract assertion below — the `lib` pin — is that state.
//!
//! The three assertions *before* it are the control. They cover the requested
//! package alone, which the current renderer already gets right against this
//! same registry, this same page and this same wheel: a red there means the
//! fixture is wrong (page shape, wheel build, or route) rather than the
//! closure this work item names. They are written as fixture self-checks and
//! their messages say so.
//!
//! # Facets
//!
//! - **Behavior**: after `init` → `add app --index-url <simple>`, `mamba.lock`
//!   pins both `app` and `lib`; `app` stays `direct = true` and its
//!   `dependencies` names the pinned edge `lib==1.0`, while `lib` is the
//!   transitive pin with no dependencies of its own. `mamba sync` converges
//!   the environment and exits 0, and `mamba run main.py` — a script that
//!   imports both packages — exits 0 on the project's own `.venv` interpreter
//!   and prints the marker each installed wheel carries, so the lock really is
//!   one a working environment can be built from. Finally `mamba lock
//!   --index-url` against the same registry rewrites `mamba.lock` byte for
//!   byte, which is the whole claim: `add --index-url` writes the lock
//!   `lock --index-url` writes.
//! - **Security (supply-chain integrity of a closure the user never named)**:
//!   a transitive pin is a package the user did not ask for by name, so its
//!   pin is the only thing standing between the user and whatever the registry
//!   decides to serve later. Each of the two entries is asserted to carry a
//!   non-empty `url` *and* a non-empty `sha256` before the pairing is compared
//!   — an empty one is the fail-open state `mamba sync` reads as
//!   nothing-to-verify, and it would otherwise satisfy a pairing comparison
//!   vacuously, so "record the closure with blank hashes" cannot read as a
//!   fix. Each `sha256` is then compared against the digest of the exact bytes
//!   the registry serves at that entry's own `url`, so one file's digest can
//!   never be paired with another file's URL. Verification is never weakened
//!   to reach the green: the `sync` step downloads through the same
//!   hash-verifying client the product ships, so a mispaired digest aborts
//!   there even if the lock assertions were somehow satisfied.
//! - **Performance**: this work item names no budget, so the case asserts no
//!   timing, no request count, and no concurrency.
//!
//! # Hermeticity
//!
//! No network, ever: both wheels are built in-process and the only registry is
//! an in-process `wiremock` server on loopback; every byte the case sees comes
//! from it. `HOME` and `MAMBA_CACHE_DIR` are pinned into the temp tree;
//! `MAMBA_FROZEN_INDEX`, `MAMBA_INDEX_URL`, `MAMBA_JOBS`, `XDG_CACHE_HOME`,
//! `VIRTUAL_ENV`, `PYTHONPATH`, and every proxy variable are removed, so
//! neither an ambient index nor a proxy can supply or divert an answer, and no
//! interpreter can import `app` or `lib` from anywhere but the project's own
//! `.venv`. The binary is only ever run inside the temporary project
//! directory. `python3` on `PATH` is a requirement of the `sync` step, not an
//! option: a host without one fails this case naming it. Nothing is skipped,
//! nothing is `#[ignore]`d, and nothing sleeps or retries.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};
use sha2::{Digest, Sha256};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The two-node graph under test: the package the user asks for, and the one
/// only the registry knows it needs.
const ROOT_DIST: &str = "app";
const LEAF_DIST: &str = "lib";
const VERSION: &str = "1.0";
const ROOT_WHEEL_FILE: &str = "app-1.0-py3-none-any.whl";
const LEAF_WHEEL_FILE: &str = "lib-1.0-py3-none-any.whl";

/// The mounted routes. Everything else the server is asked for is a `404`.
/// Both project-level JSON endpoints and the leaf's per-version endpoint are
/// mounted explicitly as `404` so each fallback is a served answer rather than
/// an accident of the harness.
const ROOT_JSON_ROUTE: &str = "/pypi/app/json";
const LEAF_JSON_ROUTE: &str = "/pypi/lib/json";
const ROOT_VERSION_JSON_ROUTE: &str = "/pypi/app/1.0/json";
const LEAF_VERSION_JSON_ROUTE: &str = "/pypi/lib/1.0/json";
const ROOT_SIMPLE_ROUTE: &str = "/simple/app/";
const LEAF_SIMPLE_ROUTE: &str = "/simple/lib/";
const ROOT_WHEEL_ROUTE: &str = "/files/app-1.0-py3-none-any.whl";
const LEAF_WHEEL_ROUTE: &str = "/files/lib-1.0-py3-none-any.whl";

/// The one place the dependency edge exists: the registry's per-version JSON
/// for the requested package. Nothing in the wheel, the simple page, or the
/// manifest mentions `lib`.
const ROOT_VERSION_JSON_BODY: &str = "{\"info\":{\"requires_dist\":[\"lib\"]}}";

/// The pinned edge as the lock renderer writes it: `name==version`, the same
/// form the local frozen-index path already emits.
const EXPECTED_EDGE: &str = "lib==1.0";

/// The module body each wheel carries. The marker is per-distribution, so the
/// script the case runs proves *which* wheel each import resolved to, not just
/// that some module of that name was importable.
const ROOT_ORIGIN: &str = "app-1.0-from-the-registry";
const LEAF_ORIGIN: &str = "lib-1.0-from-the-registry";

/// The script `mamba run` executes: the interpreter it was given, then a
/// marker from each installed package.
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
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-add-registry-transitive");
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
/// to. The digest is taken over the exact bytes mounted at `url`, so the
/// pairing assertions below compare a lock entry against one file, never
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
    /// The package the user asks for, and the one only the registry knows about.
    root: Artifact,
    leaf: Artifact,
}

impl Registry {
    fn start() -> Registry {
        let root_bytes = build_wheel(ROOT_DIST, ROOT_ORIGIN, ROOT_WHEEL_FILE);
        let leaf_bytes = build_wheel(LEAF_DIST, LEAF_ORIGIN, LEAF_WHEEL_FILE);
        let root_digest = sha256_bytes(&root_bytes);
        let leaf_digest = sha256_bytes(&leaf_bytes);
        assert_ne!(
            root_digest, leaf_digest,
            "fixture: the two served wheels must have different digests, or this \
             case cannot tell which artifact a lock entry described"
        );

        let rt = tokio::runtime::Runtime::new().expect("fixture: build a runtime for the registry");
        let (server, base, root_url, leaf_url) = rt.block_on(async {
            let server = MockServer::start().await;
            let base = server.uri();
            let root_url = format!("{base}{ROOT_WHEEL_ROUTE}");
            let leaf_url = format!("{base}{LEAF_WHEEL_ROUTE}");

            // Both project-level JSON endpoints say "no such project", so the
            // PEP 503 simple page is the only route to each release list; the
            // leaf's per-version endpoint says the same, which the client
            // reads as "declares no dependencies". Mounted explicitly rather
            // than left to the unmatched-request default, so each fallback is
            // a served answer.
            for route in [ROOT_JSON_ROUTE, LEAF_JSON_ROUTE, LEAF_VERSION_JSON_ROUTE] {
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
            ] {
                Mock::given(method("GET"))
                    .and(path(route))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .set_body_raw(bytes, "application/octet-stream"),
                    )
                    .mount(&server)
                    .await;
            }

            (server, base, root_url, leaf_url)
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
        }
    }

    /// The `--index-url` this case passes: the PEP 503 simple URL.
    fn index_url(&self) -> String {
        format!("{}/simple", self.base)
    }

    /// Every request the registry saw, as `METHOD /path`. Folded into each
    /// panic message so a red names what the client actually asked for — in
    /// particular whether it ever asked for the edge at
    /// `/pypi/app/1.0/json`.
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
            project.join("pyproject.toml").is_file(),
            "fixture: `mamba init` wrote no pyproject.toml in {}",
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

/// Everything a reader needs to act on a red: the step, the whole invocation,
/// and what the registry was asked for.
fn report(step: &str, args: &[&str], out: &Output, seen: &[String]) -> String {
    format!(
        "step: {step}\n{}\n--- requests the registry received ---\n{}",
        render(args, out),
        if seen.is_empty() {
            "(none)".to_string()
        } else {
            seen.join("\n")
        }
    )
}

/// One `[[package]]` table of `mamba.lock`, read as data. `direct` is an
/// `Option` because the single-package renderer omits the key entirely, and
/// "absent" is a different observation from "false".
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
    // A dropped pin is not a fix: an empty `url` or an empty `sha256` is the
    // fail-open state `mamba sync` reads as nothing to verify, and it would
    // otherwise satisfy the pairing comparison below vacuously.
    assert!(
        !locked.url.is_empty() && !locked.sha256.is_empty(),
        "the locked entry for `{}` must carry both a url and a sha256 — an empty \
         one is the fail-open state `mamba sync` reads as nothing to verify; got \
         url {:?} and sha256 {:?}\n{}\n--- mamba.lock ---\n{lock}",
        artifact.dist,
        locked.url,
        locked.sha256,
        report(step, args, out, seen)
    );
    assert_eq!(
        locked.url, artifact.url,
        "`{}` must be locked at the artifact URL this registry advertised for it \
         ({})\n{}\n--- mamba.lock ---\n{lock}",
        artifact.dist,
        artifact.wheel_file,
        report(step, args, out, seen)
    );
    assert_eq!(
        locked.sha256, artifact.digest,
        "the sha256 locked for `{}` must be the digest of the bytes the registry \
         serves at the locked url {}\n  expected ({}): {}\n  actual: {} — {}\n{}\n\
         --- mamba.lock ---\n{lock}",
        artifact.dist,
        locked.url,
        artifact.wheel_file,
        artifact.digest,
        locked.sha256,
        registry.describe_digest(&locked.sha256),
        report(step, args, out, seen)
    );
}

/// `init` → `add app --index-url <simple>` → the lock pins the closure with
/// paired digests → `sync` → `run main.py` imports both packages on the
/// project's own interpreter → `lock --index-url` rewrites the same bytes.
#[test]
fn add_via_index_url_records_the_transitive_closure_with_paired_digests() {
    require_python3();
    let registry = Registry::start();
    let index_url = registry.index_url();
    let project = Project::start();

    // ---- `mamba add app --index-url <base>/simple` ---------------------
    let add_args = ["add", ROOT_DIST, "--index-url", index_url.as_str()];
    let out = project.run(&add_args);
    let seen = registry.seen();
    assert!(
        out.status.success(),
        "`mamba add {ROOT_DIST} --index-url <base>/simple` must succeed: the \
         registry serves this package at {ROOT_SIMPLE_ROUTE}\n{}",
        report("add", &add_args, &out, &seen)
    );

    let lock_after_add = project.lock_body();
    let entries = parse_lock(&lock_after_add);

    // ---- the control: the package the user named ----------------------
    // The single-package renderer already gets this right against this same
    // registry, page and wheel, so a red in this block means the fixture is
    // wrong rather than the closure this work item names.
    let root = entry(&entries, ROOT_DIST).unwrap_or_else(|| {
        panic!(
            "fixture self-check: `mamba add {ROOT_DIST} --index-url` must pin the \
             package it was asked for; the lock pins {}\n{}\n--- mamba.lock ---\n{lock_after_add}",
            pins_of(&entries),
            report("add", &add_args, &out, &seen)
        )
    });
    assert_eq!(
        root.version, VERSION,
        "fixture self-check: `{ROOT_DIST}` must be pinned at the only version \
         {ROOT_SIMPLE_ROUTE} advertises; a red here means the fixture is wrong \
         rather than the closure this case judges\n{}\n--- mamba.lock ---\n{lock_after_add}",
        report("add", &add_args, &out, &seen)
    );
    assert_eq!(
        root.url, registry.root.url,
        "fixture self-check: `mamba add --index-url` must lock the wheel's url; a \
         red here means the fixture (page shape, wheel, or route) is wrong rather \
         than the closure this case judges\n{}\n--- mamba.lock ---\n{lock_after_add}",
        report("add", &add_args, &out, &seen)
    );
    assert_eq!(
        root.sha256, registry.root.digest,
        "fixture self-check: `mamba add --index-url` must lock the digest of the \
         wheel it named; it recorded {}, and that wheel's digest is {}. A red here \
         means the fixture is wrong rather than the closure this case \
         judges\n{}\n--- mamba.lock ---\n{lock_after_add}",
        registry.describe_digest(&root.sha256),
        registry.root.digest,
        report("add", &add_args, &out, &seen)
    );

    // ---- the contract: the closure the registry declared ---------------
    // `lib` exists nowhere but `GET /pypi/app/1.0/json`. It can only appear in
    // this lock if `mamba add` asked for that route and recorded the answer.
    let leaf = entry(&entries, LEAF_DIST).unwrap_or_else(|| {
        panic!(
            "`mamba add {ROOT_DIST} --index-url <base>/simple` must record the \
             transitive closure: the registry declares `requires_dist: \
             [\"{LEAF_DIST}\"]` for {ROOT_DIST}=={VERSION} at \
             {ROOT_VERSION_JSON_ROUTE}, so `{LEAF_DIST}` must be pinned in \
             mamba.lock too. The lock pins {}.\n{}\n--- mamba.lock ---\n{lock_after_add}",
            pins_of(&entries),
            report("add", &add_args, &out, &seen)
        )
    });

    assert_eq!(
        root.direct,
        Some(true),
        "the package the user asked for must stay `direct = true` — the closure is \
         recorded around it, not in place of it\n{}\n--- mamba.lock ---\n{lock_after_add}",
        report("add", &add_args, &out, &seen)
    );
    assert_eq!(
        root.dependencies,
        vec![EXPECTED_EDGE.to_string()],
        "`{ROOT_DIST}`'s `dependencies` must name the pinned edge the registry \
         declared, in the `name==version` form the lock renderer writes (an absent \
         key reads as the empty list here)\n{}\n--- mamba.lock ---\n{lock_after_add}",
        report("add", &add_args, &out, &seen)
    );
    assert_eq!(
        leaf.version, VERSION,
        "`{LEAF_DIST}` must be pinned at the only version {LEAF_SIMPLE_ROUTE} \
         advertises\n{}\n--- mamba.lock ---\n{lock_after_add}",
        report("add", &add_args, &out, &seen)
    );
    assert_eq!(
        leaf.direct,
        Some(false),
        "`{LEAF_DIST}` is a transitive pin, not something the user asked for, so it \
         must be recorded with `direct = false`\n{}\n--- mamba.lock ---\n{lock_after_add}",
        report("add", &add_args, &out, &seen)
    );
    assert!(
        leaf.dependencies.is_empty(),
        "`{LEAF_DIST}` declares no dependencies ({LEAF_VERSION_JSON_ROUTE} is a \
         404, which the client reads as none), so its `dependencies` must be \
         empty; got {:?}\n{}\n--- mamba.lock ---\n{lock_after_add}",
        leaf.dependencies,
        report("add", &add_args, &out, &seen)
    );

    // Each pin describes exactly one file: the URL the registry advertised,
    // and the digest of the bytes it serves there.
    assert_pins_artifact(
        root,
        &registry.root,
        &registry,
        "add",
        &add_args,
        &out,
        &seen,
        &lock_after_add,
    );
    assert_pins_artifact(
        leaf,
        &registry.leaf,
        &registry,
        "add",
        &add_args,
        &out,
        &seen,
        &lock_after_add,
    );

    // ---- `mamba sync`: the product's own verdict on that lock -----------
    // Both locked urls are fetched through the hash-verifying downloader, so
    // an entry whose digest belongs to another file aborts here.
    let sync_args = ["sync"];
    let out = project.run(&sync_args);
    let seen = registry.seen();
    assert!(
        out.status.success(),
        "`mamba sync` must install every package `mamba add --index-url` locked: \
         the registry serves {ROOT_WHEEL_FILE} at {ROOT_WHEEL_ROUTE} and \
         {LEAF_WHEEL_FILE} at {LEAF_WHEEL_ROUTE}\n{}\n--- mamba.lock ---\n{lock_after_add}",
        report("sync", &sync_args, &out, &seen)
    );

    // ---- `mamba run`: the environment that lock describes ---------------
    project.write("main.py", MAIN_PY);
    let run_args = ["run", "main.py"];
    let out = project.run(&run_args);
    let seen = registry.seen();
    assert!(
        out.status.success(),
        "`mamba run main.py` must execute a script importing both locked \
         packages; a closure the lock never recorded is a \
         `ModuleNotFoundError` here\n{}\n--- mamba.lock ---\n{lock_after_add}",
        report("run", &run_args, &out, &seen)
    );

    let stdout = stdout_of(&out);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "`mamba run main.py` must pass the script's stdout through unchanged — \
         three lines, nothing added\n{}",
        report("run", &run_args, &out, &seen)
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
         interpreter: it reported `{reported}`, whose environment directory is \
         {}, not {}\n{}",
        environment.display(),
        expected_env.display(),
        report("run", &run_args, &out, &seen)
    );
    assert_eq!(
        lines[1], ROOT_ORIGIN,
        "the interpreter must import `{ROOT_DIST}` from the wheel this registry \
         served\n{}",
        report("run", &run_args, &out, &seen)
    );
    assert_eq!(
        lines[2], LEAF_ORIGIN,
        "the interpreter must import `{LEAF_DIST}` from the wheel this registry \
         served — the transitive package is only in the environment because the \
         lock pinned it\n{}",
        report("run", &run_args, &out, &seen)
    );

    // ---- `mamba lock`: the two commands must agree ----------------------
    // `add --index-url` writes the lock `lock --index-url` writes, against the
    // same registry and the same manifest — byte for byte.
    let lock_args = ["lock", "--index-url", index_url.as_str()];
    let out = project.run(&lock_args);
    let seen = registry.seen();
    assert!(
        out.status.success(),
        "`mamba lock --index-url <base>/simple` must succeed against the registry \
         `mamba add` just resolved from\n{}",
        report("lock", &lock_args, &out, &seen)
    );
    let lock_after_lock = project.lock_body();
    assert_eq!(
        lock_after_lock,
        lock_after_add,
        "`mamba add --index-url` must write the lock `mamba lock --index-url` \
         writes: re-locking the same manifest against the same registry rewrote \
         mamba.lock\n{}\n--- after `mamba add` ---\n{lock_after_add}\n\
         --- after `mamba lock` ---\n{lock_after_lock}",
        report("lock", &lock_args, &out, &seen)
    );
}
