// <HANDWRITE gap="missing-generator:unit-test:5d65a1ff" tracker="standardize-gap-projects-jet-tests-stories-stories-build-rs" reason="Tests: build to a temp out_dir emits index.html + one preview per story + the transformed modules they import; emitted URLs are relative and resolve to files present in the output; dev jet stories behavior unaffected.">
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
    write(
        root.join("src/components/Button.stories.tsx"),
        BUTTON_STORIES,
    );
    write(root.join("src/surfaces/Card.stories.tsx"), CARD_STORIES);
    write(
        root.join("src/components/Button.tsx"),
        r#"
export interface ButtonProps {
  /** Main label shown in the button. */
  label: string;
  /** Emphasize the button. */
  primary?: boolean;
}

/**
 * Button component description.
 */
export const Button = (props: ButtonProps) => null;
"#,
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
    assert!(out.join("index.json").is_file(), "index.json written");
    let manager = fs::read_to_string(out.join("index.html")).expect("read manager");
    assert!(manager.contains("class=\"jet-docs-link\""));
    assert!(manager.contains("Button component description."));
    assert!(manager.contains("Main label shown in the button."));
    assert!(manager.contains(r#"<span class="jet-docs-pill">string</span>"#));
    assert!(manager.contains("preview/components-button--primary.html"));
    let index_json = fs::read_to_string(out.join("index.json")).expect("read index json");
    assert!(index_json.contains("\"schemaVersion\":1"));
    assert!(index_json.contains("\"id\":\"components-button--primary\""));
    assert!(index_json.contains("\"importPath\":\"./src/components/Button.stories.tsx\""));
    assert!(index_json.contains("\"docs\":["));

    // One preview per story, by id.
    for story in &index.stories {
        let preview = out.join("preview").join(format!("{}.html", story.id));
        assert!(
            preview.is_file(),
            "preview for {} written: {:?}",
            story.id,
            preview
        );
    }

    // The transformed story modules + their imported components are emitted as JS.
    assert!(
        out.join("modules/src/components/Button.stories.js")
            .is_file(),
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
    assert!(result.emitted.iter().any(|p| p == Path::new("index.json")));
    assert!(result
        .emitted
        .iter()
        .any(|p| p == Path::new("modules/src/components/Button.js")));
}

#[test]
fn static_manager_keeps_dev_feature_parity_checklist() {
    let dir = write_fixtures();
    let out = dir.path().join("dist-stories");
    build_stories_static(dir.path(), &out).expect("build");
    let manager = fs::read_to_string(out.join("index.html")).expect("read manager");
    let index_json = fs::read_to_string(out.join("index.json")).expect("read index json");
    let checklist = [
        ("toolbar", "id=\"jet-viewport\""),
        ("controls panel", "id=\"jet-controls-body\""),
        ("actions panel", "id=\"jet-actions-log\""),
        ("interactions panel", "id=\"jet-interactions-log\""),
        ("a11y panel", "id=\"jet-a11y-log\""),
        ("source panel", "id=\"jet-source-code\""),
        ("docs pages", "class=\"jet-docs-link\""),
        ("manager search", "id=\"jet-search\""),
        ("theme toggle", "id=\"jet-theme-toggle\""),
        ("index artifact", "\"schemaVersion\":1"),
    ];
    for (feature, needle) in checklist {
        let haystack = if feature == "index artifact" {
            &index_json
        } else {
            &manager
        };
        assert!(
            haystack.contains(needle),
            "{feature} missing from static parity checklist"
        );
    }
}

#[test]
fn mdx_docs_pages_render_core_blocks_in_static_export() {
    let dir = write_fixtures();
    write(
        dir.path().join("src/components/Button.mdx"),
        r#"
<Meta title="Components/Button" />

# Button usage
Use the primary button for the main action.

<Canvas of={ButtonStories.Primary} />
<Story id="components-button--disabled" />
<ArgTypes of={ButtonStories} />
<Source of={ButtonStories.Primary} />
"#,
    );
    let out = dir.path().join("dist-stories");
    build_stories_static(dir.path(), &out).expect("build");

    let manager = fs::read_to_string(out.join("index.html")).expect("read manager");
    assert!(manager.contains("data-docs-id=\"mdx-components-button\""));
    assert!(manager.contains("Button usage"));
    assert!(manager.contains("Use the primary button for the main action."));
    assert!(manager.contains("preview/components-button--primary.html"));
    assert!(manager.contains("preview/components-button--disabled.html"));
    assert!(manager.contains("Main label shown in the button."));
    assert!(manager.contains("export const Primary"));
}

#[test]
fn broken_mdx_surfaces_file_named_diagnostic() {
    let dir = write_fixtures();
    write(
        dir.path().join("src/components/Broken.mdx"),
        r#"
<Meta title="Components/Button" />
<CustomThing />
"#,
    );

    let index = discover(dir.path());
    assert!(
        index.diagnostics.iter().any(|diag| {
            diag.contains("src/components/Broken.mdx")
                && diag.contains("unsupported MDX JSX tag <CustomThing>")
        }),
        "diagnostics should name broken MDX file and unsupported syntax: {:?}",
        index.diagnostics
    );
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
        out.join("modules/src/components/Button.stories.js")
            .is_file(),
        "the imported module file exists"
    );
    // A static preview ships no HMR client / WebSocket.
    assert!(
        !preview.contains("WebSocket"),
        "no HMR WebSocket in static preview"
    );
    assert!(
        !preview.contains("/@react-refresh") && !preview.contains("RefreshRuntime"),
        "static preview must not reference the dev-only refresh runtime: {preview}"
    );

    // The emitted Button.stories module rewrites its `./Button` import to the
    // emitted `.js` sibling, which exists.
    let mod_js = fs::read_to_string(out.join("modules/src/components/Button.stories.js"))
        .expect("read module");
    assert!(
        mod_js.contains("./Button.js"),
        "relative import rewritten to emitted .js sibling: {mod_js}"
    );
    assert!(
        !mod_js.contains("/@react-refresh")
            && !mod_js.contains("RefreshRuntime")
            && !mod_js.contains("enqueueUpdate"),
        "static emitted modules must not carry React Fast Refresh injection: {mod_js}"
    );
    assert!(out.join("modules/src/components/Button.js").is_file());
}

#[test]
fn static_build_emits_project_preview_runtime_module() {
    let dir = write_fixtures();
    let root = dir.path();
    write(
        root.join(".storybook/preview.tsx"),
        r#"
import React from 'react';

export const globalTypes = { theme: { defaultValue: 'dark' } };
export const globals = { locale: 'en' };
export const parameters = { layout: 'centered' };
export const loaders = [async () => ({ fromPreview: true })];
export const decorators = [
  (Story, context) => <div data-theme={context.globals.theme}><Story /></div>,
];
"#,
    );

    let out = root.join("dist-stories");
    let result = build_stories_static(root, &out).expect("build");

    let preview_runtime = Path::new("modules/.storybook/preview.js");
    assert!(
        out.join(preview_runtime).is_file(),
        "project preview module is emitted"
    );
    assert!(result.emitted.iter().any(|p| p == preview_runtime));

    let primary_preview = fs::read_to_string(out.join("preview/components-button--primary.html"))
        .expect("read preview");
    assert!(
        primary_preview.contains(
            r#"import * as ProjectPreviewModule from "../modules/.storybook/preview.js";"#
        ) && primary_preview.contains(
            "const ProjectPreview = ProjectPreviewModule.default || ProjectPreviewModule;",
        ),
        "static preview imports emitted project preview runtime: {primary_preview}"
    );
    assert!(
        primary_preview.contains("ProjectPreview.decorators")
            && primary_preview.contains("ProjectPreview.globalTypes")
            && primary_preview.contains("ProjectPreview.loaders")
            && primary_preview.contains("applyLayout(parameters.layout)"),
        "static preview runtime applies CSF render-path core"
    );
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

/// (e) #197: a component importing a bare specifier installed in node_modules
/// gets the resolved dep emitted under `out_dir/deps/<key>.js`, and the emitting
/// module references it via a RELATIVE path that EXISTS — recursively for the
/// dep's own relative imports. Un-installed bare specifiers (e.g. `react`) are
/// left as-authored for the esm.sh importmap.
#[test]
fn build_emits_resolved_node_modules_dep_with_relative_url() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    // A tiny installed package `clsx` whose ESM entry imports a relative chunk.
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

    // A component that imports the installed dep + a non-installed one, and the
    // story that mounts it.
    write(
        root.join("src/components/Button.tsx"),
        "import clsx from 'clsx';\nimport React from 'react';\nexport const Button = (props) => clsx('x');\n",
    );
    write(
        root.join("src/components/Button.stories.tsx"),
        "import { Button } from './Button';\nexport default { title: 'Components/Button', component: Button };\nexport const Primary = { args: {} };\n",
    );

    let out = root.join("dist-stories");
    let result = build_stories_static(root, &out).expect("build");

    // The resolved dep + its transitive chunk are emitted under deps/.
    let dep_main = out.join("deps/clsx/dist/clsx.js");
    let dep_chunk = out.join("deps/clsx/dist/chunk.js");
    assert!(dep_main.is_file(), "resolved dep emitted: {:?}", dep_main);
    assert!(
        dep_chunk.is_file(),
        "dep's transitive chunk emitted: {:?}",
        dep_chunk
    );
    assert!(result
        .emitted
        .iter()
        .any(|p| p == Path::new("deps/clsx/dist/clsx.js")));

    // The emitting component module rewrites the bare `clsx` import to a RELATIVE
    // path into the deps/ tree that EXISTS, and leaves the un-installed `react`
    // import as-authored (importmap).
    let component = fs::read_to_string(out.join("modules/src/components/Button.js"))
        .expect("read component module");
    assert!(
        component.contains("../../../deps/clsx/dist/clsx.js"),
        "bare dep import rewritten to relative deps url: {component}"
    );
    assert!(
        !component.contains("\"clsx\"") && !component.contains("'clsx'"),
        "the bare clsx specifier no longer appears verbatim: {component}"
    );
    assert!(
        component.contains("\"react\"") || component.contains("'react'"),
        "un-installed bare import kept for the importmap: {component}"
    );

    // The dep's own relative import is rewritten to its emitted `.js` sibling,
    // which exists.
    let dep_js = fs::read_to_string(&dep_main).expect("read dep module");
    assert!(
        dep_js.contains("./chunk.js"),
        "dep's relative import rewritten to emitted .js sibling: {dep_js}"
    );
}

#[test]
// @spec .aw/tech-design/projects/jet/logic/jet-stories-build-scss-is-never-compiled-scss-files-copied-verba.md#unit-test
fn build_compiles_scss_side_effect_imports_to_static_css() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    write(
        root.join("src/components/Button.stories.tsx"),
        "import { Button } from './Button';\nexport default { title: 'Components/Button', component: Button };\nexport const Primary = { args: {} };\n",
    );
    write(
        root.join("src/components/Button.tsx"),
        "import './Button.scss';\nexport const Button = () => null;\n",
    );
    write(
        root.join("src/components/Button.scss"),
        "$radius: 4px;\n.button { &--primary { border-radius: $radius; } }\n",
    );

    let out = root.join("dist-stories");
    let result = build_stories_static(root, &out).expect("build");

    let css_rel = Path::new("modules/src/components/Button.css");
    let css_path = out.join(css_rel);
    assert!(
        css_path.is_file(),
        "SCSS side-effect import must emit a CSS asset"
    );
    assert!(result.emitted.iter().any(|p| p == css_rel));

    let css = fs::read_to_string(&css_path).expect("read css");
    assert!(
        css.contains(".button--primary") && css.contains("border-radius") && css.contains("4px"),
        "SCSS nesting and variables should be compiled, got:\n{css}"
    );
    assert!(
        !out.join("modules/src/components/Button.scss.js").exists(),
        "SCSS must not be emitted as a fake JS module"
    );

    let component_js =
        fs::read_to_string(out.join("modules/src/components/Button.js")).expect("read component");
    assert!(
        !component_js.contains("Button.scss"),
        "emitted JS should not import SCSS as a JS module: {component_js}"
    );

    let preview = fs::read_to_string(out.join("preview/components-button--primary.html"))
        .expect("read preview");
    assert!(
        preview
            .contains(r#"<link rel="stylesheet" href="../modules/src/components/Button.css" />"#),
        "preview should link emitted CSS with a relative URL: {preview}"
    );
}

#[test]
// @spec projects/jet/tech-design/logic/jet-stories-build-png-now-fixed-svg-partially-fixed-barrel-re-ex.md#unit-test
fn build_compiles_bare_specifier_css_import_to_static_css() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    write(
        root.join("node_modules/@tw-tech/ds2/package.json"),
        r#"{
  "name": "@tw-tech/ds2",
  "version": "1.0.0",
  "exports": {
    "./style.css": "./style.css"
  }
}"#,
    );
    write(
        root.join("node_modules/@tw-tech/ds2/style.css"),
        ".widget { color: blue; }\n",
    );

    write(
        root.join("src/components/Widget.stories.tsx"),
        "import { Widget } from './Widget';\nexport default { title: 'Components/Widget', component: Widget };\nexport const Primary = { args: {} };\n",
    );
    write(
        root.join("src/components/Widget.tsx"),
        "import '@tw-tech/ds2/style.css';\nexport const Widget = () => null;\n",
    );

    let out = root.join("dist-stories");
    let result = build_stories_static(root, &out).expect("build");

    let css_rel = Path::new("deps/@tw-tech/ds2/style.css");
    let css_path = out.join(css_rel);
    assert!(
        css_path.is_file(),
        "bare-specifier CSS import (via package.json exports) must emit a real CSS asset"
    );
    assert!(result.emitted.iter().any(|p| p == css_rel));

    let css = fs::read_to_string(&css_path).expect("read css");
    assert!(
        css.contains(".widget") && css.contains("color"),
        "compiled CSS should contain the source rule, got:\n{css}"
    );
    assert!(
        !out.join("deps/@tw-tech/ds2/style.css.js").exists(),
        "bare-specifier CSS must not fall through to a dangling deps/<pkg>/style.css.js dependency reference"
    );

    let component_js =
        fs::read_to_string(out.join("modules/src/components/Widget.js")).expect("read component");
    assert!(
        !component_js.contains("style.css"),
        "emitted JS should not keep a dangling import of the bare-specifier CSS: {component_js}"
    );

    let preview = fs::read_to_string(out.join("preview/components-widget--primary.html"))
        .expect("read preview");
    assert!(
        preview.contains(r#"<link rel="stylesheet" href="../deps/@tw-tech/ds2/style.css" />"#),
        "preview should link the compiled bare-specifier CSS with a relative URL: {preview}"
    );
}

#[test]
// @spec projects/jet/tech-design/logic/jet-stories-build-png-now-fixed-svg-partially-fixed-barrel-re-ex.md#unit-test
fn build_compiles_bare_specifier_scss_import_via_css_pipeline() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    write(
        root.join("node_modules/@tw-tech/ds3/package.json"),
        r#"{
  "name": "@tw-tech/ds3",
  "version": "1.0.0",
  "exports": {
    "./theme.scss": "./theme.scss"
  }
}"#,
    );
    write(
        root.join("node_modules/@tw-tech/ds3/theme.scss"),
        "$radius: 4px;\n.theme { &--rounded { border-radius: $radius; } }\n",
    );

    write(
        root.join("src/components/Theme.stories.tsx"),
        "import { Theme } from './Theme';\nexport default { title: 'Components/Theme', component: Theme };\nexport const Primary = { args: {} };\n",
    );
    write(
        root.join("src/components/Theme.tsx"),
        "import '@tw-tech/ds3/theme.scss';\nexport const Theme = () => null;\n",
    );

    let out = root.join("dist-stories");
    let result = build_stories_static(root, &out).expect("build");

    let css_rel = Path::new("deps/@tw-tech/ds3/theme.css");
    let css_path = out.join(css_rel);
    assert!(
        css_path.is_file(),
        "bare-specifier SCSS import must be compiled to a real CSS asset"
    );
    assert!(result.emitted.iter().any(|p| p == css_rel));

    let css = fs::read_to_string(&css_path).expect("read css");
    assert!(
        css.contains(".theme--rounded") && css.contains("border-radius") && css.contains("4px"),
        "SCSS nesting and variables should be compiled through the real CssPipeline, got:\n{css}"
    );
    assert!(
        !out.join("deps/@tw-tech/ds3/theme.scss").exists(),
        "raw Sass must not be copied verbatim"
    );
    assert!(
        !out.join("deps/@tw-tech/ds3/theme.scss.js").exists(),
        "bare-specifier SCSS must not fall through to a dangling deps/<pkg>/theme.scss.js dependency reference"
    );
}

