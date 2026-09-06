//! Black-box contract: when the registry cannot answer for a release's
//! `requires_dist`, `mamba lock` must refuse the resolution — not resolve the
//! release as a leaf and write a lock that quietly lacks the edge.
//!
//! A dependency edge that exists only in the registry's per-version document
//! is invisible to every other source: the wheel declares nothing, the simple
//! page declares nothing, and the manifest names only the root. So a run that
//! asks for that document, is refused an answer, and carries on has not
//! learned that the release is a leaf — it has learned nothing, and the lock
//! it writes states a closure it never read. `mamba sync` then builds a venv
//! from that lock and the import fails at runtime, which is the same failure
//! the unreadable-line case next door already refuses.
//!
//! The line between the two answers is drawn where the index client already
//! draws it, and this file pins both sides of it:
//!
//! - **an answer that says nothing** — HTTP 404, an index that simply has no
//!   per-version route — resolves to a release with no edges, exit 0;
//! - **no answer at all** — HTTP 503 on every attempt, after the client has
//!   spent its retries — refuses, non-zero, with nothing written.
//!
//! # What this case owns
//!
//! The case drives the built `mamba` binary over throwaway directory trees and
//! reads only what the binary wrote, or refused to write. It never links the
//! resolver, the index client or the lock renderer, and it never hand-writes a
//! `mamba.lock`, an index page or a wheel: every wheel is built by the
//! product's own `WheelBuilder`, and the registry serves PEP 503 anchor pages
//! plus per-version `requires_dist` documents. The wheels themselves declare
//! no `Requires-Dist`, so the one dependency edge below exists in exactly one
//! place — the registry's own `/pypi/app/1.0/json` — and that is precisely the
//! route the three cases vary. The only thing the case authors is
//! `mamba.toml`'s `dependencies` line, which is user input in this workflow
//! and exactly what `mamba lock` reads.
//!
//! # The one graph, served three ways
//!
//! ```text
//!   app 1.0 --requires dep>=1--> dep 1.0        (dep declares nothing)
//!
//!   case one    /pypi/app/1.0/json -> 503 on every request
//!               => refuse: non-zero, stderr names app==1.0 and requires_dist,
//!                  no mamba.lock, and the route was asked more than once
//!   case two    /pypi/app/1.0/json -> 200, ["dep>=1"]
//!               => lock app==1.0 with the dep==1.0 edge, and dep==1.0 itself
//!   case three  /pypi/app/1.0/json -> 404
//!               => lock app==1.0 alone, dependencies = [], exit 0
//! ```
//!
//! Cases two and three are the boundary case one must not cross: without them,
//! a resolver that refused every graph, or one that refused every empty
//! answer, would satisfy case one. Case two proves the fixture can resolve the
//! graph it serves; case three proves the 404 allowance survives.
//!
//! # Hermeticity
//!
//! No network, ever: every wheel is built in-process and the only package
//! source is an in-process `wiremock` server on loopback — nothing here
//! resolves against pypi.org. `HOME` and `MAMBA_CACHE_DIR` are pinned per
//! project into its own temp tree, so no two projects can read each other's
//! cached metadata; `MAMBA_FROZEN_INDEX`, `MAMBA_INDEX_URL`, `MAMBA_JOBS`,
//! `XDG_CACHE_HOME`, `VIRTUAL_ENV`, `PYTHONPATH` and every proxy variable are
//! removed, so no ambient index or proxy can supply or divert an answer. No
//! step builds an environment or runs a script, so no Python interpreter is
//! required or consulted. Nothing is skipped, nothing is `#[ignore]`d, and
//! nothing here sleeps or polls: the retries case one observes are counted
//! from the registry's own request log, never timed.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mamba::pkgmanage::pkgmgr::wheel_build::{
    compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
};
use sha2::{Digest, Sha256};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// --------------------------------------------------------------------------
// the graph
// --------------------------------------------------------------------------

const ROOT: &str = "app";
const ROOT_VERSION: &str = "1.0";
/// The one `requires_dist` line `app 1.0` declares. It lives only in the
/// registry's per-version document, so it is readable exactly when that
/// document is served.
const EDGE_LINE: &str = "dep>=1";
const ROOT_REQUIRES: &[&str] = &[EDGE_LINE];
const DEP: &str = "dep";
const DEP_VERSION: &str = "1.0";

