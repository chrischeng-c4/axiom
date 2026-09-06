//! Black-box contract: `mamba lock` must pin the version every requirement
//! admits — backing off a first pick that a later-arriving requirement
//! contradicts — instead of refusing a graph that has a solution, and must
//! honour a `==X.*` edge instead of dropping it without a word.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over throwaway directory trees and
//! reads only what the binary wrote. It never links the resolver, the
//! specifier grammar, or the lock renderer, and it never hand-writes a
//! `mamba.lock` or an index page: every wheel is built by the product's own
//! `WheelBuilder`, and the registry serves PEP 503 anchor pages plus
//! per-version `requires_dist` documents. The wheels themselves declare no
//! `Requires-Dist`, so every dependency edge below exists in exactly one
//! place — the registry's own `/pypi/<name>/<version>/json` — and a lock that
//! knows about an edge is a lock that asked for it. The one thing the case
//! authors is `mamba.toml`'s `dependencies` line, which is user input in this
//! workflow and precisely what `mamba lock` reads.
//!
//! # The three graphs
//!
//! Each has exactly one consistent solution, and in each that solution is not
//! what an eager highest-first walk commits to first.
//!
//! ```text
//!   one:   sharedapp 1.0   --requires sharedlib>=1--> sharedlib {1.0, 3.0}
//!          sharedother 1.0 --requires sharedlib<2 --> sharedlib {1.0, 3.0}
//!
//!   two:   deepapp 1.0     --requires deepmid>=1 --> deepmid {1.0, 2.0}
//!          deepmid 2.0     --requires deeplib>=3 --> deeplib {1.0, 3.0}
//!          deepmid 1.0     --requires deeplib>=1 --> deeplib {1.0, 3.0}
//!          deepother 1.0   --requires deeplib<2  --> deeplib {1.0, 3.0}
//!
//!   three: wildapp 1.0     --requires wildlib==1.*--> wildlib {1.0, 1.5, 2.0}
//! ```
//!
//! One: `sharedlib==1.0` is the only release satisfying both `>=1` and `<2`,
//! and it is the *lower* of the two the index advertises. Two: `deepother`
//! forces `deeplib<2`, which no `deeplib` above `1.0` meets, so `deepmid` has
//! to come down to `1.0` — the conflict is settled one decision below the name
//! it was detected on. Three: `1.*` admits `1.0` and `1.5` and excludes `2.0`,
//! so the pin is `1.5` — neither the newest release nor the oldest.
//!
//! Each graph is served by its own registry with its own distribution names,
//! so no test can inherit another's cached metadata or answer.
//!
//! # Why today's tree cannot pass
//!
//! Measured against the binary built from `81588f3e9e`:
//!
//! - One and two exit 1 with `resolution failed (NoCompatibleVersion):
//!   conflicting requirements on …` and write no `mamba.lock`. The resolver
//!   decides each name once and, when a later requirement contradicts the
//!   recorded version, refuses — nothing is undone, so a graph whose only
//!   solution needs a lower version of a decided name is refused rather than
//!   solved.
//! - Three exits 0 and writes a lock whose only package is `wildapp==1.0`
//!   carrying `dependencies = []`: `==1.*` is not a version the specifier
//!   parser accepts, and a `requires_dist` line that fails to parse is
//!   dropped, so the edge — and with it the whole of `wildlib` — vanishes from
//!   the lock silently.
//!
//! Each test therefore opens with a fixture self-check that passes on that
//! same tree, so the red it reports is about the resolution and not about the
//! registry, the wheels or the manifest: one and two lock the eager pick the
//! answer must back off from, and three locks `wildlib` alone at its newest
//! release.
//!
//! # Facets
//!
//! - **Behavior**: `mamba lock --index-url` exits 0 on a graph that has a
//!   consistent solution, pins every name at the version every requirement
//!   raised on it admits, records each edge against that pin, and is
//!   byte-stable on replay.
//! - **Security (a declared bound is a boundary)**: an upper bound — `<2`, or
//!   the `2.0`-excluding `==1.*` — is how a user keeps a known-bad release out
//!   of the environment `mamba sync` later builds from the lock. "Solving" a
//!   graph by ignoring the bound would install the very release it excludes,
//!   so each test asserts the concrete version rather than merely a successful
//!   exit, and asserts that the entry's `url` and `sha256` name the file the
//!   registry advertised *for that version* — a lock that says `1.0` while
//!   pointing at the `3.0` bytes, or that carries an empty digest for `mamba
//!   sync` to skip, fails here. Nothing in this file weakens the refusal of a
//!   genuinely unsatisfiable graph: that contract is
//!   `pkgmgr_lock_conflict_refused`, and a resolution that succeeds while
//!   still reporting a conflict is refused below as well.
//! - **Performance**: this work item names no budget, so the case asserts no
//!   timing, no request count and no concurrency.
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
// graph one: two roots, one shared name, and the lower release is the answer
// --------------------------------------------------------------------------
const G1_ROOT: &str = "sharedapp";
const G1_OTHER: &str = "sharedother";
const G1_SHARED: &str = "sharedlib";
/// `sharedapp`'s edge. Both releases satisfy it, so on its own it decides the
/// newer one.
const G1_ROOT_REQUIRES: &str = "sharedlib>=1";
/// `sharedother`'s edge, which is raised after `sharedlib` has been decided
/// and which only the lower release satisfies.
const G1_OTHER_REQUIRES: &str = "sharedlib<2";
const G1_LOW: &str = "1.0";
const G1_HIGH: &str = "3.0";