#[test]
// @spec .aw/tech-design/projects/jet/logic/jet-stories-build-scss-is-never-compiled-scss-files-copied-verba.md#unit-test
fn build_emits_svg_and_png_assets_as_url_strings() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    write(
        root.join("node_modules/@tw-tech/shared-assets/package.json"),
        r#"{
  "name": "@tw-tech/shared-assets",
  "version": "1.0.0"
}"#,
    );
    write(
        root.join("node_modules/@tw-tech/shared-assets/images/empty-default.png"),
        "png-bytes",
    );

    write(
        root.join("src/components/AssetBox.stories.tsx"),
        "import { AssetBox } from './AssetBox';\nexport default { title: 'Components/AssetBox', component: AssetBox };\nexport const Primary = { args: {} };\n",
    );
    write(
        root.join("src/components/AssetBox.tsx"),
        "import iconUrl from './error.svg?url';\nimport defaultImage from '@tw-tech/shared-assets/images/empty-default.png?url';\nexport const AssetBox = () => ({ iconUrl, defaultImage });\n",
    );
    write(
        root.join("src/components/error.svg"),
        r#"<svg viewBox="0 0 1 1"><path d="M0 0h1v1H0z"/></svg>"#,
    );

    let out = root.join("dist-stories");
    let result = build_stories_static(root, &out).expect("build");

    let svg_rel = Path::new("modules/src/components/error.svg");
    let png_rel = Path::new("deps/@tw-tech/shared-assets/images/empty-default.png");
    assert!(out.join(svg_rel).is_file(), "SVG asset must be copied");
    assert!(out.join(png_rel).is_file(), "PNG asset must be copied");
    assert!(result.emitted.iter().any(|p| p == svg_rel));
    assert!(result.emitted.iter().any(|p| p == png_rel));

    let component = fs::read_to_string(out.join("modules/src/components/AssetBox.js"))
        .expect("read component module");
    assert!(
        component.contains(r#"const iconUrl = "./error.svg";"#),
        "relative SVG import should become a URL string: {component}"
    );
    assert!(
        component.contains(
            r#"const defaultImage = "../../../deps/@tw-tech/shared-assets/images/empty-default.png";"#
        ),
        "bare PNG import should become a URL string into deps/: {component}"
    );
    assert!(
        !component.contains("import iconUrl") && !component.contains("import defaultImage"),
        "asset imports must not remain as browser module imports: {component}"
    );
}

