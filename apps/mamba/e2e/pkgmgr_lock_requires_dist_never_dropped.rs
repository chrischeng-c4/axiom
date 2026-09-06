//! Black-box contract: `mamba lock` must read every `requires_dist` line a
//! release declares — including the parenthesized `name (<4,>=2)` form PEP 508
//! spells and every Metadata 2.1 writer emits — and, when a line cannot be
//! read at all, must refuse the resolution instead of dropping the line and
//! writing a lock that quietly lacks the edge.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over throwaway directory trees and
//! reads only what the binary wrote, or refused to write. It never links the
//! resolver, the requirement grammar, the specifier grammar or the lock
//! renderer, and it never hand-writes a `mamba.lock` or an index page: every
//! wheel is built by the product's own `WheelBuilder`, and the registry serves
//! PEP 503 anchor pages plus per-version `requires_dist` documents. The wheels
//! themselves declare no `Requires-Dist`, so every dependency edge below
//! exists in exactly one place — the registry's own
//! `/pypi/<name>/<version>/json` — and a lock that knows about an edge is a
//! lock that asked for it and read the answer. The one thing the case authors
//! is `mamba.toml`'s `dependencies` line, which is user input in this workflow
//! and precisely what `mamba lock` reads.
//!
//! # The three graphs
//!
//! ```text
//!   one:   requests 2.31.0 --requires charset-normalizer (<4,>=2)    --> charset-normalizer 3.3.2
//!                          --requires idna (<4,>=2.5)                --> idna 3.6
//!                          --requires urllib3 (<3,>=1.21.1)          --> urllib3 2.1.0
//!                          --requires certifi (>=2017.4.17)          --> certifi 2023.11.17
//!                          --requires PySocks (…) ; extra == 'socks' --> excluded by policy
//!                          --requires chardet (…) ; extra == 'use_…' --> excluded by policy
//!
//!   two:   legacyapp 1.0   --requires legacylib (<3,>=1.21.1)        --> legacylib {1.0, 2.5, 3.0}
//!
//!   three: brokenapp 1.0   --requires this is not a requirement      --> unreadable
//! ```
//!
//! One is the six lines `https://pypi.org/pypi/requests/2.31.0/json` serves
//! today, copied verbatim: four unconditional, two gated behind an extra this
//! project does not activate. The lock owes exactly five packages — the root
//! and the four unconditional dependencies — so a run that reads the
//! parentheses but loses the marker locks seven and is refused here just as
//! firmly as one that drops all six.
//!
//! Two separates "the name survived" from "the bounds survived": `legacylib`
//! is published at `1.0`, `2.5` and `3.0`, `<3,>=1.21.1` admits exactly one of
//! them, and it is neither the newest nor the oldest. A parser that recovered
//! the name and threw the specifier set away would lock `3.0` and fail here.
//!
//! Three is the fail-closed path: a line no parser can read is registry-
//! supplied input this project cannot honour, and the only safe answer is to
//! stop. Exit non-zero, name the release and the offending line, and leave no
//! lock behind.
//!
//! # Why today's tree cannot pass
//!
//! Measured against the binary built from `9304d43c52`:
//!
//! - One and two exit 0 and write a lock whose only `[[package]]` is the root,
//!   carrying `dependencies = []`. A parenthesized specifier set reaches the
//!   specifier grammar with its parentheses still attached, is refused there,
//!   and a `requires_dist` line that fails to parse is dropped without a word
//!   — so every edge, and with it every dependency, vanishes between the
//!   registry and the lock.
//! - Three exits 0 as well, and writes that same one-package lock: the
//!   unreadable line takes the identical silent-drop path.
//!
//! Each test therefore opens with a fixture self-check that passes on that
//! same tree, so the red it reports is about the resolution and not about the
//! registry, the wheels or the manifest. One asks for all six of `requests`'
//! dependencies by name and requires the registry to serve every one of them,
//! which is what makes the two extras-gated absences below a policy decision
//! rather than a gap in the fixture. Two locks `legacylib` with no constraint
//! on it at all and requires the newest of its three releases. Three locks a
//! second, well-formed distribution the same registry serves.
//!
//! # Facets
//!
//! - **Behavior**: `mamba lock --index-url` records every unconditional
//!   `requires_dist` edge of every release it resolves, pins each name at the
//!   version the declared bounds admit, renders each edge as `name==version`
//!   against those pins, and refuses — with a diagnosis — a release whose
//!   `requires_dist` it cannot read.
//! - **Security (a declared bound is a boundary, and a dropped line is a
//!   silent downgrade of it)**: `<4`, `<3`, `<6` and `!=1.5.7` are how a
//!   release keeps a known-bad dependency out of the environment `mamba sync`
//!   later builds from the lock. Dropping the line drops the bound *and* the
//!   package, so the lock under-reports the closure it is trusted to describe;
//!   these tests therefore assert the concrete pinned versions, the exact set
//!   of packages, and that each entry's `url` and `sha256` name the file the
//!   registry advertised *for that version* — a lock that says `2.5` while
//!   pointing at the `3.0` bytes, or that carries an empty digest for `mamba
//!   sync` to skip, fails here. Case three is the fail-closed half: the
//!   `requires_dist` list is untrusted input from the index, and a line that
//!   cannot be parsed must stop the resolution with a diagnosis, never crash
//!   (`panicked at` is refused) and never leave a truncated or partial
//!   `mamba.lock` — not even a `mamba.lock.tmp` — behind.
//! - **Performance**: this work item names no budget, so the case asserts no
//!   timing, no request count and no concurrency.
//!
//! # Hermeticity
//!
//! No network, ever: every wheel is built in-process and the only package
//! source is an in-process `wiremock` server on loopback — nothing here
//! resolves against pypi.org, whose real answers this case only quotes. `HOME`
//! and `MAMBA_CACHE_DIR` are pinned per project into its own temp tree, so no
//! two projects can read each other's cached metadata; `MAMBA_FROZEN_INDEX`,
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

