//! Smoke gate for `ecosystem_fixture_manifest.toml` — #2551 / parent
//! #2529.
//!
//! Loads the required-ecosystem-fixture manifest and verifies:
//!
//!   1. Schema. Every entry carries `category` ("stdlib" | "3p"),
//!      `module`, `relpath`, `required_modules` (non-empty),
//!      `required_stdlib_modules` (non-empty, #2553),
//!      `expected_outcome` == "pass" (the only legal value today), and
//!      `command` (non-empty).
//!   2. File existence. `relpath` must resolve under
//!      `tests/cpython/` to an existing `.py` file.
//!   3. Consistency. `relpath` must start with `<category>/<module>/`
//!      and `command` must reference the same `relpath`. These keep
//!      manifest entries self-describing — a worker who reads the
//!      table key + relpath should never have to cross-check the
//!      command field.
//!   4. Self-consistent stdlib mapping (#2553). For stdlib fixtures
//!      the fixture's `module` must appear in
//!      `required_stdlib_modules` — a `stdlib/re/...` fixture that
//!      doesn't name `re` as a stdlib dep is structurally broken.
//!   5. No-network default gate (#2556). Every manifest-listed fixture
//!      source is scanned for known network-touching patterns
//!      (urllib.request, http.client, socket connect, requests.get/post
//!      against non-mock URLs, etc.). A required fixture that reaches
//!      the network fails the smoke gate up-front instead of getting
//!      bucketed at runtime as flaky — the default MVP gate has no
//!      network dependency by construction.
//!
//! Acceptance for #2551 says "missing required fixture files fail
//! validation" — that is what (2) enforces. The MVP-gate behaviour
//! ("required-only counts toward MVP green") is implemented in
//! #2550, which now reads this manifest. #2553 adds the
//! stdlib-dep mapping so a future failure report can group required
//! failures by their stdlib dependency ("five fixtures block on `re`").
//! #2556 enforces the no-network invariant statically.
//!
//! This is a cheap test (single TOML read + N stat()s). Stays in the
//! default test set; runs in well under a second.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("ecosystem_fixture_manifest.toml")
}

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("conformance")
}

fn load_manifest() -> toml::Value {
    let path = manifest_path();
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("manifest parse error: {e}"))
}

