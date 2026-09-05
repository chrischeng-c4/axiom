//! Black-box contract: a `mamba.lock` entry written by `mamba lock
//! --index-url` must describe **one** artifact — the `sha256` it records has
//! to be the digest of the bytes served at the `url` it records beside it.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over a throwaway directory tree
//! and reads only what the binary wrote. It never links the index client, the
//! resolver, or the lock renderer, and it never hand-writes a `mamba.lock`:
//! the artifact the registry serves is a real wheel built by the product's own
//! `WheelBuilder`, and the page that advertises it is a PEP 503 anchor list.
//! A green run therefore means `mamba lock` reached the simple page over HTTP,
//! chose an artifact, and recorded the digest *of that artifact* — not that
//! the case agreed with itself.
//!
//! # The observation point
//!
//! The registry is one `wiremock::MockServer` bound to loopback, serving a
//! simple page for `demo` whose anchors are, in this order:
//!
//! | # | anchor | digest fragment | bytes served |
//! |---|---|---|---|
//! | 1 | `demo-1.0.tar.gz` | the sha256 of a blob that is **not** the wheel | none — the route is unmounted, so it is a `404` |
//! | 2 | `demo-1.0-py3-none-any.whl` | the sha256 of the wheel's own bytes | the wheel, at `/files/demo-1.0-py3-none-any.whl` |
//!
//! Listing the sdist first is the whole fixture. `mamba` prefers wheels when
//! it picks an artifact, so the file it picks is the second anchor while the
//! first digest on the page belongs to the first. Any implementation that
//! reaches for "the first sha256 in the release" and "the best artifact URL"
//! through two independent paths describes two different files in one lock
//! entry, and this page is the smallest registry that shows it.
//!
//! Nothing here asserts which artifact the picker should choose: the wheel is
//! the pick before and after the change. What is asserted is that the digest
//! travels with it.
//!
//! # Why today's tree cannot pass
//!
//! `lock.rs::resolve_via_pypi` pins each resolved node with the first `sha256`
//! it finds in `node.files` — filled from the release in page order — while
//! the entry's `url` comes from `pick_artifact_url`, which scores wheels by
//! PEP 425 tags and skips the sdist entirely. Against the page above the
//! written entry therefore carries the **sdist** digest next to the **wheel**
//! URL. Two observations separately refuse that:
//!
//! 1. the case's own digest assertion, which compares the locked `sha256` with
//!    the sha256 of the exact bytes the registry serves at the locked `url`;
//! 2. `mamba sync`, which streams that URL through the hash-verifying
//!    downloader and aborts with `hash mismatch for
//!    demo-1.0-py3-none-any.whl: expected <sdist digest>, got <wheel digest>`.
//!
//! The first fires first, so a red names the two digests directly; the second
//! is the product's own verdict on the same lock and keeps the contract honest
//! if the case's arithmetic were ever loosened.
//!
//! # Attribution
//!
//! `mamba add --index-url` already pairs its pin (it selects digest and URL
//! from one `WheelPick`), so the same project is asserted to be paired
//! immediately after `add` and before `lock` runs. That intermediate assertion
//! is the control: it uses the same registry, the same page and the same
//! wheel, so a red there means the fixture is wrong — page shape, wheel build,
//! or route — rather than the lock path this work item names. It is written as
//! a fixture self-check, and its message says so.
//!
//! # Facets
//!
//! - **Behavior**: after `init` → `add --index-url` → `lock --index-url`, the
//!   `demo` entry of `mamba.lock` names the wheel's URL and the wheel's
//!   digest; `mamba sync` converges the environment and exits 0, and the
//!   following `mamba sync --check` exits 0, so the lock the registry produced
//!   is one a real install can actually be driven from.
//! - **Security (supply-chain integrity of the lock)**: a lock entry is a
//!   pinning claim, and this case refuses the two ways it can stop being one.
//!   A `sha256` belonging to a different file than the `url` is worse than no
//!   pin: it points the verifier at the wrong bytes, so the artifact that is
//!   actually installed is whatever the registry decides to serve at that URL
//!   the moment the hash check is relaxed or the entry is regenerated, and it
//!   makes an honest registry look like a tampering one. An empty `sha256` or
//!   an empty `url` is the fail-open state `mamba sync` reads as
//!   nothing-to-verify, so both are asserted non-empty *before* the pairing is
//!   compared — a pin dropped to satisfy the pairing would otherwise read as a
//!   fix. Verification is never weakened to reach the green: the `sync` step
//!   downloads through the same hash-verifying client the product ships, and
//!   the sdist route is deliberately absent so an implementation that switched
//!   the pick to the sdist fails to fetch instead of quietly passing.
//! - **Performance**: this work item names no budget, so the case asserts no
//!   timing and no request count.
//!
//! # Hermeticity
//!
//! No network, ever: the wheel is built in-process and the only registry is an
//! in-process `wiremock` server on loopback; every byte the case sees comes
//! from it. `HOME` and `MAMBA_CACHE_DIR` are pinned into the temp tree;
//! `MAMBA_FROZEN_INDEX`, `MAMBA_INDEX_URL`, `MAMBA_JOBS`, `XDG_CACHE_HOME`,
//! `VIRTUAL_ENV`, `PYTHONPATH`, and every proxy variable are removed, so
//! neither an ambient index nor a proxy can supply or divert an answer. The
//! binary is only ever run inside the temporary project directory. `python3`
//! on `PATH` is a requirement of the `sync` step, not an option: a host
//! without one fails this case naming it. Nothing is skipped, nothing is
//! `#[ignore]`d, and nothing sleeps or retries.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};
use sha2::{Digest, Sha256};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The distribution under test: one module, one version, two advertised files.
const DIST: &str = "demo";
const VERSION: &str = "1.0";
const MODULE: &str = "demo";
const WHEEL_FILE: &str = "demo-1.0-py3-none-any.whl";
const SDIST_FILE: &str = "demo-1.0.tar.gz";

