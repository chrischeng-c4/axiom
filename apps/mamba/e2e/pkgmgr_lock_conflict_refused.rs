//! Black-box contract: `mamba lock` must **refuse** a dependency graph whose
//! own declared constraints cannot both hold, rather than write a lock that
//! violates one of them.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over throwaway directory trees and
//! reads only what the binary wrote, or refused to write. It never links the
//! resolver, the frozen-index walker, or the lock renderer, and it never
//! hand-writes a `mamba.lock` or an index page: every wheel is built by the
//! product's own `WheelBuilder`, the registry serves PEP 503 anchor pages plus
//! per-version `requires_dist` documents, and the frozen index is whatever
//! `mamba index build` stages from those same wheels. The one thing the case
//! authors is `pyproject.toml`'s `dependencies` line — the manifest is user input
//! in this workflow, and `mamba lock` is precisely the command that reads a
//! hand-edited manifest. Every artifact the case *judges* is one the product
//! produced.
//!
//! # The graph
//!
//! ```text
//!   app 1.0    --requires lib>=2-->   lib {1.0, 3.0}
//!   other 1.0  --requires lib<2--->   lib {1.0, 3.0}
//! ```
//!
//! No version of `lib` satisfies both edges: `1.0` fails `>=2` and `3.0` fails
//! `<2`. The two edges are declared by the package source and nowhere else —
//! by the registry's own `/pypi/<name>/1.0/json` on the registry half, and by
//! the wheels' own `Requires-Dist` (as `mamba index build` transcribes it) on
//! the frozen half — so a graph the binary refuses is one it discovered, not
//! one this file asserted at itself.
//!
//! # The control, in this same file
//!
//! Each half carries a second graph differing from the first in exactly one
//! string: `other` requires `lib>=1` instead of `lib<2`. `lib` is still a name
//! two requirements reach, and `3.0` still satisfies both, so that graph must
//! keep locking exactly as it does today — one `lib` pin, both edges rendered
//! as `lib==3.0`, a verifiable digest on each entry, and byte-identical on
//! replay. A refusal rule that fires on "this name arrived twice" rather than
//! on "the recorded version contradicts the requirement that just arrived"
//! goes red there.
//!
//! Each conflict test also locks its own graph *without* the second root
//! first, as a fixture self-check: a red in that step says the fixture is
//! wrong or the refusal is over-broad, not that the conflict went undetected.
//!
//! # Why today's tree cannot pass
//!
//! Both halves drop the second constraint, by two independent mechanisms.
//!
//! - Registry: `Resolver::resolve` opens each worklist iteration with `if
//!   decided.contains_key(&name) { continue; }`, so a requirement arriving
//!   after a name has been decided is discarded without ever being compared
//!   against the recorded version. `app` sorts before `other`, so `lib>=2`
//!   decides `lib==3.0` and `lib<2` is thrown away. Today `mamba lock
//!   --index-url` exits 0 and writes a lock pinning `lib==3.0` alone, with
//!   `other`'s edge recorded as `dependencies = ["lib==3.0"]` — the registry
//!   declared `lib<2` for `other`, and the lock now says the opposite.
//! - Frozen: `resolve_transitive` keys its `seen` map on `name==version` and
//!   resolves each requirement independently, so two requirements on one name
//!   never intersect. Today `mamba lock --index DIR` exits 0 and writes a lock
//!   with two `[[package]]` blocks named `lib`, one at `1.0` and one at `3.0`.
//!
//! Neither run says anything on stdout or stderr.
//!
//! # Facets
//!
//! - **Behavior**: `mamba lock` exits non-zero on the conflicting graph, on
//!   both index kinds, with a message naming `lib` and both conflicting
//!   requirements — and the compatible graph still locks, with the same pins,
//!   the same edges and the same bytes it produces today.
//! - **Security (fail-closed, and a declared bound is a boundary)**: a version
//!   bound in a manifest or in a package's own metadata is how a user keeps a
//!   known-bad release out of the environment `mamba sync` builds from the
//!   lock. A lock that silently replaces `lib<2` with `lib==3.0` pins the very
//!   release the graph excluded, so the refusal has to be a refusal: exit
//!   non-zero, never a warning with exit 0, never a panic. It also has to be
//!   total — a project that held no lock still holds none afterwards, with no
//!   `mamba.lock.tmp` left from a half-finished atomic write, and a project
//!   whose earlier run left a good lock finds it byte-for-byte unchanged, so
//!   the refusal can never be the thing that destroys a working pin set.
//! - **Performance**: this work item names no budget, so the case asserts no
//!   timing, no request count, and no concurrency.
//!
//! # Hermeticity
//!
//! No network, ever: every wheel is built in-process, the only registry is an
//! in-process `wiremock` server on loopback, and the only other package source
//! is a directory the case had `mamba index build` write. `HOME` and
//! `MAMBA_CACHE_DIR` are pinned per project into its own temp tree, so no two
//! tests can read each other's cached metadata; `MAMBA_FROZEN_INDEX`,
//! `MAMBA_INDEX_URL`, `MAMBA_JOBS`, `XDG_CACHE_HOME`, `VIRTUAL_ENV`,
//! `PYTHONPATH` and every proxy variable are removed, so no ambient index or
//! proxy can supply or divert an answer. No step builds an environment or runs
//! a script, so no Python interpreter is required or consulted. Nothing is
//! skipped, nothing is `#[ignore]`d, and nothing sleeps or retries.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};
use sha2::{Digest, Sha256};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The three-node graph both halves resolve. `app` and `other` are the roots
/// the manifest declares; `lib` is the name they both reach.
const ROOT_DIST: &str = "app";
const OTHER_DIST: &str = "other";
const SHARED_DIST: &str = "lib";
const ROOT_VERSION: &str = "1.0";
const OTHER_VERSION: &str = "1.0";
const SHARED_LOW: &str = "1.0";
const SHARED_HIGH: &str = "3.0";