#[test]
fn ecosystem_fixture_manifest_is_well_formed_and_files_exist() {
    let doc = load_manifest();
    let fixtures = doc
        .get("fixtures")
        .and_then(|v| v.as_table())
        .unwrap_or_else(|| panic!("manifest missing top-level [fixtures] table"));

    let root = fixtures_root();
    let mut violations: Vec<String> = Vec::new();
    let mut seen_relpaths: HashSet<String> = HashSet::new();

    assert!(
        !fixtures.is_empty(),
        "manifest [fixtures] table is empty — MVP gate cannot be \
         exercised without at least one required entry",
    );

    for (id, entry) in fixtures {
        let table = match entry.as_table() {
            Some(t) => t,
            None => {
                violations.push(format!(
                    "  - {id}: entry must be a TOML table"
                ));
                continue;
            }
        };

        let category = match table.get("category").and_then(|v| v.as_str()) {
            Some(s) if s == "stdlib" || s == "3p" => s,
            Some(other) => {
                violations.push(format!(
                    "  - {id}: category = {other:?}; must be \"stdlib\" or \"3p\""
                ));
                continue;
            }
            None => {
                violations.push(format!(
                    "  - {id}: missing required `category` field"
                ));
                continue;
            }
        };

        let module = match table.get("module").and_then(|v| v.as_str()) {
            Some(s) if !s.is_empty() => s,
            _ => {
                violations.push(format!(
                    "  - {id}: missing or empty required `module` field"
                ));
                continue;
            }
        };

        let relpath = match table.get("relpath").and_then(|v| v.as_str()) {
            Some(s) if !s.is_empty() => s,
            _ => {
                violations.push(format!(
                    "  - {id}: missing or empty required `relpath` field"
                ));
                continue;
            }
        };

        // Consistency: relpath should begin with "<category>/<module>/" so
        // the table key + relpath are sufficient to navigate the fixture
        // without ever reading the `command` field.
        let expected_prefix = format!("{category}/{module}/");
        if !relpath.starts_with(&expected_prefix) {
            violations.push(format!(
                "  - {id}: relpath = {relpath:?} must start with {expected_prefix:?}",
            ));
        }
        if !relpath.ends_with(".py") {
            violations.push(format!(
                "  - {id}: relpath = {relpath:?} must end with .py",
            ));
        }

        // File existence — the headline acceptance.
        let abs = root.join(relpath);
        if !abs.exists() {
            violations.push(format!(
                "  - {id}: relpath {relpath:?} does not exist under \
                 tests/cpython/ (resolved {}). Restore the \
                 fixture or remove the manifest entry in the same commit.",
                abs.display(),
            ));
        }

        // Detect accidental duplicate relpaths — two different ids must
        // not point at the same file.
        if !seen_relpaths.insert(relpath.to_string()) {
            violations.push(format!(
                "  - {id}: relpath {relpath:?} already used by an earlier \
                 fixture entry; pick a unique relpath",
            ));
        }

        // required_modules — non-empty array of strings.
        match table.get("required_modules").and_then(|v| v.as_array()) {
            Some(arr) if !arr.is_empty() && arr.iter().all(|v| v.is_str()) => {}
            _ => {
                violations.push(format!(
                    "  - {id}: required_modules must be a non-empty array of \
                     strings"
                ));
            }
        }

        // required_stdlib_modules (#2553) — non-empty array of strings.
        // The headline #2553 acceptance: "Missing required_stdlib_modules
        // fails manifest validation" — enforced here.
        let stdlib_deps: Vec<String> = match table
            .get("required_stdlib_modules")
            .and_then(|v| v.as_array())
        {
            Some(arr) if !arr.is_empty() && arr.iter().all(|v| v.is_str()) => arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => {
                violations.push(format!(
                    "  - {id}: required_stdlib_modules must be a non-empty \
                     array of strings (#2553). Declare which stdlib modules \
                     this fixture exercises so a future report can group \
                     failures by stdlib dependency."
                ));
                Vec::new()
            }
        };

        // Self-consistent stdlib mapping (#2553). A stdlib fixture's
        // own module must appear in its stdlib dep set; otherwise the
        // mapping is structurally broken — a `stdlib/re/...` fixture
        // that doesn't list `re` as a stdlib dep cannot be grouped
        // correctly in the failure rollup.
        if category == "stdlib" && !stdlib_deps.iter().any(|m| m == module) {
            violations.push(format!(
                "  - {id}: stdlib fixture's own module {module:?} must appear \
                 in required_stdlib_modules = {stdlib_deps:?}"
            ));
        }

        // expected_outcome — pass | xfail | skip (#2555).
        let outcome = match table.get("expected_outcome").and_then(|v| v.as_str()) {
            Some(s @ ("pass" | "xfail" | "skip")) => Some(s),
            Some(other) => {
                violations.push(format!(
                    "  - {id}: expected_outcome = {other:?}; must be one of \
                     \"pass\" / \"xfail\" / \"skip\""
                ));
                None
            }
            None => {
                violations.push(format!(
                    "  - {id}: missing required `expected_outcome` field"
                ));
                None
            }
        };

        // blocker (#2555) — required iff expected_outcome is xfail or skip.
        // Without a tracked blocker an xfail / skip entry is debt that can't
        // be paid down: a future reader has no idea what would graduate it
        // back to pass, and the failure report can't name the cause.
        let blocker = table.get("blocker").and_then(|v| v.as_str());
        match (outcome, blocker) {
            (Some("xfail" | "skip"), None) => violations.push(format!(
                "  - {id}: expected_outcome is non-pass; `blocker` field \
                 (issue ref or short reason) is required so the report can \
                 name what would graduate the fixture back to pass"
            )),
            (Some("xfail" | "skip"), Some(s)) if s.is_empty() => violations.push(format!(
                "  - {id}: `blocker` must be non-empty for non-pass outcomes"
            )),
            (Some("pass"), Some(_)) => violations.push(format!(
                "  - {id}: `blocker` is only legal when expected_outcome \
                 is xfail or skip; remove it from the pass entry"
            )),
            _ => {}
        }

        // command — must reference relpath so it stays self-describing.
        match table.get("command").and_then(|v| v.as_str()) {
            Some(cmd) if cmd.contains(relpath) => {}
            Some(cmd) => violations.push(format!(
                "  - {id}: command = {cmd:?} must reference relpath \
                 {relpath:?} so manifest entries stay self-describing",
            )),
            None => violations.push(format!(
                "  - {id}: missing required `command` field"
            )),
        }
    }

    assert!(
        violations.is_empty(),
        "ecosystem fixture manifest has {} violation{}:\n{}",
        violations.len(),
        if violations.len() == 1 { "" } else { "s" },
        violations.join("\n"),
    );
}

