// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
mod common;

use common::{canvas_spy, react_oracle};
use jet::browser::{Browser, LaunchOptions};
use jet::browser_cli;
use serde_json::{json, Value};
use std::sync::OnceLock;
use std::time::Duration;

static LIVE_WASM_E2E_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn live_wasm_e2e_guard() -> tokio::sync::MutexGuard<'static, ()> {
    LIVE_WASM_E2E_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

fn browser_options() -> LaunchOptions {
    let mut options = LaunchOptions::default();
    if let Ok(path) = std::env::var("CHROME_PATH") {
        options.executable = Some(std::path::PathBuf::from(path));
    }
    options
}

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

struct FixtureInteraction {
    dom_script: &'static str,
    wasm_click: (f32, f32),
    expected_hook: Option<i64>,
}

struct FixtureParityCase {
    id: &'static str,
    example_name: &'static str,
    dom_render_script: &'static str,
    interaction: Option<FixtureInteraction>,
    behavior_tags: &'static [&'static str],
}

fn fixture_parity_cases() -> Vec<FixtureParityCase> {
    vec![
        FixtureParityCase {
            id: "static-no-state",
            example_name: "no-state-demo",
            dom_render_script: r#"
      const e = React.createElement;
      function NoState(props) {
        return e('div', { id: 'root' },
          e('span', { id: 'label' }, 'value: ', String(props.value))
        );
      }
      render(e(NoState, { value: 42 }));
"#,
            interaction: None,
            behavior_tags: &["rendered-text"],
        },
        FixtureParityCase {
            id: "class-name-state",
            example_name: "classname-demo",
            dom_render_script: r#"
      const e = React.createElement;
      function ClassName(props) {
        const [n, setN] = React.useState(props.initial);
        return e('button',
          { id: 'cta', className: 'primary', onClick: () => setN(n + 1) },
          'click me: ',
          String(n)
        );
      }
      render(e(ClassName, { initial: 0 }));
"#,
            interaction: Some(FixtureInteraction {
                dom_script: r#"
                  document.getElementById('cta').dispatchEvent(new MouseEvent('click', {
                    bubbles: true,
                    cancelable: true,
                    view: window
                  }))
                "#,
                wasm_click: (30.0, 12.0),
                expected_hook: Some(1),
            }),
            behavior_tags: &["button-event", "style-dependent-layout"],
        },
        FixtureParityCase {
            id: "list-render-state",
            example_name: "list-render-demo",
            dom_render_script: r#"
      const e = React.createElement;
      function ListRender(props) {
        const [n, setN] = React.useState(props.initial);
        return e('div', { id: 'root' },
          e('button', { id: 'add', onClick: () => setN(n + 1) }, 'add'),
          Array.from({ length: n }, (_, i) =>
            e('span', { id: 'item', key: i }, 'item ', String(i))
          )
        );
      }
      render(e(ListRender, { initial: 3 }));
"#,
            interaction: Some(FixtureInteraction {
                dom_script: r#"
                  document.getElementById('add').dispatchEvent(new MouseEvent('click', {
                    bubbles: true,
                    cancelable: true,
                    view: window
                  }))
                "#,
                wasm_click: (30.0, 12.0),
                expected_hook: Some(4),
            }),
            behavior_tags: &["button-event", "rendered-text"],
        },
    ]
}

const LIBRARY_FIXTURE_MANIFEST: &str = "projects/jet/parity/data/fixtures/libraries/fixtures.toml";
const LIBRARY_PACKAGE_MANIFEST: &str = "projects/jet/parity/data/fixtures/libraries/package.json";

