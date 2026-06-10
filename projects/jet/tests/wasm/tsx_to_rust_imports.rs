// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Import handling for the TSX -> Rust WASM compiler.
//!
//! Runtime imports are a codegen boundary: React hooks are known to
//! the compiler, CSS side-effect imports are owned by the host build,
//! type-only imports disappear after type checking, and supported UI
//! library adapters lower to Jet-native WASM intrinsics.

use jet::tsx_to_rust::{transpile, transpile_compat_with_source};

const MUI_VISUAL_FIXTURE_TSX: &str =
    include_str!("../../../../examples/mui-visual-demo/src/MuiVisualFixture.tsx");
const ANTD_VISUAL_FIXTURE_TSX: &str =
    include_str!("../../../../examples/antd-visual-demo/src/AntdVisualFixture.tsx");

#[test]
fn react_imports_do_not_block_transpile() {
    let out = transpile(
        r#"
import React, { useState } from 'react';

interface CounterProps {
  start: number;
}

export function Counter({ start }: CounterProps) {
  const [n, setN] = useState(start);
  return <button id="inc" onClick={() => setN(n + 1)}>count: {n}</button>;
}
"#,
    )
    .expect("react imports should be compiler-known");

    assert!(out.contains("pub struct CounterProps"));
    assert!(out.contains("use_state::<i64>(props.start)"));
}

#[test]
fn type_only_and_css_imports_do_not_block_transpile() {
    let out = transpile(
        r#"
import type { Project } from './api';
import { type WorkItem } from './api';
import './styles.css';

interface AppProps {
  count: number;
}

export function App({ count }: AppProps) {
  return <div id="root">count: {count}</div>;
}
"#,
    )
    .expect("type-only and CSS side-effect imports should be accepted");

    assert!(out.contains("pub struct AppProps"));
    assert!(out.contains("Element::intrinsic(\"div\""));
}

#[test]
fn mui_runtime_imports_lower_with_wasm_adapter() {
    let out = transpile(
        r#"
import Box from '@mui/material/Box';

interface AppProps {
  count: number;
}

export function App({ count }: AppProps) {
  return <Box id="root">count: {count}</Box>;
}
"#,
    )
    .expect("supported MUI imports should lower through WASM adapters");

    assert!(out.contains("Element::intrinsic(\"div\""));
    assert!(out.contains("Element::text(count.to_string())"));
    assert!(!out.contains("@mui/material"));
}

#[test]
fn local_runtime_imports_fail_before_silent_drop() {
    let err = transpile(
        r#"
import { fetchProjects } from './api';

interface AppProps {
  count: number;
}

export function App({ count }: AppProps) {
  return <div id="root">count: {count}</div>;
}
"#,
    )
    .unwrap_err();
    let msg = format!("{err:#}");

    assert!(
        msg.contains("runtime import from `./api` is not lowered by jet build --wasm yet"),
        "unexpected diagnostic: {msg}"
    );
}

#[test]
fn compat_lowering_maps_mui_imports_to_wasm_intrinsics() {
    let out = transpile_compat_with_source(
        r#"
import React from 'react';
import AddRoundedIcon from '@mui/icons-material/AddRounded';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import CssBaseline from '@mui/material/CssBaseline';
import { ThemeProvider } from '@mui/material/styles';
import { fetchProjects } from './api';

export function App() {
  return (
    <ThemeProvider theme={{}}>
      <CssBaseline />
      <Box id="root">
        <Button startIcon={<AddRoundedIcon />}>Create Project</Button>
      </Box>
    </ThemeProvider>
  );
}
"#,
        "src/App.tsx",
        "App",
    )
    .expect("compat lowering should accept MUI/runtime imports");

    assert!(out.rust_source.contains("Element::Fragment"));
    assert!(out.rust_source.contains("Element::intrinsic(\"div\""));
    assert!(out.rust_source.contains("Element::intrinsic(\"button\""));
    assert!(out
        .rust_source
        .contains("Element::text(\"Create Project\")"));
    assert!(!out.rust_source.contains("@mui"));
    assert!(!out.rust_source.contains("fetchProjects"));
}