// --------------------------------------------------------------------------
// case one: the six lines pypi.org serves for requests 2.31.0, verbatim
// --------------------------------------------------------------------------
const R_ROOT: &str = "requests";
const R_ROOT_VERSION: &str = "2.31.0";

/// The four unconditional lines. Each is parenthesized, which is the form
/// `Metadata-Version: 2.1` writers emit and which PEP 508's grammar spells as
/// `versionspec = ( '(' version_many ')' ) | version_many`.
const R_LINE_CHARSET: &str = "charset-normalizer (<4,>=2)";
const R_LINE_IDNA: &str = "idna (<4,>=2.5)";
const R_LINE_URLLIB3: &str = "urllib3 (<3,>=1.21.1)";
const R_LINE_CERTIFI: &str = "certifi (>=2017.4.17)";
/// The two lines gated behind an extra this project never activates. They are
/// parenthesized too, so they separate "the parentheses were read" from "the
/// marker was still honoured": a run that keeps them locks seven packages.
const R_LINE_PYSOCKS: &str = "PySocks (!=1.5.7,>=1.5.6) ; extra == 'socks'";
const R_LINE_CHARDET: &str = "chardet (<6,>=3.0.2) ; extra == 'use_chardet_on_py3'";

const R_REQUIRES_DIST: &[&str] = &[
    R_LINE_CHARSET,
    R_LINE_IDNA,
    R_LINE_URLLIB3,
    R_LINE_CERTIFI,
    R_LINE_PYSOCKS,
    R_LINE_CHARDET,
];

