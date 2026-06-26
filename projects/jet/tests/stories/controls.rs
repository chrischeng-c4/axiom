// HANDWRITE-BEGIN gap="missing-generator:unit-test:16a93e6f" tracker="standardize-gap-projects-jet-tests-stories-controls-rs" reason="Tests: prop-type extraction for a typed component; control inference for bool/string/number/string-literal-union; meta.argTypes override wins; the Controls panel HTML seeds current arg values; editing posts new args."
//! Integration tests for B3: the `jet stories` Controls panel.
//!
//! Covers the full B3 surface:
//! (a) [`prop_extractor::extract_props`] reads a typed `ButtonProps` interface,
//! (b) [`controls::infer_control`] maps each prop type to its control kind,
//! (c) a `meta.argTypes` override wins over inference,
//! (d) the rendered Controls panel HTML seeds the story's current arg values,
//! (e) editing a control wires an args-update that targets the preview render
//!     hook (the manager posts `jet-stories-args`; the preview applies it).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use jet::stories::controls::{infer_control, resolve_controls, Control, ControlKind};
use jet::stories::csf::CsfValue;
use jet::stories::prop_extractor::{extract_props, extract_props_at, PropDef};
use jet::stories::{discover, server};
use tempfile::TempDir;
use tower::ServiceExt; // for `oneshot`

/// A typed component with the four canonical prop shapes.
const BUTTON_COMPONENT: &str = r#"
import React from 'react';

interface ButtonProps {
  primary: boolean;
  label: string;
  size: "sm" | "lg";
  count?: number;
}

export function Button(props: ButtonProps) {
  return null;
}
"#;

// ── (a) extract_props on a typed interface ───────────────────────────────────

#[test]
fn extracts_typed_button_props() {
    let props = extract_props(BUTTON_COMPONENT, "Button");
    assert_eq!(props.len(), 4, "all four props read: {props:?}");

    let expect = [
        ("primary", "boolean", false),
        ("label", "string", false),
        ("size", "\"sm\" | \"lg\"", false),
        ("count", "number", true),
    ];
    for (got, (name, ty, optional)) in props.iter().zip(expect) {
        assert_eq!(got.name, name);
        assert_eq!(got.type_text, ty, "type_text for {name}");
        assert_eq!(got.optional, optional, "optional for {name}");
    }
}

// ── (b) infer_control maps each prop type ────────────────────────────────────

#[test]
fn infers_control_per_prop_type() {
    let props = extract_props(BUTTON_COMPONENT, "Button");
    let kinds: Vec<ControlKind> = props.iter().map(infer_control).collect();
    assert_eq!(
        kinds,
        vec![
            ControlKind::Toggle,
            ControlKind::Text,
            ControlKind::Select {
                options: vec!["sm".to_string(), "lg".to_string()]
            },
            ControlKind::Number,
        ]
    );
}

// ── (c) meta.argTypes override wins over inference ───────────────────────────

#[test]
fn arg_type_select_override_wins_over_inference() {
    // `size` would infer to Select from its union — but here we start from a
    // plain `string` prop and let the argType force a select with options.
    let props = vec![PropDef {
        name: "size".into(),
        type_text: "string".into(),
        optional: false,
    }];

    let mut control_obj = BTreeMap::new();
    control_obj.insert("type".into(), CsfValue::Str("select".into()));
    control_obj.insert("options".into(), CsfValue::Raw("[\"sm\", \"md\", \"lg\"]".into()));

    let mut arg_type = BTreeMap::new();
    arg_type.insert("control".into(), CsfValue::Object(control_obj));

    let mut arg_types = BTreeMap::new();
    arg_types.insert("size".into(), CsfValue::Object(arg_type));

    let controls = resolve_controls(&props, &arg_types, &BTreeMap::new());
    assert_eq!(
        controls[0].kind,
        ControlKind::Select {
            options: vec!["sm".into(), "md".into(), "lg".into()]
        },
        "argType options override inference"
    );
}

#[test]
fn arg_type_control_false_disables_widget() {
    let props = extract_props(BUTTON_COMPONENT, "Button");
    let mut arg_types = BTreeMap::new();
    let mut disabled = BTreeMap::new();
    disabled.insert("control".into(), CsfValue::Bool(false));
    arg_types.insert("count".into(), CsfValue::Object(disabled));

    let controls = resolve_controls(&props, &arg_types, &BTreeMap::new());
    assert!(
        controls.iter().all(|c| c.name != "count"),
        "control:false omits the widget: {controls:?}"
    );
    assert_eq!(controls.len(), 3, "the other three remain");
}

// ── (d) + (e) the rendered Controls panel + args wiring (via the router) ─────

const BUTTON_STORIES: &str = r#"
import { Button } from './Button';
import type { Meta, StoryObj } from '@storybook/react';

