//! Black-box contract: `mamba lock --index-url` must search the whole space
//! its requirements admit before it refuses — including the case where the
//! contradiction lands on a name nothing has decided yet, and the case where
//! the decision that has to move is a *more recent* one than the name the
//! contradiction surfaced on — while still refusing a graph that has no
//! consistent pin set at all.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over throwaway directory trees and
//! reads only what the binary wrote, or refused to write. It never links the
//! resolver, the specifier grammar, or the lock renderer, it asserts nothing
//! about a search order, a stack, a candidate list or any other internal, and
//! it never hand-writes a `mamba.lock` or an index page: every wheel is built
//! by the product's own `WheelBuilder`, and the registry serves PEP 503 anchor
//! pages plus per-version `requires_dist` documents. The wheels themselves
//! declare no `Requires-Dist`, so every dependency edge below exists in
//! exactly one place — the registry's own `/pypi/<name>/<version>/json` — and
//! a lock that knows about an edge is a lock that asked for it. The one thing
//! the case authors is `mamba.toml`'s `dependencies` line, which is user input
//! in this workflow and precisely what `mamba lock` reads.
//!
//! # The three graphs
//!
//! ```text
//!   one:   lateapp 3.0  --requires latelib>=2--> latelib {1.5, 2.5}
//!          lateapp 2.0  --requires latelib>=1--> latelib {1.5, 2.5}
//!          roots: lateapp>=2, latelib<2
//!
//!   two:   chainapp 3.0 --requires chainmid>=2--> chainmid {1.0, 2.0}
//!          chainapp 2.0 --requires chainmid>=1--> chainmid {1.0, 2.0}
//!          chainmid 2.0 --requires chainlib>=2--> chainlib {1.5, 2.5}
//!          chainmid 1.0 --requires chainlib>=1--> chainlib {1.5, 2.5}
//!          roots: chainapp>=1, chainlib<2
//!
//!   three: deadapp 1.0  --requires deadlib>=2--> deadlib {1.5, 2.5}
//!          roots: deadapp>=1, deadlib<2
//! ```
//!
//! One has exactly one consistent pin set: `lateapp==3.0` needs `latelib>=2`,
//! which the root's `latelib<2` forbids outright, so `lateapp` has to be
//! `2.0`, whose `latelib>=1` leaves `latelib==1.5`. Two has exactly one as
//! well: `chainlib<2` pins `chainlib==1.5`, which `chainmid==2.0` cannot have,
//! so `chainmid==1.0` — and `chainapp==3.0` demands `chainmid>=2`, so
//! `chainapp` has to be `2.0`. Three has none: `deadapp` is published once,
//! its only release requires `deadlib>=2`, and the root requires `deadlib<2`.
//!
//! The names are chosen so that the constrained name sorts *after* the
//! dependant that raises the offending requirement on it — `lateapp` before
//! `latelib`, `chainapp` before `chainlib` before `chainmid` — because that is
//! the shape in which the incompleteness shows.
//!
//! Each graph is served by its own registry with its own distribution names,
//! so no test can inherit another's cached metadata or answer.
//!
//! # Why today's tree cannot pass
//!
//! Measured against the binary built from `cbdcecfce7`:
//!
//! - One exits 1 and writes no lock, in *both* manifest orders of its two
//!   roots, with stderr `Error: resolution failed: resolution failed
//!   (EmptyIntersection): no version of latelib satisfies 2 specifier(s)`. The
//!   contradiction is between two requirements raised on `latelib`, which is
//!   not decided yet, so it arrives as "this name has no candidate" rather
//!   than as a conflict with a recorded pin — and that answer ends the search
//!   although `lateapp` still had `2.0` to try.
//! - Two exits 1 and writes no lock, with stderr `Error: resolution failed:
//!   resolution failed (NoCompatibleVersion): conflicting requirements on
//!   chainlib: `chainlib<2` decided chainlib==1.5 but a later requirement
//!   needs `chainlib>=2`, which that version does not satisfy`. Here the
//!   conflict *is* seen on a decided name, but the decision that has to move
//!   is the newer one taken above it: `chainmid` still had `1.0` to try, and
//!   it is discarded rather than advanced.
//! - Three exits 1 and writes no lock. That is the answer this file demands
//!   too: it is the guard that neither of the two above may be bought by
//!   dropping, widening or ignoring a declared bound. It is green today and
//!   has to stay green.
//!
//! Each test therefore opens with fixture self-checks that pass on that same
//! tree — the eager pin set each answer must back off from, and the lower
//! release each upper bound selects on its own — so the red the first two
//! report is about the search and not about the registry, the wheels or the
//! manifest.
//!
//! # Facets
//!
//! - **Behavior**: `mamba lock --index-url` exits 0 on a graph that has a
//!   consistent pin set, pins every name at the version every requirement
//!   raised on it admits, records each edge against that pin, gives the same
//!   answer whichever order the manifest lists its roots in, and is
//!   byte-stable on replay. A graph with no consistent pin set still exits
//!   non-zero, still names the contested package, and still writes nothing.
//! - **Security (a declared bound is a boundary, and the refusal stays a
//!   refusal)**: `latelib<2`, `chainlib<2` and `deadlib<2` are how a user
//!   keeps a known-bad release out of the environment `mamba sync` later
//!   builds from the lock. Searching further must never mean bounding less:
//!   every test asserts the concrete pinned version rather than a successful
//!   exit, and asserts that each entry's `url` and `sha256` name the file the
//!   registry advertised *for that very version*, so a lock that says `1.5`
//!   while pointing at the `2.5` bytes, or that carries an empty digest for
//!   `mamba sync` to skip, fails here. Test three is the fail-closed control:
//!   an unsatisfiable graph must still be refused outright — no lock, no
//!   fragment of one, no warning-with-exit-0, no panic — so "solve everything"
//!   cannot pass this file either.
//! - **Performance**: this work item names no budget, so the case asserts no
//!   timing, no request count and no concurrency. A complete search is allowed
//!   to be slower than an incomplete one.
//!
//! # Hermeticity
//!
//! No network, ever: every wheel is built in-process and the only package
//! source is an in-process `wiremock` server on loopback. `HOME` and
//! `MAMBA_CACHE_DIR` are pinned per project into its own temp tree, so no two
//! projects can read each other's cached metadata; `MAMBA_FROZEN_INDEX`,
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
// graph one: the contradiction lands on a name nothing has decided yet
// --------------------------------------------------------------------------
const G1_APP: &str = "lateapp";
const G1_LIB: &str = "latelib";
/// The manifest's two roots. `lateapp>=2` admits both `lateapp` releases;
/// `latelib<2` admits only the lower `latelib`.
const G1_APP_ROOT: &str = "lateapp>=2";
const G1_LIB_ROOT: &str = "latelib<2";
/// The newer `lateapp` demands a `latelib` the root forbids, so this release
/// is the one the answer has to give up.
const G1_APP_HIGH: &str = "3.0";
const G1_APP_HIGH_REQUIRES: &str = "latelib>=2";
/// The older `lateapp`, whose edge the root's upper bound can live with.
const G1_APP_LOW: &str = "2.0";
const G1_APP_LOW_REQUIRES: &str = "latelib>=1";
const G1_LIB_LOW: &str = "1.5";
const G1_LIB_HIGH: &str = "2.5";

