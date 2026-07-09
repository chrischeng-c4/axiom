// <HANDWRITE gap="codegen:browser-semantic-surface" tracker="jet-bb-semantic-surface" reason="Companion integration test for the handwritten semantic Browser Bridge surface; becomes CODEGEN with the interact module.">
//! Live-browser contract for the semantic `jet bb` surface
//! (playwright-mcp-shaped): snapshot/ref interaction, navigation, and
//! the console/fetch observability rings.
//!
//! One end-to-end flow against a real Chromium on a local fixture page:
//!
//! 1. `snapshot` mints refs for the interactable elements.
//! 2. `fill`/`select`/`check`/`click` work by ref AND by locator
//!    selector, and the page state proves each action landed.
//! 3. `console`/`requests` return the activity the init-script rings
//!    buffered — including after `goto`/`reload`, which must re-arm the
//!    hooks on the new document (CDP drops new-document scripts when
//!    the registering session disconnects).
//! 4. A ref minted before navigation fails with the re-snapshot hint
//!    instead of acting on a dead element.
//!
//! Skips gracefully when Chromium is absent, matching the repo's
//! real-services-over-mocks policy.

use jet::browser_cli::interact::{self, Target};
use jet::browser_cli::prepare_session;

fn chromium_available() -> bool {
    if std::env::var("CHROME_PATH").is_ok() {
        return true;
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let xdg = std::env::var("XDG_CACHE_HOME").unwrap_or_else(|_| format!("{home}/.cache"));
    if std::path::Path::new(&format!("{home}/Library/Caches/ms-playwright")).exists() {
        return true;
    }
    if std::path::Path::new(&format!("{xdg}/ms-playwright")).exists() {
        return true;
    }
    if std::path::Path::new(&format!("{home}/.cache/jet/chromium")).exists() {
        return true;
    }
    false
}

const PAGE_ONE: &str = r#"<!doctype html>
<html><head><meta charset="utf-8"><title>BB Semantic Fixture</title></head>
<body>
  <h1>Demo Form</h1>
  <form>
    <label for="name">Name</label>
    <input id="name" type="text" placeholder="your name">
    <select id="lang">
      <option value="">choose</option>
      <option value="zh">Chinese</option>
      <option value="en">English</option>
    </select>
    <input id="agree" type="checkbox">
    <button id="save" type="button"
      onclick="window.__clicks=(window.__clicks||0)+1; console.log('saved', document.getElementById('name').value); fetch('data:text/plain,pong').catch(()=>{})">
      Save</button>
  </form>
  <a id="next" href="page2.html">Next page</a>
</body></html>"#;

const PAGE_TWO: &str = r#"<!doctype html>
<html><head><meta charset="utf-8"><title>Page Two</title></head>
<body><h1>Second</h1><p>arrived</p></body></html>"#;

#[tokio::test]
async fn semantic_surface_drives_a_real_page_end_to_end() {
    if !chromium_available() {
        eprintln!("skipping: Chromium not installed");
        return;
    }
    let fixture = tempfile::tempdir().expect("fixture dir");
    std::fs::write(fixture.path().join("index.html"), PAGE_ONE).expect("writing page one");
    std::fs::write(fixture.path().join("page2.html"), PAGE_TWO).expect("writing page two");
    let url_one = format!("file://{}/index.html", fixture.path().display());
    let url_two = format!("file://{}/page2.html", fixture.path().display());

    // The session root holds .jet/browser-session.json — the same
    // re-dial model every `jet bb` command uses.
    let root = tempfile::tempdir().expect("session root");
    let Ok(browser) = prepare_session(root.path(), &url_one).await else {
        eprintln!("skipping: Browser launch failed");
        return;
    };

    // 1. Snapshot mints refs and parks the map on the page.
    let snap = interact::snapshot(root.path()).await.expect("snapshot");
    let text = snap["snapshot"].as_str().expect("snapshot text");
    assert!(
        snap["ref_count"].as_u64().unwrap_or(0) >= 5,
        "snapshot: {text}"
    );
    assert!(text.contains("heading \"Demo Form\""), "snapshot: {text}");
    assert!(text.contains("button \"Save\""), "snapshot: {text}");
    let save_ref = text
        .lines()
        .find(|l| l.contains("button \"Save\""))
        .and_then(|l| l.split("[ref=").nth(1))
        .and_then(|l| l.split(']').next())
        .expect("Save button must carry a ref")
        .to_string();
    let name_ref = text
        .lines()
        .find(|l| l.contains("textbox"))
        .and_then(|l| l.split("[ref=").nth(1))
        .and_then(|l| l.split(']').next())
        .expect("name input must carry a ref")
        .to_string();

    // 2. Act by ref and by selector; verify on the page itself.
    interact::fill(root.path(), &Target::Ref(name_ref), "Chris")
        .await
        .expect("fill by ref");
    interact::select(root.path(), &Target::Selector("#lang".into()), "Chinese")
        .await
        .expect("select by css + label");
    interact::set_checked(root.path(), &Target::Selector("#agree".into()), true)
        .await
        .expect("check by css");
    interact::click(root.path(), &Target::Ref(save_ref.clone()), false)
        .await
        .expect("click by ref");
    interact::click(
        root.path(),
        &Target::Selector("role=button[name=\"Save\"]".into()),
        false,
    )
    .await
    .expect("click by role selector");

    let page = jet::browser_cli::attach(root.path()).await.expect("attach");
    let state = page
        .evaluate(
            "JSON.stringify({ name: document.getElementById('name').value, \
             lang: document.getElementById('lang').value, \
             agree: document.getElementById('agree').checked, \
             clicks: window.__clicks })",
        )
        .await
        .expect("state probe");
    let state: serde_json::Value =
        serde_json::from_str(state.as_str().expect("state json")).expect("state parse");
    assert_eq!(state["name"], "Chris");
    assert_eq!(state["lang"], "zh");
    assert_eq!(state["agree"], true);
    assert_eq!(state["clicks"], 2, "both click paths must land");

    // 3. Observability rings captured the click side effects.
    let console = interact::console(root.path(), Some("log"), 10, false)
        .await
        .expect("console read");
    let entries = console["entries"].as_array().expect("console entries");
    assert!(
        entries.iter().any(|e| e["text"]
            .as_str()
            .is_some_and(|t| t.contains("saved Chris"))),
        "console must hold the click log: {console}"
    );
    let requests = interact::requests(root.path(), 10, false)
        .await
        .expect("requests read");
    assert!(
        requests["entries"]
            .as_array()
            .expect("request entries")
            .iter()
            .any(|e| e["api"] == "fetch"),
        "requests must hold the fetch the click issued: {requests}"
    );

    // 4. Navigation: goto + wait-for-text, then hooks must still work
    //    on the new document (re-armed by the navigating verb).
    interact::goto(root.path(), &url_two)
        .await
        .expect("goto page two");
    interact::wait(root.path(), None, Some("arrived"), None, 5_000)
        .await
        .expect("wait for page-two text");
    let page = jet::browser_cli::attach(root.path())
        .await
        .expect("re-attach");
    page.evaluate("console.warn('on page two') || true")
        .await
        .expect("warn on page two");
    let console = interact::console(root.path(), Some("warn"), 10, false)
        .await
        .expect("console read after navigation");
    assert!(
        console.get("note").is_none(),
        "hooks must survive goto without lazy healing: {console}"
    );
    assert!(
        console["entries"]
            .as_array()
            .expect("entries")
            .iter()
            .any(|e| e["text"] == "on page two"),
        "post-navigation console must capture: {console}"
    );

    // 5. Stale refs minted on page one fail with the re-snapshot hint.
    let err = interact::click(root.path(), &Target::Ref(save_ref), false)
        .await
        .expect_err("page-one ref must not act on page two");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("snapshot"),
        "stale-ref error must tell the agent to re-snapshot: {msg}"
    );

    // 6. History: back lands on page one with its form intact.
    interact::history_step(root.path(), -1).await.expect("back");
    interact::wait(root.path(), Some("#save"), None, None, 5_000)
        .await
        .expect("wait for page-one form after back");

    browser.close().await.expect("closing browser");
}
// </HANDWRITE>