const meta = {
  title: 'Components/Button',
  component: Button,
  args: { label: 'Default', size: 'sm' },
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: { primary: true, label: 'Click me', size: 'lg' },
};
"#;

fn write_fixtures() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();
    write(root.join("src/Button.stories.tsx"), BUTTON_STORIES);
    write(root.join("src/Button.tsx"), BUTTON_COMPONENT);
    dir
}

fn write(path: std::path::PathBuf, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(path, contents).expect("write fixture");
}

async fn get(router: &axum::Router, path: &str) -> (StatusCode, String) {
    let response = router
        .clone()
        .oneshot(Request::builder().uri(path).body(Body::empty()).expect("request"))
        .await
        .expect("router response");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 4 * 1024 * 1024)
        .await
        .expect("body bytes");
    (status, String::from_utf8_lossy(&bytes).to_string())
}

/// (d) the manager HTML embeds a Controls panel seeded with the selected
/// story's *current merged arg values* (story args override meta args).
#[tokio::test]
async fn manager_renders_controls_seeded_with_current_args() {
    let dir = write_fixtures();
    let index = discover(dir.path());
    let router = server::build_router(index, dir.path().to_path_buf());

    let (status, html) = get(&router, "/").await;
    assert_eq!(status, StatusCode::OK);

    // The panel renders one widget per resolved control.
    assert!(html.contains("id=\"jet-controls\""), "controls panel present");
    assert!(html.contains("data-control=\"primary\""), "boolean -> toggle");
    assert!(html.contains("data-control=\"label\""), "string -> text");
    assert!(html.contains("data-control=\"size\""), "union -> select");
    assert!(html.contains("data-control=\"count\""), "number -> number");

    // Current values are seeded from the merged story args:
    //   primary=true (story), label='Click me' (story overrides meta), size='lg'.
    assert!(
        html.contains("data-kind=\"toggle\" checked"),
        "primary toggle seeded true: {html}"
    );
    assert!(html.contains("value=\"Click me\""), "label text seeded with story value");
    assert!(
        html.contains("<option value=\"lg\" selected>"),
        "size select seeds the current option"
    );
    // The size select offers both union options.
    assert!(html.contains("<option value=\"sm\""));
    assert!(html.contains("<option value=\"lg\""));
}

/// (e) editing a control posts the new args to the preview render hook: the
/// manager wires onchange handlers that `postMessage` a `jet-stories-args`
/// update, and the preview applies it via `window.__jetStoriesRender`.
#[tokio::test]
async fn editing_a_control_targets_the_preview_render_hook() {
    let dir = write_fixtures();
    let index = discover(dir.path());
    let router = server::build_router(index, dir.path().to_path_buf());

    // Manager side: onchange handlers post the args set into the preview frame.
    let (_, manager_html) = get(&router, "/").await;
    assert!(
        manager_html.contains("addEventListener('change'")
            || manager_html.contains("addEventListener('input'"),
        "controls have edit handlers"
    );
    assert!(manager_html.contains("postMessage"), "edits post to the preview frame");
    assert!(
        manager_html.contains("jet-stories-args"),
        "uses the args-update message channel"
    );

    // Preview side: the frame listens for that message and re-renders via the
    // exposed render hook.
    let (status, preview_html) =
        get(&router, "/__jet_stories_preview/components-button--primary").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        preview_html.contains("window.__jetStoriesRender = renderStory"),
        "render hook exposed"
    );
    assert!(
        preview_html.contains("jet-stories-args"),
        "preview listens for control updates"
    );
    assert!(
        preview_html.contains("liveArgs = data.args"),
        "preview swaps live args on update"
    );
}

/// A story whose component source can't be resolved degrades to no controls
/// (the manager still renders).
#[tokio::test]
async fn missing_component_source_degrades_to_no_controls() {
    let dir = TempDir::new().expect("temp dir");
    // Story file with a bare (unresolvable) component import — no local file.
    write(
        dir.path().join("src/Widget.stories.tsx"),
        r#"
import { Widget } from '@acme/widget';
const meta = { title: 'X/Widget', component: Widget };
export default meta;
export const Basic = { args: { a: 1 } };
"#,
    );
    let index = discover(dir.path());
    let router = server::build_router(index, dir.path().to_path_buf());
    let (status, html) = get(&router, "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("No controls for this story."), "graceful empty panel");
}

/// Sanity: the controls panel uses real props for a story whose meta lacks an
/// explicit `args` block (defaults pulled from inference, current=None).
#[test]
fn controls_without_current_values_still_render_widgets() {
    let props = extract_props(BUTTON_COMPONENT, "Button");
    let controls: Vec<Control> = resolve_controls(&props, &BTreeMap::new(), &BTreeMap::new());
    assert_eq!(controls.len(), 4);
    assert!(controls.iter().all(|c| c.current.is_none()));
}

// ── #198: generic / cross-file / intersection prop-type extraction ───────────

/// (a) A component whose `Props` is imported from a sibling file yields controls
/// for those props (the extractor follows the relative `import type` to disk).
#[test]
fn imported_sibling_props_yield_controls() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();
    write(
        root.join("src/types.ts"),
        "export interface Props { primary: boolean; label: string; size: \"sm\" | \"lg\"; }\n",
    );
    let component = root.join("src/Widget.tsx");
    write(
        component.clone(),
        r#"
import type { Props } from "./types";
export function Widget(props: Props) { return null; }
"#,
    );

    let source = fs::read_to_string(&component).expect("read component");
    let props = extract_props_at(&source, "Widget", Some(&component));
    let names: Vec<&str> = props.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["primary", "label", "size"], "imported props read");

    let controls = resolve_controls(&props, &BTreeMap::new(), &BTreeMap::new());
    let kinds: Vec<ControlKind> = controls.into_iter().map(|c| c.kind).collect();
    assert_eq!(
        kinds,
        vec![
            ControlKind::Toggle,
            ControlKind::Text,
            ControlKind::Select {
                options: vec!["sm".into(), "lg".into()]
            },
        ],
        "controls inferred for cross-file props"
    );
}