#[test]
fn mui_visual_fixture_strict_lowering_preserves_form_controls() {
    let out = transpile(MUI_VISUAL_FIXTURE_TSX)
        .expect("strict lowering should preserve the visible MUI visual fixture");

    let rust = out;
    assert!(rust.contains("Element::intrinsic(\"main\""));
    assert!(rust.contains("Element::intrinsic(\"table\""));
    assert!(rust.contains("Element::intrinsic(\"tr\""));
    assert!(rust.contains("Element::intrinsic(\"td\""));
    assert!(rust.contains("Element::intrinsic(\"input\""));
    assert!(rust.contains("Element::intrinsic(\"button\""));
    assert!(rust.contains("id: Some(\"visual-root\".to_string())"));
    assert!(rust.contains("id: Some(\"large-table\".to_string())"));
    assert!(rust.contains("MUI visual table fixture"));
    assert!(rust.contains("id: Some(\"mui-name\".to_string())"));
    assert!(rust.contains("id: Some(\"mui-accept\".to_string())"));
    assert!(rust.contains("class_name: Some(\"ui-case mui-card\".to_string())"));
    assert!(rust.contains("use_state::<String>(\"Ada\".to_string())"));
    assert!(rust.contains("use_state::<bool>(true)"));
    assert!(rust.contains("on_checked_change: Some"));
    assert!(!rust.contains("@mui/material"));
}

#[test]
fn antd_visual_fixture_strict_lowering_preserves_form_controls() {
    let out = transpile(ANTD_VISUAL_FIXTURE_TSX)
        .expect("strict lowering should preserve the visible AntD visual fixture");

    let rust = out;
    assert!(rust.contains("Element::intrinsic(\"main\""));
    assert!(rust.contains("Element::intrinsic(\"table\""));
    assert!(rust.contains("Element::intrinsic(\"tr\""));
    assert!(rust.contains("Element::intrinsic(\"td\""));
    assert!(rust.contains("Element::intrinsic(\"input\""));
    assert!(rust.contains("Element::intrinsic(\"button\""));
    assert!(rust.contains("id: Some(\"visual-root\".to_string())"));
    assert!(rust.contains("id: Some(\"large-table\".to_string())"));
    assert!(rust.contains("AntD visual table fixture"));
    assert!(rust.contains("id: Some(\"antd-name\".to_string())"));
    assert!(rust.contains("id: Some(\"antd-accept\".to_string())"));
    assert!(rust.contains("class_name: Some(\"ui-case antd-card\".to_string())"));
    assert!(rust.contains("use_state::<String>(\"Ada\".to_string())"));
    assert!(rust.contains("use_state::<bool>(true)"));
    assert!(rust.contains("on_checked_change: Some"));
}

#[test]
fn clipboard_write_text_lowers_to_host_bridge() {
    let out = transpile(
        r#"
import React from 'react';

export function App() {
  const copied = "hello\tworld";
  return <button id="copy" onClick={() => navigator.clipboard.writeText(copied)}>Copy</button>;
}
"#,
    )
    .expect("clipboard writes should lower through the WASM host bridge");

    assert!(out.contains("let copied = \"hello\\tworld\".to_string();"));
    assert!(out.contains("jet_wasm::host::write_clipboard_text(copied.as_ref())"));
}

#[test]
fn compat_lowering_does_not_add_mui_button_style_when_class_name_is_explicit() {
    let out = transpile_compat_with_source(
        r#"
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
        "src/MuiButtonFixture.tsx",
        "MuiButtonFixture",
    )
    .expect("compat lowering should preserve explicit library fixture class shape");

    let rust = out.rust_source;
    assert!(rust.contains("class_name: Some(\"MuiButton-root\".to_string()),"));
    assert!(
        !rust.contains("text-transform: uppercase"),
        "explicit library class fixtures should not get fallback MUI Button inline style.\nGENERATED:\n{rust}"
    );
}

#[test]
fn compat_lowering_does_not_add_mui_textfield_style_when_class_name_is_explicit() {
    let out = transpile_compat_with_source(
        r#"
import React from 'react';
import TextField from '@mui/material/TextField';

export function MuiTextFieldFixture() {
  return (
    <form id="form">
      <TextField id="mui-name" className="MuiInputBase-input" label="Name" />
    </form>
  );
}
"#,
        "src/MuiTextFieldFixture.tsx",
        "MuiTextFieldFixture",
    )
    .expect("compat lowering should preserve explicit TextField class shape");

    let rust = out.rust_source;
    assert!(rust.contains("class_name: Some(\"MuiInputBase-input\".to_string()),"));
    assert!(
        !rust.contains("box-sizing: border-box"),
        "explicit library class fixtures should not get fallback MUI TextField inline style.\nGENERATED:\n{rust}"
    );
}
// CODEGEN-END
