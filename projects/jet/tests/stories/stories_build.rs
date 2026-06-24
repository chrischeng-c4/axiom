// HANDWRITE-BEGIN gap="missing-generator:unit-test:5d65a1ff" tracker="pending-tracker" reason="Tests: build to a temp out_dir emits index.html + one preview per story + the transformed modules they import; emitted URLs are relative and resolve to files present in the output; dev jet stories behavior unaffected."
//! Integration tests for B4: `jet stories build` — static export of the
//! workbench (#190).
//!
//! These exercise the real [`jet::stories::build_stories_static`] against a temp
//! fixture project and cover:
//! (a) building writes `index.html` + one `preview/<id>.html` per discovered
//!     story + the transformed module file(s) the previews import,
//! (b) the manager sidebar + each preview reference RELATIVE paths that EXIST in
//!     the output (links resolve to emitted files),
//! (c) building twice is clean / idempotent (a stale file from a prior build is
//!     gone), and
//! (d) the dev `render_manager_html` / `render_preview_html` default output is
//!     unchanged — no absolute→relative regression for the dev server.

use std::fs;
use std::path::Path;

use jet::stories::{build_stories_static, discover};
use tempfile::TempDir;

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

fn write(path: std::path::PathBuf, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(path, contents).expect("write fixture");
}

/// Lay down two valid story fixtures + their components in nested dirs.
fn write_fixtures() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();
    write(root.join("src/components/Button.stories.tsx"), BUTTON_STORIES);
    write(root.join("src/surfaces/Card.stories.tsx"), CARD_STORIES);
    write(
        root.join("src/components/Button.tsx"),
        "export const Button = (props) => null;\n",
    );
    write(
        root.join("src/surfaces/Card.tsx"),
        "export const Card = (props) => null;\n",
    );
    dir
}

/// (a) Build writes index.html + one preview per story + the transformed modules.
#[test]
fn build_emits_manager_previews_and_modules() {
    let dir = write_fixtures();
    let out = dir.path().join("dist-stories");

    // Discover BEFORE building — the build emits `*.stories.js` modules that
    // would otherwise be re-picked-up by a post-build discover walk.
    let index = discover(dir.path());

    let result = build_stories_static(dir.path(), &out).expect("build");

    // Three stories discovered (Primary + Disabled from Button; WithFooter from Card).
    assert_eq!(result.story_count, index.stories.len());
    assert_eq!(result.story_count, 3, "Primary + Disabled + WithFooter");

    // Manager shell.
    assert!(out.join("index.html").is_file(), "index.html written");

    // One preview per story, by id.
    for story in &index.stories {
        let preview = out.join("preview").join(format!("{}.html", story.id));
        assert!(preview.is_file(), "preview for {} written: {:?}", story.id, preview);
    }

    // The transformed story modules + their imported components are emitted as JS.
    assert!(
        out.join("modules/src/components/Button.stories.js").is_file(),
        "Button.stories transformed to JS"
    );
    assert!(
        out.join("modules/src/components/Button.js").is_file(),
        "imported Button component transformed to JS"
    );
    assert!(
        out.join("modules/src/surfaces/Card.stories.js").is_file(),
        "Card.stories transformed to JS"
    );
    assert!(
        out.join("modules/src/surfaces/Card.js").is_file(),
        "imported Card component transformed to JS"
    );

    // The result lists every emitted relative path.
    assert!(result.emitted.iter().any(|p| p == Path::new("index.html")));
    assert!(result
        .emitted
        .iter()
        .any(|p| p == Path::new("modules/src/components/Button.js")));
}

/// (b) The manager + previews reference RELATIVE paths that exist in the output.
#[test]
fn emitted_urls_are_relative_and_resolve() {
    let dir = write_fixtures();
    let out = dir.path().join("dist-stories");
    build_stories_static(dir.path(), &out).expect("build");

    let manager = fs::read_to_string(out.join("index.html")).expect("read index");
    // No absolute dev-server routes leaked into the static manager.
    assert!(
        !manager.contains("/__jet_stories_preview"),
        "static manager must not reference dev routes"
    );
    // The iframe + sidebar link relative `preview/<id>.html` files that EXIST.
    assert!(manager.contains("preview/components-button--primary.html"));
    let primary_preview = out.join("preview/components-button--primary.html");
    assert!(primary_preview.is_file(), "linked preview exists");

    // The preview imports its module via a relative `../modules/...js` path that
    // resolves to an emitted file (preview/ → ../modules/).
    let preview = fs::read_to_string(&primary_preview).expect("read preview");
    assert!(
        preview.contains("../modules/src/components/Button.stories.js"),
        "preview imports the relative module url: {preview}"
    );
    assert!(
        out.join("modules/src/components/Button.stories.js").is_file(),
        "the imported module file exists"
    );
    // A static preview ships no HMR client / WebSocket.
    assert!(!preview.contains("WebSocket"), "no HMR WebSocket in static preview");

    // The emitted Button.stories module rewrites its `./Button` import to the
    // emitted `.js` sibling, which exists.
    let mod_js = fs::read_to_string(out.join("modules/src/components/Button.stories.js"))
        .expect("read module");
    assert!(
        mod_js.contains("./Button.js"),
        "relative import rewritten to emitted .js sibling: {mod_js}"
    );
    assert!(out.join("modules/src/components/Button.js").is_file());
}

/// (c) Building twice is clean — a stale file from a prior build is removed.
#[test]
fn rebuild_is_idempotent_and_cleans_stale_files() {
    let dir = write_fixtures();
    let out = dir.path().join("dist-stories");

    let first = build_stories_static(dir.path(), &out).expect("first build");

    // Drop a stale artifact into the output dir as if a previous build left it.
    let stale = out.join("preview/old--gone.html");
    write(stale.clone(), "<html>stale</html>");
    assert!(stale.is_file());

    let second = build_stories_static(dir.path(), &out).expect("second build");

    // The stale file is gone after a clean rebuild.
    assert!(!stale.exists(), "stale preview removed on rebuild");
    // The two builds emit the same set of files.
    assert_eq!(first.emitted, second.emitted, "rebuild is deterministic");
    assert_eq!(first.story_count, second.story_count);
}

/// (d) The dev renderers' default output is unchanged (no absolute→relative
/// regression for the dev server).
#[test]
fn dev_renderers_default_output_is_unchanged() {
    use jet::stories::manager::{render_manager_html, render_preview_html};
    use jet::stories::StoryEntry;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    let mut index = jet::stories::StoryIndex::default();
    let story = StoryEntry {
        id: "components-button--primary".into(),
        name: "Primary".into(),
        export_name: "Primary".into(),
        args: BTreeMap::new(),
        has_render: false,
        file: PathBuf::from("/x/Button.stories.tsx"),
        title_path: vec!["Components".into(), "Button".into()],
    };
    index.stories.push(story.clone());

    // The dev manager still emits absolute dev-server preview routes.
    let manager = render_manager_html(&index, None, &[]);
    assert!(
        manager.contains("/__jet_stories_preview/components-button--primary"),
        "dev manager keeps absolute routes"
    );
    assert!(!manager.contains("preview/components-button--primary.html"));

    // The dev preview still imports the absolute module URL + ships the HMR client.
    let preview = render_preview_html(&story, "/src/Button.stories.tsx");
    assert!(preview.contains("import * as Story from \"/src/Button.stories.tsx\""));
    assert!(preview.contains("HMR connected"), "dev preview keeps the HMR client");
}
// HANDWRITE-END