/// The edge `app` declares, in both graphs.
const ROOT_REQUIRES: &str = "lib>=2";
/// The edge `other` declares in the conflicting graph. No version of `lib`
/// satisfies this and `ROOT_REQUIRES` at once.
const CONFLICTING_OTHER_REQUIRES: &str = "lib<2";
/// The edge `other` declares in the control graph: `lib` is still reached
/// twice, and `3.0` still satisfies both requirements.
const COMPATIBLE_OTHER_REQUIRES: &str = "lib>=1";

/// The two bounds the refusal must name. Compared against a whitespace-free
/// copy of stderr, so `lib >= 2` and `lib>=2` both count.
const LOWER_BOUND: &str = ">=2";
const UPPER_BOUND: &str = "<2";

/// The pinned edge both roots must carry in the control graph, in the
/// `name==version` form the lock renderer writes.
const SHARED_EDGE: &str = "lib==3.0";

/// The binary under test. `CARGO_BIN_EXE_*` is resolved by cargo at compile
/// time, so a missing binary is a build failure rather than a skipped case.
fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

/// Run the binary in `cwd` with an environment that can reach neither an
/// ambient index, nor a user-global cache, nor a proxy.
fn run_in(cwd: &Path, home: &Path, args: &[&str]) -> Output {
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

fn sha256_file(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    sha256_bytes(&bytes)
}

/// Whether `needle` occurs in `haystack` as a standalone token rather than as
/// part of a longer word or path segment. `lib` has to be named as the
/// package, so a `library` in prose or a `/usr/lib/` in a path must not pass
/// for it.
fn names_token(haystack: &str, needle: &str) -> bool {
    let is_word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let bytes = haystack.as_bytes();
    let mut from = 0usize;
    while let Some(hit) = haystack[from..].find(needle) {
        let start = from + hit;
        let end = start + needle.len();
        let before_ok = start == 0 || !is_word(bytes[start - 1]);
        let after_ok = end == bytes.len() || !is_word(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
    }
    false
}

/// One `[[package]]` table of `mamba.lock`, read as data. Kept as a `Vec`
/// rather than a map keyed by name, because "the same name twice" is one of
/// the states this file exists to observe.
struct LockEntry {
    name: String,
    version: String,
    sha256: String,
    url: String,
    source_kind: String,
    path: String,
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
            source_kind: string_at(t, "source_kind"),
            path: string_at(t, "path"),
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

/// Every `name==version` the lock pins, in file order — the observation that
/// tells "one `lib`" from "two `lib`s".
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
        for path in [&dir, &home] {
            std::fs::create_dir_all(path)
                .unwrap_or_else(|e| panic!("fixture: create {}: {e}", path.display()));
        }
        let init_args = ["init"];
        let out = run_in(&dir, &home, &init_args);
        assert!(
            out.status.success(),
            "fixture: `mamba init` must succeed: {}",
            render(&init_args, &out)
        );
        assert!(
            dir.join("pyproject.toml").is_file(),
            "fixture: `mamba init` wrote no pyproject.toml in {}",
            dir.display()
        );
        Project {
            _root: root_dir,
            dir,
            home,
        }
    }

    fn run(&self, args: &[&str]) -> Output {
        run_in(&self.dir, &self.home, args)
    }

    /// Rewrite the manifest's one `dependencies` line, leaving every other
    /// byte `mamba init` wrote in place. This is the case's only authored
    /// input; `dev-dependencies` is a different key and is never touched.
    fn set_dependencies(&self, deps: &[&str]) {
        let manifest = self.dir.join("pyproject.toml");
        let body = std::fs::read_to_string(&manifest)
            .unwrap_or_else(|e| panic!("fixture: read {}: {e}", manifest.display()));
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
            replaced, 1,
            "fixture: the manifest must carry exactly one `dependencies = …` \
             line to rewrite\n--- {} ---\n{body}",
            manifest.display()
        );
        std::fs::write(&manifest, out)
            .unwrap_or_else(|e| panic!("fixture: write {}: {e}", manifest.display()));
    }

    fn lock_body(&self) -> String {
        let path = self.dir.join("mamba.lock");
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    /// Every directory entry whose name begins `mamba.lock` — the lock itself,
    /// and the `mamba.lock.tmp` a half-finished atomic write would leave.
    fn lock_like_entries(&self) -> Vec<String> {
        let mut out: Vec<String> = std::fs::read_dir(&self.dir)
            .unwrap_or_else(|e| panic!("read dir {}: {e}", self.dir.display()))
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with("mamba.lock"))
            .collect();
        out.sort();
        out
    }
}

