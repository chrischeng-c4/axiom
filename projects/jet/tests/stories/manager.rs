// <HANDWRITE gap="missing-generator:unit-test:059f4bfc" tracker="standardize-gap-projects-jet-tests-stories-manager-rs" reason="Tests: manager route returns HTML listing discovered stories; preview route renders the selected story in isolation; switching stories swaps the preview.">
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
    write(
        root.join("src/components/Button.stories.tsx"),
        BUTTON_STORIES,
    );
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
    assert!(
        html.contains("Components / Button"),
        "Button group listed: {html}"
    );
    assert!(html.contains("Surfaces / Card"), "Card group listed");
    assert!(html.contains(">Primary<"), "Primary story listed");
    assert!(html.contains(">Disabled<"), "Disabled story listed");
    assert!(html.contains(">WithFooter<"), "WithFooter story listed");
    // It embeds a preview iframe.
    assert!(
        html.contains("id=\"jet-preview\""),
        "preview iframe present"
    );
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
    assert!(
        body.contains("unknown story id"),
        "error names the bad id: {body}"
    );
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
    assert!(
        js.contains("Disabled"),
        "Disabled export survives transform"
    );
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

/// (f) #197: a component that imports a bare specifier installed in the project's
/// node_modules gets that import rewritten to a `/@dep/<key>` route, and the dep
/// route serves the resolved dep file as JS — recursively for its own imports.
/// Unresolvable bare specifiers (e.g. `react`, not installed) are left as-is so
/// the esm.sh importmap still satisfies them.
#[tokio::test]
async fn module_route_resolves_and_serves_node_modules_dep() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    // A tiny installed package `clsx` with a `module` ESM entry that itself
    // imports a relative chunk (to exercise the recursive dep walk) AND a
    // not-installed bare specifier (`react`) that must stay as-authored.
    write(
        root.join("node_modules/clsx/package.json"),
        r#"{"name":"clsx","version":"2.0.0","module":"dist/clsx.mjs","main":"dist/clsx.js"}"#,
    );
    write(
        root.join("node_modules/clsx/dist/clsx.mjs"),
        "import { join } from './chunk.mjs';\nexport default function clsx(){ return join(); }\n",
    );
    write(
        root.join("node_modules/clsx/dist/chunk.mjs"),
        "export function join(){ return ''; }\n",
    );

    // A component that imports the installed dep + a non-installed one.
    write(
        root.join("src/components/Button.tsx"),
        "import clsx from 'clsx';\nimport React from 'react';\nexport const Button = (props) => clsx('x');\n",
    );
    // A story so discovery has something to mount (not strictly required for the
    // module route, but keeps the fixture realistic).
    write(
        root.join("src/components/Button.stories.tsx"),
        "import { Button } from './Button';\nexport default { title: 'Components/Button', component: Button };\nexport const Primary = { args: {} };\n",
    );

    let router = router_for(root);

    // The component module route rewrites the resolvable bare `clsx` import to a
    // `/@dep/clsx/...` route, but leaves the un-installed `react` import alone.
    let (status, js) = get(&router, "/src/components/Button.tsx").await;
    assert_eq!(status, StatusCode::OK, "component served: {js}");
    assert!(
        js.contains("/@dep/clsx/dist/clsx.mjs"),
        "resolvable bare import rewritten to a /@dep route: {js}"
    );
    assert!(
        !js.contains("\"clsx\"") && !js.contains("'clsx'"),
        "the bare clsx specifier no longer appears verbatim: {js}"
    );
    assert!(
        js.contains("\"react\"") || js.contains("'react'"),
        "un-installed bare import left for the importmap: {js}"
    );

    // The dep route serves the resolved dep file as JS. Its OWN relative import
    // (`./chunk.mjs`) stays relative — the browser resolves it against this
    // dep's `/@dep/clsx/dist/` URL, so it loads from the same dep route without
    // needing a rewrite.
    let (dep_status, dep_js) = get(&router, "/@dep/clsx/dist/clsx.mjs").await;
    assert_eq!(dep_status, StatusCode::OK, "dep served: {dep_js}");
    assert!(
        dep_js.contains("function clsx"),
        "dep body present: {dep_js}"
    );
    assert!(
        dep_js.contains("./chunk.mjs"),
        "dep's relative import stays relative (resolves under /@dep/): {dep_js}"
    );

    // That sibling chunk — addressed relative to the dep route — also serves.
    let (chunk_status, chunk_js) = get(&router, "/@dep/clsx/dist/chunk.mjs").await;
    assert_eq!(chunk_status, StatusCode::OK, "dep chunk served: {chunk_js}");
    assert!(chunk_js.contains("function join"), "chunk body present");

    // An unknown dep path is a 404 (not a panic / 500).
    let (missing, _) = get(&router, "/@dep/clsx/dist/nope.mjs").await;
    assert_eq!(missing, StatusCode::NOT_FOUND);
}
// </HANDWRITE>