#[test]
fn build_rewrites_svg_reactcomponent_barrel_reexports() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    write(
        root.join("src/components/IconBox.stories.tsx"),
        "import { IconBox } from './IconBox';\nexport default { title: 'Components/IconBox', component: IconBox };\nexport const Primary = { args: {} };\n",
    );
    write(
        root.join("src/components/IconBox.tsx"),
        "import { ErrorIcon } from './assets';\nexport const IconBox = () => ErrorIcon;\n",
    );
    write(
        root.join("src/components/assets/index.tsx"),
        "export { ReactComponent as ErrorIcon } from './error.svg';\n",
    );
    write(
        root.join("src/components/assets/error.svg"),
        r#"<svg viewBox="0 0 16 16"><path d="M0 0h16v16H0z"/></svg>"#,
    );

    let out = root.join("dist-stories");
    let result = build_stories_static(root, &out).expect("build");

    let barrel = fs::read_to_string(out.join("modules/src/components/assets/index.js"))
        .expect("read emitted asset barrel");
    assert!(
        barrel.contains("React.forwardRef") && barrel.contains("export { SvgErrorIcon as ErrorIcon };"),
        "SVG ReactComponent barrel re-export should become an inline React component export: {barrel}"
    );
    assert!(
        !barrel.contains("from './error.svg'") && !barrel.contains("from \"./error.svg\""),
        "browser JS must not keep a module export from raw SVG: {barrel}"
    );
    assert!(
        result
            .emitted
            .iter()
            .any(|p| p == Path::new("modules/src/components/assets/error.svg")),
        "raw SVG file should still be emitted for URL consumers"
    );
}