// --------------------------------------------------------------------------
// graph two: the decision that has to move sits above the contested name
// --------------------------------------------------------------------------
const G2_APP: &str = "chainapp";
const G2_MID: &str = "chainmid";
const G2_LIB: &str = "chainlib";
const G2_APP_ROOT: &str = "chainapp>=1";
const G2_LIB_ROOT: &str = "chainlib<2";
const G2_APP_HIGH: &str = "3.0";
const G2_APP_HIGH_REQUIRES: &str = "chainmid>=2";
const G2_APP_LOW: &str = "2.0";
const G2_APP_LOW_REQUIRES: &str = "chainmid>=1";
const G2_MID_HIGH: &str = "2.0";
const G2_MID_HIGH_REQUIRES: &str = "chainlib>=2";
const G2_MID_LOW: &str = "1.0";
const G2_MID_LOW_REQUIRES: &str = "chainlib>=1";
const G2_LIB_LOW: &str = "1.5";
const G2_LIB_HIGH: &str = "2.5";

// --------------------------------------------------------------------------
// graph three: no consistent pin set exists, and none may be invented
// --------------------------------------------------------------------------
const G3_APP: &str = "deadapp";
const G3_LIB: &str = "deadlib";
const G3_APP_ROOT: &str = "deadapp>=1";
const G3_LIB_ROOT: &str = "deadlib<2";
const G3_APP_ONLY: &str = "1.0";
const G3_APP_REQUIRES: &str = "deadlib>=2";
const G3_LIB_LOW: &str = "1.5";
const G3_LIB_HIGH: &str = "2.5";