// --------------------------------------------------------------------------
// graph two: the conflict is settled one decision below where it was found
// --------------------------------------------------------------------------
const G2_ROOT: &str = "deepapp";
const G2_OTHER: &str = "deepother";
const G2_MID: &str = "deepmid";
const G2_SHARED: &str = "deeplib";
const G2_ROOT_REQUIRES: &str = "deepmid>=1";
/// The newer `deepmid` drags in a `deeplib` the other root forbids, so this is
/// the release the answer has to give up.
const G2_MID_HIGH_REQUIRES: &str = "deeplib>=3";
const G2_MID_LOW_REQUIRES: &str = "deeplib>=1";
const G2_OTHER_REQUIRES: &str = "deeplib<2";
const G2_MID_LOW: &str = "1.0";
const G2_MID_HIGH: &str = "2.0";
const G2_LOW: &str = "1.0";
const G2_HIGH: &str = "3.0";

// --------------------------------------------------------------------------
// graph three: a wildcard edge, whose answer is neither newest nor oldest
// --------------------------------------------------------------------------
const G3_ROOT: &str = "wildapp";
const G3_SHARED: &str = "wildlib";
const G3_ROOT_REQUIRES: &str = "wildlib==1.*";
const G3_LOW: &str = "1.0";
const G3_MATCH: &str = "1.5";
const G3_EXCLUDED: &str = "2.0";

