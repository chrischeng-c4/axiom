// HANDWRITE-BEGIN gap="missing-generator:unit-test:de93f93a" tracker="standardize-gap-projects-jet-tests-stories-csf-discovery-rs" reason="Fixtures (Button.stories.tsx, Card.stories.tsx, malformed) + tests: glob finds both, meta+named stories parsed with merged args, title hierarchy + stable ids, malformed file -> diagnostic without aborting discovery."
//! Integration tests for B1: CSF story discovery + parse (`jet stories`).
//!
//! Fixtures are written into a temp dir so the WalkDir/globset discovery path
//! runs end-to-end. We cover:
//! (a) glob discovery of both valid `.stories.tsx` files,
//! (b) `Button` parsing to meta(title=Components/Button) + 2 named stories with
//!     args (story args merged over meta args),
//! (c) the index exposing a title hierarchy + stable, slugged ids,
//! (d) a malformed file (no default export) yielding a diagnostic WITHOUT
//!     aborting discovery of the valid files.

use std::fs;
use std::path::Path;

use jet::stories::csf::CsfValue;
use jet::stories::{discover, StoryIndex};
use tempfile::TempDir;

const BUTTON_STORIES: &str = r#"
import { Button } from './Button';
import type { Meta, StoryObj } from '@storybook/react';

const meta = {
  title: 'Components/Button',
  component: Button,
  args: { size: 'md', label: 'Default' },
  argTypes: { size: { control: 'select' } },
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

const BROKEN_STORIES: &str = r#"
// No default export -> not valid CSF.
export const Orphan = {
  args: { value: 1 },
};
"#;

/// CSF2: `const Primary = Template.bind({})` + later `Primary.args = {...}`.
const LEGACY_STORIES: &str = r#"
import { Toggle } from './Toggle';

export default {
  title: 'Legacy/Toggle',
  component: Toggle,
};

const Template = (args) => <Toggle {...args} />;

export const Primary = Template.bind({});
Primary.args = { label: "Hi", on: true };
Primary.storyName = "The Primary";
"#;

/// Spread args: `args: { ...base, x: 2 }` where `base` is a static const, plus
/// a story that spreads another CSF2 story's args.
const SPREAD_STORIES: &str = r#"
import { Panel } from './Panel';

export default {
  title: 'Layout/Panel',
  component: Panel,
};

const base = { x: 1, y: 1, label: 'base' };

export const Spread = {
  args: { ...base, x: 2 },
};

// Unresolvable spread (imported base) -> explicit keys kept, spread dropped.
export const Dynamic = {
  args: { ...imported, only: 9 },
};
"#;

/// A barrel file that re-exports stories from a sibling story file.
const REEXPORT_STORIES: &str = r#"
export { Primary } from './LegacyToggle.stories';
export { Primary as Renamed } from './LegacyToggle.stories';
"#;

/// Lay down the three fixtures in nested dirs (exercises `**/` globbing).
fn write_fixtures() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    write(root.join("src/components/Button.stories.tsx"), BUTTON_STORIES);
    write(root.join("src/surfaces/Card.stories.tsx"), CARD_STORIES);
    write(root.join("src/broken/Broken.stories.tsx"), BROKEN_STORIES);

    // A decoy that must NOT match the story globs.
    write(root.join("src/components/Button.tsx"), "export const Button = () => null;\n");
    // node_modules must be skipped entirely.
    write(
        root.join("node_modules/dep/Vendor.stories.tsx"),
        BUTTON_STORIES,
    );

    dir
}

/// Fixtures for the stories follow-up (CSF2 bind, spread args, re-exports).
///
/// Kept separate from [`write_fixtures`] so the original exact-count assertions
/// stay stable. `LegacyToggle.stories.tsx` is the sibling that
/// `Barrel.stories.tsx` re-exports from.
fn write_followup_fixtures() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();

    write(root.join("src/legacy/LegacyToggle.stories.tsx"), LEGACY_STORIES);
    write(root.join("src/layout/Panel.stories.tsx"), SPREAD_STORIES);
    write(root.join("src/legacy/Barrel.stories.tsx"), REEXPORT_STORIES);

    dir
}

fn write(path: std::path::PathBuf, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(path, contents).expect("write fixture");
}