/// The refusal a conflicting graph must produce, whichever index kind carried
/// it: a non-zero exit, a diagnosed error rather than a panic, and a message
/// naming the contested package and both requirements that disagree.
fn assert_refuses(context: &str, args: &[&str], out: &Output, extra: &str) {
    let stderr = stderr_of(out);
    assert!(
        !out.status.success(),
        "{context}: `mamba lock` must refuse a graph in which no version of \
         `{SHARED_DIST}` satisfies both `{ROOT_REQUIRES}` (declared by \
         `{ROOT_DIST}`) and `{CONFLICTING_OTHER_REQUIRES}` (declared by \
         `{OTHER_DIST}`); a lock written here is a lock that violates one of \
         them\n{}\n{extra}",
        render(args, out)
    );
    assert!(
        !stderr.contains("panicked at"),
        "{context}: the refusal must be a diagnosed resolution error, not a \
         panic — a crash is not a fail-closed path\n{}\n{extra}",
        render(args, out)
    );
    assert!(
        names_token(&stderr, SHARED_DIST),
        "{context}: the refusal must name `{SHARED_DIST}`, the package whose \
         two requirements disagree; nothing in stderr names it\n{}\n{extra}",
        render(args, out)
    );
    let squeezed: String = stderr.chars().filter(|c| !c.is_whitespace()).collect();
    for bound in [LOWER_BOUND, UPPER_BOUND] {
        assert!(
            squeezed.contains(bound),
            "{context}: the refusal must name both conflicting requirements — \
             `{ROOT_REQUIRES}` and `{CONFLICTING_OTHER_REQUIRES}` — and \
             `{bound}` is absent from stderr. The comparison ignores \
             whitespace, so `{SHARED_DIST} {bound}` counts; a `{{:?}}` dump of \
             a `VersionSpecifier` does not spell the operator and does not \
             count, and neither does a message naming only the requirement \
             that arrived last\n{}\n{extra}",
            render(args, out)
        );
    }
}

/// The refusal left the project with no lock at all, and no fragment of one.
fn assert_no_lock_written(context: &str, project: &Project, args: &[&str], out: &Output) {
    let found = project.lock_like_entries();
    assert!(
        found.is_empty(),
        "{context}: a refused resolution must write no lock and leave no \
         fragment of one behind; {} holds {found:?}\n{}",
        project.dir.display(),
        render(args, out)
    );
}

// --------------------------------------------------------------------------
// the registry half
// --------------------------------------------------------------------------

/// One artifact the registry serves: its filename, where it lives, and what
/// its bytes hash to. The digest is taken over the exact bytes mounted at
/// `url`, so a lock entry is always compared against one real file.
struct Artifact {
    file: String,
    url: String,
    digest: String,
    bytes: Vec<u8>,
}

/// Build one real wheel through the product's own wheel builder and describe
/// it as the artifact a registry would advertise.
fn build_artifact(dist: &str, version: &str, requires: &[&str]) -> Artifact {
    let dir = tempfile::tempdir().expect("fixture: create a temp dir for the wheel");
    let built = build_wheel_file(dir.path(), dist, version, requires);
    let file = built
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_else(|| panic!("fixture: the wheel builder named the artifact {}", built.display()))
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

/// A loopback registry that stays up for the whole test.
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
    root: Artifact,
    other: Artifact,
    shared_low: Artifact,
    shared_high: Artifact,
}