#[test]
fn build_emits_react_is_as_browser_esm_shim() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    write(
        root.join("node_modules/react-is/package.json"),
        r#"{"name":"react-is","version":"16.13.1","main":"index.js"}"#,
    );
    write(
        root.join("node_modules/react-is/index.js"),
        "module.exports = require('./cjs/react-is.development.js');\n",
    );
    write(
        root.join("src/components/Probe.stories.tsx"),
        "import { Probe } from './Probe';\nexport default { title: 'Components/Probe', component: Probe };\nexport const Primary = { args: {} };\n",
    );
    write(
        root.join("src/components/Probe.tsx"),
        "import { ForwardRef, isMemo } from 'react-is';\nexport const Probe = () => String(Boolean(ForwardRef) && !isMemo({}));\n",
    );

    let out = root.join("dist-stories");
    build_stories_static(root, &out).expect("build");

    let shim = fs::read_to_string(out.join("deps/react-is/index.js")).expect("read react-is shim");
    assert!(
        shim.contains("const ForwardRef") && shim.contains("function isMemo"),
        "react-is should be emitted as an ESM shim: {shim}"
    );
    assert!(
        shim.contains("export {") && !shim.contains("module.exports") && !shim.contains("require("),
        "react-is shim must be browser ESM, not CommonJS: {shim}"
    );

    let component = fs::read_to_string(out.join("modules/src/components/Probe.js"))
        .expect("read component module");
    assert!(
        component.contains("../../../deps/react-is/index.js"),
        "component should import the emitted react-is ESM shim: {component}"
    );
}