/// (b) An intersection `Base & Extra` yields the union of both operands' members
/// as controls (deduped, first operand winning), each inferring its widget.
#[test]
fn intersection_props_union_into_controls() {
    let src = r#"
interface Base { id: string; primary: boolean; }
type Extra = { count: number; };
type Props = Base & Extra;
export function Widget(props: Props) { return null; }
"#;
    let props = extract_props(src, "Widget");
    let names: Vec<&str> = props.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["id", "primary", "count"], "union of operands");

    let controls = resolve_controls(&props, &BTreeMap::new(), &BTreeMap::new());
    let by_name: BTreeMap<&str, &ControlKind> =
        controls.iter().map(|c| (c.name.as_str(), &c.kind)).collect();
    assert_eq!(by_name.get("id"), Some(&&ControlKind::Text));
    assert_eq!(by_name.get("primary"), Some(&&ControlKind::Toggle));
    assert_eq!(by_name.get("count"), Some(&&ControlKind::Number));
}

/// (c) A simple generic (`GenProps<number>`) resolves: the concrete type argument
/// substitutes for the parameter, so the param-typed prop infers its real widget.
#[test]
fn simple_generic_resolves_with_substitution() {
    let src = r#"
interface GenProps<T> { value: T; label: string; }
export function Field(props: GenProps<number>) { return null; }
"#;
    let props = extract_props(src, "Field");
    assert_eq!(props.len(), 2);

    let controls = resolve_controls(&props, &BTreeMap::new(), &BTreeMap::new());
    let by_name: BTreeMap<&str, &ControlKind> =
        controls.iter().map(|c| (c.name.as_str(), &c.kind)).collect();
    assert_eq!(
        by_name.get("value"),
        Some(&&ControlKind::Number),
        "T -> number gives a Number control"
    );
    assert_eq!(by_name.get("label"), Some(&&ControlKind::Text));
}

/// (d) An unresolvable generic (a utility-type rhs we don't destructure) degrades
/// to no controls without error — the panel still renders empty.
#[test]
fn unresolvable_generic_degrades_to_no_controls() {
    let src = r#"
type Props = Partial<{ a: string; b: number }>;
export function Widget(props: Props) { return null; }
"#;
    let props = extract_props(src, "Widget");
    assert!(props.is_empty(), "no props extracted from utility-type rhs");
    let controls = resolve_controls(&props, &BTreeMap::new(), &BTreeMap::new());
    assert!(controls.is_empty(), "no controls, no crash");
}

/// (a, end-to-end) The manager renders controls for a story whose component's
/// props are imported from a sibling file — exercising the live server path that
/// now threads the component file path into extraction.
#[tokio::test]
async fn manager_renders_controls_for_cross_file_props() {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();
    write(
        root.join("src/types.ts"),
        "export interface Props { primary: boolean; label: string; }\n",
    );
    write(
        root.join("src/Widget.tsx"),
        r#"
import type { Props } from "./types";
export function Widget(props: Props) { return null; }
"#,
    );
    write(
        root.join("src/Widget.stories.tsx"),
        r#"
import { Widget } from './Widget';
const meta = { title: 'X/Widget', component: Widget, args: { label: 'Hi' } };
export default meta;
export const Basic = { args: { primary: true, label: 'Hello' } };
"#,
    );

    let index = discover(root);
    let router = server::build_router(index, root.to_path_buf());
    let (status, html) = get(&router, "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("data-control=\"primary\""), "cross-file bool control");
    assert!(html.contains("data-control=\"label\""), "cross-file string control");
    assert!(html.contains("value=\"Hello\""), "seeded with story arg");
}

// Silence unused-path lint on the helper module path constant if added later.
#[allow(dead_code)]
fn _unused(_: &Path) {}
// HANDWRITE-END