/// The word a refusal spells, whitespace-free and lowercased. A resolution
/// that succeeds while still reporting a conflict is not a resolution.
const CONFLICT_WORD: &str = "conflictingrequirements";

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

/// `stderr` with every whitespace byte removed and lowercased, so a message
/// counts however it happened to wrap.
fn squeezed_stderr(out: &Output) -> String {
    stderr_of(out)
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// `needle` occurs in `haystack` as a whole word, so a message naming
/// `deadlibrary` does not count as naming `deadlib`.
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
    /// answer, and return the lock it wrote, parsed. Every test reads its
    /// answer through here, so no test can accept a lock the binary refused.
    fn lock_ok(&self, context: &str, registry: &Registry) -> (String, Vec<LockEntry>) {
        let index_url = registry.index_url();
        let args = ["lock", "--index-url", index_url.as_str()];
        let out = self.run(&args);
        assert!(
            out.status.success(),
            "{context}: `mamba lock --index-url` must resolve this graph — it \
             has an assignment in which every requirement raised on every name \
             holds, so there is nothing here to refuse\n{}\n{}",
            render(&args, &out),
            registry.seen()
        );
        assert!(
            !stderr_of(&out).contains("panicked at"),
            "{context}: the resolution must be a resolution, not a crash\n{}\n{}",
            render(&args, &out),
            registry.seen()
        );
        assert!(
            !squeezed_stderr(&out).contains(CONFLICT_WORD),
            "{context}: this graph has a consistent pin set, so nothing may \
             report a conflict on it — an exit-0 run that still prints one \
             leaves the caller unable to tell a pin set from a \
             refusal\n{}\n{}",
            render(&args, &out),
            registry.seen()
        );
        let body = self.lock_body();
        let entries = parse_lock(&body);
        (body, entries)
    }
}