/// #2553 acceptance — exercise the stdlib-dependency rollup helper.
///
/// The headline value of the `required_stdlib_modules` field is that
/// a failure report can answer "which stdlib modules block the most
/// required fixtures?". This test builds that rollup at the manifest
/// level (every fixture is treated as a potential failure) and
/// asserts:
///
///   1. The rollup is non-empty.
///   2. Each rollup bucket carries at least one fixture id.
///   3. The 3p `idna` fixture's transitive stdlib dep `unicodedata`
///      shows up as a bucket — so the cross-category mapping works,
///      not just the trivial `stdlib-mod-uses-mod-itself` case.
///
/// Coverage-correctness (does the fixture really exercise the
/// declared stdlib?) is explicitly out of scope per #2553.
#[test]
fn ecosystem_fixture_stdlib_dependency_rollup_groups_by_module() {
    let doc = load_manifest();
    let fixtures = doc
        .get("fixtures")
        .and_then(|v| v.as_table())
        .expect("manifest [fixtures] table");

    // module name -> sorted list of fixture ids that depend on it
    let mut rollup: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (id, entry) in fixtures {
        let Some(table) = entry.as_table() else { continue };
        let Some(arr) = table
            .get("required_stdlib_modules")
            .and_then(|v| v.as_array())
        else {
            continue;
        };
        for v in arr {
            let Some(s) = v.as_str() else { continue };
            rollup.entry(s.to_string()).or_default().push(id.clone());
        }
    }

    for ids in rollup.values_mut() {
        ids.sort();
    }

    assert!(
        !rollup.is_empty(),
        "stdlib-dependency rollup is empty — every required fixture \
         should declare at least one stdlib dep",
    );
    for (module, ids) in &rollup {
        assert!(
            !ids.is_empty(),
            "stdlib dep {module:?} has an empty fixture list — rollup invariant broken",
        );
    }

    // The idna 3p fixture's transitive stdlib dep on unicodedata must
    // show up in the rollup — this is the cross-category mapping
    // #2553 is really about. A stdlib-mod-uses-mod-itself rollup would
    // not have caught the case where the field was forgotten on the
    // 3p entry.
    let unicodedata = rollup
        .get("unicodedata")
        .expect("unicodedata bucket must be present once the 3p idna fixture is registered");
    assert!(
        unicodedata.iter().any(|id| id == "3p-idna-encode_idn"),
        "3p-idna-encode_idn must appear under the `unicodedata` rollup \
         bucket; got fixtures={unicodedata:?}",
    );

    // Echo the full rollup to stderr so a worker can grep it from CI
    // logs — this is the "summary can list which stdlib modules block
    // a third-party fixture" acceptance criterion in #2553.
    eprintln!("[ecosystem_manifest] stdlib_dependency_rollup (module -> fixture_ids):");
    for (module, ids) in &rollup {
        eprintln!("  {module}: {}", ids.join(", "));
    }
}

