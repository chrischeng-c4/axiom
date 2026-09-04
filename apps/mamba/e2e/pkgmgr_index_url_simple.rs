//! Black-box contract: `--index-url` must accept the PEP 503 *simple* URL a
//! `uv` user already has in their fingers — `https://<host>/simple` — and not
//! only the registry base URL `mamba` happens to build its own paths from.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over a throwaway directory tree
//! and reads only what the binary wrote. It never links the index client, and
//! it never hand-writes a `mamba.lock`: the one artifact the registry serves
//! is a real wheel built by the product's own `WheelBuilder`, and the page
//! that advertises it is a PEP 503 anchor list. A green run therefore means
//! `mamba add` reached the simple page over HTTP, parsed the anchor, and
//! recorded what that anchor said — not that the case agreed with itself.
//!
//! # The observation point
//!
//! The registry is one `wiremock::MockServer` bound to loopback for the whole
//! test, serving exactly three routes and nothing else:
//!
//! | route | answer |
//! |---|---|
//! | `GET /pypi/demo/json` | `404`, so the client's JSON-first probe must fall through |
//! | `GET /simple/demo/` | `200 text/html`, one PEP 503 anchor with a `#sha256=` fragment |
//! | `GET /files/demo-1.0-py3-none-any.whl` | `200`, the wheel's own bytes |
//!
//! Every other path — including the `/simple/simple/demo/` and
//! `/simple/pypi/demo/json` a doubled prefix produces — is unmounted, and an
//! unmounted path is a `404`. So "the client asked for the right URL" is not
//! asserted as a string: it is the only way any of these three observations
//! can be satisfied at all. What the registry actually received is folded into
//! every panic message, so a red says which URL was requested rather than
//! leaving it to be guessed.
//!
//! # Why today's tree cannot pass
//!
//! `IndexClient` builds `{index_url}/pypi/{name}/json` and
//! `{index_url}/simple/{name}/` from a base trimmed of trailing `/` only.
//! Handed `http://127.0.0.1:<port>/simple`, it asks for
//! `/simple/pypi/demo/json` and then `/simple/simple/demo/`; both `404`, the
//! fetch ends in `NotFound`, and `mamba add` exits non-zero with
//! ``package `demo` not found on index …``. Two of the three cases below name
//! that form, and the third names the base URL that already works.
//!
//! # Facets
//!
//! - **Behavior**: the same registry, the same package, and three spellings of
//!   the same index — `…/simple`, `…/simple/`, and the bare base — must all
//!   exit 0 and pin `demo` at `1.0` in `mamba.lock` with the artifact URL and
//!   `sha256` the simple page advertised. The bare base is the control: it is
//!   green before the change, so a red there means the fixture is wrong (page
//!   shape, wheel, or route) rather than the client.
//! - **Security (supply-chain integrity across the accepted spellings)**: the
//!   locked `sha256` must equal the digest of the wheel bytes the registry
//!   serves, for every accepted URL form. An entry with an empty digest is the
//!   fail-open state `mamba sync` reads as nothing-to-verify, so a
//!   normalisation that widened the accepted URL set while dropping the hash
//!   from the parsed metadata would be a silent downgrade; asserting the
//!   digest on each form refuses it. The normalisation is also bounded: no
//!   host is special-cased here, and nothing but a loopback registry is ever
//!   contacted.
//! - **Performance**: the work item names no budget, so this case asserts no
//!   timing.
//!
//! # Hermeticity
//!
//! No network, ever: the wheel is built in-process and the only registry is an
//! in-process `wiremock` server on loopback. `HOME` and `MAMBA_CACHE_DIR` are
//! pinned into the temp tree; `MAMBA_FROZEN_INDEX`, `MAMBA_INDEX_URL`,
//! `MAMBA_JOBS`, `XDG_CACHE_HOME`, `VIRTUAL_ENV`, `PYTHONPATH`, and every
//! proxy variable are removed, so neither an ambient index nor a proxy can
//! supply or divert an answer. Each case gets its own `HOME`, so the on-disk
//! metadata cache — which is keyed by package name, not by index URL — can
//! never let one URL form answer for another. Nothing is skipped, nothing is
//! `#[ignore]`d, and nothing sleeps or retries: a failed spawn panics naming
//! the step that needed it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};
use sha2::{Digest, Sha256};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The distribution under test: one module, one version, one wheel.
const DIST: &str = "demo";
const VERSION: &str = "1.0";
const MODULE: &str = "demo";
const WHEEL_FILE: &str = "demo-1.0-py3-none-any.whl";