#[test]
fn build_emits_classnames_as_browser_esm_default_shim() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    write(
        root.join("node_modules/classnames/package.json"),
        r#"{"name":"classnames","version":"2.5.1","main":"index.js"}"#,
    );
    write(
        root.join("node_modules/classnames/index.js"),
        "module.exports = function classNames() { return ''; };\n",
    );
    write(
        root.join("src/components/Probe.stories.tsx"),
        "import { Probe } from './Probe';\nexport default { title: 'Components/Probe', component: Probe };\nexport const Primary = { args: {} };\n",
    );
    write(
        root.join("src/components/Probe.tsx"),
        "import classNames from 'classnames';\nexport const Probe = () => classNames('a', { b: true });\n",
    );

    let out = root.join("dist-stories");
    build_stories_static(root, &out).expect("build");

    let shim =
        fs::read_to_string(out.join("deps/classnames/index.js")).expect("read classnames shim");
    assert!(
        shim.contains("function classNames") && shim.contains("export default classNames"),
        "classnames should be emitted as an ESM default shim: {shim}"
    );
    assert!(
        !shim.contains("module.exports") && !shim.contains("require("),
        "classnames shim must be browser ESM, not CommonJS: {shim}"
    );
}

#[test]
fn build_wraps_dayjs_plugins_as_browser_esm_default() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    write(
        root.join("node_modules/dayjs/package.json"),
        r#"{"name":"dayjs","version":"1.11.13","main":"dayjs.min.js"}"#,
    );
    write(
        root.join("node_modules/dayjs/plugin/advancedFormat.js"),
        "module.exports = function advancedFormat() {};\n",
    );
    write(
        root.join("src/components/Probe.stories.tsx"),
        "import { Probe } from './Probe';\nexport default { title: 'Components/Probe', component: Probe };\nexport const Primary = { args: {} };\n",
    );
    write(
        root.join("src/components/Probe.tsx"),
        "import advancedFormat from 'dayjs/plugin/advancedFormat';\nexport const Probe = () => Boolean(advancedFormat);\n",
    );

    let out = root.join("dist-stories");
    build_stories_static(root, &out).expect("build");

    let wrapper = fs::read_to_string(out.join("deps/dayjs/plugin/advancedFormat.js"))
        .expect("read dayjs plugin wrapper");
    assert!(
        wrapper.contains("const module = { exports: {} }")
            && wrapper.contains("export default module.exports"),
        "dayjs plugin should be wrapped as an ESM default export: {wrapper}"
    );
}