const R_CHARSET: &str = "charset-normalizer";
const R_CHARSET_V: &str = "3.3.2";
const R_IDNA: &str = "idna";
const R_IDNA_V: &str = "3.6";
const R_URLLIB3: &str = "urllib3";
const R_URLLIB3_V: &str = "2.1.0";
const R_CERTIFI: &str = "certifi";
const R_CERTIFI_V: &str = "2023.11.17";
/// Published, and inside `!=1.5.7,>=1.5.6`, so its absence from the lock is a
/// marker decision rather than a registry that could not serve it.
const R_PYSOCKS: &str = "pysocks";
const R_PYSOCKS_V: &str = "1.7.1";
/// Published, and inside `<6,>=3.0.2`, for the same reason.
const R_CHARDET: &str = "chardet";
const R_CHARDET_V: &str = "5.2.0";

// --------------------------------------------------------------------------
// case two: the bounds decide, and their answer is neither newest nor oldest
// --------------------------------------------------------------------------
const L_ROOT: &str = "legacyapp";
const L_ROOT_VERSION: &str = "1.0";
const L_LINE: &str = "legacylib (<3,>=1.21.1)";
const L_LIB: &str = "legacylib";
/// Below `>=1.21.1` — `1.0` precedes `1.21.1`, so the oldest release is out.
const L_TOO_LOW: &str = "1.0";
const L_MATCH: &str = "2.5";
/// Above `<3`, and the release an unconstrained resolution takes.
const L_TOO_HIGH: &str = "3.0";

// --------------------------------------------------------------------------
// case three: a line no parser can read
// --------------------------------------------------------------------------
const B_ROOT: &str = "brokenapp";
const B_ROOT_VERSION: &str = "1.0";
const B_LINE: &str = "this is not a requirement";
/// A second, well-formed distribution on the same registry, so the fixture
/// self-check can prove the registry answers without touching `brokenapp`.
const B_CONTROL: &str = "sanelib";
const B_CONTROL_VERSION: &str = "1.0";

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

/// `stderr` with every whitespace byte removed, so a message counts however it
/// happened to wrap.
fn squeezed_stderr(out: &Output) -> String {
    stderr_of(out)
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

/// Whether `needle` occurs in `haystack` as a standalone token rather than as
/// part of a longer word or path segment, so a distribution name has to be
/// named as itself and not matched inside a longer word or a directory.
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

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

// --------------------------------------------------------------------------
// reading the lock the product wrote
// --------------------------------------------------------------------------

/// One `[[package]]` table of `mamba.lock`, read as data. Kept as a `Vec`
/// rather than a map keyed by name, because "the same name twice" and "the
/// name not at all" are both states this file exists to observe.
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
            dir.join("mamba.toml").is_file(),
            "fixture: `mamba init` wrote no mamba.toml in {}",
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
        let manifest = self.dir.join("mamba.toml");
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

    /// Lock against `registry`, require the run to have stood behind its
    /// answer, and return the lock it wrote, parsed. Every passing test reads
    /// its answer through here, so no test can accept a lock the binary
    /// refused.
    fn lock_ok(&self, context: &str, registry: &Registry) -> (String, Vec<LockEntry>) {
        let index_url = registry.index_url();
        let args = ["lock", "--index-url", index_url.as_str()];
        let out = self.run(&args);
        assert!(
            out.status.success(),
            "{context}: `mamba lock --index-url` must resolve this graph — \
             every `requires_dist` line in it is a requirement PEP 508 spells, \
             and every name one raises has a release satisfying it, so there is \
             nothing here to refuse\n{}\n{}",
            render(&args, &out),
            registry.seen()
        );
        assert!(
            !stderr_of(&out).contains("panicked at"),
            "{context}: the resolution must be a resolution, not a crash\n{}\n{}",
            render(&args, &out),
            registry.seen()
        );
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
    url: String,
    digest: String,
    bytes: Vec<u8>,
}

/// One release the registry publishes: a distribution at a version, with the
/// `requires_dist` lines the registry declares for it. The wheel itself
/// declares nothing, so this list is the only place the edge exists.
struct Release {
    dist: &'static str,
    version: &'static str,
    requires: &'static [&'static str],
}