/// Scaffold a project whose manifest declares exactly `roots`, lock it against
/// `registry`, and hand back the lock it wrote. Each call gets its own project
/// and its own cache, so nothing carries over between the orders a test tries.
fn lock_roots(context: &str, registry: &Registry, roots: &[&str]) -> (String, Vec<LockEntry>) {
    let project = Project::start();
    project.set_dependencies(roots);
    project.lock_ok(context, registry)
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
fn build_wheel_file(out_dir: &Path, dist: &str, version: &str) -> PathBuf {
    let filename = compose_filename(dist, version, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-lock-search-is-complete");
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
/// registry advertised for that very version — not the release the
/// requirements excluded, and not nothing at all.
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
        "{context}: `{name}` must be pinned at {version}, the version every \
         requirement raised on it admits\n--- mamba.lock ---\n{body}\n{}",
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
        "{context}: `{name}` must be recorded with `direct = {direct}`\n\
         --- mamba.lock ---\n{body}"
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

// --------------------------------------------------------------------------
// graph one
// --------------------------------------------------------------------------

/// Two roots, and the contradiction between them lands on `latelib` — a name
/// nothing has decided yet at the moment it is discovered. `lateapp` still has
/// an older release whose edge the upper bound admits, so the graph has an
/// answer, and reaching it means giving up the first pick of a *different*
/// name than the one the contradiction was found on.
///
/// The manifest's roots are tried in both orders: which of the two the user
/// happened to list first is not a fact about the graph, so it may not be a
/// fact about the answer either.
#[test]
fn lock_solves_a_conflict_raised_on_a_name_nothing_has_decided_yet() {
    let releases = [
        Release {
            dist: G1_APP,
            version: G1_APP_HIGH,
            requires: &[G1_APP_HIGH_REQUIRES],
        },
        Release {
            dist: G1_APP,
            version: G1_APP_LOW,
            requires: &[G1_APP_LOW_REQUIRES],
        },
        Release {
            dist: G1_LIB,
            version: G1_LIB_LOW,
            requires: &[],
        },
        Release {
            dist: G1_LIB,
            version: G1_LIB_HIGH,
            requires: &[],
        },
    ];
    let registry = Registry::start(&releases);

    // ---- fixture self-check: the pin set the answer must back off from ----
    // `lateapp>=2` on its own resolves against this same registry, these same
    // pages and these same wheels, and takes the newest release plus the
    // `latelib` its edge then demands. A red here says the fixture is wrong; a
    // green says the pins asserted below are ones the resolver had to come
    // down to, not the only ones on offer.
    let (eager_body, eager_entries) =
        lock_roots("fixture self-check, eager pin set", &registry, &[G1_APP_ROOT]);
    assert_eq!(
        pins_of(&eager_entries),
        vec![
            format!("{G1_APP}=={G1_APP_HIGH}"),
            format!("{G1_LIB}=={G1_LIB_HIGH}"),
        ],
        "fixture self-check: `{G1_APP_ROOT}` alone must take \
         `{G1_APP}=={G1_APP_HIGH}` and, through its `{G1_APP_HIGH_REQUIRES}`, \
         `{G1_LIB}=={G1_LIB_HIGH}` — the pair the second root's `{G1_LIB_ROOT}` \
         then rules out\n--- mamba.lock ---\n{eager_body}\n{}",
        registry.seen()
    );

    // ---- fixture self-check: the bound alone selects the lower release ----
    // A red here says /simple/latelib/ does not advertise 1.5, or that `<2`
    // cannot select it — either way the contract below would be measuring the
    // registry rather than the search.
    let (bounded_body, bounded_entries) = lock_roots(
        "fixture self-check, the bound alone",
        &registry,
        &[G1_LIB_ROOT],
    );
    assert_eq!(
        pins_of(&bounded_entries),
        vec![format!("{G1_LIB}=={G1_LIB_LOW}")],
        "fixture self-check: `{G1_LIB_ROOT}` alone must resolve to \
         `{G1_LIB}=={G1_LIB_LOW}`, the one release /simple/{G1_LIB}/ advertises \
         below 2\n--- mamba.lock ---\n{bounded_body}\n{}",
        registry.seen()
    );

    // ---- the contract, in both manifest orders ---------------------------
    let orders: [(&str, [&str; 2]); 2] = [
        ("graph one, roots as declared", [G1_APP_ROOT, G1_LIB_ROOT]),
        ("graph one, roots reversed", [G1_LIB_ROOT, G1_APP_ROOT]),
    ];
    let mut locks: Vec<(&str, String)> = Vec::new();
    for (context, roots) in orders {
        let (body, entries) = lock_roots(context, &registry, &roots);

        assert_eq!(
            pins_of(&entries),
            vec![
                format!("{G1_APP}=={G1_APP_LOW}"),
                format!("{G1_LIB}=={G1_LIB_LOW}"),
            ],
            "{context}: `{G1_LIB_ROOT}` leaves `{G1_LIB}=={G1_LIB_LOW}` as the \
             only admissible `{G1_LIB}`, and `{G1_APP}=={G1_APP_HIGH}` requires \
             `{G1_APP_HIGH_REQUIRES}`, which that release fails — so `{G1_APP}` \
             has to come down to `{G1_APP_LOW}`, whose `{G1_APP_LOW_REQUIRES}` \
             holds. The contradiction shows up on `{G1_LIB}` before anything \
             has decided it, and abandoning the search there gives up a pin set \
             that exists\n--- mamba.lock ---\n{body}\n{}",
            registry.seen()
        );
        assert_pinned_at(context, &registry, &entries, &body, G1_APP, G1_APP_LOW);
        assert_pinned_at(context, &registry, &entries, &body, G1_LIB, G1_LIB_LOW);

        // Both names are manifest roots, so both are direct; the edge is
        // rendered against the version the lock actually pinned.
        assert_edges(
            context,
            &entries,
            &body,
            G1_APP,
            true,
            &[format!("{G1_LIB}=={G1_LIB_LOW}")],
        );
        assert_edges(context, &entries, &body, G1_LIB, true, &[]);

        locks.push((context, body));
    }

    let (first_context, first_lock) = &locks[0];
    let (second_context, second_lock) = &locks[1];
    assert_eq!(
        second_lock, first_lock,
        "graph one: the two runs differ only in the order `mamba.toml` lists \
         its two roots in, which is not a fact about the graph — so the locks \
         must be byte-identical. `{second_context}` disagrees with \
         `{first_context}`\n--- {first_context} ---\n{first_lock}\n\
         --- {second_context} ---\n{second_lock}"
    );

    // A search that reaches its answer by a route it does not record would be
    // free to reach a different one next time. The lock is the product's
    // output, so the bytes are part of the contract too.
    let (replayed, _) = lock_roots("graph one, replayed", &registry, &[G1_APP_ROOT, G1_LIB_ROOT]);
    assert_eq!(
        replayed, *first_lock,
        "graph one: re-locking the same manifest against the same registry \
         must produce the same bytes — a resolution whose answer depends on \
         search order is not a pin set anyone can rely on\n{}",
        registry.seen()
    );
}

// --------------------------------------------------------------------------
// graph two
// --------------------------------------------------------------------------

/// The conflict surfaces on `chainlib`, which has nothing left to try, and the
/// decision that has to move is `chainmid` — taken *after* `chainlib`, and
/// still holding a release whose edge the bound admits. Discarding it and
/// reaching further down instead retakes `chainmid` at the very release that
/// raised the offending requirement, so the graph is refused although its one
/// consistent pin set is a single step away.
#[test]
fn lock_moves_the_most_recent_decision_when_the_contested_name_is_exhausted() {
    let releases = [
        Release {
            dist: G2_APP,
            version: G2_APP_HIGH,
            requires: &[G2_APP_HIGH_REQUIRES],
        },
        Release {
            dist: G2_APP,
            version: G2_APP_LOW,
            requires: &[G2_APP_LOW_REQUIRES],
        },
        Release {
            dist: G2_MID,
            version: G2_MID_HIGH,
            requires: &[G2_MID_HIGH_REQUIRES],
        },
        Release {
            dist: G2_MID,
            version: G2_MID_LOW,
            requires: &[G2_MID_LOW_REQUIRES],
        },
        Release {
            dist: G2_LIB,
            version: G2_LIB_LOW,
            requires: &[],
        },
        Release {
            dist: G2_LIB,
            version: G2_LIB_HIGH,
            requires: &[],
        },
    ];
    let registry = Registry::start(&releases);

    // ---- fixture self-check: the eager chain, top to bottom --------------
    let (eager_body, eager_entries) =
        lock_roots("fixture self-check, eager pin set", &registry, &[G2_APP_ROOT]);
    assert_eq!(
        pins_of(&eager_entries),
        vec![
            format!("{G2_APP}=={G2_APP_HIGH}"),
            format!("{G2_LIB}=={G2_LIB_HIGH}"),
            format!("{G2_MID}=={G2_MID_HIGH}"),
        ],
        "fixture self-check: `{G2_APP_ROOT}` alone must take \
         `{G2_APP}=={G2_APP_HIGH}`, through `{G2_APP_HIGH_REQUIRES}` reach \
         `{G2_MID}=={G2_MID_HIGH}`, and through `{G2_MID_HIGH_REQUIRES}` reach \
         `{G2_LIB}=={G2_LIB_HIGH}` — the chain the second root's \
         `{G2_LIB_ROOT}` then rules out\n--- mamba.lock ---\n{eager_body}\n{}",
        registry.seen()
    );

    // ---- fixture self-check: the bound alone selects the lower release ----
    let (bounded_body, bounded_entries) = lock_roots(
        "fixture self-check, the bound alone",
        &registry,
        &[G2_LIB_ROOT],
    );
    assert_eq!(
        pins_of(&bounded_entries),
        vec![format!("{G2_LIB}=={G2_LIB_LOW}")],
        "fixture self-check: `{G2_LIB_ROOT}` alone must resolve to \
         `{G2_LIB}=={G2_LIB_LOW}`, the one release /simple/{G2_LIB}/ advertises \
         below 2\n--- mamba.lock ---\n{bounded_body}\n{}",
        registry.seen()
    );

    // ---- the contract ----------------------------------------------------
    let context = "graph two";
    let (body, entries) = lock_roots(context, &registry, &[G2_APP_ROOT, G2_LIB_ROOT]);

    assert_eq!(
        pins_of(&entries),
        vec![
            format!("{G2_APP}=={G2_APP_LOW}"),
            format!("{G2_LIB}=={G2_LIB_LOW}"),
            format!("{G2_MID}=={G2_MID_LOW}"),
        ],
        "{context}: `{G2_LIB_ROOT}` leaves `{G2_LIB}=={G2_LIB_LOW}`, which \
         `{G2_MID}=={G2_MID_HIGH}` cannot have — its `{G2_MID_HIGH_REQUIRES}` \
         fails — so `{G2_MID}` has to be `{G2_MID_LOW}`; and \
         `{G2_APP}=={G2_APP_HIGH}` demands `{G2_APP_HIGH_REQUIRES}`, which \
         `{G2_MID}=={G2_MID_LOW}` fails, so `{G2_APP}` has to be \
         `{G2_APP_LOW}`. That is the whole of the graph's one consistent pin \
         set; refusing it, or keeping either newer release, abandons \
         it\n--- mamba.lock ---\n{body}\n{}",
        registry.seen()
    );
    assert_pinned_at(context, &registry, &entries, &body, G2_APP, G2_APP_LOW);
    assert_pinned_at(context, &registry, &entries, &body, G2_MID, G2_MID_LOW);
    assert_pinned_at(context, &registry, &entries, &body, G2_LIB, G2_LIB_LOW);

    assert_edges(
        context,
        &entries,
        &body,
        G2_APP,
        true,
        &[format!("{G2_MID}=={G2_MID_LOW}")],
    );
    assert_edges(
        context,
        &entries,
        &body,
        G2_MID,
        false,
        &[format!("{G2_LIB}=={G2_LIB_LOW}")],
    );
    assert_edges(context, &entries, &body, G2_LIB, true, &[]);

    let (replayed, _) = lock_roots(
        "graph two, replayed",
        &registry,
        &[G2_APP_ROOT, G2_LIB_ROOT],
    );
    assert_eq!(
        replayed, body,
        "{context}: re-locking the same manifest against the same registry \
         must produce the same bytes\n{}",
        registry.seen()
    );
}

// --------------------------------------------------------------------------
// graph three
// --------------------------------------------------------------------------

/// The control, and the reason the two graphs above may not be bought by
/// searching less carefully: `deadapp` is published once, its only release
/// requires `deadlib>=2`, and the manifest requires `deadlib<2`. There is no
/// assignment, so there is nothing to find — however long the search looks.
/// The refusal has to stay a refusal: a non-zero exit, a named package, and
/// not one byte of a lock.
#[test]
fn lock_still_refuses_a_graph_with_no_consistent_pin_set() {
    let releases = [
        Release {
            dist: G3_APP,
            version: G3_APP_ONLY,
            requires: &[G3_APP_REQUIRES],
        },
        Release {
            dist: G3_LIB,
            version: G3_LIB_LOW,
            requires: &[],
        },
        Release {
            dist: G3_LIB,
            version: G3_LIB_HIGH,
            requires: &[],
        },
    ];
    let registry = Registry::start(&releases);
    let index_url = registry.index_url();
    let lock_args = ["lock", "--index-url", index_url.as_str()];

    // ---- fixture self-check: each root resolves on its own ---------------
    // A red in either step says the registry, the pages or the wheels are
    // wrong — so the refusal below would be a refusal of the fixture rather
    // than of the graph.
    let (app_body, app_entries) = lock_roots(
        "fixture self-check, the root alone",
        &registry,
        &[G3_APP_ROOT],
    );
    assert_eq!(
        pins_of(&app_entries),
        vec![
            format!("{G3_APP}=={G3_APP_ONLY}"),
            format!("{G3_LIB}=={G3_LIB_HIGH}"),
        ],
        "fixture self-check: `{G3_APP_ROOT}` alone must resolve to \
         `{G3_APP}=={G3_APP_ONLY}` and, through its `{G3_APP_REQUIRES}`, \
         `{G3_LIB}=={G3_LIB_HIGH}`\n--- mamba.lock ---\n{app_body}\n{}",
        registry.seen()
    );

    let (lib_body, lib_entries) = lock_roots(
        "fixture self-check, the bound alone",
        &registry,
        &[G3_LIB_ROOT],
    );
    assert_eq!(
        pins_of(&lib_entries),
        vec![format!("{G3_LIB}=={G3_LIB_LOW}")],
        "fixture self-check: `{G3_LIB_ROOT}` alone must resolve to \
         `{G3_LIB}=={G3_LIB_LOW}`, so the release below 2 is genuinely on \
         offer and the refusal below is about the graph, not about a missing \
         file\n--- mamba.lock ---\n{lib_body}\n{}",
        registry.seen()
    );

    // ---- the contract ----------------------------------------------------
    let project = Project::start();
    project.set_dependencies(&[G3_APP_ROOT, G3_LIB_ROOT]);
    let out = project.run(&lock_args);
    let stderr = stderr_of(&out);

    assert!(
        !out.status.success(),
        "graph three: `mamba lock --index-url` must refuse a graph in which no \
         `{G3_LIB}` satisfies both `{G3_APP_REQUIRES}` (declared by \
         `{G3_APP}=={G3_APP_ONLY}`, its only release) and `{G3_LIB_ROOT}` \
         (declared by the manifest); searching harder finds nothing here, and a \
         lock written anyway is a lock that violates one of them\n{}\n{}",
        render(&lock_args, &out),
        registry.seen()
    );
    assert!(
        !stderr.contains("panicked at"),
        "graph three: the refusal must be a diagnosed resolution error, not a \
         panic — a crash is not a fail-closed path\n{}\n{}",
        render(&lock_args, &out),
        registry.seen()
    );
    assert!(
        names_token(&stderr, G3_LIB),
        "graph three: the refusal must name `{G3_LIB}`, the package whose \
         requirements cannot be reconciled; nothing in stderr names \
         it\n{}\n{}",
        render(&lock_args, &out),
        registry.seen()
    );
    assert_eq!(
        project.lock_like_entries(),
        Vec::<String>::new(),
        "graph three: a refused resolution must write no lock and leave no \
         fragment of one behind; {} holds {:?}\n{}",
        project.dir.display(),
        project.lock_like_entries(),
        render(&lock_args, &out)
    );
}