/// The three mounted routes. Everything else the server is asked for is a
/// `404`, which is what makes a doubled `/simple` prefix observable.
const JSON_ROUTE: &str = "/pypi/demo/json";
const SIMPLE_ROUTE: &str = "/simple/demo/";
const WHEEL_ROUTE: &str = "/files/demo-1.0-py3-none-any.whl";

/// The module body the wheel carries. Nothing here imports it — the case stops
/// at resolution — but a real body keeps the artifact a real wheel.
const MODULE_BODY: &str = "def answer():\n    return 42\n";

/// The binary under test. `CARGO_BIN_EXE_*` is resolved by cargo at compile
/// time, so a missing binary is a build failure rather than a skipped case.
fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

/// Run the binary in `cwd` with an environment that can reach neither an
/// ambient registry, nor a user-global cache, nor a proxy.
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

/// Build the real wheel the registry serves, through the product's own wheel
/// builder, and hand back its bytes.
fn wheel_bytes() -> Vec<u8> {
    let dir = tempfile::tempdir().expect("fixture: create a temp dir for the wheel");
    let filename = compose_filename(DIST, VERSION, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-index-url-simple");
    wheel_meta.tags.push("py3-none-any".into());
    let core_meta = CoreMetadata::new(DIST, VERSION);
    let mut builder = WheelBuilder::new(filename, wheel_meta, core_meta);
    builder.add_file(format!("{MODULE}/__init__.py"), MODULE_BODY.to_string());
    let wheel = builder
        .build_to_dir(dir.path())
        .unwrap_or_else(|e| panic!("fixture: build wheel {DIST}-{VERSION}: {e:?}"));
    assert_eq!(
        wheel.file_name().and_then(|n| n.to_str()),
        Some(WHEEL_FILE),
        "fixture: the wheel builder named the artifact {}",
        wheel.display()
    );
    std::fs::read(&wheel).unwrap_or_else(|e| panic!("fixture: read {}: {e}", wheel.display()))
}

/// The PEP 503 page the registry serves at `/simple/demo/`: one anchor whose
/// text is the filename and whose `href` carries the artifact URL plus the
/// `#sha256=` fragment, exactly as a real simple index writes it.
fn simple_page(wheel_url: &str, digest: &str) -> String {
    format!(
        "<!DOCTYPE html>\n\
         <html>\n\
         <head><meta name=\"pypi:repository-version\" content=\"1.0\"><title>Links for demo</title></head>\n\
         <body>\n\
         <h1>Links for demo</h1>\n\
         <a href=\"{wheel_url}#sha256={digest}\" data-requires-python=\"&gt;=3.8\">{WHEEL_FILE}</a><br/>\n\
         </body>\n\
         </html>\n"
    )
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
    /// The absolute artifact URL the simple page advertises.
    wheel_url: String,
    /// The sha256 of the exact bytes the registry serves for that URL.
    digest: String,
}

impl Registry {
    fn start() -> Registry {
        let bytes = wheel_bytes();
        let digest = sha256_bytes(&bytes);
        let rt =
            tokio::runtime::Runtime::new().expect("fixture: build a runtime for the registry");
        let (server, base, wheel_url) = rt.block_on(async {
            let server = MockServer::start().await;
            let base = server.uri();
            let wheel_url = format!("{base}{WHEEL_ROUTE}");

            // The JSON API says "no such project", so the simple fallback is
            // the only way to the metadata. This route is mounted explicitly
            // rather than left to the unmatched-request default, so the
            // fallback is a served answer and not an accident of the harness.
            Mock::given(method("GET"))
                .and(path(JSON_ROUTE))
                .respond_with(ResponseTemplate::new(404))
                .mount(&server)
                .await;

            Mock::given(method("GET"))
                .and(path(SIMPLE_ROUTE))
                .respond_with(ResponseTemplate::new(200).set_body_raw(
                    simple_page(&wheel_url, &digest).into_bytes(),
                    "text/html; charset=utf-8",
                ))
                .mount(&server)
                .await;

            Mock::given(method("GET"))
                .and(path(WHEEL_ROUTE))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_raw(bytes.clone(), "application/octet-stream"),
                )
                .mount(&server)
                .await;

            (server, base, wheel_url)
        });
        Registry {
            server,
            rt,
            base,
            wheel_url,
            digest,
        }
    }

    /// One spelling of this registry's index URL.
    fn index_url(&self, suffix: &str) -> String {
        format!("{}{}", self.base, suffix)
    }

    /// Every request the registry saw, as `METHOD /path`. Folded into each
    /// panic message so a red names the URL the client actually built.
    fn seen(&self) -> Vec<String> {
        self.rt
            .block_on(self.server.received_requests())
            .unwrap_or_default()
            .iter()
            .map(|r| format!("{} {}", r.method, r.url.path()))
            .collect()
    }
}