/// Build one real wheel through the product's own wheel builder.
///
/// PEP 427 escapes every run of `-` in the distribution component of a wheel
/// filename to `_`, and a real index serves `charset_normalizer-3.3.2-….whl`
/// under `/simple/charset-normalizer/`. The escape is applied here for the
/// same reason a real build backend applies it: the filename's second
/// `-`-separated field is what an index reads as the version.
fn build_wheel_file(out_dir: &Path, dist: &str, version: &str) -> PathBuf {
    let escaped = dist.replace('-', "_");
    let filename = compose_filename(&escaped, version, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-requires-dist");
    wheel_meta.tags.push("py3-none-any".into());
    let core_meta = CoreMetadata::new(dist, version);
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

/// Build one wheel and describe it as the artifact a registry advertises.
fn build_artifact(dist: &str, version: &str) -> Artifact {
    let dir = tempfile::tempdir().expect("fixture: create a temp dir for the wheel");
    let built = build_wheel_file(dir.path(), dist, version);
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

/// A loopback registry serving one graph, up for the whole test.
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
    /// releases, one per-version JSON document per release carrying its
    /// `requires_dist` lines, and the wheel bytes each anchor points at.
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
    /// particular which per-version documents it read the edges from.
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

/// The entry for `name` pins `version`, and the file it names is the one the
/// registry advertised for that very version — not a release the declared
/// bounds excluded, and not nothing at all.
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
        entry.version, version,
        "{context}: `{name}` must be pinned at {version}, the version the \
         declared `requires_dist` bounds admit\n--- mamba.lock ---\n{body}\n{}",
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
/// says, and its edges are exactly `edges`, rendered against the versions
/// actually pinned.
///
/// Both sides are sorted before the comparison: which order a lock lists one
/// package's edges in is not this contract's business, but *which* edges it
/// lists, and how many, is the whole of it.
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
        "{context}: `{name}` must be recorded with `direct = {direct}`\n\
         --- mamba.lock ---\n{body}"
    );
    let mut got = entry.dependencies.clone();
    got.sort();
    let mut want = edges.to_vec();
    want.sort();
    assert_eq!(
        got, want,
        "{context}: `{name}`'s edges must be every `requires_dist` line the \
         registry declared for it that this environment admits, rendered \
         against the versions the lock pinned — an edge that vanished between \
         the registry and the lock leaves `mamba sync` building an environment \
         that cannot import, and an edge a marker excluded installs a package \
         nobody asked for\n--- mamba.lock ---\n{body}"
    );
}

// --------------------------------------------------------------------------
// case one
// --------------------------------------------------------------------------

/// The six `requires_dist` lines `requests` 2.31.0 publishes, served verbatim:
/// four unconditional and parenthesized, two parenthesized behind an extra.
/// The lock owes the root plus the four, at the versions the bounds admit, and
/// owes nothing at all for the two the marker excludes.
#[test]
fn lock_carries_every_unconditional_parenthesized_requires_dist_edge() {
    let releases = [
        Release {
            dist: R_ROOT,
            version: R_ROOT_VERSION,
            requires: R_REQUIRES_DIST,
        },
        Release {
            dist: R_CHARSET,
            version: R_CHARSET_V,
            requires: &[],
        },
        Release {
            dist: R_IDNA,
            version: R_IDNA_V,
            requires: &[],
        },
        Release {
            dist: R_URLLIB3,
            version: R_URLLIB3_V,
            requires: &[],
        },
        Release {
            dist: R_CERTIFI,
            version: R_CERTIFI_V,
            requires: &[],
        },
        Release {
            dist: R_PYSOCKS,
            version: R_PYSOCKS_V,
            requires: &[],
        },
        Release {
            dist: R_CHARDET,
            version: R_CHARDET_V,
            requires: &[],
        },
    ];
    let registry = Registry::start(&releases);

    // ---- fixture self-check: all six dependencies are on offer -----------
    // Every name the six lines raise — including the two behind an extra — is
    // published by this registry, inside the bounds its line declares, and
    // resolves when asked for by name. A red here says the pages or the wheels
    // are wrong; a green says the two absences asserted below are a marker
    // policy decision and not a registry that had nothing to serve.
    let offered = Project::start();
    offered.set_dependencies(&[
        R_CHARSET, R_IDNA, R_URLLIB3, R_CERTIFI, R_PYSOCKS, R_CHARDET,
    ]);
    let (offered_body, offered_entries) = offered.lock_ok("fixture self-check", &registry);
    assert_eq!(
        pins_of(&offered_entries),
        vec![
            format!("{R_CERTIFI}=={R_CERTIFI_V}"),
            format!("{R_CHARDET}=={R_CHARDET_V}"),
            format!("{R_CHARSET}=={R_CHARSET_V}"),
            format!("{R_IDNA}=={R_IDNA_V}"),
            format!("{R_PYSOCKS}=={R_PYSOCKS_V}"),
            format!("{R_URLLIB3}=={R_URLLIB3_V}"),
        ],
        "fixture self-check: every one of the six names `{R_ROOT}`'s \
         `requires_dist` raises must resolve against this registry when the \
         manifest asks for it directly\n--- mamba.lock ---\n{offered_body}\n{}",
        registry.seen()
    );

    // ---- the contract ----------------------------------------------------
    let project = Project::start();
    let root_requirement = format!("{R_ROOT}=={R_ROOT_VERSION}");
    project.set_dependencies(&[root_requirement.as_str()]);
    let (body, entries) = project.lock_ok("case one", &registry);

    assert_eq!(
        pins_of(&entries),
        vec![
            format!("{R_CERTIFI}=={R_CERTIFI_V}"),
            format!("{R_CHARSET}=={R_CHARSET_V}"),
            format!("{R_IDNA}=={R_IDNA_V}"),
            format!("{R_ROOT}=={R_ROOT_VERSION}"),
            format!("{R_URLLIB3}=={R_URLLIB3_V}"),
        ],
        "case one: `{R_ROOT}=={R_ROOT_VERSION}` declares `{R_LINE_CHARSET}`, \
         `{R_LINE_IDNA}`, `{R_LINE_URLLIB3}` and `{R_LINE_CERTIFI}` \
         unconditionally, so the lock owes five packages — the root and those \
         four, at the versions their bounds admit. A lock holding only the \
         root dropped every line because the parentheses PEP 508 permits \
         reached the specifier grammar; a lock holding seven read the \
         parentheses but forgot that `{R_LINE_PYSOCKS}` and \
         `{R_LINE_CHARDET}` are gated behind extras this project never \
         activated\n--- mamba.lock ---\n{body}\n{}",
        registry.seen()
    );

    for (name, version) in [
        (R_ROOT, R_ROOT_VERSION),
        (R_CHARSET, R_CHARSET_V),
        (R_IDNA, R_IDNA_V),
        (R_URLLIB3, R_URLLIB3_V),
        (R_CERTIFI, R_CERTIFI_V),
    ] {
        assert_pinned_at("case one", &registry, &entries, &body, name, version);
    }

    assert_edges(
        "case one",
        &entries,
        &body,
        R_ROOT,
        true,
        &[
            format!("{R_CHARSET}=={R_CHARSET_V}"),
            format!("{R_IDNA}=={R_IDNA_V}"),
            format!("{R_URLLIB3}=={R_URLLIB3_V}"),
            format!("{R_CERTIFI}=={R_CERTIFI_V}"),
        ],
    );
    for name in [R_CHARSET, R_IDNA, R_URLLIB3, R_CERTIFI] {
        assert_edges("case one", &entries, &body, name, false, &[]);
    }

    // The two extras-gated names must be absent from the whole file, not
    // merely from the package list: an entry, an edge, or a url naming either
    // one is a package `mamba sync` would install for an extra nobody asked
    // for.
    let lowered = body.to_ascii_lowercase();
    for excluded in [R_PYSOCKS, R_CHARDET] {
        assert!(
            !lowered.contains(excluded),
            "case one: nothing in mamba.lock may name `{excluded}` — the \
             registry publishes it, and the fixture self-check above proves it \
             resolves, but the only line raising it is gated behind an extra \
             this project never activated\n--- mamba.lock ---\n{body}\n{}",
            registry.seen()
        );
    }
}

// --------------------------------------------------------------------------
// case two
// --------------------------------------------------------------------------

/// A parenthesized requirement is a requirement with bounds, not merely a
/// name: `legacylib (<3,>=1.21.1)` picks one of three published releases, and
/// it is neither the newest nor the oldest.
#[test]
fn lock_applies_the_bounds_a_parenthesized_requires_dist_line_declares() {
    let releases = [
        Release {
            dist: L_ROOT,
            version: L_ROOT_VERSION,
            requires: &[L_LINE],
        },
        Release {
            dist: L_LIB,
            version: L_TOO_LOW,
            requires: &[],
        },
        Release {
            dist: L_LIB,
            version: L_MATCH,
            requires: &[],
        },
        Release {
            dist: L_LIB,
            version: L_TOO_HIGH,
            requires: &[],
        },
    ];
    let registry = Registry::start(&releases);

    // ---- fixture self-check: all three releases are on offer -------------
    // Asked for with no constraint at all, `legacylib` resolves to its newest
    // release. A red here says the page or the wheels are wrong; a green says
    // the `2.5` asserted below is what the bounds selected, not what the
    // registry happened to serve.
    let unconstrained = Project::start();
    unconstrained.set_dependencies(&[L_LIB]);
    let (unconstrained_body, unconstrained_entries) =
        unconstrained.lock_ok("fixture self-check", &registry);
    assert_eq!(
        only_entry(&unconstrained_entries, L_LIB, &unconstrained_body).version,
        L_TOO_HIGH,
        "fixture self-check: with no constraint on it, `{L_LIB}` must resolve \
         to `{L_TOO_HIGH}`, the newest of the three releases /simple/{L_LIB}/ \
         advertises\n--- mamba.lock ---\n{unconstrained_body}\n{}",
        registry.seen()
    );

    // ---- the contract ----------------------------------------------------
    let project = Project::start();
    let root_requirement = format!("{L_ROOT}=={L_ROOT_VERSION}");
    project.set_dependencies(&[root_requirement.as_str()]);
    let (body, entries) = project.lock_ok("case two", &registry);

    assert_eq!(
        pins_of(&entries),
        vec![
            format!("{L_ROOT}=={L_ROOT_VERSION}"),
            format!("{L_LIB}=={L_MATCH}"),
        ],
        "case two: `{L_ROOT}=={L_ROOT_VERSION}` declares `{L_LINE}`, so the \
         lock owes two packages and `{L_LIB}` belongs in it at `{L_MATCH}` — \
         `{L_TOO_LOW}` precedes `1.21.1` and `{L_TOO_HIGH}` is excluded by \
         `<3`. A lock without `{L_LIB}` at all dropped the line; one at \
         `{L_TOO_HIGH}` recovered the name and threw the bounds away, and \
         `{L_TOO_HIGH}` is the release the declaration exists to keep \
         out\n--- mamba.lock ---\n{body}\n{}",
        registry.seen()
    );
    assert_pinned_at("case two", &registry, &entries, &body, L_LIB, L_MATCH);
    assert_edges(
        "case two",
        &entries,
        &body,
        L_ROOT,
        true,
        &[format!("{L_LIB}=={L_MATCH}")],
    );
    assert_edges("case two", &entries, &body, L_LIB, false, &[]);
}

// --------------------------------------------------------------------------
// case three
// --------------------------------------------------------------------------

/// A `requires_dist` line no parser can read is registry-supplied input this
/// project cannot honour. Resolving around it produces a lock that silently
/// under-reports its own closure, so the only safe answer is to stop: exit
/// non-zero, say which release and which line, and write nothing.
#[test]
fn lock_refuses_a_release_whose_requires_dist_line_cannot_be_read() {
    let releases = [
        Release {
            dist: B_ROOT,
            version: B_ROOT_VERSION,
            requires: &[B_LINE],
        },
        Release {
            dist: B_CONTROL,
            version: B_CONTROL_VERSION,
            requires: &[],
        },
    ];
    let registry = Registry::start(&releases);

    // ---- fixture self-check: this registry resolves and locks ------------
    // A second distribution on the same server, reached the same way, locks
    // normally. A red here says the registry, the wheels or the manifest are
    // wrong; a green says the refusal asserted below is about the unreadable
    // line and not about a fixture that could not resolve anything.
    let control = Project::start();
    control.set_dependencies(&[B_CONTROL]);
    let (control_body, control_entries) = control.lock_ok("fixture self-check", &registry);
    assert_eq!(
        pins_of(&control_entries),
        vec![format!("{B_CONTROL}=={B_CONTROL_VERSION}")],
        "fixture self-check: `{B_CONTROL}` must resolve and lock against this \
         registry\n--- mamba.lock ---\n{control_body}\n{}",
        registry.seen()
    );

    // ---- the contract ----------------------------------------------------
    let project = Project::start();
    let root_requirement = format!("{B_ROOT}=={B_ROOT_VERSION}");
    project.set_dependencies(&[root_requirement.as_str()]);
    let index_url = registry.index_url();
    let args = ["lock", "--index-url", index_url.as_str()];
    let out = project.run(&args);

    assert!(
        !out.status.success(),
        "case three: `{B_ROOT}=={B_ROOT_VERSION}` declares the one \
         `requires_dist` line `{B_LINE}`, which is not a requirement in any \
         grammar. `mamba lock` must refuse the resolution: a run that drops \
         the line and exits 0 writes a lock claiming a closure it never read, \
         and `mamba sync` then builds an environment from it\n{}\n{}",
        render(&args, &out),
        registry.seen()
    );

    let stderr = stderr_of(&out);
    assert!(
        !stderr.contains("panicked at"),
        "case three: the refusal must be a diagnosed error, not a panic — a \
         crash on registry-supplied input is not a fail-closed path\n{}\n{}",
        render(&args, &out),
        registry.seen()
    );
    assert!(
        names_token(&stderr, B_ROOT),
        "case three: the refusal must name `{B_ROOT}`, the release whose \
         `requires_dist` could not be read; nothing in stderr names it, so the \
         caller cannot tell which release of which graph to look at\n{}\n{}",
        render(&args, &out),
        registry.seen()
    );

    let squeezed = squeezed_stderr(&out);
    for fragment in [B_ROOT_VERSION, B_LINE] {
        let wanted: String = fragment.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            squeezed.contains(&wanted),
            "case three: the refusal must name the version `{B_ROOT_VERSION}` \
             and quote the offending line `{B_LINE}` verbatim — `{fragment}` \
             is absent from stderr. The comparison ignores whitespace, so any \
             wrapping counts; a message naming only the package, or only that \
             a requirement failed to parse, leaves the reader unable to tell \
             which line of which release to report\n{}\n{}",
            render(&args, &out),
            registry.seen()
        );
    }

    let left_behind = project.lock_like_entries();
    assert_eq!(
        left_behind,
        Vec::<String>::new(),
        "case three: a refused resolution must write no lock and leave no \
         fragment of one behind; {} holds {left_behind:?}\n{}",
        project.dir.display(),
        render(&args, &out)
    );
}
