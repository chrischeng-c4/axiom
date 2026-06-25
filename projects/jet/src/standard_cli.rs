//! Standard agent-facing CLI commands every axiom tool ships:
//! `llm` (offline self-documentation), `upgrade` (self-update from GitHub
//! releases), and `report-issue` (file a structured issue).
//!
//! See `CONTRIBUTING.md` → "CLI convention: every CLI ships `llm`, `upgrade`,
//! `report-issue`". `llm`'s topic content is the single in-code source of truth
//! for how an agent drives jet.

use anyhow::{Context, Result};
use clap::{Arg, ArgAction, ArgMatches, Command};
use sha2::{Digest, Sha256};

/// GitHub repo that owns jet's releases + issue tracker.
const REPO: &str = "chrischeng-c4/axiom";
/// Release-tag / asset prefix for this tool (`jet@<version>`, `jet-<target>`).
const PROJECT: &str = "jet";

// ---------------------------------------------------------------------------
// clap registration — called from `cli::command()`.
// ---------------------------------------------------------------------------

/// `jet llm [topic] [--format md|json]`
pub fn llm_command() -> Command {
    Command::new("llm")
        .about("Print agent-facing docs for driving jet — offline, no network")
        .arg(
            Arg::new("topic")
                .help("Topic: outline (default), workflow, quickstart, recipes")
                .default_value("outline"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .value_parser(["md", "json"])
                .default_value("md")
                .help("Output format"),
        )
}

/// `jet upgrade [--version <tag>] [--check]`
pub fn upgrade_command() -> Command {
    Command::new("upgrade")
        .about("Update jet to the latest jet@* GitHub release")
        .arg(
            Arg::new("version")
                .long("version")
                .help("Install a specific release tag (e.g. jet@0.4.1 or 0.4.1)"),
        )
        .arg(
            Arg::new("check")
                .long("check")
                .action(ArgAction::SetTrue)
                .help("Only report whether a newer release exists; do not install"),
        )
}

/// `jet report-issue [--title <t>] [--dry-run] [message...]`
pub fn report_issue_command() -> Command {
    Command::new("report-issue")
        .about("File a structured issue report against the axiom tracker")
        .arg(
            Arg::new("title")
                .long("title")
                .help("Issue title (default: derived from the message)"),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .action(ArgAction::SetTrue)
                .help("Print the issue that would be filed (and its URL) without creating it"),
        )
        .arg(
            Arg::new("message")
                .num_args(0..)
                .help("Free-text description of the problem"),
        )
}

// ---------------------------------------------------------------------------
// `llm` — offline self-documentation.
// ---------------------------------------------------------------------------

struct Topic {
    id: &'static str,
    summary: &'static str,
    body: &'static str,
}

const TOPICS: &[Topic] = &[
    Topic {
        id: "workflow",
        summary: "mental model: install → dev → build → test",
        body: "\
# jet workflow

jet is a Rust-native build tool + package manager for JavaScript/TypeScript
(bun/vite/npm in one binary). The mental model:

1. `jet init`              scaffold a new project
2. `jet install`          resolve + install deps from package.json → jet-lock.yaml
3. `jet add <pkg>`        add a dependency (`-D` for devDependencies)
4. `jet dev`              dev server with hot module reload (HMR)
5. `jet build`            production build (app, or a library with `--lib`)
6. `jet test` / `jet e2e` native unit/component tests and product-flow E2E
7. `jet check`            TypeScript type-check

Packages live in a global content-addressed store (`jet store`). The lockfile is
`jet-lock.yaml`; configuration is `jet.toml` (inspect with `jet config`).",
    },
    Topic {
        id: "quickstart",
        summary: "copy-paste create → dev → build",
        body: "\
# jet quickstart

    jet init my-app
    cd my-app
    jet install
    jet dev            # serves with HMR
    # ...edit src...
    jet build          # production bundle in dist/
    jet test           # run the native test runner",
    },
    Topic {
        id: "recipes",
        summary: "task → command cheat-sheet",
        body: "\
# jet recipes

| task                       | command                       |
|----------------------------|-------------------------------|
| add a dependency           | `jet add lodash`              |
| add a dev dependency       | `jet add -D vitest`           |
| remove a dependency        | `jet remove lodash`           |
| run a package.json script  | `jet run build`               |
| run a one-off binary (npx) | `jet jtx cowsay hi`           |
| type-check                 | `jet check`                   |
| build a library            | `jet build --lib`             |
| start the dev server       | `jet dev`                     |
| run e2e flows              | `jet e2e run`                 |
| inspect / lint config      | `jet config lint`             |
| update this tool           | `jet upgrade`                 |
| file a bug                 | `jet report-issue \"...\"`      |",
    },
];

/// `jet llm` — print the requested topic offline.
pub fn run_llm(matches: &ArgMatches) -> Result<()> {
    let topic = matches
        .get_one::<String>("topic")
        .map(String::as_str)
        .unwrap_or("outline");
    let json = matches.get_one::<String>("format").map(String::as_str) == Some("json");

    if topic == "outline" {
        if json {
            let topics: Vec<_> = TOPICS
                .iter()
                .map(|t| serde_json::json!({ "id": t.id, "summary": t.summary }))
                .collect();
            let out = serde_json::json!({
                "project": PROJECT,
                "version": env!("CARGO_PKG_VERSION"),
                "topics": topics,
            });
            println!("{}", serde_json::to_string_pretty(&out)?);
        } else {
            println!("{}", outline_md());
        }
        return Ok(());
    }

    let Some(t) = TOPICS.iter().find(|t| t.id == topic) else {
        let ids: Vec<&str> = TOPICS.iter().map(|t| t.id).collect();
        anyhow::bail!("unknown llm topic '{topic}'. Try: outline, {}", ids.join(", "));
    };

    if json {
        let out = serde_json::json!({
            "project": PROJECT,
            "topic": t.id,
            "summary": t.summary,
            "body": t.body,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("{}", t.body);
    }
    Ok(())
}

fn outline_md() -> String {
    let mut s = String::new();
    s.push_str("# jet — agent topic outline\n\n");
    s.push_str(
        "jet is a Rust-native build tool + package manager for JavaScript/TypeScript.\n\
         Run `jet llm <topic>` for detail (add `--format json` for a machine-readable form).\n\n",
    );
    s.push_str("## Topics\n\n");
    for t in TOPICS {
        s.push_str(&format!("- `{}` — {}\n", t.id, t.summary));
    }
    s.push_str("\n## Standard agent commands\n\n");
    s.push_str("- `jet llm [topic] [--format md|json]` — this self-documentation (offline)\n");
    s.push_str("- `jet upgrade [--version <tag>] [--check]` — self-update from GitHub releases\n");
    s.push_str("- `jet report-issue [--title <t>] [message...]` — file a structured issue\n");
    s
}

// ---------------------------------------------------------------------------
// `upgrade` — self-update from the per-project GitHub release pipeline.
// ---------------------------------------------------------------------------

/// `jet upgrade` — replace the running binary with a release build.
pub async fn run_upgrade(matches: &ArgMatches) -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");
    let check_only = matches.get_flag("check");
    let pin = matches.get_one::<String>("version").cloned();

    let client = reqwest::Client::builder()
        .user_agent(concat!("jet-upgrade/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("failed to build HTTP client")?;

    let tag = match pin {
        Some(v) if v.contains('@') => v,
        Some(v) => format!("{PROJECT}@{v}"),
        None => latest_release_tag(&client).await?,
    };
    let latest_ver = tag.split('@').nth(1).unwrap_or(tag.as_str()).to_string();

    if check_only {
        if version_is_newer(&latest_ver, current) {
            println!("jet {current} → {latest_ver} available (tag {tag}). Run `jet upgrade` to install.");
        } else {
            println!("jet {current} is up to date (latest release: {tag}).");
        }
        return Ok(());
    }

    let target = release_target()?;
    let base = format!("https://github.com/{REPO}/releases/download/{tag}");
    let asset = format!("{PROJECT}-{target}.tar.gz");
    let tar_url = format!("{base}/{asset}");
    let sha_url = format!("{tar_url}.sha256");

    println!("downloading {tar_url}");
    let bytes = client
        .get(&tar_url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .with_context(|| format!("download failed: {tar_url}"))?
        .bytes()
        .await
        .context("failed to read release tarball")?;

    // Best-effort integrity check — refuse on mismatch, tolerate a missing .sha256.
    if let Ok(resp) = client.get(&sha_url).send().await.and_then(|r| r.error_for_status()) {
        let expected = resp.text().await.unwrap_or_default();
        let expected = expected.split_whitespace().next().unwrap_or("").to_string();
        if !expected.is_empty() {
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let actual = hex_lower(&hasher.finalize());
            if actual != expected {
                anyhow::bail!("sha256 mismatch (expected {expected}, got {actual})");
            }
            println!("sha256 verified");
        }
    }

    let exe = std::env::current_exe().context("cannot locate the running jet executable")?;
    let dir = exe.parent().context("running executable has no parent directory")?;
    let tmp = dir.join(format!(".jet-upgrade-{}.tmp", std::process::id()));

    let gz = flate2::read::GzDecoder::new(std::io::Cursor::new(&bytes[..]));
    let mut archive = tar::Archive::new(gz);
    let wanted = format!("{PROJECT}-{target}/{PROJECT}");
    let mut found = false;
    for entry in archive.entries().context("malformed release tarball")? {
        let mut entry = entry?;
        let path = entry.path()?.to_string_lossy().to_string();
        if path == wanted {
            entry.unpack(&tmp).with_context(|| format!("failed to write {}", tmp.display()))?;
            found = true;
            break;
        }
    }
    if !found {
        let _ = std::fs::remove_file(&tmp);
        anyhow::bail!("binary not found in archive: {wanted}");
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(&tmp)?.permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(&tmp, perm)?;
    }

    // Atomic swap: rename over the current exe (same dir → same filesystem).
    // The running process keeps the old inode; the next launch is the new build.
    std::fs::rename(&tmp, &exe)
        .with_context(|| format!("failed to replace {}", exe.display()))?;

    println!("upgraded jet {current} → {latest_ver} ({})", exe.display());
    Ok(())
}

async fn latest_release_tag(client: &reqwest::Client) -> Result<String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases?per_page=30");
    let releases: serde_json::Value = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .context("failed to query GitHub releases")?
        .json()
        .await
        .context("failed to parse GitHub releases response")?;

    let prefix = format!("{PROJECT}@");
    releases
        .as_array()
        .and_then(|arr| {
            arr.iter()
                .filter_map(|r| r.get("tag_name").and_then(|t| t.as_str()))
                .find(|t| t.starts_with(&prefix))
                .map(str::to_string)
        })
        .with_context(|| format!("no {PROJECT}@* release found in {REPO}"))
}

/// Release asset target triple for the host. Mirrors the release workflow's
/// matrix (Apple Silicon only on macOS).
fn release_target() -> Result<String> {
    let arch = match std::env::consts::ARCH {
        "aarch64" => "aarch64",
        "x86_64" => "x86_64",
        other => anyhow::bail!("unsupported architecture: {other}"),
    };
    let os = match std::env::consts::OS {
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-gnu",
        other => anyhow::bail!("unsupported OS: {other}"),
    };
    if std::env::consts::OS == "macos" && arch == "x86_64" {
        anyhow::bail!("jet provides no Intel-macOS binary (Apple Silicon only)");
    }
    Ok(format!("{arch}-{os}"))
}

/// True when `candidate` is a strictly newer version than `current`.
/// Parses dotted numeric versions; falls back to string inequality.
fn version_is_newer(candidate: &str, current: &str) -> bool {
    fn parse(v: &str) -> Option<(u64, u64, u64)> {
        let v = v.trim().trim_start_matches('v');
        let mut it = v.split('.');
        let major = it.next()?.parse().ok()?;
        let minor = it.next().unwrap_or("0").parse().ok()?;
        let patch = it.next().unwrap_or("0").parse().ok()?;
        Some((major, minor, patch))
    }
    match (parse(candidate), parse(current)) {
        (Some(c), Some(cur)) => c > cur,
        _ => candidate != current,
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

// ---------------------------------------------------------------------------
// `report-issue` — file a structured issue report.
// ---------------------------------------------------------------------------

/// `jet report-issue` — open a pre-filled issue (via `gh`, else print the URL).
pub fn run_report_issue(matches: &ArgMatches) -> Result<()> {
    let dry = matches.get_flag("dry-run");
    let msg = matches
        .get_many::<String>("message")
        .map(|v| v.cloned().collect::<Vec<_>>().join(" "))
        .unwrap_or_default();
    let version = env!("CARGO_PKG_VERSION");
    let (os, arch) = (std::env::consts::OS, std::env::consts::ARCH);

    let title = matches.get_one::<String>("title").cloned().unwrap_or_else(|| {
        if msg.trim().is_empty() {
            "jet: issue report".to_string()
        } else {
            let head: String = msg.lines().next().unwrap_or("").chars().take(72).collect();
            format!("jet: {head}")
        }
    });
    let described = if msg.trim().is_empty() { "_(no description provided)_" } else { &msg };
    let body = format!(
        "{described}\n\n---\n_Filed via `jet report-issue`._\n- jet: {version}\n- os/arch: {os}/{arch}\n"
    );
    let new_issue_url = format!(
        "https://github.com/{REPO}/issues/new?title={}&body={}",
        percent_encode(&title),
        percent_encode(&body),
    );

    if dry {
        println!("repo:  {REPO}");
        println!("title: {title}");
        println!("---");
        println!("{body}");
        println!("url:   {new_issue_url}");
        return Ok(());
    }

    // Prefer `gh issue create`; fall back to a pre-filled URL when gh is
    // absent (NotFound) or the create fails (e.g. unauthenticated).
    match std::process::Command::new("gh")
        .args(["issue", "create", "--repo", REPO, "--title", &title, "--body", &body])
        .status()
    {
        Ok(s) if s.success() => return Ok(()),
        Ok(_) => eprintln!("jet report-issue: `gh issue create` failed; falling back to a URL."),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => eprintln!("jet report-issue: could not run `gh` ({e}); falling back to a URL."),
    }

    println!("Open this URL to file the issue:\n{new_issue_url}");
    Ok(())
}

/// Minimal RFC 3986 percent-encoding (unreserved chars pass through). Avoids a
/// url/urlencoding dependency for the issue-URL fallback.
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