#[test]
fn build_wraps_dayjs_main_as_browser_esm_default() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    write(
        root.join("node_modules/dayjs/package.json"),
        r#"{"name":"dayjs","version":"1.11.13","main":"dayjs.min.js"}"#,
    );
    write(
        root.join("node_modules/dayjs/dayjs.min.js"),
        "module.exports = function dayjs() {};\n",
    );
    write(
        root.join("src/components/Probe.stories.tsx"),
        "import { Probe } from './Probe';\nexport default { title: 'Components/Probe', component: Probe };\nexport const Primary = { args: {} };\n",
    );
    write(
        root.join("src/components/Probe.tsx"),
        "import dayjs from 'dayjs';\nexport const Probe = () => Boolean(dayjs);\n",
    );

    let out = root.join("dist-stories");
    build_stories_static(root, &out).expect("build");

    let wrapper =
        fs::read_to_string(out.join("deps/dayjs/dayjs.min.js")).expect("read dayjs main wrapper");
    assert!(
        wrapper.contains("const module = { exports: {} }")
            && wrapper.contains("export default module.exports"),
        "dayjs main should be wrapped as an ESM default export: {wrapper}"
    );
}

#[test]
fn build_emits_json2mq_as_browser_esm_default_shim() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    write(
        root.join("node_modules/json2mq/package.json"),
        r#"{"name":"json2mq","version":"0.2.0","main":"index.js"}"#,
    );
    write(
        root.join("node_modules/json2mq/index.js"),
        "module.exports = function json2mq() { return ''; };\n",
    );
    write(
        root.join("src/components/Probe.stories.tsx"),
        "import { Probe } from './Probe';\nexport default { title: 'Components/Probe', component: Probe };\nexport const Primary = { args: {} };\n",
    );
    write(
        root.join("src/components/Probe.tsx"),
        "import json2mq from 'json2mq';\nexport const Probe = () => json2mq({ minWidth: 1 });\n",
    );

    let out = root.join("dist-stories");
    build_stories_static(root, &out).expect("build");

    let shim = fs::read_to_string(out.join("deps/json2mq/index.js")).expect("read json2mq shim");
    assert!(
        shim.contains("function json2mq") && shim.contains("export default json2mq"),
        "json2mq should be emitted as an ESM default shim: {shim}"
    );
    assert!(
        !shim.contains("module.exports") && !shim.contains("require("),
        "json2mq shim must be browser ESM, not CommonJS: {shim}"
    );
}