/// One scaffolded project: a throwaway `HOME` and a `mamba init`ed directory.
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

/// Everything a reader needs to act on a red: which spelling was passed, what
/// the command exited with, both streams, and what the registry was asked for.
fn report(form: &str, index_url: &str, args: &[&str], out: &Output, seen: &[String]) -> String {
    format!(
        "--index-url form: {form}\nurl passed: {index_url}\n{}\n\
         --- requests the registry received ---\n{}",
        render(args, out),
        if seen.is_empty() {
            "(none)".to_string()
        } else {
            seen.join("\n")
        }
    )
}

/// One `[[package]]` table of `mamba.lock`, read as data.
struct LockEntry {
    name: String,
    version: String,
    sha256: String,
    url: String,
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
        })
        .collect()
}

/// Drive one `--index-url` spelling end to end: `mamba add demo` against a
/// fresh project must exit 0 and pin the package the simple page advertised.
fn add_records_demo(form: &str, index_url: &str, registry: &Registry) {
    let project = Project::start();
    let args = ["add", DIST, "--index-url", index_url];
    let out = project.run(&args);
    let seen = registry.seen();

    assert!(
        out.status.success(),
        "`mamba add {DIST} --index-url <{form}>` must succeed: the registry serves this \
         package at {SIMPLE_ROUTE}\n{}",
        report(form, index_url, &args, &out, &seen)
    );

    let lock = project.lock_body();
    let entries = parse_lock(&lock);
    let entry = entries.iter().find(|e| e.name == DIST).unwrap_or_else(|| {
        panic!(
            "mamba.lock must pin `{DIST}` after `mamba add {DIST} --index-url <{form}>`\n{}\
             \n--- mamba.lock ---\n{lock}",
            report(form, index_url, &args, &out, &seen)
        )
    });

    assert_eq!(
        entry.version, VERSION,
        "`{DIST}` must be pinned at the only version {SIMPLE_ROUTE} advertises, for the \
         `{form}` form\n{}\n--- mamba.lock ---\n{lock}",
        report(form, index_url, &args, &out, &seen)
    );
    assert_eq!(
        entry.url, registry.wheel_url,
        "the locked artifact URL must be the one the simple page's anchor carried, for the \
         `{form}` form\n{}\n--- mamba.lock ---\n{lock}",
        report(form, index_url, &args, &out, &seen)
    );
    assert_eq!(
        entry.sha256, registry.digest,
        "the locked digest must be the sha256 of the bytes the registry serves (an empty \
         digest is the fail-open state `mamba sync` reads as nothing-to-verify), for the \
         `{form}` form\n{}\n--- mamba.lock ---\n{lock}",
        report(form, index_url, &args, &out, &seen)
    );
}

/// The form `uv` documents and the form the demo that opened this Milestone
/// passed: the PEP 503 simple URL itself.
#[test]
fn add_accepts_the_simple_index_url() {
    let registry = Registry::start();
    let index_url = registry.index_url("/simple");
    add_records_demo("<base>/simple", &index_url, &registry);
}

/// The same URL as a directory, which is how a browser and most registry
/// documentation spell it. Trailing slashes must not change the answer.
#[test]
fn add_accepts_the_simple_index_url_with_trailing_slash() {
    let registry = Registry::start();
    let index_url = registry.index_url("/simple/");
    add_records_demo("<base>/simple/", &index_url, &registry);
}

/// The registry base URL, which `mamba` already accepts. This case is the
/// control: it exercises the same page, the same wheel, and the same routes,
/// so a red here means the fixture is wrong rather than the client, and a
/// green here is what makes the two `/simple` reds attributable to the URL the
/// client built.
#[test]
fn add_accepts_the_bare_registry_base_url() {
    let registry = Registry::start();
    let index_url = registry.index_url("");
    add_records_demo("<base>", &index_url, &registry);
}