impl Registry {
    /// Serve the graph in which `app` requires `lib>=2` and `other` requires
    /// `other_requires`. That one argument is the whole difference between
    /// the conflicting graph and the control.
    fn start(other_requires: &'static str) -> Registry {
        // The wheels carry no `Requires-Dist` here: on this half the edge
        // exists only in the registry's per-version JSON, so a lock that knows
        // about it is a lock that asked the registry.
        let root = build_artifact(ROOT_DIST, ROOT_VERSION, &[]);
        let other = build_artifact(OTHER_DIST, OTHER_VERSION, &[]);
        let shared_low = build_artifact(SHARED_DIST, SHARED_LOW, &[]);
        let shared_high = build_artifact(SHARED_DIST, SHARED_HIGH, &[]);
        let digests = [
            &root.digest,
            &other.digest,
            &shared_low.digest,
            &shared_high.digest,
        ];
        for (i, left) in digests.iter().enumerate() {
            for right in digests.iter().skip(i + 1) {
                assert_ne!(
                    left, right,
                    "fixture: the served wheels must have distinct digests, or \
                     this case cannot tell which artifact a lock entry described"
                );
            }
        }

        let rt = tokio::runtime::Runtime::new().expect("fixture: build a runtime for the registry");
        let (server, base) = rt.block_on(async {
            let server = MockServer::start().await;
            let base = server.uri();

            // Every project-level JSON endpoint says "no such project", so the
            // PEP 503 simple page is the only route to a release list; `lib`'s
            // per-version endpoints say the same, which the client reads as
            // "declares no dependencies". Mounted explicitly rather than left
            // to the unmatched-request default, so each fallback is a served
            // answer.
            for route in [
                format!("/pypi/{ROOT_DIST}/json"),
                format!("/pypi/{OTHER_DIST}/json"),
                format!("/pypi/{SHARED_DIST}/json"),
                format!("/pypi/{SHARED_DIST}/{SHARED_LOW}/json"),
                format!("/pypi/{SHARED_DIST}/{SHARED_HIGH}/json"),
            ] {
                Mock::given(method("GET"))
                    .and(path(route))
                    .respond_with(ResponseTemplate::new(404))
                    .mount(&server)
                    .await;
            }

            // The only place the two dependency edges exist.
            for (route, requires) in [
                (
                    format!("/pypi/{ROOT_DIST}/{ROOT_VERSION}/json"),
                    ROOT_REQUIRES,
                ),
                (
                    format!("/pypi/{OTHER_DIST}/{OTHER_VERSION}/json"),
                    other_requires,
                ),
            ] {
                let body = format!("{{\"info\":{{\"requires_dist\":[\"{requires}\"]}}}}");
                Mock::given(method("GET"))
                    .and(path(route))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .set_body_raw(body.into_bytes(), "application/json"),
                    )
                    .mount(&server)
                    .await;
            }

            for (route, page) in [
                (
                    format!("/simple/{ROOT_DIST}/"),
                    simple_page(ROOT_DIST, &[&root], &base),
                ),
                (
                    format!("/simple/{OTHER_DIST}/"),
                    simple_page(OTHER_DIST, &[&other], &base),
                ),
                (
                    format!("/simple/{SHARED_DIST}/"),
                    simple_page(SHARED_DIST, &[&shared_low, &shared_high], &base),
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

            for artifact in [&root, &other, &shared_low, &shared_high] {
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
            root,
            other,
            shared_low,
            shared_high,
        }
    }

    /// The `--index-url` this case passes: the PEP 503 simple URL, which the
    /// client normalises back to the registry base before building its own
    /// paths.
    fn index_url(&self) -> String {
        format!("{}/simple", self.base)
    }

    /// The absolute URL an artifact is advertised at.
    fn absolute(&self, artifact: &Artifact) -> String {
        format!("{}{}", self.base, artifact.url)
    }

    /// Every request the registry saw, as `METHOD /path`, folded into each
    /// panic message so a red names what the client actually asked for — in
    /// particular whether it ever asked for the two edges.
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
        for artifact in [&self.root, &self.other, &self.shared_low, &self.shared_high] {
            if artifact.digest == digest {
                return format!("the wheel {}", artifact.file);
            }
        }
        if digest.is_empty() {
            return "nothing — the digest is empty, which `mamba sync` reads as nothing to verify"
                .to_string();
        }
        "no artifact this registry advertised".to_string()
    }
}

/// The contract on `mamba lock --index-url`: a graph whose two declared edges
/// cannot both hold is refused, and neither a fresh project nor one holding an
/// earlier good lock is left with a lock that violates an edge.
#[test]
fn registry_lock_refuses_a_conflicting_graph_and_writes_no_lock() {
    let registry = Registry::start(CONFLICTING_OTHER_REQUIRES);
    let index_url = registry.index_url();
    let lock_args = ["lock", "--index-url", index_url.as_str()];

    // ---- fixture self-check, and the lock that must survive --------------
    // One root alone resolves cleanly against this same registry, this same
    // page and these same wheels. A red here means the fixture is wrong, or
    // the refusal is so broad it now rejects a graph with no conflict in it.
    let held = Project::start();
    held.set_dependencies(&[ROOT_DIST]);
    let out = held.run(&lock_args);
    assert!(
        out.status.success(),
        "fixture self-check: `mamba lock --index-url` must resolve `{ROOT_DIST}` \
         alone — its one edge `{ROOT_REQUIRES}` is satisfiable by \
         `{SHARED_DIST}=={SHARED_HIGH}`, so nothing here conflicts\n{}\n{}",
        render(&lock_args, &out),
        registry.seen()
    );
    let good_lock = held.lock_body();
    let good_entries = parse_lock(&good_lock);
    let good_shared = only_entry(&good_entries, SHARED_DIST, &good_lock);
    assert_eq!(
        good_shared.version, SHARED_HIGH,
        "fixture self-check: `{ROOT_REQUIRES}` must select \
         `{SHARED_DIST}=={SHARED_HIGH}` from the two releases \
         /simple/{SHARED_DIST}/ advertises — that pick is what the second root's \
         `{CONFLICTING_OTHER_REQUIRES}` then contradicts\n{}\n--- mamba.lock ---\n{good_lock}",
        registry.seen()
    );

    // ---- the contract, on a project that has never held a lock -----------
    let fresh = Project::start();
    fresh.set_dependencies(&[ROOT_DIST, OTHER_DIST]);
    let out = fresh.run(&lock_args);
    assert_refuses(
        "registry, fresh project",
        &lock_args,
        &out,
        &registry.seen(),
    );
    assert_no_lock_written("registry, fresh project", &fresh, &lock_args, &out);

    // ---- the contract, on a project whose good lock must survive ---------
    held.set_dependencies(&[ROOT_DIST, OTHER_DIST]);
    let out = held.run(&lock_args);
    assert_refuses(
        "registry, project holding an earlier lock",
        &lock_args,
        &out,
        &registry.seen(),
    );
    assert_eq!(
        held.lock_like_entries(),
        vec!["mamba.lock".to_string()],
        "registry, project holding an earlier lock: the refusal must leave the \
         lock that was already there and nothing else — a `mamba.lock.tmp` here \
         is a half-finished write\n{}",
        render(&lock_args, &out)
    );
    assert_eq!(
        held.lock_body(),
        good_lock,
        "registry, project holding an earlier lock: a refused resolution must \
         leave the existing mamba.lock byte-for-byte as it was; truncating or \
         rewriting it destroys a pin set the project was working from\n{}\n\
         --- mamba.lock before ---\n{good_lock}",
        render(&lock_args, &out)
    );
}

/// The control for the registry half: `{SHARED_DIST}` is still a name two
/// requirements reach, they agree, and the lock is exactly what it is today.
#[test]
fn registry_lock_still_pins_a_name_two_requirements_agree_on() {
    let registry = Registry::start(COMPATIBLE_OTHER_REQUIRES);
    let index_url = registry.index_url();
    let lock_args = ["lock", "--index-url", index_url.as_str()];

    let project = Project::start();
    project.set_dependencies(&[ROOT_DIST, OTHER_DIST]);
    let out = project.run(&lock_args);
    assert!(
        out.status.success(),
        "control: `{ROOT_DIST}` requires `{ROOT_REQUIRES}` and `{OTHER_DIST}` \
         requires `{COMPATIBLE_OTHER_REQUIRES}`, and \
         `{SHARED_DIST}=={SHARED_HIGH}` satisfies both — a rule that refuses \
         this graph is refusing on `the name arrived twice` rather than on the \
         constraints disagreeing\n{}\n{}",
        render(&lock_args, &out),
        registry.seen()
    );

    let body = project.lock_body();
    let entries = parse_lock(&body);
    assert_eq!(
        pins_of(&entries),
        vec![
            format!("{ROOT_DIST}=={ROOT_VERSION}"),
            format!("{SHARED_DIST}=={SHARED_HIGH}"),
            format!("{OTHER_DIST}=={OTHER_VERSION}"),
        ],
        "control: the compatible graph must pin each name once, with \
         `{SHARED_DIST}` at the version both requirements admit\n{}\n\
         --- mamba.lock ---\n{body}",
        registry.seen()
    );

    let root = only_entry(&entries, ROOT_DIST, &body);
    let other = only_entry(&entries, OTHER_DIST, &body);
    let shared = only_entry(&entries, SHARED_DIST, &body);
    for direct in [root, other] {
        assert_eq!(
            direct.direct,
            Some(true),
            "control: `{}` is a root the manifest names, so it must stay \
             `direct = true`\n--- mamba.lock ---\n{body}",
            direct.name
        );
        assert_eq!(
            direct.dependencies,
            vec![SHARED_EDGE.to_string()],
            "control: `{}`'s edge must be rendered as the pinned \
             `{SHARED_EDGE}`, which is the version its own declared \
             requirement admits\n--- mamba.lock ---\n{body}",
            direct.name
        );
    }
    assert_eq!(
        shared.direct,
        Some(false),
        "control: `{SHARED_DIST}` is reached through the two roots, not named \
         by the manifest, so it must be recorded with `direct = \
         false`\n--- mamba.lock ---\n{body}"
    );
    assert!(
        shared.dependencies.is_empty(),
        "control: `{SHARED_DIST}` declares no dependencies of its own, so its \
         `dependencies` must be empty; got {:?}\n--- mamba.lock ---\n{body}",
        shared.dependencies
    );

    // Security: the surviving pin still names exactly one file — a non-empty
    // url and sha256, the url the registry advertised, and the digest of the
    // bytes it serves there. An empty digest is the fail-open state `mamba
    // sync` reads as nothing to verify.
    assert!(
        !shared.url.is_empty() && !shared.sha256.is_empty(),
        "control: the entry for `{SHARED_DIST}` must carry both a url and a \
         sha256; got url {:?} and sha256 {:?}\n--- mamba.lock ---\n{body}",
        shared.url,
        shared.sha256
    );
    assert_eq!(
        shared.url,
        registry.absolute(&registry.shared_high),
        "control: `{SHARED_DIST}` must be locked at the artifact URL this \
         registry advertised for {}\n--- mamba.lock ---\n{body}",
        registry.shared_high.file
    );
    assert_eq!(
        shared.sha256,
        registry.shared_high.digest,
        "control: the sha256 locked for `{SHARED_DIST}` must be the digest of \
         the bytes served at the locked url; it recorded {}\n--- mamba.lock ---\n{body}",
        registry.describe_digest(&shared.sha256)
    );

    // The bytes are the contract too: a compatible re-arrival must resolve
    // exactly as it does today, replay included.
    let out = project.run(&lock_args);
    assert!(
        out.status.success(),
        "control: re-locking the same manifest against the same registry must \
         succeed\n{}\n{}",
        render(&lock_args, &out),
        registry.seen()
    );
    assert_eq!(
        project.lock_body(),
        body,
        "control: `mamba lock --index-url` must be byte-stable on replay\n{}",
        render(&lock_args, &out)
    );
}

// --------------------------------------------------------------------------
// the frozen-index half
// --------------------------------------------------------------------------

/// The wheels of one graph, frozen into an index by `mamba index build`. The
/// dependency edges live in each wheel's own `Requires-Dist`, and nothing
/// beside the wheels is authored: the index layout, its `metadata.toml`s and
/// its digests are all the product's output.
struct FrozenIndex {
    _root: tempfile::TempDir,
    dir: PathBuf,
    /// `<index>/lib/<version>/<wheel>` for each release, so a lock entry can
    /// be compared against the file it claims to pin.
    shared_low_wheel: PathBuf,
    shared_high_wheel: PathBuf,
}

impl FrozenIndex {
    /// Freeze the graph in which `app` requires `lib>=2` and `other` requires
    /// `other_requires`. As on the registry half, that one argument is the
    /// whole difference between the conflicting graph and the control.
    fn build(other_requires: &str) -> FrozenIndex {
        let root_dir = tempfile::tempdir().expect("fixture: create temp root for the index");
        let root = root_dir.path().to_path_buf();
        let wheels = root.join("wheels");
        let index = root.join("index");
        let home = root.join("home");
        for path in [&wheels, &home] {
            std::fs::create_dir_all(path)
                .unwrap_or_else(|e| panic!("fixture: create {}: {e}", path.display()));
        }

        let built: Vec<PathBuf> = [
            (ROOT_DIST, ROOT_VERSION, vec![ROOT_REQUIRES]),
            (OTHER_DIST, OTHER_VERSION, vec![other_requires]),
            (SHARED_DIST, SHARED_LOW, vec![]),
            (SHARED_DIST, SHARED_HIGH, vec![]),
        ]
        .iter()
        .map(|(dist, version, requires)| build_wheel_file(&wheels, dist, version, requires))
        .collect();

        let index_str = index.to_str().expect("index path is utf-8").to_string();
        let mut build_args = vec!["index", "build", "--out", index_str.as_str()];
        let built_strs: Vec<String> = built
            .iter()
            .map(|p| p.to_str().expect("wheel path is utf-8").to_string())
            .collect();
        for wheel in &built_strs {
            build_args.push(wheel.as_str());
        }
        let out = run_in(&root, &home, &build_args);
        assert!(
            out.status.success(),
            "fixture: `mamba index build` must stage the four wheels: {}",
            render(&build_args, &out)
        );

        // Fixture self-check: a broken wheel builder or a changed index layout
        // must fail here, naming itself, rather than surfacing as a lock
        // assertion further down. Every name in this graph is already PEP 503
        // normal form, so the directory name is the package name.
        let staged = |dist: &str, version: &str, wheel: &Path| -> PathBuf {
            let want = index
                .join(dist)
                .join(version)
                .join(wheel.file_name().expect("wheel has a filename"));
            assert!(
                want.is_file(),
                "fixture: `mamba index build` did not stage {dist}-{version} at {}",
                want.display()
            );
            want
        };
        staged(ROOT_DIST, ROOT_VERSION, &built[0]);
        staged(OTHER_DIST, OTHER_VERSION, &built[1]);
        let shared_low_wheel = staged(SHARED_DIST, SHARED_LOW, &built[2]);
        let shared_high_wheel = staged(SHARED_DIST, SHARED_HIGH, &built[3]);

        FrozenIndex {
            _root: root_dir,
            dir: index,
            shared_low_wheel,
            shared_high_wheel,
        }
    }

    fn arg(&self) -> String {
        self.dir.to_str().expect("index path is utf-8").to_string()
    }

    /// What the index holds, for a panic message: the release directories of
    /// the contested name.
    fn shared_releases(&self) -> String {
        format!(
            "--- index ---\n{}\n{}",
            self.shared_low_wheel.display(),
            self.shared_high_wheel.display()
        )
    }
}

/// Build a real, importable wheel through the product's own wheel builder.
/// `requires` becomes the wheel's `Requires-Dist:` lines verbatim — the only
/// place this half of the case states a dependency edge.
fn build_wheel_file(out_dir: &Path, dist: &str, version: &str, requires: &[&str]) -> PathBuf {
    let filename = compose_filename(dist, version, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-lock-conflict-refused");
    wheel_meta.tags.push("py3-none-any".into());
    let mut core_meta = CoreMetadata::new(dist, version);
    core_meta.requires_dist = requires.iter().map(|r| (*r).to_string()).collect();
    let rendered = filename.to_filename();
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

/// The contract on `mamba lock --index DIR`: the frozen walker refuses the
/// same conflict the resolver does, and leaves the same two projects alone.
#[test]
fn frozen_index_lock_refuses_a_conflicting_graph_and_writes_no_lock() {
    let index = FrozenIndex::build(CONFLICTING_OTHER_REQUIRES);
    let index_arg = index.arg();
    let lock_args = ["lock", "--index", index_arg.as_str()];
    // `Pin::parse` requires `==` for a direct dependency on this path, so both
    // roots are pinned; the conflict is reached through their transitive edges.
    let root_pin = format!("{ROOT_DIST}=={ROOT_VERSION}");
    let other_pin = format!("{OTHER_DIST}=={OTHER_VERSION}");

    // ---- fixture self-check, and the lock that must survive --------------
    let held = Project::start();
    held.set_dependencies(&[root_pin.as_str()]);
    let out = held.run(&lock_args);
    assert!(
        out.status.success(),
        "fixture self-check: `mamba lock --index` must resolve `{root_pin}` \
         alone — its one edge `{ROOT_REQUIRES}` is satisfiable by \
         `{SHARED_DIST}=={SHARED_HIGH}`, so nothing here conflicts\n{}\n{}",
        render(&lock_args, &out),
        index.shared_releases()
    );
    let good_lock = held.lock_body();
    let good_entries = parse_lock(&good_lock);
    let good_shared = only_entry(&good_entries, SHARED_DIST, &good_lock);
    assert_eq!(
        good_shared.version, SHARED_HIGH,
        "fixture self-check: `{ROOT_REQUIRES}` must select \
         `{SHARED_DIST}=={SHARED_HIGH}` from the two releases the index holds — \
         that pick is what the second root's `{CONFLICTING_OTHER_REQUIRES}` then \
         contradicts\n{}\n--- mamba.lock ---\n{good_lock}",
        index.shared_releases()
    );

    // ---- the contract, on a project that has never held a lock -----------
    let fresh = Project::start();
    fresh.set_dependencies(&[root_pin.as_str(), other_pin.as_str()]);
    let out = fresh.run(&lock_args);
    assert_refuses(
        "frozen index, fresh project",
        &lock_args,
        &out,
        &index.shared_releases(),
    );
    assert_no_lock_written("frozen index, fresh project", &fresh, &lock_args, &out);

    // ---- the contract, on a project whose good lock must survive ---------
    held.set_dependencies(&[root_pin.as_str(), other_pin.as_str()]);
    let out = held.run(&lock_args);
    assert_refuses(
        "frozen index, project holding an earlier lock",
        &lock_args,
        &out,
        &index.shared_releases(),
    );
    assert_eq!(
        held.lock_like_entries(),
        vec!["mamba.lock".to_string()],
        "frozen index, project holding an earlier lock: the refusal must leave \
         the lock that was already there and nothing else — a `mamba.lock.tmp` \
         here is a half-finished write\n{}",
        render(&lock_args, &out)
    );
    assert_eq!(
        held.lock_body(),
        good_lock,
        "frozen index, project holding an earlier lock: a refused resolution \
         must leave the existing mamba.lock byte-for-byte as it was\n{}\n\
         --- mamba.lock before ---\n{good_lock}",
        render(&lock_args, &out)
    );
}

/// The control for the frozen half: two requirements reach `lib`, they agree,
/// and the lock is exactly what it is today — one `lib` block, not two.
#[test]
fn frozen_index_lock_still_pins_a_name_two_requirements_agree_on() {
    let index = FrozenIndex::build(COMPATIBLE_OTHER_REQUIRES);
    let index_arg = index.arg();
    let lock_args = ["lock", "--index", index_arg.as_str()];
    let root_pin = format!("{ROOT_DIST}=={ROOT_VERSION}");
    let other_pin = format!("{OTHER_DIST}=={OTHER_VERSION}");

    let project = Project::start();
    project.set_dependencies(&[root_pin.as_str(), other_pin.as_str()]);
    let out = project.run(&lock_args);
    assert!(
        out.status.success(),
        "control: `{ROOT_DIST}` requires `{ROOT_REQUIRES}` and `{OTHER_DIST}` \
         requires `{COMPATIBLE_OTHER_REQUIRES}`, and \
         `{SHARED_DIST}=={SHARED_HIGH}` satisfies both — a rule that refuses \
         this graph is refusing on `the name arrived twice` rather than on the \
         constraints disagreeing\n{}\n{}",
        render(&lock_args, &out),
        index.shared_releases()
    );

    let body = project.lock_body();
    let entries = parse_lock(&body);
    assert_eq!(
        pins_of(&entries),
        vec![
            format!("{ROOT_DIST}=={ROOT_VERSION}"),
            format!("{SHARED_DIST}=={SHARED_HIGH}"),
            format!("{OTHER_DIST}=={OTHER_VERSION}"),
        ],
        "control: the compatible graph must pin each name once — two \
         [[package]] blocks named `{SHARED_DIST}` describe an environment that \
         cannot exist\n{}\n--- mamba.lock ---\n{body}",
        index.shared_releases()
    );

    let root = only_entry(&entries, ROOT_DIST, &body);
    let other = only_entry(&entries, OTHER_DIST, &body);
    let shared = only_entry(&entries, SHARED_DIST, &body);
    for direct in [root, other] {
        assert_eq!(
            direct.direct,
            Some(true),
            "control: `{}` is a root the manifest names, so it must stay \
             `direct = true`\n--- mamba.lock ---\n{body}",
            direct.name
        );
        assert_eq!(
            direct.dependencies,
            vec![SHARED_EDGE.to_string()],
            "control: `{}`'s edge must be rendered as the pinned \
             `{SHARED_EDGE}`, which is the version its own declared \
             requirement admits\n--- mamba.lock ---\n{body}",
            direct.name
        );
    }
    assert!(
        shared.dependencies.is_empty(),
        "control: `{SHARED_DIST}` declares no `Requires-Dist`, so its \
         `dependencies` must be empty; got {:?}\n--- mamba.lock ---\n{body}",
        shared.dependencies
    );

    // Security: the surviving pin still points at one file inside the index it
    // was told to use, with that file's digest.
    assert_eq!(
        shared.source_kind, "index",
        "control: `{SHARED_DIST}` came from a frozen index and must say \
         so\n--- mamba.lock ---\n{body}"
    );
    assert_eq!(
        shared.url, "",
        "control: `{SHARED_DIST}` came from a frozen index, so it has no \
         artifact URL\n--- mamba.lock ---\n{body}"
    );
    assert_eq!(
        shared.sha256,
        sha256_file(&index.shared_high_wheel),
        "control: `{SHARED_DIST}` must lock the sha256 of {} — an empty or \
         mismatched digest is the fail-open state `mamba sync` reads as \
         nothing to verify\n--- mamba.lock ---\n{body}",
        index.shared_high_wheel.display()
    );
    let index_real = std::fs::canonicalize(&index.dir)
        .unwrap_or_else(|e| panic!("canonicalize {}: {e}", index.dir.display()));
    let locked_real = std::fs::canonicalize(Path::new(&shared.path))
        .unwrap_or_else(|e| panic!("canonicalize locked path {:?}: {e}", shared.path));
    assert!(
        locked_real.starts_with(&index_real),
        "control: `{SHARED_DIST}` locks path `{}`, which resolves outside the \
         index {}\n--- mamba.lock ---\n{body}",
        locked_real.display(),
        index_real.display()
    );
    assert_eq!(
        locked_real,
        std::fs::canonicalize(&index.shared_high_wheel).expect("canonicalize staged wheel"),
        "control: `{SHARED_DIST}` must lock the wheel `mamba index build` \
         staged for {SHARED_HIGH}\n--- mamba.lock ---\n{body}"
    );

    // The bytes are the contract too: a compatible re-arrival must resolve
    // exactly as it does today, replay included.
    let out = project.run(&lock_args);
    assert!(
        out.status.success(),
        "control: re-locking the same manifest against the same index must \
         succeed\n{}",
        render(&lock_args, &out)
    );
    assert_eq!(
        project.lock_body(),
        body,
        "control: `mamba lock --index` must be byte-stable on replay\n{}",
        render(&lock_args, &out)
    );
}