#[test]
fn build_emits_copy_to_clipboard_browser_esm_default_shim() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    write(
        root.join("node_modules/copy-to-clipboard/package.json"),
        r#"{"name":"copy-to-clipboard","version":"3.3.3","main":"index.js","dependencies":{"toggle-selection":"^1.0.6"}}"#,
    );
    write(
        root.join("node_modules/copy-to-clipboard/index.js"),
        "var deselectCurrent = require('toggle-selection');\nmodule.exports = function copy() { return Boolean(deselectCurrent); };\n",
    );
    write(
        root.join("node_modules/toggle-selection/package.json"),
        r#"{"name":"toggle-selection","version":"1.0.6","main":"index.js"}"#,
    );
    write(
        root.join("node_modules/toggle-selection/index.js"),
        "module.exports = function toggleSelection() { return function noop() {}; };\n",
    );
    write(
        root.join("src/components/Probe.stories.tsx"),
        "import { Probe } from './Probe';\nexport default { title: 'Components/Probe', component: Probe };\nexport const Primary = { args: {} };\n",
    );
    write(
        root.join("src/components/Probe.tsx"),
        "import copy from 'copy-to-clipboard';\nexport const Probe = () => String(copy('ok'));\n",
    );

    let out = root.join("dist-stories");
    build_stories_static(root, &out).expect("build");

    let copy_shim = fs::read_to_string(out.join("deps/copy-to-clipboard/index.js"))
        .expect("read copy-to-clipboard shim");
    assert!(
        copy_shim.contains("import deselectCurrent from \"../toggle-selection/index.js\"")
            && copy_shim.contains("export default copy"),
        "copy-to-clipboard should be emitted as an ESM default shim: {copy_shim}"
    );
    assert!(
        !copy_shim.contains("module.exports") && !copy_shim.contains("require("),
        "copy-to-clipboard shim must be browser ESM, not CommonJS: {copy_shim}"
    );

    let toggle_shim = fs::read_to_string(out.join("deps/toggle-selection/index.js"))
        .expect("read toggle-selection shim");
    assert!(
        toggle_shim.contains("function toggleSelection")
            && toggle_shim.contains("export default toggleSelection"),
        "toggle-selection should be emitted as an ESM default shim: {toggle_shim}"
    );
    assert!(
        !toggle_shim.contains("module.exports") && !toggle_shim.contains("require("),
        "toggle-selection shim must be browser ESM, not CommonJS: {toggle_shim}"
    );
}