/// The mounted routes. Everything else — including the sdist's own
/// `/files/demo-1.0.tar.gz` — is unmounted, and an unmounted path is a `404`.
/// Both JSON endpoints are mounted explicitly as `404` so the simple-API
/// fallback is a served answer rather than an accident of the harness.
const JSON_ROUTE: &str = "/pypi/demo/json";
const VERSION_JSON_ROUTE: &str = "/pypi/demo/1.0/json";
const SIMPLE_ROUTE: &str = "/simple/demo/";
const WHEEL_ROUTE: &str = "/files/demo-1.0-py3-none-any.whl";
const SDIST_ROUTE: &str = "/files/demo-1.0.tar.gz";

/// The module body the wheel carries. Nothing imports it here — the case stops
/// at install — but a real body keeps the artifact a real wheel.
const MODULE_BODY: &str = "def answer():\n    return 42\n";

/// The bytes the sdist anchor's `#sha256=` fragment is taken over. They are
/// never served: only their digest reaches the product, as the first digest on
/// the page. Any resemblance to the wheel's bytes would make the fixture
/// unable to tell the two artifacts apart, which the fixture self-check below
/// refuses.
const SDIST_BYTES: &[u8] =
    b"mamba e2e wi-4220: these bytes are the sdist's, and are not the wheel's\n";

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
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-lock-registry-sha-pairs-url");
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