fn index_of(dir: &Path) -> StoryIndex {
    discover(dir)
}

/// (a) discovery finds both valid files via the glob, skips decoy + node_modules.
#[test]
fn discovers_valid_story_files() {
    let dir = write_fixtures();
    let index = index_of(dir.path());

    // Two valid metas (Button + Card); Broken contributes none.
    assert_eq!(
        index.metas.len(),
        2,
        "expected Button + Card metas, got {:?}",
        index
            .metas
            .iter()
            .map(|m| m.file.display().to_string())
            .collect::<Vec<_>>()
    );

    let titles: Vec<_> = index.metas.iter().filter_map(|m| m.title.clone()).collect();
    assert!(titles.contains(&"Components/Button".to_string()));
    assert!(titles.contains(&"Surfaces/Card".to_string()));

    // node_modules vendor file must not leak in.
    assert!(
        index
            .metas
            .iter()
            .all(|m| !m.file.to_string_lossy().contains("node_modules")),
        "node_modules story files must be skipped"
    );
}

/// (b) Button parses to meta(title=Components/Button) + 2 named stories with
/// merged args.
#[test]
fn button_meta_and_stories_parsed_with_merged_args() {
    let dir = write_fixtures();
    let index = index_of(dir.path());

    let button_meta = index
        .metas
        .iter()
        .find(|m| m.title.as_deref() == Some("Components/Button"))
        .expect("Button meta present");
    assert_eq!(button_meta.component.as_deref(), Some("Button"));
    assert_eq!(button_meta.title_path, vec!["Components", "Button"]);
    assert_eq!(
        button_meta.args.get("size"),
        Some(&CsfValue::Str("md".into()))
    );
    assert!(button_meta.arg_types.contains_key("size"));

    let button_stories: Vec<_> = index
        .stories
        .iter()
        .filter(|s| s.file == button_meta.file)
        .collect();
    assert_eq!(button_stories.len(), 2, "Button has Primary + Disabled");

    let primary = button_stories
        .iter()
        .find(|s| s.name == "Primary")
        .expect("Primary story");
    // Story arg overrides meta `label`, meta `size` is inherited.
    assert_eq!(
        primary.args.get("label"),
        Some(&CsfValue::Str("Click me".into()))
    );
    assert_eq!(primary.args.get("size"), Some(&CsfValue::Str("md".into())));
    assert_eq!(primary.args.get("primary"), Some(&CsfValue::Bool(true)));
    assert!(!primary.has_render);

    let disabled = button_stories
        .iter()
        .find(|s| s.name == "Disabled")
        .expect("Disabled story");
    assert_eq!(
        disabled.args.get("disabled"),
        Some(&CsfValue::Bool(true))
    );
    assert!(disabled.has_render, "Disabled declares a render fn");
}

/// (c) the index exposes a title hierarchy + stable, slugged ids.
#[test]
fn index_exposes_title_hierarchy_and_stable_ids() {
    let dir = write_fixtures();
    let index = index_of(dir.path());

    let hierarchy = index.title_hierarchy();
    assert!(hierarchy.contains(&vec!["Components".to_string()]));
    assert!(hierarchy.contains(&vec!["Components".to_string(), "Button".to_string()]));
    assert!(hierarchy.contains(&vec!["Surfaces".to_string()]));
    assert!(hierarchy.contains(&vec!["Surfaces".to_string(), "Card".to_string()]));

    // Stable, deterministic, unique ids.
    let ids: Vec<_> = index.stories.iter().map(|s| s.id.clone()).collect();
    assert!(ids.contains(&"components-button--primary".to_string()));
    assert!(ids.contains(&"components-button--disabled".to_string()));
    // `WithFooter` is a single identifier (no separator) -> `withfooter`.
    assert!(ids.contains(&"surfaces-card--withfooter".to_string()));

    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "stories are returned id-sorted");

    let unique: std::collections::BTreeSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), ids.len(), "ids are unique");

    // Re-running discovery yields identical ids (stability).
    let again = index_of(dir.path());
    let ids_again: Vec<_> = again.stories.iter().map(|s| s.id.clone()).collect();
    assert_eq!(ids, ids_again);
}