#[test]
fn build_applies_production_defines_to_static_modules() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    write(
        root.join("node_modules/env-probe/package.json"),
        r#"{"name":"env-probe","version":"1.0.0","module":"index.js"}"#,
    );
    write(
        root.join("node_modules/env-probe/index.js"),
        "export const mode = process.env.NODE_ENV;\nexport const dev = __DEV__;\n",
    );
    write(
        root.join("src/components/Probe.stories.tsx"),
        "import { Probe } from './Probe';\nexport default { title: 'Components/Probe', component: Probe };\nexport const Primary = { args: {} };\n",
    );
    write(
        root.join("src/components/Probe.tsx"),
        "import { mode, dev } from 'env-probe';\nexport const Probe = () => `${mode}:${dev}`;\n",
    );

    let out = root.join("dist-stories");
    build_stories_static(root, &out).expect("build");

    let dep = fs::read_to_string(out.join("deps/env-probe/index.js")).expect("read env-probe dep");
    assert!(
        dep.contains(r#""production""#) && dep.contains("false"),
        "static modules should receive production defines: {dep}"
    );
    assert!(
        !dep.contains("process.env") && !dep.contains("__DEV__"),
        "static modules should not leave browser-undefined env globals: {dep}"
    );
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
        description: String::new(),
        args: BTreeMap::new(),
        parameters: BTreeMap::new(),
        source: None,
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
    assert!(
        preview.contains("HMR connected"),
        "dev preview keeps the HMR client"
    );
}
// </HANDWRITE>
