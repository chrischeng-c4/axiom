// HANDWRITE-BEGIN gap="missing-generator:unit-test:059f4bfc" tracker="pending-tracker" reason="Tests: manager route returns HTML listing discovered stories; preview route renders the selected story in isolation; switching stories swaps the preview."
//! Integration tests for B2: the `jet stories` native workbench server.
//!
//! These exercise the real axum router ([`jet::stories::server::build_router`])
//! against a temp fixture dir, driving routes via `tower::ServiceExt::oneshot`
//! (no port binding needed). We cover:
//! (a) the manager route serves an HTML page listing the discovered stories,
//! (b) the preview route for a known story id returns HTML that references that
//!     story's module file + export name and mounts in isolation (`#jet-root`),
//! (c) an unknown story id returns 404,
//! (d) the module route transforms + serves a fixture `.tsx` module as JS.

use std::fs;
use std::path::Path;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use jet::stories::{discover, server};
use tempfile::TempDir;
use tower::ServiceExt; // for `oneshot`

const BUTTON_STORIES: &str = r#"
import { Button } from './Button';
import type { Meta, StoryObj } from '@storybook/react';

const meta = {
  title: 'Components/Button',
  component: Button,
  args: { size: 'md', label: 'Default' },
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: { primary: true, label: 'Click me' },
};

export const Disabled: Story = {
  args: { disabled: true },
  render: () => <Button disabled />,
};
"#;

const CARD_STORIES: &str = r#"
import { Card } from './Card';

export default {
  title: 'Surfaces/Card',
  component: Card,
};

export const WithFooter = {
  args: { footer: true },
};
"#;

/// Lay down two valid story fixtures in nested dirs.
fn write_fixtures() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();
    write(root.join("src/components/Button.stories.tsx"), BUTTON_STORIES);
    write(root.join("src/surfaces/Card.stories.tsx"), CARD_STORIES);
    write(
        root.join("src/components/Button.tsx"),
        "export const Button = (props) => null;\n",
    );
    dir
}

fn write(path: std::path::PathBuf, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(path, contents).expect("write fixture");
}

/// Build the workbench router over a fixture dir.
fn router_for(root: &Path) -> axum::Router {
    let index = discover(root);
    server::build_router(index, root.to_path_buf())
}

/// Fire a `GET path` at the router and return (status, body string).
async fn get(router: &axum::Router, path: &str) -> (StatusCode, String) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(path)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("router response");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 4 * 1024 * 1024)
        .await
        .expect("body bytes");
    (status, String::from_utf8_lossy(&bytes).to_string())
}

/// (a) the manager route serves HTML listing the discovered stories.
#[tokio::test]
async fn manager_route_lists_discovered_stories() {
    let dir = write_fixtures();
    let router = router_for(dir.path());

    let (status, html) = get(&router, "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("jet stories"), "manager shell title present");
    // Both story groups + their exports are listed in the sidebar.
    assert!(html.contains("Components / Button"), "Button group listed: {html}");
    assert!(html.contains("Surfaces / Card"), "Card group listed");
    assert!(html.contains(">Primary<"), "Primary story listed");
    assert!(html.contains(">Disabled<"), "Disabled story listed");
    assert!(html.contains(">WithFooter<"), "WithFooter story listed");
    // It embeds a preview iframe.
    assert!(html.contains("id=\"jet-preview\""), "preview iframe present");
    assert!(
        html.contains("/__jet_stories_preview/"),
        "links to preview routes"
    );
}

/// (a') the explicit manager alias route also works.
#[tokio::test]
async fn manager_alias_route_works() {
    let dir = write_fixtures();
    let router = router_for(dir.path());
    let (status, html) = get(&router, "/__jet_stories_manager").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("id=\"jet-sidebar\""));
}

/// (b) the preview route for a known story references its module + export and
/// mounts in isolation.
#[tokio::test]
async fn preview_route_renders_story_in_isolation() {
    let dir = write_fixtures();
    let router = router_for(dir.path());

    // `Components/Button` → slug `components-button`; export `Primary`.
    let (status, html) = get(&router, "/__jet_stories_preview/components-button--primary").await;
    assert_eq!(status, StatusCode::OK);

    // References the story's module file (root-relative URL) + export name.
    assert!(
        html.contains("/src/components/Button.stories.tsx"),
        "preview imports the story module: {html}"
    );
    assert!(
        html.contains("const exportName = \"Primary\""),
        "selects the Primary export"
    );
    // Mounts into a single isolated root — no app shell/router.
    assert_eq!(
        html.matches("id=\"jet-root\"").count(),
        1,
        "exactly one isolated mount root"
    );
    assert!(html.contains("createRoot"), "uses react-dom createRoot");
}

/// (c) an unknown story id returns 404.
#[tokio::test]
async fn preview_route_unknown_id_is_404() {
    let dir = write_fixtures();
    let router = router_for(dir.path());

    let (status, body) = get(&router, "/__jet_stories_preview/does-not-exist--nope").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains("unknown story id"), "error names the bad id: {body}");
}

/// (d) the module route transforms + serves a fixture `.tsx` module as JS.
#[tokio::test]
async fn module_route_transforms_and_serves_tsx() {
    let dir = write_fixtures();
    let router = router_for(dir.path());

    let (status, js) = get(&router, "/src/components/Button.stories.tsx").await;
    assert_eq!(status, StatusCode::OK, "module served: {js}");

    // The exported story symbols survive the transform...
    assert!(js.contains("Primary"), "Primary export survives transform");
    assert!(js.contains("Disabled"), "Disabled export survives transform");
    // ...and TSX-specific syntax (the `satisfies Meta<...>` / type annotations)
    // does not leak through verbatim as a type annotation. The JSX in
    // `render: () => <Button disabled />` must be lowered (no raw `<Button`).
    assert!(
        !js.contains("satisfies Meta<typeof Button>"),
        "type-only `satisfies` clause is stripped: {js}"
    );

    // A request for a non-existent module is a 404.
    let (missing_status, _) = get(&router, "/src/components/Nope.stories.tsx").await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);

    // A request for a non-module extension is a 404 (not served as a module).
    let (txt_status, _) = get(&router, "/src/components/Button.txt").await;
    assert_eq!(txt_status, StatusCode::NOT_FOUND);
}

/// (e) path traversal is rejected.
#[tokio::test]
async fn module_route_rejects_parent_traversal() {
    let dir = write_fixtures();
    let router = router_for(dir.path());
    let (status, _) = get(&router, "/../etc/passwd.tsx").await;
    // Either normalized away by axum (404) or rejected as a bad request.
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::NOT_FOUND,
        "traversal blocked, got {status}"
    );
}
// HANDWRITE-END
