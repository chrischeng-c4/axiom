// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Import handling for the TSX -> Rust WASM compiler.
//!
//! Runtime imports are a codegen boundary: React hooks are known to
//! the compiler, CSS side-effect imports are owned by the host build,
//! and type-only imports disappear after type checking. Everything
//! else must fail clearly until Jet can lower that module.

use jet::tsx_to_rust::{transpile, transpile_compat_with_source};

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
fn runtime_imports_fail_with_wasm_diagnostic() {
    let err = transpile(
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
    .unwrap_err();
    let msg = format!("{err:#}");

    assert!(
        msg.contains(
            "runtime import from `@mui/material/Box` is not lowered by jet build --wasm yet"
        ),
        "unexpected diagnostic: {msg}"
    );
    assert!(
        msg.contains("2:1"),
        "diagnostic should include source position: {msg}"
    );
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
fn compat_lowering_preserves_mui_visual_fixture_state_and_style() {
    let out = transpile_compat_with_source(
        r#"
import React, { useState } from 'react';
import Button from '@mui/material/Button';
import Checkbox from '@mui/material/Checkbox';
import TextField from '@mui/material/TextField';

export function MuiVisualFixture() {
  const [name, setName] = useState("Ada");
  const [accepted, setAccepted] = useState(true);

  return (
    <main
      id="visual-root"
      style={{
        fontFamily: "Inter, system-ui, sans-serif",
        maxWidth: 440,
        margin: "32px auto",
        padding: 24,
        border: "1px solid #d7dde8",
        borderRadius: 8,
      }}
    >
      <h1 style={{ fontSize: 24, margin: "0 0 16px" }}>MUI visual fixture</h1>
      <TextField
        id="mui-name"
        label="Name"
        value={name}
        onChange={(event) => setName(event.target.value)}
      />
      <label style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 12 }}>
        <Checkbox
          id="mui-accept"
          checked={accepted}
          onChange={(event) => setAccepted(event.target.checked)}
        />
        Accept library terms
      </label>
      <Button id="mui-button">MUI Primary</Button>
      <p id="echo" style={{ marginTop: 18 }}>
        hello {name}; accepted: {String(accepted)}
      </p>
    </main>
  );
}
"#,
        "src/MuiVisualFixture.tsx",
        "MuiVisualFixture",
    )
    .expect("compat lowering should preserve the visible MUI fixture state");

    let rust = out.rust_source;
    assert!(rust.contains("let (name, setName) = use_state::<String>(\"Ada\".to_string());"));
    assert!(rust.contains("let (accepted, setAccepted) = use_state::<bool>(true);"));
    assert!(rust.contains("max-width: 440px"));
    assert!(rust.contains("border-radius: 8px"));
    assert!(rust.contains("font-size: 24px"));
    assert!(rust.contains("value: Some(name.clone()),"));
    assert!(rust.contains("on_change: Some(Callback::new"));
    assert!(rust.contains("input_type: Some(\"checkbox\".to_string()),"));
    assert!(rust.contains("checked: Some(accepted),"));
    assert!(rust.contains("on_checked_change: Some(Callback::new"));
    assert!(rust.contains("Element::text(name.to_string())"));
    assert!(rust.contains("Element::text(accepted.to_string())"));
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