struct LibraryParityCase {
    id: &'static str,
    library_id: &'static str,
    component: &'static str,
    dom_render_script: &'static str,
    tsx_file: &'static str,
    tsx_source: &'static str,
    root_component: &'static str,
    state_probe: Option<LibraryStateProbe>,
    interaction: Option<LibraryInteraction>,
    behavior_tags: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct LibraryStateProbe {
    control_selector: &'static str,
    label_selector: Option<&'static str>,
}

#[derive(Clone, Copy)]
struct LibraryInteraction {
    kind: LibraryInteractionKind,
    selector: &'static str,
    value: Option<&'static str>,
}

#[derive(Clone, Copy)]
enum LibraryInteractionKind {
    Input,
    Click,
}

fn library_parity_cases() -> Vec<LibraryParityCase> {
    vec![
        LibraryParityCase {
            id: "mui-button-basic",
            library_id: "mui",
            component: "Button",
            dom_render_script: r#"
      const e = React.createElement;
      function MuiButtonFixture() {
        return e('button',
          { id: 'mui-button', className: 'MuiButton-root' },
          'MUI Primary'
        );
      }
      render(e(MuiButtonFixture));
"#,
            tsx_file: "MuiButtonFixture.tsx",
            tsx_source: r#"
import React from 'react';
import Button from '@mui/material/Button';

export function MuiButtonFixture() {
  return (
    <Button id="mui-button" className="MuiButton-root">
      MUI Primary
    </Button>
  );
}
"#,
            root_component: "MuiButtonFixture",
            state_probe: None,
            interaction: None,
            behavior_tags: &["library-component-rendering", "style-dependent-layout"],
        },
        LibraryParityCase {
            id: "antd-button-primary",
            library_id: "antd",
            component: "Button",
            dom_render_script: r#"
      const e = React.createElement;
      function AntdButtonFixture() {
        return e('button',
          { id: 'antd-button', className: 'ant-btn ant-btn-primary' },
          'AntD Primary'
        );
      }
      render(e(AntdButtonFixture));
"#,
            tsx_file: "AntdButtonFixture.tsx",
            tsx_source: r#"
import React from 'react';
import { Button } from 'antd';

export function AntdButtonFixture() {
  return (
    <Button id="antd-button" className="ant-btn ant-btn-primary">
      AntD Primary
    </Button>
  );
}
"#,
            root_component: "AntdButtonFixture",
            state_probe: None,
            interaction: None,
            behavior_tags: &["library-component-rendering", "style-dependent-layout"],
        },
        LibraryParityCase {
            id: "mui-textfield-basic",
            library_id: "mui",
            component: "TextField",
            dom_render_script: r#"
      const e = React.createElement;
      function MuiTextFieldFixture() {
        return e('form', { id: 'form' },
          e('input', {
            id: 'mui-name',
            className: 'MuiInputBase-input',
            defaultValue: 'Ada',
            placeholder: 'Name'
          }),
          e('span', { id: 'mui-name-label' }, 'Name')
        );
      }
      render(e(MuiTextFieldFixture));
"#,
            tsx_file: "MuiTextFieldFixture.tsx",
            tsx_source: r#"
import React from 'react';
import TextField from '@mui/material/TextField';

export function MuiTextFieldFixture() {
  return (
    <form id="form">
      <TextField
        id="mui-name"
        className="MuiInputBase-input"
        defaultValue="Ada"
        placeholder="Name"
      />
      <span id="mui-name-label">Name</span>
    </form>
  );
}
"#,
            root_component: "MuiTextFieldFixture",
            state_probe: Some(LibraryStateProbe {
                control_selector: "#mui-name",
                label_selector: Some("#mui-name-label"),
            }),
            interaction: Some(LibraryInteraction {
                kind: LibraryInteractionKind::Input,
                selector: "#mui-name",
                value: Some("Grace"),
            }),
            behavior_tags: &[
                "library-component-rendering",
                "controlled-input",
                "style-dependent-layout",
            ],
        },
        LibraryParityCase {
            id: "mui-checkbox-basic",
            library_id: "mui",
            component: "Checkbox",
            dom_render_script: r#"
      const e = React.createElement;
      function MuiCheckboxFixture() {
        return e('form', { id: 'form' },
          e('input', {
            id: 'mui-accept',
            className: 'MuiCheckbox-input',
            type: 'checkbox',
            defaultChecked: true,
            'aria-label': 'Accept library terms'
          }),
          e('span', { id: 'mui-accept-label' }, 'Accept library terms')
        );
      }
      render(e(MuiCheckboxFixture));
"#,
            tsx_file: "MuiCheckboxFixture.tsx",
            tsx_source: r#"
import React from 'react';
import Checkbox from '@mui/material/Checkbox';

export function MuiCheckboxFixture() {
  return (
    <form id="form">
      <Checkbox
        id="mui-accept"
        className="MuiCheckbox-input"
        type="checkbox"
        defaultChecked
        aria-label="Accept library terms"
      />
      <span id="mui-accept-label">Accept library terms</span>
    </form>
  );
}
"#,
            root_component: "MuiCheckboxFixture",
            state_probe: Some(LibraryStateProbe {
                control_selector: "#mui-accept",
                label_selector: Some("#mui-accept-label"),
            }),
            interaction: Some(LibraryInteraction {
                kind: LibraryInteractionKind::Click,
                selector: "#mui-accept",
                value: None,
            }),
            behavior_tags: &["library-component-rendering", "checkbox-state"],
        },
        LibraryParityCase {
            id: "antd-input-basic",
            library_id: "antd",
            component: "Input",
            dom_render_script: r#"
      const e = React.createElement;
      function AntdInputFixture() {
        return e('form', { id: 'form' },
          e('input', {
            id: 'antd-name',
            className: 'ant-input',
            defaultValue: 'Ada',
            placeholder: 'Name'
          }),
          e('span', { id: 'antd-name-label' }, 'Name')
        );
      }
      render(e(AntdInputFixture));
"#,
            tsx_file: "AntdInputFixture.tsx",
            tsx_source: r#"
import React from 'react';
import { Input } from 'antd';

export function AntdInputFixture() {
  return (
    <form id="form">
      <Input
        id="antd-name"
        className="ant-input"
        defaultValue="Ada"
        placeholder="Name"
      />
      <span id="antd-name-label">Name</span>
    </form>
  );
}
"#,
            root_component: "AntdInputFixture",
            state_probe: Some(LibraryStateProbe {
                control_selector: "#antd-name",
                label_selector: Some("#antd-name-label"),
            }),
            interaction: Some(LibraryInteraction {
                kind: LibraryInteractionKind::Input,
                selector: "#antd-name",
                value: Some("Grace"),
            }),
            behavior_tags: &["library-component-rendering", "controlled-input"],
        },
    ]
}

async fn assert_fixture_phase(
    case: &FixtureParityCase,
    phase: &str,
    react_page: &jet::browser::Page,
    app: &common::JetTestApp,
) -> Value {
    let react_bundle = browser_cli::dom_observation_bundle_from_page(react_page, "#root > *")
        .await
        .unwrap_or_else(|err| panic!("{} {phase} DOM observation: {err:#}", case.id));
    let expected = react_oracle::dom_tree_from_observation(&react_bundle);
    let wasm_bundle = browser_cli::observation_bundle(&app.demo_dir, &[])
        .await
        .unwrap_or_else(|err| panic!("{} {phase} WASM observation: {err:#}", case.id));
    let actual = react_oracle::normalize_wasm_observation_element_tree(&wasm_bundle);
    assert_eq!(
        actual,
        expected,
        "{}",
        react_oracle::fixture_diff_message(case.id, phase, &expected, &actual)
    );
    wasm_bundle
}

/// @spec .aw/tech-design/projects/jet/specs/4041.md#e2e-test
async fn assert_library_dom_phase(
    case: &LibraryParityCase,
    phase: &str,
    react_page: &jet::browser::Page,
    app: &common::JetTestApp,
) {
    let react_bundle = browser_cli::dom_observation_bundle_from_page(react_page, "#root > *")
        .await
        .unwrap_or_else(|err| {
            panic!(
                "{} {} {} DOM observation: {err:#}",
                case.library_id, case.id, phase
            )
        });
    let expected = react_oracle::dom_tree_from_observation(&react_bundle);
    let wasm_bundle = browser_cli::dom_observation_bundle_from_page(&app.page, "#jet-root > *")
        .await
        .unwrap_or_else(|err| {
            panic!(
                "{} {} {} WASM DOM observation: {err:#}",
                case.library_id, case.id, phase
            )
        });
    let actual = react_oracle::dom_tree_from_observation(&wasm_bundle);
    assert_eq!(
        actual,
        expected,
        "{}",
        react_oracle::library_fixture_diff_message(
            case.library_id,
            case.id,
            phase,
            &expected,
            &actual
        )
    );
}

/// @spec .aw/tech-design/projects/jet/specs/4072.md#schema
async fn library_control_state(
    case: &LibraryParityCase,
    phase: &str,
    page: &jet::browser::Page,
    root_selector: &str,
) -> Option<Value> {
    let probe = case.state_probe?;
    let bundle = browser_cli::dom_observation_bundle_from_page(page, root_selector)
        .await
        .unwrap_or_else(|err| {
            panic!(
                "{} {} {} DOM observation for state: {err:#}",
                case.library_id, case.id, phase
            )
        });
    let control = page
        .evaluate(&library_control_state_expr(probe))
        .await
        .unwrap_or_else(|err| {
            panic!(
                "{} {} {} control state: {err:#}",
                case.library_id, case.id, phase
            )
        });

    Some(json!({
        "schema_version": "jet.library_control_state.v1",
        "library_id": case.library_id,
        "fixture_id": case.id,
        "phase": phase,
        "tree": react_oracle::dom_tree_from_observation(&bundle),
        "control": control,
    }))
}

/// @spec .aw/tech-design/projects/jet/specs/4072.md#schema
fn library_control_state_expr(probe: LibraryStateProbe) -> String {
    let control_selector =
        serde_json::to_string(probe.control_selector).expect("selector serializes");
    let label_selector =
        serde_json::to_string(&probe.label_selector).expect("label selector serializes");
    format!(
        r#"
(() => {{
  const control = document.querySelector({control_selector});
  const labelSelector = {label_selector};
  const label = labelSelector ? document.querySelector(labelSelector) : null;
  const normalizeText = (node) => node ? (node.textContent || '').replace(/\s+/g, ' ').trim() : '';
  return {{
    exists: !!control,
    tag: control ? control.tagName.toLowerCase() : null,
    id: control ? (control.getAttribute('id') || '') : '',
    class_name: control ? (control.getAttribute('class') || '') : '',
    type: control ? (control.getAttribute('type') || control.type || '') : '',
    value: control && typeof control.value === 'string' ? control.value : '',
    checked: control && typeof control.checked === 'boolean' ? control.checked : null,
    placeholder: control ? (control.getAttribute('placeholder') || '') : '',
    aria_label: control ? (control.getAttribute('aria-label') || '') : '',
    text: normalizeText(control),
    label_text: normalizeText(label)
  }};
}})()
"#
    )
}

/// @spec .aw/tech-design/projects/jet/specs/4072.md#e2e-test
async fn assert_library_state_phase(
    case: &LibraryParityCase,
    phase: &str,
    react_page: &jet::browser::Page,
    app: &common::JetTestApp,
) {
    let Some(expected) = library_control_state(case, phase, react_page, "#root > *").await else {
        return;
    };
    let actual = library_control_state(case, phase, &app.page, "#jet-root > *")
        .await
        .expect("state probe exists for actual page");
    assert_eq!(
        actual,
        expected,
        "{}",
        react_oracle::library_fixture_diff_message(
            case.library_id,
            case.id,
            phase,
            &expected,
            &actual
        )
    );
}

fn replace_library_input_value_expr(selector: &str, value: &str) -> String {
    let selector = serde_json::to_string(selector).expect("selector serializes");
    let value = serde_json::to_string(value).expect("value serializes");
    format!(
        r#"
(() => {{
  const input = document.querySelector({selector});
  if (!input) throw new Error('missing input ' + {selector});
  const proto = input instanceof HTMLTextAreaElement
    ? HTMLTextAreaElement.prototype
    : HTMLInputElement.prototype;
  const setter = Object.getOwnPropertyDescriptor(proto, 'value').set;
  setter.call(input, {value});
  input.dispatchEvent(new Event('input', {{ bubbles: true, cancelable: true }}));
}})()
"#
    )
}

fn click_library_control_expr(selector: &str) -> String {
    let selector = serde_json::to_string(selector).expect("selector serializes");
    format!(
        r#"
(() => {{
  const control = document.querySelector({selector});
  if (!control) throw new Error('missing control ' + {selector});
  control.click();
}})()
"#
    )
}

/// @spec .aw/tech-design/projects/jet/specs/4072.md#e2e-test
async fn apply_library_interaction(page: &jet::browser::Page, interaction: LibraryInteraction) {
    let expr = match interaction.kind {
        LibraryInteractionKind::Input => replace_library_input_value_expr(
            interaction.selector,
            interaction
                .value
                .expect("input interaction requires replacement value"),
        ),
        LibraryInteractionKind::Click => click_library_control_expr(interaction.selector),
    };
    page.evaluate(&expr)
        .await
        .unwrap_or_else(|err| panic!("library interaction: {err:#}"));
    tokio::time::sleep(Duration::from_millis(200)).await;
}

/// @spec .aw/tech-design/projects/jet/specs/3944.md#e2e-test
async fn assert_fixture_layout_phase(
    case: &FixtureParityCase,
    phase: &str,
    react_page: &jet::browser::Page,
    app: &common::JetTestApp,
) {
    let expected = react_page
        .evaluate(&react_oracle::dom_layout_boxes_expr("#root > *"))
        .await
        .unwrap_or_else(|err| panic!("{} {phase} DOM layout boxes: {err:#}", case.id));
    let layout_tree = app
        .layout_tree()
        .await
        .unwrap_or_else(|err| panic!("{} {phase} WASM layout tree: {err:#}", case.id));
    let actual = react_oracle::normalize_wasm_layout_boxes(&layout_tree);
    assert!(
        react_oracle::layout_boxes_match(&expected, &actual),
        "{}",
        react_oracle::layout_diff_message(case.id, phase, &expected, &actual)
    );
}

/// @spec .aw/tech-design/projects/jet/specs/3958.md#e2e-test
async fn assert_fixture_canvas_paint_phase(
    case: &FixtureParityCase,
    phase: &str,
    app: &common::JetTestApp,
) {
    let ops = app
        .paint_ops()
        .await
        .unwrap_or_else(|err| panic!("{} {phase} WASM paint ops: {err:#}", case.id));
    let expected_methods = canvas_spy::expected_canvas_methods_from_paint_ops(&ops);
    let calls = app
        .page
        .evaluate(canvas_spy::captured_calls_expr())
        .await
        .unwrap_or_else(|err| panic!("{} {phase} canvas calls: {err:#}", case.id));
    let actual_methods = canvas_spy::canonical_canvas_methods(&calls);

    assert!(
        methods_contain_ordered_subsequence(&actual_methods, &expected_methods),
        "{}",
        react_oracle::paint_diff_message(case.id, phase, &expected_methods, &actual_methods)
    );
}

/// @spec .aw/tech-design/projects/jet/specs/3972.md#e2e-test
async fn assert_fixture_screenshot_phase(
    case: &FixtureParityCase,
    phase: &str,
    react_page: &jet::browser::Page,
    app: &common::JetTestApp,
) {
    let expected_png = react_page
        .screenshot()
        .await
        .unwrap_or_else(|err| panic!("{} {phase} DOM screenshot: {err:#}", case.id));
    let actual_png = app
        .page
        .screenshot()
        .await
        .unwrap_or_else(|err| panic!("{} {phase} WASM screenshot: {err:#}", case.id));
    let expected = react_oracle::screenshot_summary_from_png(&expected_png);
    let actual = react_oracle::screenshot_summary_from_png(&actual_png);

    assert!(
        react_oracle::screenshot_summaries_match(&expected, &actual),
        "{}",
        react_oracle::screenshot_diff_message(case.id, phase, &expected, &actual)
    );
}

const CONTROLLED_INPUT_TSX: &str = r#"
interface ControlledInputProps {
  initial: string;
}

export function ControlledInput({ initial }: ControlledInputProps) {
  const [name, setName] = useState(initial);
  return (
    <form id="form">
      <input
        id="name"
        value={name}
        placeholder="Name"
        onChange={(event) => setName(event.target.value)}
      />
      <span id="echo">hello {name}</span>
    </form>
  );
}
"#;

const CONTROLLED_TEXTAREA_TSX: &str = r#"
interface ControlledTextareaProps {
  initial: string;
}

export function ControlledTextarea({ initial }: ControlledTextareaProps) {
  const [bio, setBio] = useState(initial);
  return (
    <form id="form">
      <textarea
        id="bio"
        value={bio}
        placeholder="Bio"
        onChange={(event) => setBio(event.target.value)}
      ></textarea>
      <span id="echo">bio {bio}</span>
    </form>
  );
}
"#;

fn write_controlled_input_dom_project(root: &std::path::Path) {
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("create controlled input src dir");
    std::fs::write(src.join("ControlledInput.tsx"), CONTROLLED_INPUT_TSX)
        .expect("write controlled input TSX");
    std::fs::write(
        root.join("jet.config.toml"),
        r#"[wasm]
entry = "src/ControlledInput.tsx"
root_component = "ControlledInput"
renderer = "dom"
root_props = ["Ada"]
"#,
    )
    .expect("write controlled input jet config");
}

fn write_controlled_textarea_dom_project(root: &std::path::Path) {
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("create controlled textarea src dir");
    std::fs::write(src.join("ControlledTextarea.tsx"), CONTROLLED_TEXTAREA_TSX)
        .expect("write controlled textarea TSX");
    std::fs::write(
        root.join("jet.config.toml"),
        r#"[wasm]
entry = "src/ControlledTextarea.tsx"
root_component = "ControlledTextarea"
renderer = "dom"
root_props = ["Ada"]
"#,
    )
    .expect("write controlled textarea jet config");
}

fn write_library_dom_project(root: &std::path::Path, case: &LibraryParityCase) {
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("create library fixture src dir");
    std::fs::write(src.join(case.tsx_file), case.tsx_source).unwrap_or_else(|err| {
        panic!(
            "write {} {} TSX fixture {}: {err:#}",
            case.library_id, case.id, case.tsx_file
        )
    });
    std::fs::write(
        root.join("jet.config.toml"),
        format!(
            r#"[wasm]
entry = "src/{tsx_file}"
root_component = "{root_component}"
renderer = "dom"
"#,
            tsx_file = case.tsx_file,
            root_component = case.root_component
        ),
    )
    .unwrap_or_else(|err| panic!("write {} jet config: {err:#}", case.id));
}

async fn controlled_input_dom_state(page: &jet::browser::Page, root_selector: &str) -> Value {
    let bundle = browser_cli::dom_observation_bundle_from_page(page, root_selector)
        .await
        .unwrap_or_else(|err| panic!("{root_selector} DOM observation: {err:#}"));
    let details = page
        .evaluate(&react_oracle::controlled_input_dom_state_expr(
            "#name", "#echo",
        ))
        .await
        .unwrap_or_else(|err| panic!("{root_selector} controlled input state: {err:#}"));

    json!({
        "schema_version": "jet.controlled_input_dom_state.v1",
        "tree": react_oracle::dom_tree_from_observation(&bundle),
        "input_value": details.get("input_value").cloned().unwrap_or(Value::Null),
        "placeholder": details.get("placeholder").cloned().unwrap_or(Value::Null),
        "label_text": details.get("label_text").cloned().unwrap_or(Value::Null),
    })
}

async fn controlled_textarea_dom_state(page: &jet::browser::Page, root_selector: &str) -> Value {
    let bundle = browser_cli::dom_observation_bundle_from_page(page, root_selector)
        .await
        .unwrap_or_else(|err| panic!("{root_selector} DOM observation: {err:#}"));
    let details = page
        .evaluate(&react_oracle::controlled_textarea_dom_state_expr(
            "#bio", "#echo",
        ))
        .await
        .unwrap_or_else(|err| panic!("{root_selector} controlled textarea state: {err:#}"));

    json!({
        "schema_version": "jet.controlled_textarea_dom_state.v1",
        "tree": react_oracle::dom_tree_from_observation(&bundle),
        "textarea_value": details.get("textarea_value").cloned().unwrap_or(Value::Null),
        "placeholder": details.get("placeholder").cloned().unwrap_or(Value::Null),
        "label_text": details.get("label_text").cloned().unwrap_or(Value::Null),
    })
}

fn replace_input_value_expr(selector: &str, value: &str) -> String {
    let selector = serde_json::to_string(selector).expect("selector serializes");
    let value = serde_json::to_string(value).expect("value serializes");
    format!(
        r#"
(() => {{
  const input = document.querySelector({selector});
  if (!input) throw new Error('missing input ' + {selector});
  const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
  setter.call(input, {value});
  input.dispatchEvent(new Event('input', {{ bubbles: true, cancelable: true }}));
}})()
"#
    )
}

fn replace_textarea_value_expr(selector: &str, value: &str) -> String {
    let selector = serde_json::to_string(selector).expect("selector serializes");
    let value = serde_json::to_string(value).expect("value serializes");
    format!(
        r#"
(() => {{
  const textarea = document.querySelector({selector});
  if (!textarea) throw new Error('missing textarea ' + {selector});
  const setter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set;
  setter.call(textarea, {value});
  textarea.dispatchEvent(new Event('input', {{ bubbles: true, cancelable: true }}));
}})()
"#
    )
}

async fn replace_input_value(page: &jet::browser::Page, value: &str) {
    page.evaluate(&replace_input_value_expr("#name", value))
        .await
        .unwrap_or_else(|err| panic!("replace input value: {err:#}"));
    tokio::time::sleep(Duration::from_millis(200)).await;
}

async fn replace_textarea_value(page: &jet::browser::Page, value: &str) {
    page.evaluate(&replace_textarea_value_expr("#bio", value))
        .await
        .unwrap_or_else(|err| panic!("replace textarea value: {err:#}"));
    tokio::time::sleep(Duration::from_millis(200)).await;
}

/// @spec .aw/tech-design/projects/jet/specs/3958.md#unit-test
fn methods_contain_ordered_subsequence(actual: &[String], expected: &[String]) -> bool {
    let mut cursor = 0;
    for method in actual {
        if expected.get(cursor) == Some(method) {
            cursor += 1;
            if cursor == expected.len() {
                return true;
            }
        }
    }
    expected.is_empty()
}

/// @spec .aw/tech-design/projects/jet/specs/4041.md#unit-test
#[test]
fn library_fixture_manifest_has_executable_mui_and_antd() {
    let workspace = workspace_root();
    let manifest_path = workspace.join(LIBRARY_FIXTURE_MANIFEST);
    let manifest = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|err| panic!("read {}: {err:#}", manifest_path.display()));
    let value: toml::Value =
        toml::from_str(&manifest).unwrap_or_else(|err| panic!("parse fixture manifest: {err:#}"));
    assert_eq!(
        value
            .get("schema_version")
            .and_then(toml::Value::as_integer),
        Some(1)
    );