/// The PEP 503 page the registry serves at `/simple/demo/`. The sdist anchor
/// is first and the wheel anchor second, exactly as a real index that uploaded
/// the source distribution before the built one would write it.
fn simple_page(sdist_url: &str, sdist_digest: &str, wheel_url: &str, wheel_digest: &str) -> String {
    format!(
        "<!DOCTYPE html>\n\
         <html>\n\
         <head><meta name=\"pypi:repository-version\" content=\"1.0\"><title>Links for demo</title></head>\n\
         <body>\n\
         <h1>Links for demo</h1>\n\
         <a href=\"{sdist_url}#sha256={sdist_digest}\" data-requires-python=\"&gt;=3.8\">{SDIST_FILE}</a><br/>\n\
         <a href=\"{wheel_url}#sha256={wheel_digest}\" data-requires-python=\"&gt;=3.8\">{WHEEL_FILE}</a><br/>\n\
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
    /// The absolute wheel URL the simple page advertises, and the sha256 of
    /// the exact bytes the registry serves for it.
    wheel_url: String,
    wheel_digest: String,
    /// The absolute sdist URL the simple page advertises, and the sha256 of
    /// the bytes it claims. The route is never mounted, so a picker that
    /// selected it would fail to download rather than pass quietly.
    sdist_url: String,
    sdist_digest: String,
}

impl Registry {
    fn start() -> Registry {
        let bytes = wheel_bytes();
        let wheel_digest = sha256_bytes(&bytes);
        let sdist_digest = sha256_bytes(SDIST_BYTES);
        assert_ne!(
            wheel_digest, sdist_digest,
            "fixture: the two advertised artifacts must have different digests, \
             or this case cannot tell which one the lock described"
        );
        let rt = tokio::runtime::Runtime::new().expect("fixture: build a runtime for the registry");
        let (server, base, wheel_url, sdist_url) = rt.block_on(async {
            let server = MockServer::start().await;
            let base = server.uri();
            let wheel_url = format!("{base}{WHEEL_ROUTE}");
            let sdist_url = format!("{base}{SDIST_ROUTE}");

            // Both JSON endpoints say "no such project", so the PEP 503 simple
            // page is the only route to the metadata and to the release's file
            // list. Mounted explicitly rather than left to the unmatched
            // default, so the fallback is a served answer.
            for json_route in [JSON_ROUTE, VERSION_JSON_ROUTE] {
                Mock::given(method("GET"))
                    .and(path(json_route))
                    .respond_with(ResponseTemplate::new(404))
                    .mount(&server)
                    .await;
            }

            Mock::given(method("GET"))
                .and(path(SIMPLE_ROUTE))
                .respond_with(ResponseTemplate::new(200).set_body_raw(
                    simple_page(&sdist_url, &sdist_digest, &wheel_url, &wheel_digest).into_bytes(),
                    "text/html; charset=utf-8",
                ))
                .mount(&server)
                .await;

            // The wheel is the only artifact whose bytes exist. `SDIST_ROUTE`
            // is deliberately unmounted.
            Mock::given(method("GET"))
                .and(path(WHEEL_ROUTE))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_raw(bytes.clone(), "application/octet-stream"),
                )
                .mount(&server)
                .await;

            (server, base, wheel_url, sdist_url)
        });
        Registry {
            server,
            rt,
            base,
            wheel_url,
            wheel_digest,
            sdist_url,
            sdist_digest,
        }
    }

    /// The `--index-url` this case passes: the PEP 503 simple URL.
    fn index_url(&self) -> String {
        format!("{}/simple", self.base)
    }

    /// Every request the registry saw, as `METHOD /path`. Folded into each
    /// panic message so a red names what the client actually asked for.
    fn seen(&self) -> Vec<String> {
        self.rt
            .block_on(self.server.received_requests())
            .unwrap_or_default()
            .iter()
            .map(|r| format!("{} {}", r.method, r.url.path()))
            .collect()
    }

    /// Which advertised artifact a digest belongs to, in words, so a failure
    /// message says *what* the lock described rather than only that it was
    /// wrong.
    fn describe_digest(&self, digest: &str) -> String {
        if digest == self.wheel_digest {
            format!("the wheel {WHEEL_FILE}")
        } else if digest == self.sdist_digest {
            format!("the sdist {SDIST_FILE}, which is not the artifact at the locked url")
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

/// The `demo` entry of the lock the binary just wrote, or a panic naming the
/// step that was supposed to have written it.
fn demo_entry(project: &Project, step: &str, args: &[&str], out: &Output, seen: &[String]) -> LockEntry {
    let body = project.lock_body();
    parse_lock(&body)
        .into_iter()
        .find(|e| e.name == DIST)
        .unwrap_or_else(|| {
            panic!(
                "mamba.lock must pin `{DIST}` after this step\n{}\n--- mamba.lock ---\n{body}",
                report(step, args, out, seen)
            )
        })
}

/// The `python3` `mamba sync` will create the project environment from. It is
/// a requirement of this case, not an option: a host without one fails here,
/// naming it, and the case never skips.
fn require_python3() -> PathBuf {
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
    let real = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim().to_string());
    assert!(
        real.is_file(),
        "`{}` reported the interpreter {}, which is not a file",
        candidate.display(),
        real.display()
    );
    real
}

/// `init` → `add demo --index-url <simple>` → `lock --index-url <simple>` →
/// the locked digest is the digest of the bytes at the locked url → `sync` →
/// `sync --check`.
#[test]
fn lock_via_index_url_records_the_digest_of_the_artifact_it_picked() {
    require_python3();
    let registry = Registry::start();
    let index_url = registry.index_url();
    let project = Project::start();

    // ---- `mamba add`: the control -------------------------------------
    // `add` selects digest and URL from one pick, so it already writes a
    // paired entry. Asserting that here, against this registry and this
    // wheel, makes the `lock` red below attributable: if these two fail the
    // fixture is wrong, not the lock path.
    let add_args = ["add", DIST, "--index-url", index_url.as_str()];
    let out = project.run(&add_args);
    let seen = registry.seen();
    assert!(
        out.status.success(),
        "`mamba add {DIST} --index-url <base>/simple` must succeed: the registry \
         serves this package at {SIMPLE_ROUTE}\n{}",
        report("add", &add_args, &out, &seen)
    );
    let added = demo_entry(&project, "add", &add_args, &out, &seen);
    assert_eq!(
        added.url, registry.wheel_url,
        "fixture self-check: `mamba add --index-url` must lock the wheel's url; \
         a red here means the fixture (page shape, wheel, or route) is wrong \
         rather than the `mamba lock` path this case judges\n{}",
        report("add", &add_args, &out, &seen)
    );
    assert_eq!(
        added.sha256, registry.wheel_digest,
        "fixture self-check: `mamba add --index-url` must lock the digest of the \
         wheel it named; it recorded {}, and the wheel's digest is {}. A red here \
         means the fixture is wrong rather than the `mamba lock` path this case \
         judges\n{}",
        registry.describe_digest(&added.sha256),
        registry.wheel_digest,
        report("add", &add_args, &out, &seen)
    );

    // ---- `mamba lock`: the contract -----------------------------------
    let lock_args = ["lock", "--index-url", index_url.as_str()];
    let out = project.run(&lock_args);
    let seen = registry.seen();
    assert!(
        out.status.success(),
        "`mamba lock --index-url <base>/simple` must succeed: the registry serves \
         this package at {SIMPLE_ROUTE}\n{}",
        report("lock", &lock_args, &out, &seen)
    );

    let locked = demo_entry(&project, "lock", &lock_args, &out, &seen);
    assert_eq!(
        locked.version, VERSION,
        "`{DIST}` must stay pinned at the only version {SIMPLE_ROUTE} advertises\n{}\
         \n--- mamba.lock ---\n{}",
        report("lock", &lock_args, &out, &seen),
        project.lock_body()
    );

    // A dropped pin is not a fix: an empty `url` or an empty `sha256` is the
    // fail-open state `mamba sync` reads as nothing to verify, and it would
    // otherwise satisfy a pairing comparison vacuously.
    assert!(
        !locked.url.is_empty() && !locked.sha256.is_empty(),
        "the locked entry for `{DIST}` must carry both a url and a sha256 — an \
         empty one is the fail-open state `mamba sync` reads as nothing to \
         verify; got url {:?} and sha256 {:?}\n{}\n--- mamba.lock ---\n{}",
        locked.url,
        locked.sha256,
        report("lock", &lock_args, &out, &seen),
        project.lock_body()
    );

    assert_eq!(
        locked.url, registry.wheel_url,
        "`mamba lock --index-url` must keep locking the wheel the picker \
         selects; the sdist advertised first at {} is not the pick, and this \
         work item does not change which artifact is chosen\n{}\n\
         --- mamba.lock ---\n{}",
        registry.sdist_url,
        report("lock", &lock_args, &out, &seen),
        project.lock_body()
    );

    assert_eq!(
        locked.sha256, registry.wheel_digest,
        "the locked sha256 must be the digest of the bytes the registry serves at \
         the locked url {}\n  expected (the wheel {WHEEL_FILE}): {}\n  \
         actual: {} — {}\n  for reference, the sdist {SDIST_FILE} is advertised \
         first on the page with digest {}\n{}\n--- mamba.lock ---\n{}",
        locked.url,
        registry.wheel_digest,
        locked.sha256,
        registry.describe_digest(&locked.sha256),
        registry.sdist_digest,
        report("lock", &lock_args, &out, &seen),
        project.lock_body()
    );

    // ---- `mamba sync`: the product's own verdict on that lock ----------
    // The locked url is fetched through the hash-verifying downloader, so an
    // entry whose digest belongs to another file aborts here with
    // `hash mismatch for <file>: expected <locked digest>, got <served digest>`.
    let sync_args = ["sync"];
    let out = project.run(&sync_args);
    let seen = registry.seen();
    assert!(
        out.status.success(),
        "`mamba sync` must install from the lock `mamba lock --index-url` wrote: \
         the registry serves {} at {WHEEL_ROUTE} with digest {}\n{}\n\
         --- mamba.lock ---\n{}",
        WHEEL_FILE,
        registry.wheel_digest,
        report("sync", &sync_args, &out, &seen),
        project.lock_body()
    );

    let check_args = ["sync", "--check"];
    let out = project.run(&check_args);
    let seen = registry.seen();
    assert!(
        out.status.success(),
        "`mamba sync --check` must report the environment synchronized right \
         after a successful `mamba sync`\n{}\n--- mamba.lock ---\n{}",
        report("sync --check", &check_args, &out, &seen),
        project.lock_body()
    );
}