/// Every root is published once, at this version: the graphs differ in their
/// edges, not in how many releases their roots have.
const ROOT_VERSION: &str = "1.0";

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

    /// Lock against `registry`, require the run to have stood behind its
    /// answer, and return the lock it wrote, parsed. Every test reads its
    /// answer through here, so no test can accept a lock the binary refused.
    fn lock_ok(&self, context: &str, registry: &Registry) -> (String, Vec<LockEntry>) {
        let index_url = registry.index_url();
        let args = ["lock", "--index-url", index_url.as_str()];
        let out = self.run(&args);
        assert!(
            out.status.success(),
            "{context}: `mamba lock --index-url` must resolve this graph — \
             every name in it has a release satisfying every requirement raised \
             on it, so there is nothing here to refuse\n{}\n{}",
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
            "{context}: this graph has a consistent solution, so nothing may \
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
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-lock-backtracks");
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

/// Two roots reach one name. The first requirement admits both releases and
/// takes the newer; the second admits only the older. The graph has exactly
/// one consistent solution, and reaching it means giving up the first pick.
#[test]
fn lock_backtracks_off_a_first_pick_a_later_root_forbids() {
    let releases = [
        Release {
            dist: G1_ROOT,
            version: ROOT_VERSION,
            requires: &[G1_ROOT_REQUIRES],
        },
        Release {
            dist: G1_OTHER,
            version: ROOT_VERSION,
            requires: &[G1_OTHER_REQUIRES],
        },
        Release {
            dist: G1_SHARED,
            version: G1_LOW,
            requires: &[],
        },
        Release {
            dist: G1_SHARED,
            version: G1_HIGH,
            requires: &[],
        },
    ];
    let registry = Registry::start(&releases);

    // ---- fixture self-check: the pick the answer must back off from -------
    // `sharedapp` alone resolves against this same registry, this same page
    // and these same wheels, and `sharedlib>=1` takes the newer release. A red
    // here says the fixture is wrong; a green says the `1.0` asserted below is
    // a version the resolver had to come back down to, not the only one on
    // offer.
    let eager = Project::start();
    eager.set_dependencies(&[G1_ROOT]);
    let (eager_body, eager_entries) = eager.lock_ok("fixture self-check", &registry);
    assert_eq!(
        only_entry(&eager_entries, G1_SHARED, &eager_body).version,
        G1_HIGH,
        "fixture self-check: `{G1_ROOT_REQUIRES}` alone must select \
         `{G1_SHARED}=={G1_HIGH}` from the two releases /simple/{G1_SHARED}/ \
         advertises — that pick is what `{G1_OTHER}`'s `{G1_OTHER_REQUIRES}` \
         then forbids\n--- mamba.lock ---\n{eager_body}\n{}",
        registry.seen()
    );

    // ---- the contract ----------------------------------------------------
    let project = Project::start();
    project.set_dependencies(&[G1_ROOT, G1_OTHER]);
    let (body, entries) = project.lock_ok("graph one", &registry);

    assert_eq!(
        pins_of(&entries),
        vec![
            format!("{G1_ROOT}=={ROOT_VERSION}"),
            format!("{G1_SHARED}=={G1_LOW}"),
            format!("{G1_OTHER}=={ROOT_VERSION}"),
        ],
        "graph one: `{G1_ROOT}` requires `{G1_ROOT_REQUIRES}` and `{G1_OTHER}` \
         requires `{G1_OTHER_REQUIRES}`, so `{G1_SHARED}=={G1_LOW}` is the one \
         release both admit — the newer `{G1_HIGH}` satisfies only the first, \
         and refusing the graph outright abandons a solution that \
         exists\n--- mamba.lock ---\n{body}\n{}",
        registry.seen()
    );
    assert_pinned_at("graph one", &registry, &entries, &body, G1_SHARED, G1_LOW);

    let shared_edge = format!("{G1_SHARED}=={G1_LOW}");
    let edges = [shared_edge];
    assert_edges("graph one", &entries, &body, G1_ROOT, true, &edges);
    assert_edges("graph one", &entries, &body, G1_OTHER, true, &edges);
    assert_edges("graph one", &entries, &body, G1_SHARED, false, &[]);

    // A search that reaches its answer by a route it does not record would be
    // free to reach a different one next time. The lock is the product's
    // output, so the bytes are part of the contract too.
    let (replayed, _) = project.lock_ok("graph one, replayed", &registry);
    assert_eq!(
        replayed, body,
        "graph one: re-locking the same manifest against the same registry \
         must produce the same bytes — a resolution whose answer depends on \
         search order is not a pin set anyone can rely on\n{}",
        registry.seen()
    );
}

// --------------------------------------------------------------------------
// graph two
// --------------------------------------------------------------------------

/// The name the conflict is detected on has no release satisfying both
/// requirements: the decision that has to move is the one below it — an
/// intermediate package whose own newer release is what raised the impossible
/// requirement.
#[test]
fn lock_backtracks_past_an_intermediate_package_to_the_version_that_holds() {
    let releases = [
        Release {
            dist: G2_ROOT,
            version: ROOT_VERSION,
            requires: &[G2_ROOT_REQUIRES],
        },
        Release {
            dist: G2_OTHER,
            version: ROOT_VERSION,
            requires: &[G2_OTHER_REQUIRES],
        },
        Release {
            dist: G2_MID,
            version: G2_MID_LOW,
            requires: &[G2_MID_LOW_REQUIRES],
        },
        Release {
            dist: G2_MID,
            version: G2_MID_HIGH,
            requires: &[G2_MID_HIGH_REQUIRES],
        },
        Release {
            dist: G2_SHARED,
            version: G2_LOW,
            requires: &[],
        },
        Release {
            dist: G2_SHARED,
            version: G2_HIGH,
            requires: &[],
        },
    ];
    let registry = Registry::start(&releases);

    // ---- fixture self-check: the two picks the answer must back off from --
    let eager = Project::start();
    eager.set_dependencies(&[G2_ROOT]);
    let (eager_body, eager_entries) = eager.lock_ok("fixture self-check", &registry);
    assert_eq!(
        pins_of(&eager_entries),
        vec![
            format!("{G2_ROOT}=={ROOT_VERSION}"),
            format!("{G2_SHARED}=={G2_HIGH}"),
            format!("{G2_MID}=={G2_MID_HIGH}"),
        ],
        "fixture self-check: `{G2_ROOT}` alone must take \
         `{G2_MID}=={G2_MID_HIGH}` and, through its `{G2_MID_HIGH_REQUIRES}`, \
         `{G2_SHARED}=={G2_HIGH}` — the pair `{G2_OTHER}`'s \
         `{G2_OTHER_REQUIRES}` then forbids\n--- mamba.lock ---\n{eager_body}\n{}",
        registry.seen()
    );

    // ---- the contract ----------------------------------------------------
    let project = Project::start();
    project.set_dependencies(&[G2_ROOT, G2_OTHER]);
    let (body, entries) = project.lock_ok("graph two", &registry);

    assert_eq!(
        pins_of(&entries),
        vec![
            format!("{G2_ROOT}=={ROOT_VERSION}"),
            format!("{G2_SHARED}=={G2_LOW}"),
            format!("{G2_MID}=={G2_MID_LOW}"),
            format!("{G2_OTHER}=={ROOT_VERSION}"),
        ],
        "graph two: `{G2_OTHER_REQUIRES}` leaves `{G2_SHARED}=={G2_LOW}` as the \
         only admissible release, and `{G2_MID}=={G2_MID_HIGH}` requires \
         `{G2_MID_HIGH_REQUIRES}`, which that release fails — so `{G2_MID}` has \
         to come down to `{G2_MID_LOW}`, whose `{G2_MID_LOW_REQUIRES}` holds. \
         Keeping the newer `{G2_MID}`, or refusing the graph, both abandon the \
         one consistent solution\n--- mamba.lock ---\n{body}\n{}",
        registry.seen()
    );
    assert_pinned_at("graph two", &registry, &entries, &body, G2_MID, G2_MID_LOW);
    assert_pinned_at("graph two", &registry, &entries, &body, G2_SHARED, G2_LOW);

    let shared_edge = format!("{G2_SHARED}=={G2_LOW}");
    assert_edges(
        "graph two",
        &entries,
        &body,
        G2_ROOT,
        true,
        &[format!("{G2_MID}=={G2_MID_LOW}")],
    );
    assert_edges(
        "graph two",
        &entries,
        &body,
        G2_OTHER,
        true,
        &[shared_edge.clone()],
    );
    assert_edges("graph two", &entries, &body, G2_MID, false, &[shared_edge]);
    assert_edges("graph two", &entries, &body, G2_SHARED, false, &[]);
}

// --------------------------------------------------------------------------
// graph three
// --------------------------------------------------------------------------

/// A `==X.*` edge is a bound like any other: it admits a prefix of releases
/// and excludes the rest. The pin it selects here is neither the newest
/// release nor the oldest, so a lock that drops the edge, that reads it as
/// "any version", or that answers it with the lowest release each say so.
#[test]
fn lock_pins_the_highest_release_a_wildcard_edge_admits() {
    let releases = [
        Release {
            dist: G3_ROOT,
            version: ROOT_VERSION,
            requires: &[G3_ROOT_REQUIRES],
        },
        Release {
            dist: G3_SHARED,
            version: G3_LOW,
            requires: &[],
        },
        Release {
            dist: G3_SHARED,
            version: G3_MATCH,
            requires: &[],
        },
        Release {
            dist: G3_SHARED,
            version: G3_EXCLUDED,
            requires: &[],
        },
    ];
    let registry = Registry::start(&releases);

    // ---- fixture self-check: all three releases are on offer -------------
    // Asked for with no constraint at all, `wildlib` resolves to its newest
    // release. A red here says the page or the wheels are wrong; a green says
    // the `1.5` asserted below is what the edge selected, not what the
    // registry happened to serve.
    let unconstrained = Project::start();
    unconstrained.set_dependencies(&[G3_SHARED]);
    let (unconstrained_body, unconstrained_entries) =
        unconstrained.lock_ok("fixture self-check", &registry);
    assert_eq!(
        only_entry(&unconstrained_entries, G3_SHARED, &unconstrained_body).version,
        G3_EXCLUDED,
        "fixture self-check: with no constraint on it, `{G3_SHARED}` must \
         resolve to `{G3_EXCLUDED}`, the newest of the three releases \
         /simple/{G3_SHARED}/ advertises\n--- mamba.lock ---\n{unconstrained_body}\n{}",
        registry.seen()
    );

    // ---- the contract ----------------------------------------------------
    let project = Project::start();
    project.set_dependencies(&[G3_ROOT]);
    let (body, entries) = project.lock_ok("graph three", &registry);

    assert_eq!(
        pins_of(&entries),
        vec![
            format!("{G3_ROOT}=={ROOT_VERSION}"),
            format!("{G3_SHARED}=={G3_MATCH}"),
        ],
        "graph three: `{G3_ROOT}` requires `{G3_ROOT_REQUIRES}`, so \
         `{G3_SHARED}` belongs in the lock at `{G3_MATCH}` — the highest \
         release whose version begins `1.`. A lock without `{G3_SHARED}` at all \
         dropped the edge; one at `{G3_EXCLUDED}` read the wildcard as `any \
         version` and admitted the release it excludes; one at `{G3_LOW}` \
         answered a bound with the lowest release rather than the highest it \
         admits\n--- mamba.lock ---\n{body}\n{}",
        registry.seen()
    );
    assert_pinned_at(
        "graph three",
        &registry,
        &entries,
        &body,
        G3_SHARED,
        G3_MATCH,
    );
    assert_edges(
        "graph three",
        &entries,
        &body,
        G3_ROOT,
        true,
        &[format!("{G3_SHARED}=={G3_MATCH}")],
    );
    assert_edges("graph three", &entries, &body, G3_SHARED, false, &[]);
}