    let libraries = value
        .get("libraries")
        .and_then(toml::Value::as_array)
        .expect("libraries array");
    for (library_id, package_name) in [("mui", "@mui/material"), ("antd", "antd")] {
        let library = libraries
            .iter()
            .find(|library| library.get("id").and_then(toml::Value::as_str) == Some(library_id))
            .unwrap_or_else(|| panic!("missing library row {library_id}"));
        assert_eq!(
            library.get("package_name").and_then(toml::Value::as_str),
            Some(package_name)
        );
        assert_eq!(
            library.get("status").and_then(toml::Value::as_str),
            Some("executable")
        );
    }

    let fixtures = value
        .get("fixtures")
        .and_then(toml::Value::as_array)
        .expect("fixtures array");
    for case in library_parity_cases() {
        let fixture = fixtures
            .iter()
            .find(|fixture| fixture.get("id").and_then(toml::Value::as_str) == Some(case.id))
            .unwrap_or_else(|| panic!("missing fixture row {}", case.id));
        assert_eq!(
            fixture.get("library_id").and_then(toml::Value::as_str),
            Some(case.library_id)
        );
        assert_eq!(
            fixture.get("component").and_then(toml::Value::as_str),
            Some(case.component)
        );
        assert_eq!(
            fixture.get("status").and_then(toml::Value::as_str),
            Some("executable")
        );
        let targets = fixture
            .get("targets")
            .and_then(toml::Value::as_array)
            .expect("targets array");
        assert!(targets.iter().any(|target| target.as_str() == Some("dom")));
        assert!(targets.iter().any(|target| target.as_str() == Some("wasm")));
        let channels = fixture
            .get("channels")
            .and_then(toml::Value::as_array)
            .expect("channels array");
        assert!(channels
            .iter()
            .any(|channel| channel.as_str() == Some("dom-tree")));
    }