/// The route `/pypi/app/1.0/json` — the one URL the three cases vary, and the
/// one whose request count case one reads to show the client spent its retries
/// before giving up.
fn root_version_route() -> String {
    format!("/pypi/{ROOT}/{ROOT_VERSION}/json")
}

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

    /// Rewrite the manifest's one `dependencies` line, leaving every other byte
    /// `mamba init` wrote in place. This is the case's only authored input;
    /// `dev-dependencies` is a different key and is never touched.
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
    /// answer, and return the lock it wrote, parsed. Every assertion about a
    /// written lock reads it through here, so no case can accept a lock the
    /// binary refused.
    fn lock_ok(&self, context: &str, registry: &Registry) -> (String, Vec<LockEntry>) {
        let index_url = registry.index_url();
        let args = ["lock", "--index-url", index_url.as_str()];
        let out = self.run(&args);
        assert!(
            out.status.success(),
            "{context}: `mamba lock --index-url` must resolve this graph — \
             every route it needs is served, and every name it raises has a \
             release satisfying it, so there is nothing here to refuse\n{}\n{}",
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

/// What the registry answers on one release's `/pypi/<dist>/<version>/json`.
/// This is the whole of what the three cases vary; the simple pages, the
/// wheels and the manifest are identical in all three.
enum VersionRoute {
    /// `200` carrying the release's declared `requires_dist` lines.
    Healthy,
    /// `404` — an index that has no per-version route at all. The client reads
    /// this as "declared nothing", and the resolution continues.
    Absent,
    /// `503` on every request — a retryable status the client never gets past.
    /// Nothing is declared and nothing is denied: the answer is unknown.
    Unavailable,
}

/// One release the registry publishes: a distribution at a version, the
/// `requires_dist` lines the registry declares for it, and how its per-version
/// route answers. The wheel itself declares nothing, so this list is the only
/// place an edge exists.
struct Release {
    dist: &'static str,
    version: &'static str,
    requires: &'static [&'static str],
    route: VersionRoute,
}

/// Build one real wheel through the product's own wheel builder.
///
/// PEP 427 escapes every run of `-` in the distribution component of a wheel
/// filename to `_`. The escape is applied here for the same reason a real
/// build backend applies it: the filename's second `-`-separated field is what
/// an index reads as the version.
fn build_wheel_file(out_dir: &Path, dist: &str, version: &str) -> PathBuf {
    let escaped = dist.replace('-', "_");
    let filename = compose_filename(&escaped, version, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-e2e-requires-dist-unavailable");
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
    /// releases, one per-version route per release answering as its
    /// `VersionRoute` says, and the wheel bytes each anchor points at.
    fn start(releases: &[Release]) -> Registry {
        for release in releases {
            if !matches!(release.route, VersionRoute::Healthy) {
                assert!(
                    release.requires.is_empty(),
                    "fixture: {}=={} declares {:?}, but its per-version route \
                     never serves a document — a case cannot expect an edge \
                     from a line the registry never hands over",
                    release.dist,
                    release.version,
                    release.requires
                );
            }
        }
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

            // The only place a dependency edge exists, and the one route the
            // three cases vary.
            for release in releases {
                let response = match release.route {
                    VersionRoute::Healthy => {
                        let body = format!(
                            "{{\"info\":{{\"requires_dist\":[{}]}}}}",
                            release
                                .requires
                                .iter()
                                .map(|r| format!("{r:?}"))
                                .collect::<Vec<_>>()
                                .join(",")
                        );
                        ResponseTemplate::new(200)
                            .set_body_raw(body.into_bytes(), "application/json")
                    }
                    VersionRoute::Absent => ResponseTemplate::new(404),
                    VersionRoute::Unavailable => ResponseTemplate::new(503),
                };
                Mock::given(method("GET"))
                    .and(path(format!(
                        "/pypi/{}/{}/json",
                        release.dist, release.version
                    )))
                    .respond_with(response)
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

    /// The `--index-url` every case passes: the PEP 503 simple URL, which the
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

    /// Every request the registry saw, as `METHOD /path`, in order.
    fn request_paths(&self) -> Vec<String> {
        self.rt
            .block_on(self.server.received_requests())
            .unwrap_or_default()
            .iter()
            .map(|r| format!("{} {}", r.method, r.url.path()))
            .collect()
    }

    /// How many times the registry was asked for exactly `route`. This is how
    /// case one shows the client spent its retries: counted from the server's
    /// own log rather than timed, so the retry policy's delays are not part of
    /// what this file pins.
    fn hits(&self, route: &str) -> usize {
        let wanted = format!("GET {route}");
        self.request_paths()
            .iter()
            .filter(|seen| **seen == wanted)
            .count()
    }

    /// Every request the registry saw, folded into each panic message so a red
    /// names what the client actually asked for — in particular whether it got
    /// as far as the per-version document at all.
    fn seen(&self) -> String {
        let lines = self.request_paths();
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
/// registry advertised for that very version.
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
        "{context}: `{name}` must be pinned at {version}\n\
         --- mamba.lock ---\n{body}\n{}",
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
        "{context}: `{name}`'s edges must be exactly what the registry served \
         for it, rendered against the versions the lock pinned — an edge that \
         vanished between the registry and the lock leaves `mamba sync` \
         building an environment that cannot import\n\
         --- mamba.lock ---\n{body}"
    );
}

/// The graph, with `app`'s per-version route answering as `route` says. A
/// route that serves no document declares no lines, so the three cases differ
/// in exactly one respect: whether, and how, `/pypi/app/1.0/json` answers.
fn graph(route: VersionRoute) -> [Release; 2] {
    let requires: &'static [&'static str] = match route {
        VersionRoute::Healthy => ROOT_REQUIRES,
        _ => &[],
    };
    [
        Release {
            dist: ROOT,
            version: ROOT_VERSION,
            requires,
            route,
        },
        Release {
            dist: DEP,
            version: DEP_VERSION,
            requires: &[],
            route: VersionRoute::Healthy,
        },
    ]
}

/// `dep` resolves and locks against this registry when the manifest asks for
/// it by name. A red here says the simple pages, the wheels or the manifest
/// are wrong; a green says whatever the case asserts about `app` is about
/// `app`'s own per-version route, and not about a fixture that could resolve
/// nothing at all.
fn assert_fixture_resolves(registry: &Registry) {
    let control = Project::start();
    control.set_dependencies(&[DEP]);
    let (body, entries) = control.lock_ok("fixture self-check", registry);
    assert_eq!(
        pins_of(&entries),
        vec![format!("{DEP}=={DEP_VERSION}")],
        "fixture self-check: `{DEP}` must resolve and lock against this \
         registry\n--- mamba.lock ---\n{body}\n{}",
        registry.seen()
    );
}

// --------------------------------------------------------------------------
// case one
// --------------------------------------------------------------------------

/// A `requires_dist` document the registry will not serve is not an answer of
/// "no dependencies" — it is no answer at all. `app 1.0` declares `dep>=1`,
/// and that line lives nowhere but the route being refused, so a run that
/// carries on cannot know whether the release has edges. The only safe answer
/// is to stop: exit non-zero, say which release and which fetch, and write
/// nothing.
#[test]
fn lock_refuses_when_a_releases_requires_dist_cannot_be_fetched() {
    let releases = graph(VersionRoute::Unavailable);
    let registry = Registry::start(&releases);

    // ---- fixture self-check: this registry resolves and locks ------------
    assert_fixture_resolves(&registry);

    // ---- the contract ----------------------------------------------------
    let project = Project::start();
    let root_requirement = format!("{ROOT}=={ROOT_VERSION}");
    project.set_dependencies(&[root_requirement.as_str()]);
    let index_url = registry.index_url();
    let args = ["lock", "--index-url", index_url.as_str()];
    let out = project.run(&args);

    assert!(
        !out.status.success(),
        "case one: `{root_requirement}`'s `requires_dist` document answers 503 \
         on every request, so nothing was learned about its dependencies — and \
         `{EDGE_LINE}` is declared nowhere else. `mamba lock` must refuse the \
         resolution: a run that treats the unfetched release as a leaf and \
         exits 0 writes a lock claiming a closure it never read, and `mamba \
         sync` then builds an environment from it that cannot import \
         `{DEP}`\n{}\n{}",
        render(&args, &out),
        registry.seen()
    );

    let stderr = stderr_of(&out);
    assert!(
        !stderr.contains("panicked at"),
        "case one: the refusal must be a diagnosed error, not a panic — a \
         crash on a registry that is merely unavailable is not a fail-closed \
         path\n{}\n{}",
        render(&args, &out),
        registry.seen()
    );

    let squeezed = squeezed_stderr(&out);
    for fragment in [root_requirement.as_str(), "requires_dist"] {
        let wanted: String = fragment.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            squeezed.contains(&wanted),
            "case one: the refusal must name the release `{root_requirement}` \
             and the `requires_dist` fetch that failed — `{fragment}` is \
             absent from stderr. The comparison ignores whitespace, so any \
             wrapping counts; a bare `resolution failed` leaves the reader \
             unable to tell which release of which graph the registry would \
             not answer for, and so unable to tell a broken index from a \
             broken manifest\n{}\n{}",
            render(&args, &out),
            registry.seen()
        );
    }

    let left_behind = project.lock_like_entries();
    assert_eq!(
        left_behind,
        Vec::<String>::new(),
        "case one: a refused resolution must write no lock and leave no \
         fragment of one behind — a lock on disk is what the next `mamba sync` \
         reads, whatever the exit code said; {} holds {left_behind:?}\n{}",
        project.dir.display(),
        render(&args, &out)
    );

    let route = root_version_route();
    let hits = registry.hits(&route);
    assert!(
        hits > 1,
        "case one: the refusal must come after the index client has spent its \
         retries on a retryable status, not on the first 503 — the registry \
         logged {hits} request(s) for `{route}`. Counted from the server's own \
         log rather than timed, so this says nothing about how long the \
         backoff waits\n{}\n{}",
        render(&args, &out),
        registry.seen()
    );
}

// --------------------------------------------------------------------------
// case two
// --------------------------------------------------------------------------

/// The same graph with every route answering: the edge the per-version
/// document declares is read, `dep` joins the lock at the version its bound
/// admits, and the run exits 0. This is one of the two boundaries case one
/// must not cross — a resolver that refused whenever it saw this graph, or
/// refused every registry, would satisfy case one and be useless.
#[test]
fn lock_carries_the_declared_edge_when_every_route_answers() {
    let releases = graph(VersionRoute::Healthy);
    let registry = Registry::start(&releases);

    assert_fixture_resolves(&registry);

    let project = Project::start();
    let root_requirement = format!("{ROOT}=={ROOT_VERSION}");
    project.set_dependencies(&[root_requirement.as_str()]);
    let (body, entries) = project.lock_ok("case two", &registry);

    assert_eq!(
        pins_of(&entries),
        vec![
            format!("{ROOT}=={ROOT_VERSION}"),
            format!("{DEP}=={DEP_VERSION}"),
        ],
        "case two: `{root_requirement}` declares `{EDGE_LINE}` and the \
         registry serves that document, so the lock owes two packages — the \
         root and the release its one line raises\n\
         --- mamba.lock ---\n{body}\n{}",
        registry.seen()
    );
    assert_pinned_at("case two", &registry, &entries, &body, ROOT, ROOT_VERSION);
    assert_pinned_at("case two", &registry, &entries, &body, DEP, DEP_VERSION);
    assert_edges(
        "case two",
        &entries,
        &body,
        ROOT,
        true,
        &[format!("{DEP}=={DEP_VERSION}")],
    );
    assert_edges("case two", &entries, &body, DEP, false, &[]);
}

// --------------------------------------------------------------------------
// case three
// --------------------------------------------------------------------------

/// An index with no per-version route at all answers 404, and that *is* an
/// answer: this registry has nothing further to say about the release, so it
/// resolves with no edges and the run exits 0. The distinction case one draws
/// is between an answer and a failure to answer — not between an empty edge
/// list and a full one — so a resolver that refuses every release it learned
/// no edges for fails here.
#[test]
fn lock_treats_an_absent_per_version_route_as_no_dependencies() {
    let releases = graph(VersionRoute::Absent);
    let registry = Registry::start(&releases);

    assert_fixture_resolves(&registry);

    let project = Project::start();
    let root_requirement = format!("{ROOT}=={ROOT_VERSION}");
    project.set_dependencies(&[root_requirement.as_str()]);
    let (body, entries) = project.lock_ok("case three", &registry);

    assert_eq!(
        pins_of(&entries),
        vec![format!("{ROOT}=={ROOT_VERSION}")],
        "case three: `/pypi/{ROOT}/{ROOT_VERSION}/json` answers 404, which is \
         this index saying it has no per-version document to offer. The \
         allowance stands: `{root_requirement}` resolves alone, exit 0, and \
         nothing raises `{DEP}`\n--- mamba.lock ---\n{body}\n{}",
        registry.seen()
    );
    assert_pinned_at("case three", &registry, &entries, &body, ROOT, ROOT_VERSION);
    assert_edges("case three", &entries, &body, ROOT, true, &[]);
}