/// #2556 acceptance — every required fixture source must be statically
/// no-network. A fixture that imports `urllib.request`, `http.client`,
/// `socket.create_connection`, or that calls `requests.get`/`.post`
/// against a real URL is rejected at validation time, not at runtime —
/// the default MVP gate must hold without network connectivity, and
/// the only safe way to guarantee that is to keep the fixtures
/// themselves offline by construction.
///
/// Allowed escape hatches (each one represents a deliberate offline
/// pattern the manifest already relies on):
///
///   - `requests` with a mounted custom adapter against `mock://`
///     URLs (see `3p-requests-local_adapter_roundtrip`). The session's
///     `send()` is intercepted before urllib3 / sockets run; the
///     literal string `mock://` is the offline marker.
///   - Flask's `app.test_client()` — in-process WSGI, no sockets.
///   - pytest's `pytest.main([...])` against a `TemporaryDirectory`
///     test file — local filesystem only.
///   - click's `CliRunner.invoke(...)` — in-process parser callback,
///     no shell.
///
/// Anything else that imports a network module fails this gate. Live
/// integration / network checks belong behind an explicit opt-in flag
/// (the `--ignored` runner can host that as a separate gate).
#[test]
fn ecosystem_fixture_manifest_required_fixtures_are_no_network() {
    let doc = load_manifest();
    let fixtures = doc
        .get("fixtures")
        .and_then(|v| v.as_table())
        .expect("manifest [fixtures] table");

    let root = fixtures_root();

    // Patterns that unambiguously reach the network. Each pattern is a
    // substring match against the fixture source. Comments and string
    // literals would be false positives, but the manifest's fixtures
    // are tiny end-user scripts; if a comment ever mentions one of
    // these by accident, the right move is to rename the offending
    // identifier in the fixture, not to weaken the gate.
    //
    // `socket.create_connection` is the cleanest "I am about to open a
    // TCP connection" marker. `socket` alone is too coarse (sockets
    // are also used by Flask's test_client transitively).
    const BANNED: &[&str] = &[
        "urllib.request",
        "urllib.urlopen",
        "from http.client",
        "import http.client",
        "socket.create_connection",
        "socket.connect(",
        "ftplib",
        "smtplib",
        "telnetlib",
        "poplib",
        "imaplib",
        "nntplib",
        "xmlrpc.client",
    ];

    let mut violations: Vec<String> = Vec::new();

    for (id, entry) in fixtures {
        let Some(table) = entry.as_table() else { continue };
        let Some(relpath) = table.get("relpath").and_then(|v| v.as_str()) else {
            continue;
        };
        let abs = root.join(relpath);
        let Ok(src) = std::fs::read_to_string(&abs) else {
            // File existence is enforced by the schema test; if it's
            // missing here just skip rather than double-report.
            continue;
        };

        for needle in BANNED {
            if src.contains(needle) {
                violations.push(format!(
                    "  - {id} ({relpath}): contains banned network pattern \
                     {needle:?}. Required fixtures must be no-network. \
                     If this is a deliberate live-integration check, \
                     move it behind an explicit opt-in gate; required \
                     fixtures stay offline by construction."
                ));
            }
        }

        // requests is allowed *only* if it goes through a custom
        // adapter against a `mock://` URL. Catching this here means a
        // future requests-using fixture can't accidentally call
        // `requests.get("https://...")` and have it slip into the
        // required set.
        if src.contains("import requests") || src.contains("from requests") {
            let uses_mock_adapter = src.contains("mock://") && src.contains("HTTPAdapter");
            if !uses_mock_adapter {
                violations.push(format!(
                    "  - {id} ({relpath}): imports `requests` without a \
                     mounted custom adapter against a `mock://` URL. \
                     Required fixtures must not perform external HTTP I/O. \
                     Use `session.mount(\"mock://\", CustomAdapter())` \
                     or mark the fixture optional."
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} fixture(s) reach the network in the required set:\n{}",
        violations.len(),
        violations.join("\n"),
    );
}