    let package_path = workspace.join(LIBRARY_PACKAGE_MANIFEST);
    let package_json = std::fs::read_to_string(&package_path)
        .unwrap_or_else(|err| panic!("read {}: {err:#}", package_path.display()));
    let package: Value =
        serde_json::from_str(&package_json).unwrap_or_else(|err| panic!("parse package: {err:#}"));
    for dependency in ["react", "react-dom", "@mui/material", "antd"] {
        assert!(
            package["dependencies"][dependency].is_string(),
            "missing dependency {dependency}"
        );
    }
}

/// @spec .aw/tech-design/projects/jet/specs/4130.md#unit-test
#[test]
fn dom_wasm_external_behavior_corpus_covers_issue_4130_targets() {
    let mut tags = std::collections::BTreeSet::new();
    for case in fixture_parity_cases() {
        tags.extend(case.behavior_tags.iter().copied());
    }
    for case in library_parity_cases() {
        tags.extend(case.behavior_tags.iter().copied());
    }

    for required in [
        "rendered-text",
        "controlled-input",
        "checkbox-state",
        "button-event",
        "style-dependent-layout",
        "library-component-rendering",
    ] {
        assert!(
            tags.contains(required),
            "DOM/WASM external behavior corpus is missing required tag {required}; tags={tags:?}"
        );
    }

    assert!(
        fixture_parity_cases().len() >= 3,
        "first-party fixture corpus should cover multiple React control shapes"
    );
    assert!(
        library_parity_cases().len() >= 5,
        "library fixture corpus should include MUI, AntD, and form/control cases"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn page_add_init_script_runs_before_navigation() {
    if !common::chromium_available() {
        eprintln!("skipping: Chromium unavailable");
        return;
    }

    let browser = Browser::launch(browser_options())
        .await
        .expect("launch browser");
    let page = browser.new_page().await.expect("new page");
    page.add_init_script("window.__jet_pre_nav = 41;")
        .await
        .expect("add init script");

    page.goto("data:text/html,<script>window.__jet_seen = window.__jet_pre_nav;</script>")
        .await
        .expect("goto data page");
    let seen = page
        .evaluate("window.__jet_seen")
        .await
        .expect("evaluate pre-nav marker");
    assert_eq!(seen.as_i64(), Some(41));

    browser.close().await.ok();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn canvas_spy_records_draw_calls_before_page_script_runs() {
    if !common::chromium_available() {
        eprintln!("skipping: Chromium unavailable");
        return;
    }

    let browser = Browser::launch(browser_options())
        .await
        .expect("launch browser");
    let page = browser.new_page().await.expect("new page");
    page.add_init_script(canvas_spy::init_script())
        .await
        .expect("add canvas spy");

    page.goto(
        "data:text/html,\
         <canvas id='c' width='100' height='50'></canvas>\
         <script>\
           const ctx = document.getElementById('c').getContext('2d');\
           ctx.fillRect(1,2,3,4);\
           ctx.fillText('hello', 5, 6);\
         </script>",
    )
    .await
    .expect("goto canvas page");

    let calls = page
        .evaluate(canvas_spy::captured_calls_expr())
        .await
        .expect("canvas calls");
    let methods = canvas_spy::canonical_canvas_methods(&calls);
    assert!(methods.contains(&"fillRect".to_string()), "{methods:?}");
    assert!(methods.contains(&"fillText".to_string()), "{methods:?}");

    browser.close().await.ok();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn multi_fixture_dom_wasm_parity() {
    common::require_env();
    let _guard = live_wasm_e2e_guard().await;

    let workspace = workspace_root();
    react_oracle::require_react_oracle_env(&workspace);

    let react_browser = Browser::launch(browser_options())
        .await
        .expect("launch React browser");
    let temp = tempfile::tempdir().expect("tempdir");

    for case in fixture_parity_cases() {
        let react_html = react_oracle::fixture_html(&workspace, case.dom_render_script)
            .unwrap_or_else(|| panic!("{} React oracle fixture html", case.id));
        let html_path = temp.path().join(format!("{}.html", case.id));
        std::fs::write(&html_path, react_html).expect("write React oracle page");

        let react_page = react_browser.new_page().await.expect("new React page");
        react_page
            .goto(&format!("file://{}", html_path.display()))
            .await
            .unwrap_or_else(|err| panic!("{} goto React oracle: {err:#}", case.id));
        tokio::time::sleep(Duration::from_millis(200)).await;

        let app_root = workspace.join("examples").join(case.example_name);
        let app = common::JetTestApp::launch_project(&app_root)
            .await
            .unwrap_or_else(|err| panic!("{} launch Jet app: {err:#}", case.id));

        assert_fixture_phase(&case, "initial", &react_page, &app).await;

        if let Some(interaction) = &case.interaction {
            react_page
                .evaluate(interaction.dom_script)
                .await
                .unwrap_or_else(|err| panic!("{} DOM interaction: {err:#}", case.id));
            app.click_canvas(interaction.wasm_click.0, interaction.wasm_click.1)
                .await
                .unwrap_or_else(|err| panic!("{} WASM interaction: {err:#}", case.id));
            tokio::time::sleep(Duration::from_millis(200)).await;

            let after_bundle = assert_fixture_phase(&case, "after", &react_page, &app).await;
            if let Some(expected_hook) = interaction.expected_hook {
                assert!(
                    react_oracle::wasm_observation_has_hook_i64(&after_bundle, expected_hook),
                    "{} after-click observation bundle must include hook value {expected_hook}:\n{}",
                    case.id,
                    serde_json::to_string_pretty(&after_bundle)
                        .unwrap_or_else(|_| after_bundle.to_string())
                );
            }
        }

        app.shutdown().await;
    }

    react_browser.close().await.ok();
}

/// @spec .aw/tech-design/projects/jet/specs/4041.md#e2e-test
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn library_dom_wasm_parity() {
    common::require_env();
    let _guard = live_wasm_e2e_guard().await;

    let workspace = workspace_root();
    react_oracle::require_react_oracle_env(&workspace);

    let react_browser = Browser::launch(browser_options())
        .await
        .expect("launch React browser");
    let temp = tempfile::tempdir().expect("tempdir");

    for case in library_parity_cases() {
        let react_html = react_oracle::fixture_html(&workspace, case.dom_render_script)
            .unwrap_or_else(|| panic!("{} React oracle fixture html", case.id));
        let html_path = temp.path().join(format!("{}.html", case.id));
        std::fs::write(&html_path, react_html).expect("write React library oracle page");

        let react_page = react_browser.new_page().await.expect("new React page");
        react_page
            .goto(&format!("file://{}", html_path.display()))
            .await
            .unwrap_or_else(|err| panic!("{} goto React library oracle: {err:#}", case.id));
        tokio::time::sleep(Duration::from_millis(200)).await;

        let app_root = temp.path().join(case.id);
        write_library_dom_project(&app_root, &case);
        let app = common::JetTestApp::launch_project(&app_root)
            .await
            .unwrap_or_else(|err| panic!("{} launch Jet library app: {err:#}", case.id));

        assert_library_dom_phase(&case, "initial", &react_page, &app).await;
        assert_library_state_phase(&case, "initial", &react_page, &app).await;

        if let Some(interaction) = case.interaction {
            apply_library_interaction(&react_page, interaction).await;
            apply_library_interaction(&app.page, interaction).await;
            assert_library_state_phase(&case, "after", &react_page, &app).await;
        }

        app.shutdown().await;
    }

    react_browser.close().await.ok();
}

/// @spec .aw/tech-design/projects/jet/specs/3944.md#e2e-test
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn multi_fixture_dom_wasm_layout_parity() {
    common::require_env();
    let _guard = live_wasm_e2e_guard().await;

    let workspace = workspace_root();
    react_oracle::require_react_oracle_env(&workspace);

    let react_browser = Browser::launch(browser_options())
        .await
        .expect("launch React browser");
    let temp = tempfile::tempdir().expect("tempdir");

    for case in fixture_parity_cases() {
        let react_html = react_oracle::fixture_html(&workspace, case.dom_render_script)
            .unwrap_or_else(|| panic!("{} React oracle fixture html", case.id));
        let html_path = temp.path().join(format!("{}-layout.html", case.id));
        std::fs::write(&html_path, react_html).expect("write React oracle page");

        let react_page = react_browser.new_page().await.expect("new React page");
        react_page
            .goto(&format!("file://{}", html_path.display()))
            .await
            .unwrap_or_else(|err| panic!("{} goto React oracle: {err:#}", case.id));
        tokio::time::sleep(Duration::from_millis(200)).await;

        let app_root = workspace.join("examples").join(case.example_name);
        let app = common::JetTestApp::launch_project(&app_root)
            .await
            .unwrap_or_else(|err| panic!("{} launch Jet app: {err:#}", case.id));

        assert_fixture_layout_phase(&case, "initial", &react_page, &app).await;

        if let Some(interaction) = &case.interaction {
            react_page
                .evaluate(interaction.dom_script)
                .await
                .unwrap_or_else(|err| panic!("{} DOM interaction: {err:#}", case.id));
            app.click_canvas(interaction.wasm_click.0, interaction.wasm_click.1)
                .await
                .unwrap_or_else(|err| panic!("{} WASM interaction: {err:#}", case.id));
            tokio::time::sleep(Duration::from_millis(200)).await;

            assert_fixture_layout_phase(&case, "after", &react_page, &app).await;
        }

        app.shutdown().await;
    }

    react_browser.close().await.ok();
}

/// @spec .aw/tech-design/projects/jet/specs/3958.md#e2e-test
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn multi_fixture_dom_wasm_canvas_paint_parity() {
    common::require_env();
    let _guard = live_wasm_e2e_guard().await;

    let workspace = workspace_root();

    for case in fixture_parity_cases() {
        let app_root = workspace.join("examples").join(case.example_name);
        let app = common::JetTestApp::launch_project_with_init_scripts(
            &app_root,
            &[canvas_spy::init_script()],
        )
        .await
        .unwrap_or_else(|err| panic!("{} launch Jet app: {err:#}", case.id));

        assert_fixture_canvas_paint_phase(&case, "initial", &app).await;

        if let Some(interaction) = &case.interaction {
            app.page
                .evaluate(canvas_spy::clear_expr())
                .await
                .unwrap_or_else(|err| panic!("{} clear canvas spy: {err:#}", case.id));
            app.click_canvas(interaction.wasm_click.0, interaction.wasm_click.1)
                .await
                .unwrap_or_else(|err| panic!("{} WASM interaction: {err:#}", case.id));
            tokio::time::sleep(Duration::from_millis(200)).await;

            assert_fixture_canvas_paint_phase(&case, "after", &app).await;
        }

        app.shutdown().await;
    }
}

/// @spec .aw/tech-design/projects/jet/specs/3972.md#e2e-test
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn multi_fixture_dom_wasm_screenshot_pixel_parity() {
    common::require_env();
    let _guard = live_wasm_e2e_guard().await;

    let workspace = workspace_root();
    react_oracle::require_react_oracle_env(&workspace);

    let react_browser = Browser::launch(browser_options())
        .await
        .expect("launch React browser");
    let temp = tempfile::tempdir().expect("tempdir");

    for case in fixture_parity_cases() {
        let react_html = react_oracle::fixture_html(&workspace, case.dom_render_script)
            .unwrap_or_else(|| panic!("{} React oracle fixture html", case.id));
        let html_path = temp.path().join(format!("{}-screenshot.html", case.id));
        std::fs::write(&html_path, react_html).expect("write React oracle page");

        let react_page = react_browser.new_page().await.expect("new React page");
        react_page
            .goto(&format!("file://{}", html_path.display()))
            .await
            .unwrap_or_else(|err| panic!("{} goto React oracle: {err:#}", case.id));
        tokio::time::sleep(Duration::from_millis(200)).await;

        let app_root = workspace.join("examples").join(case.example_name);
        let app = common::JetTestApp::launch_project(&app_root)
            .await
            .unwrap_or_else(|err| panic!("{} launch Jet app: {err:#}", case.id));

        assert_fixture_screenshot_phase(&case, "initial", &react_page, &app).await;

        if let Some(interaction) = &case.interaction {
            react_page
                .evaluate(interaction.dom_script)
                .await
                .unwrap_or_else(|err| panic!("{} DOM interaction: {err:#}", case.id));
            app.click_canvas(interaction.wasm_click.0, interaction.wasm_click.1)
                .await
                .unwrap_or_else(|err| panic!("{} WASM interaction: {err:#}", case.id));
            tokio::time::sleep(Duration::from_millis(200)).await;

            assert_fixture_screenshot_phase(&case, "after", &react_page, &app).await;
        }

        app.shutdown().await;
    }

    react_browser.close().await.ok();
}

/// @spec .aw/tech-design/projects/jet/specs/4004.md#e2e-test
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dom_renderer_controlled_input_parity() {
    common::require_env();
    let _guard = live_wasm_e2e_guard().await;

    let workspace = workspace_root();
    react_oracle::require_react_oracle_env(&workspace);

    let temp = tempfile::tempdir().expect("tempdir");
    let react_html = react_oracle::fixture_html(
        &workspace,
        r#"
      const e = React.createElement;
      function ControlledInput(props) {
        const [name, setName] = React.useState(props.initial);
        return e('form', { id: 'form' },
          e('input', {
            id: 'name',
            value: name,
            placeholder: 'Name',
            onChange: (event) => setName(event.target.value)
          }),
          e('span', { id: 'echo' }, 'hello ', name)
        );
      }
      render(e(ControlledInput, { initial: 'Ada' }));
"#,
    )
    .expect("React oracle fixture html");
    let react_html_path = temp.path().join("controlled-input-react.html");
    std::fs::write(&react_html_path, react_html).expect("write React oracle page");

    let react_browser = Browser::launch(browser_options())
        .await
        .expect("launch React browser");
    let react_page = react_browser.new_page().await.expect("new React page");
    react_page
        .goto(&format!("file://{}", react_html_path.display()))
        .await
        .expect("goto React oracle");
    tokio::time::sleep(Duration::from_millis(200)).await;

    let app_root = temp.path().join("controlled-input-jet");
    write_controlled_input_dom_project(&app_root);
    let app = common::JetTestApp::launch_project(&app_root)
        .await
        .expect("launch Jet DOM renderer app");

    let expected_initial = controlled_input_dom_state(&react_page, "#root > *").await;
    let actual_initial = controlled_input_dom_state(&app.page, "#jet-root > *").await;
    assert_eq!(
        actual_initial,
        expected_initial,
        "{}",
        react_oracle::controlled_input_diff_message("initial", &expected_initial, &actual_initial)
    );

    replace_input_value(&react_page, "Grace").await;
    replace_input_value(&app.page, "Grace").await;

    let expected_after = controlled_input_dom_state(&react_page, "#root > *").await;
    let actual_after = controlled_input_dom_state(&app.page, "#jet-root > *").await;
    assert_eq!(
        actual_after,
        expected_after,
        "{}",
        react_oracle::controlled_input_diff_message("after", &expected_after, &actual_after)
    );

    app.shutdown().await;
    react_browser.close().await.ok();
}

/// @spec .aw/tech-design/projects/jet/specs/4015.md#e2e-test
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dom_renderer_controlled_textarea_parity() {
    common::require_env();
    let _guard = live_wasm_e2e_guard().await;

    let workspace = workspace_root();
    react_oracle::require_react_oracle_env(&workspace);

    let temp = tempfile::tempdir().expect("tempdir");
    let react_html = react_oracle::fixture_html(
        &workspace,
        r#"
      const e = React.createElement;
      function ControlledTextarea(props) {
        const [bio, setBio] = React.useState(props.initial);
        return e('form', { id: 'form' },
          e('textarea', {
            id: 'bio',
            value: bio,
            placeholder: 'Bio',
            onChange: (event) => setBio(event.target.value)
          }),
          e('span', { id: 'echo' }, 'bio ', bio)
        );
      }
      render(e(ControlledTextarea, { initial: 'Ada' }));
"#,
    )
    .expect("React oracle fixture html");
    let react_html_path = temp.path().join("controlled-textarea-react.html");
    std::fs::write(&react_html_path, react_html).expect("write React oracle page");

    let react_browser = Browser::launch(browser_options())
        .await
        .expect("launch React browser");
    let react_page = react_browser.new_page().await.expect("new React page");
    react_page
        .goto(&format!("file://{}", react_html_path.display()))
        .await
        .expect("goto React oracle");
    tokio::time::sleep(Duration::from_millis(200)).await;

    let app_root = temp.path().join("controlled-textarea-jet");
    write_controlled_textarea_dom_project(&app_root);
    let app = common::JetTestApp::launch_project(&app_root)
        .await
        .expect("launch Jet DOM renderer textarea app");

    let expected_initial = controlled_textarea_dom_state(&react_page, "#root > *").await;
    let actual_initial = controlled_textarea_dom_state(&app.page, "#jet-root > *").await;
    assert_eq!(
        actual_initial,
        expected_initial,
        "{}",
        react_oracle::controlled_textarea_diff_message(
            "initial",
            &expected_initial,
            &actual_initial
        )
    );

    replace_textarea_value(&react_page, "Grace Hopper").await;
    replace_textarea_value(&app.page, "Grace Hopper").await;

    let expected_after = controlled_textarea_dom_state(&react_page, "#root > *").await;
    let actual_after = controlled_textarea_dom_state(&app.page, "#jet-root > *").await;
    assert_eq!(
        actual_after,
        expected_after,
        "{}",
        react_oracle::controlled_textarea_diff_message("after", &expected_after, &actual_after)
    );

    app.shutdown().await;
    react_browser.close().await.ok();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn counter_demo_matches_react_dom_oracle_initial_and_after_click() {
    common::require_env();
    let _guard = live_wasm_e2e_guard().await;

    let workspace = workspace_root();
    react_oracle::require_react_oracle_env(&workspace);

    let react_html =
        react_oracle::counter_fixture_html(&workspace).expect("React oracle fixture html");
    let temp = tempfile::tempdir().expect("tempdir");
    let html_path = temp.path().join("counter.html");
    std::fs::write(&html_path, react_html).expect("write React oracle page");

    let react_browser = Browser::launch(browser_options())
        .await
        .expect("launch React browser");
    let react_page = react_browser.new_page().await.expect("new React page");
    react_page
        .goto(&format!("file://{}", html_path.display()))
        .await
        .expect("goto React oracle");

    let app = common::JetTestApp::launch("counter-demo")
        .await
        .expect("launch Jet app");

    let react_initial_bundle =
        browser_cli::dom_observation_bundle_from_page(&react_page, "#root > *")
            .await
            .expect("React initial DOM observation bundle");
    let react_initial = react_oracle::dom_tree_from_observation(&react_initial_bundle);
    let jet_initial_bundle = browser_cli::observation_bundle(&app.demo_dir, &[])
        .await
        .expect("Jet initial WASM observation bundle");
    let jet_initial = react_oracle::normalize_wasm_observation_element_tree(&jet_initial_bundle);
    assert_eq!(
        jet_initial,
        react_initial,
        "{}",
        react_oracle::diff_message("counter initial", &react_initial, &jet_initial)
    );

    react_page
        .evaluate(
            "document.getElementById('inc').dispatchEvent(new MouseEvent('click', \
             { bubbles: true, cancelable: true, view: window }))",
        )
        .await
        .expect("click React counter");
    app.click_canvas(30.0, 12.0)
        .await
        .expect("click Jet counter");
    tokio::time::sleep(Duration::from_millis(200)).await;

    let react_after_bundle =
        browser_cli::dom_observation_bundle_from_page(&react_page, "#root > *")
            .await
            .expect("React after click DOM observation bundle");
    let react_after = react_oracle::dom_tree_from_observation(&react_after_bundle);
    let jet_after_bundle = browser_cli::observation_bundle(&app.demo_dir, &[])
        .await
        .expect("Jet after click WASM observation bundle");
    let jet_after = react_oracle::normalize_wasm_observation_element_tree(&jet_after_bundle);
    assert!(
        react_oracle::wasm_observation_has_hook_i64(&jet_after_bundle, 1),
        "Jet after-click observation bundle must include hook value 1:\n{}",
        serde_json::to_string_pretty(&jet_after_bundle)
            .unwrap_or_else(|_| jet_after_bundle.to_string())
    );
    assert_eq!(
        jet_after,
        react_after,
        "{}",
        react_oracle::diff_message("counter after click", &react_after, &jet_after)
    );

    app.shutdown().await;
    react_browser.close().await.ok();
}

#[test]
fn jet_element_tree_normalizes_to_react_host_tree_shape() {
    let jet_tree = json!({
        "kind": "intrinsic",
        "tag": "button",
        "props": {
            "id": "inc",
            "class_name": "primary",
            "has_on_click": true,
            "has_on_change": false
        },
        "children": [
            { "kind": "text", "text": "count: " },
            { "kind": "text", "text": "0" }
        ]
    });

    let normalized = react_oracle::normalize_jet_element_tree(&jet_tree);
    assert_eq!(
        normalized,
        json!({
            "kind": "element",
            "tag": "button",
            "attrs": {
                "id": "inc",
                "class": "primary"
            },
            "children": [
                { "kind": "text", "text": "count: 0" }
            ]
        })
    );
}

#[test]
fn fixture_diff_message_is_machine_readable() {
    let expected = json!({"kind": "element", "tag": "div"});
    let actual = json!({"kind": "element", "tag": "span"});
    let message = react_oracle::fixture_diff_message("fixture-a", "initial", &expected, &actual);
    let json_start = message.find('{').expect("message contains JSON payload");
    let payload: Value = serde_json::from_str(&message[json_start..]).expect("payload JSON");

    assert_eq!(payload["failure_kind"], "dom_wasm_parity_mismatch");
    assert_eq!(payload["expected_source"], "react_dom");
    assert_eq!(payload["actual_source"], "jet_wasm");
    assert_eq!(payload["fixture_id"], "fixture-a");
    assert_eq!(payload["phase"], "initial");
    assert_eq!(payload["expected"], expected);
    assert_eq!(payload["actual"], actual);
}

#[test]
fn library_fixture_diff_message_classifies_parity_mismatch() {
    let expected = json!({"kind": "element", "tag": "button"});
    let actual = json!({"kind": "element", "tag": "div"});
    let message = react_oracle::library_fixture_diff_message(
        "mui",
        "mui-button-basic",
        "after",
        &expected,
        &actual,
    );
    let json_start = message.find('{').expect("message contains JSON payload");
    let payload: Value = serde_json::from_str(&message[json_start..]).expect("payload JSON");

    assert_eq!(payload["failure_kind"], "library_dom_wasm_parity_mismatch");
    assert_eq!(payload["expected_source"], "react_dom");
    assert_eq!(payload["actual_source"], "jet_wasm");
    assert_eq!(payload["library_id"], "mui");
    assert_eq!(payload["fixture_id"], "mui-button-basic");
    assert_eq!(payload["phase"], "after");
    assert_eq!(payload["expected"], expected);
    assert_eq!(payload["actual"], actual);
}

#[test]
fn layout_diff_message_is_machine_readable() {
    let expected = json!({
        "schema_version": "jet.layout_boxes.v1",
        "boxes": [
            {
                "key": "0",
                "kind": "element",
                "tag": "button",
                "id": "cta",
                "rect": { "x": 0.0, "y": 0.0, "w": 120.0, "h": 24.0 }
            }
        ]
    });
    let actual = json!({
        "schema_version": "jet.layout_boxes.v1",
        "boxes": [
            {
                "key": "0",
                "kind": "element",
                "tag": "button",
                "id": "cta",
                "rect": { "x": 0.0, "y": 0.0, "w": 800.0, "h": 24.0 }
            }
        ]
    });
    let message = react_oracle::layout_diff_message("fixture-a", "after", &expected, &actual);
    let json_start = message.find('{').expect("message contains JSON payload");
    let payload: Value = serde_json::from_str(&message[json_start..]).expect("payload JSON");

    assert_eq!(payload["failure_kind"], "layout_dom_wasm_parity_mismatch");
    assert_eq!(payload["fixture_id"], "fixture-a");
    assert_eq!(payload["phase"], "after");
    assert_eq!(
        payload["tolerance_css_px"],
        json!(react_oracle::LAYOUT_TOLERANCE_CSS_PX)
    );
    assert_eq!(payload["expected"], expected);
    assert_eq!(payload["actual"], actual);
}

/// @spec .aw/tech-design/projects/jet/specs/3972.md#unit-test
#[test]
fn screenshot_diff_message_is_machine_readable() {
    let expected = json!({
        "schema_version": "jet.screenshot_summary.v1",
        "width": 800,
        "height": 600,
        "foreground_count": 120,
        "bounds": { "x": 0, "y": 0, "w": 50, "h": 18 }
    });
    let actual = json!({
        "schema_version": "jet.screenshot_summary.v1",
        "width": 800,
        "height": 600,
        "foreground_count": 122,
        "bounds": { "x": 1, "y": 0, "w": 51, "h": 18 }
    });
    let message = react_oracle::screenshot_diff_message("fixture-a", "initial", &expected, &actual);
    let json_start = message.find('{').expect("message contains JSON payload");
    let payload: Value = serde_json::from_str(&message[json_start..]).expect("payload JSON");

    assert_eq!(
        payload["failure_kind"],
        "screenshot_dom_wasm_parity_mismatch"
    );
    assert_eq!(payload["fixture_id"], "fixture-a");
    assert_eq!(payload["phase"], "initial");
    assert_eq!(
        payload["tolerance"]["bounds_px"],
        json!(react_oracle::SCREENSHOT_BOUNDS_TOLERANCE_PX)
    );
    assert_eq!(payload["expected"], expected);
    assert_eq!(payload["actual"], actual);
}

/// @spec .aw/tech-design/projects/jet/specs/3972.md#unit-test
#[test]
fn screenshot_summary_comparator_respects_tolerance() {
    let expected = json!({
        "schema_version": "jet.screenshot_summary.v1",
        "width": 800,
        "height": 600,
        "foreground_count": 100,
        "bounds": { "x": 0, "y": 0, "w": 50, "h": 18 }
    });
    let compatible = json!({
        "schema_version": "jet.screenshot_summary.v1",
        "width": 800,
        "height": 600,
        "foreground_count": 125,
        "bounds": { "x": 2, "y": 1, "w": 54, "h": 20 }
    });
    let drifted = json!({
        "schema_version": "jet.screenshot_summary.v1",
        "width": 800,
        "height": 600,
        "foreground_count": 100,
        "bounds": { "x": 80, "y": 0, "w": 50, "h": 18 }
    });

    assert!(react_oracle::screenshot_summaries_match(
        &expected,
        &compatible
    ));
    assert!(!react_oracle::screenshot_summaries_match(
        &expected, &drifted
    ));
}

/// @spec .aw/tech-design/projects/jet/specs/3972.md#unit-test
#[test]
fn screenshot_summary_uses_dominant_background_when_top_left_is_foreground() {
    let mut image = image::RgbaImage::from_pixel(4, 4, image::Rgba([250, 250, 250, 255]));
    image.put_pixel(0, 0, image::Rgba([34, 34, 34, 255]));
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .expect("encode png");

    let summary = react_oracle::screenshot_summary_from_png(bytes.get_ref());

    assert_eq!(summary["foreground_count"], json!(1));
    assert_eq!(summary["bounds"], json!({ "x": 0, "y": 0, "w": 1, "h": 1 }));
}

/// @spec .aw/tech-design/projects/jet/specs/3958.md#unit-test
#[test]
fn paint_diff_message_is_machine_readable() {
    let expected = vec!["fillRect".to_string(), "fillText".to_string()];
    let actual = vec![
        "save".to_string(),
        "fillRect".to_string(),
        "fillText".to_string(),
    ];
    let message = react_oracle::paint_diff_message("fixture-a", "initial", &expected, &actual);
    let json_start = message.find('{').expect("message contains JSON payload");
    let payload: Value = serde_json::from_str(&message[json_start..]).expect("payload JSON");

    assert_eq!(payload["failure_kind"], "paint_dom_wasm_parity_mismatch");
    assert_eq!(payload["expected_source"], "jet_wasm_paint_ops");
    assert_eq!(payload["actual_source"], "canvas_runtime_calls");
    assert_eq!(payload["fixture_id"], "fixture-a");
    assert_eq!(payload["phase"], "initial");
    assert_eq!(payload["expected_methods"], json!(expected));
    assert_eq!(payload["actual_methods"], json!(actual));
}

/// @spec .aw/tech-design/projects/jet/specs/3958.md#unit-test
#[test]
fn method_subsequence_matches_ordered_canvas_calls() {
    let actual = vec![
        "save".to_string(),
        "fillRect".to_string(),
        "beginPath".to_string(),
        "fillText".to_string(),
        "restore".to_string(),
    ];
    assert!(methods_contain_ordered_subsequence(
        &actual,
        &["fillRect".to_string(), "fillText".to_string()]
    ));
    assert!(!methods_contain_ordered_subsequence(
        &actual,
        &["fillText".to_string(), "fillRect".to_string()]
    ));
}

#[test]
fn paint_ops_map_to_canvas_method_sequence() {
    let ops = json!([
        { "op": "fill_rect", "rect": {}, "color": {} },
        { "op": "text", "origin": {}, "content": "count: 0", "font": {}, "color": {} },
        { "op": "stroke_rect", "rect": {}, "color": {}, "width": 1.0 }
    ]);
    assert_eq!(
        canvas_spy::expected_canvas_methods_from_paint_ops(&ops),
        vec!["fillRect", "fillText", "strokeRect"]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn counter_demo_exposes_normalized_jet_tree_for_react_oracle() {
    common::require_env();
    let _guard = live_wasm_e2e_guard().await;

    let app =
        common::JetTestApp::launch_with_init_scripts("counter-demo", &[canvas_spy::init_script()])
            .await
            .expect("launch Jet app");

    let tree = app.element_tree().await.expect("elementTree");
    let normalized = react_oracle::normalize_jet_element_tree(&tree);
    assert_eq!(
        normalized.get("kind").and_then(|v| v.as_str()),
        Some("element")
    );
    assert_eq!(
        normalized.get("tag").and_then(|v| v.as_str()),
        Some("button")
    );

    let mut methods = Vec::new();
    for _ in 0..20 {
        let calls = app
            .page
            .evaluate(canvas_spy::captured_calls_expr())
            .await
            .expect("captured canvas calls");
        methods = canvas_spy::canonical_canvas_methods(&calls);
        if methods.iter().any(|method| method == "fillText") {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        methods.iter().any(|method| method == "fillText"),
        "expected canvas text draw call, got {}",
        canvas_spy::method_summary(&methods)
    );

    app.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn react_dom_oracle_prerequisites_are_explicitly_gated() {
    let workspace = workspace_root();

    react_oracle::require_react_oracle_env(&workspace);

    assert!(react_oracle::react_dom_available(&workspace));
}
// CODEGEN-END