/// (d) the malformed file produces a diagnostic and does NOT abort discovery of
/// the valid files.
#[test]
fn malformed_file_yields_diagnostic_without_aborting() {
    let dir = write_fixtures();
    let index = index_of(dir.path());

    // Valid files still fully discovered.
    assert_eq!(index.metas.len(), 2);
    assert_eq!(index.stories.len(), 3); // Primary + Disabled + WithFooter

    // The malformed file is surfaced as a diagnostic.
    assert!(
        index
            .diagnostics
            .iter()
            .any(|d| d.contains("Broken.stories.tsx") && d.contains("parse error")),
        "expected a parse-error diagnostic for Broken.stories.tsx, got {:?}",
        index.diagnostics
    );
}

/// (e) CSF2 `const Primary = Template.bind({}); Primary.args = {...}` surfaces
/// `Primary` as a story carrying its mutated args + a render (the bound
/// template).
#[test]
fn csf2_template_bind_surfaces_story_with_args() {
    let dir = write_followup_fixtures();
    let index = index_of(dir.path());

    let primary = index
        .stories
        .iter()
        .find(|s| s.name == "Primary" && s.file.ends_with("LegacyToggle.stories.tsx"))
        .expect("CSF2 Primary discovered");

    assert_eq!(
        primary.args.get("label"),
        Some(&CsfValue::Str("Hi".into())),
        "Primary.args picked up `label`"
    );
    assert_eq!(primary.args.get("on"), Some(&CsfValue::Bool(true)));
    // The bound template supplies the render.
    assert!(primary.has_render, "bound-template story renders via template");
    assert_eq!(primary.id, "legacy-toggle--primary");
}

/// (f) a barrel file re-exporting `export { Primary } from './LegacyToggle.stories'`
/// includes `Primary` (and the renamed `Renamed`) in its discovered set, under
/// the barrel's own title.
#[test]
fn re_exported_stories_are_resolved_from_sibling() {
    let dir = write_followup_fixtures();
    let index = index_of(dir.path());

    let barrel = dir
        .path()
        .join("src/legacy/Barrel.stories.tsx");

    let from_barrel: Vec<_> = index.stories.iter().filter(|s| s.file == barrel).collect();
    let names: Vec<_> = from_barrel.iter().map(|s| s.name.as_str()).collect();
    assert!(
        names.contains(&"Primary"),
        "barrel re-exports Primary, got {names:?}"
    );
    assert!(
        names.contains(&"Renamed"),
        "barrel re-exports Primary as Renamed, got {names:?}"
    );

    // The re-exported story adopts the sibling's args.
    let primary = from_barrel
        .iter()
        .find(|s| s.name == "Primary")
        .expect("re-exported Primary");
    assert_eq!(primary.args.get("label"), Some(&CsfValue::Str("Hi".into())));
    assert!(
        !index
            .diagnostics
            .iter()
            .any(|d| d.contains("Barrel.stories.tsx")),
        "no diagnostic expected for a resolvable barrel, got {:?}",
        index.diagnostics
    );
}

/// (g) spread args `args: { ...base, x: 2 }` merge the static `base` (explicit
/// `x` overrides); an unresolvable spread keeps only its explicit keys.
#[test]
fn spread_args_merge_static_base() {
    let dir = write_followup_fixtures();
    let index = index_of(dir.path());

    let spread = index
        .stories
        .iter()
        .find(|s| s.name == "Spread")
        .expect("Spread story discovered");
    // `base` members merged in...
    assert_eq!(spread.args.get("y"), Some(&CsfValue::Number("1".into())));
    assert_eq!(
        spread.args.get("label"),
        Some(&CsfValue::Str("base".into()))
    );
    // ...with the explicit `x: 2` overriding `base.x = 1`.
    assert_eq!(spread.args.get("x"), Some(&CsfValue::Number("2".into())));

    // Unresolvable spread: explicit key kept, spread silently dropped.
    let dynamic = index
        .stories
        .iter()
        .find(|s| s.name == "Dynamic")
        .expect("Dynamic story discovered");
    assert_eq!(dynamic.args.get("only"), Some(&CsfValue::Number("9".into())));
    assert!(
        !dynamic.args.contains_key("x"),
        "imported-base spread must not leak resolved keys"
    );
}
// HANDWRITE-END
