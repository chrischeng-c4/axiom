// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
// CODEGEN-BEGIN
use super::*;

#[test]
fn test_tsx_simple_component() {
    let source = r#"const App: React.FC = () => <div>Hello</div>;"#;
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();

    assert!(!result.code.contains(": React.FC"));
    assert!(result.code.contains("jsx(\"div\""));
    assert!(result.code.contains("Hello"));
}

#[test]
fn test_tsx_nested_jsx() {
    let source = r#"
const App = () => (
  <React.StrictMode>
    <App />
  </React.StrictMode>
);"#;
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();

    assert!(result.code.contains("React.StrictMode"));
    assert!(result.code.contains("App"));
    assert!(!result.code.contains("<React.StrictMode>"));
}

#[test]
fn test_tsx_function_call_with_jsx() {
    let source = r#"root.render(<App />);"#;
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();

    assert!(result.code.contains("root.render"));
    assert!(result.code.contains("jsx(App"));
    assert!(!result.code.contains("<App />"));
}

#[test]
fn test_tsx_with_type_annotations() {
    let source = r#"
interface Props {
    name: string;
}

const Greeting: React.FC<Props> = ({ name }: Props) => <div>Hello {name}</div>;
"#;
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();

    assert!(!result.code.contains("interface Props"));
    assert!(!result.code.contains(": React.FC"));
    assert!(!result.code.contains(": Props"));
    assert!(result.code.contains("jsxs(\"div\"") || result.code.contains("jsx(\"div\""));
}

#[test]
fn test_tsx_self_closing_with_props() {
    let source = r#"/** @jsx createElement */
const x = <input className="new-todo" data-testid="new-todo" placeholder="What?" />;"#;
    let options = TransformOptions {
        jsx_automatic: false,
        ..TransformOptions::default()
    };
    let result = transform_tsx(source, &options).unwrap();

    assert!(
        result.code.contains("className"),
        "missing className: {}",
        result.code
    );
    assert!(
        result.code.contains("data-testid"),
        "missing data-testid: {}",
        result.code
    );
    assert!(
        result.code.contains("placeholder"),
        "missing placeholder: {}",
        result.code
    );
}

// ── T17–T32: AST-Based TypeScript Type Stripping ──────────────────────

/// T17: Strip export type Statement
#[test]
fn t17_strip_export_type_statement() {
    let source = "export type { Foo } from './foo'\nexport const bar = 1;";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        result.code.contains("export const bar = 1"),
        "must preserve value export: {}",
        result.code
    );
    assert!(
        !result.code.contains("export type"),
        "must strip export type: {}",
        result.code
    );
}

/// T18: Strip import type Statement
#[test]
fn t18_strip_import_type_statement() {
    let source = "import type { Config } from './config'\nconst x = 1;";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        !result.code.contains("import type"),
        "must remove import type: {}",
        result.code
    );
    assert!(
        result.code.contains("const x = 1"),
        "must preserve value code: {}",
        result.code
    );
}

/// T19: Strip Inline Type Import Specifier
#[test]
fn t19_strip_inline_type_import_specifier() {
    let source = "import { type ClassValue, clsx } from 'clsx'";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        result.code.contains("clsx"),
        "must keep value specifier: {}",
        result.code
    );
    assert!(
        !result.code.contains("ClassValue"),
        "must strip type specifier: {}",
        result.code
    );
}

/// T20: Remove Empty Type-Only Import
#[test]
fn t20_remove_empty_type_only_import() {
    let source = "import { type Foo } from './foo'\nconst x = 1;";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        !result.code.contains("import"),
        "must remove entire import when only type specifiers remain: {}",
        result.code
    );
    assert!(
        result.code.contains("const x = 1"),
        "must preserve value code: {}",
        result.code
    );
}

#[test]
fn t20b_strip_arrow_function_type_predicate() {
    let source = "const edges = items.filter((edge): edge is { id: string; sourceIndex: number } => edge !== null);";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        result
            .code
            .contains("items.filter((edge) => edge !== null)"),
        "must preserve arrow callback without type predicate: {}",
        result.code
    );
    assert!(
        !result.code.contains("edge is"),
        "must strip type predicate annotation: {}",
        result.code
    );
}

/// T21: Strip Multi-Line Interface (Short)
#[test]
fn t21_strip_multiline_interface_short() {
    let source = "export interface Props {\n  name: string\n  age: number\n}\nexport const x = 1;";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        result.code.contains("export const x = 1"),
        "must preserve value export: {}",
        result.code
    );
    assert!(
        !result.code.contains("interface"),
        "must strip interface keyword: {}",
        result.code
    );
    // No orphan "export" on its own line
    for line in result.code.lines() {
        assert!(
            line.trim() != "export",
            "must not have orphan 'export' keyword: {}",
            result.code
        );
    }
}

/// T22: Strip Multi-Line Interface (10+ Lines)
#[test]
fn t22_strip_multiline_interface_long() {
    let source = r#"export interface BigProps {
  a: string
  b: number
  c: boolean
  d: string
  e: number
  f: boolean
  g: string
  h: number
  i: boolean
  j: string
  k: number
  l: boolean
}
const y = 2;"#;
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        result.code.contains("const y = 2"),
        "must preserve value code: {}",
        result.code
    );
    assert!(
        !result.code.contains("interface"),
        "must remove entire interface block: {}",
        result.code
    );
    // No orphan braces from the interface
    assert!(
        !result.code.contains("  a: string"),
        "no interface fields must remain: {}",
        result.code
    );
}

/// T23: Strip Standalone Interface (No Export)
#[test]
fn t23_strip_standalone_interface() {
    let source = "interface InternalProps {\n  id: number\n}\nconst y = 2;";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        result.code.contains("const y = 2"),
        "must preserve value code: {}",
        result.code
    );
    assert!(
        !result.code.contains("interface"),
        "must strip standalone interface: {}",
        result.code
    );
}

/// T24: Strip Declare Function
#[test]
fn t24_strip_declare_function() {
    let source = "declare function fetchData(): Promise<void>\nconst x = 1;";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        !result.code.contains("declare"),
        "must strip declare function: {}",
        result.code
    );
    assert!(
        result.code.contains("const x = 1"),
        "must preserve value code: {}",
        result.code
    );
}

/// T25: Strip Declare Module
#[test]
fn t25_strip_declare_module() {
    let source = "declare module '*.css' {\n  const styles: Record<string, string>\n  export default styles\n}\nconst z = 3;";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        !result.code.contains("declare"),
        "must strip declare module block: {}",
        result.code
    );
    assert!(
        result.code.contains("const z = 3"),
        "must preserve value code: {}",
        result.code
    );
}

/// T26: Strip Declare Const
#[test]
fn t26_strip_declare_const() {
    let source = "declare const __DEV__: boolean;\nconst x = 1;";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        !result.code.contains("declare const __DEV__"),
        "must strip declare const: {}",
        result.code
    );
    assert!(
        result.code.contains("const x = 1"),
        "must preserve value code: {}",
        result.code
    );
}

/// T27: Strip Declare Global Block
#[test]
fn t27_strip_declare_global() {
    let source = "declare global {\n  interface Window {\n    __APP__: any\n  }\n}\nconst a = 1;";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        !result.code.contains("declare"),
        "must strip declare global block: {}",
        result.code
    );
    assert!(
        result.code.contains("const a = 1"),
        "must preserve value code: {}",
        result.code
    );
}

/// T28: Strip Satisfies Expression
#[test]
fn t28_strip_satisfies_expression() {
    let source = "const cfg = { port: 3000 } satisfies Config";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        result.code.contains("{ port: 3000 }"),
        "must keep expression: {}",
        result.code
    );
    assert!(
        !result.code.contains("satisfies"),
        "must strip satisfies keyword: {}",
        result.code
    );
    assert!(
        !result.code.contains("Config"),
        "must strip type name: {}",
        result.code
    );
}

/// T28b: Strip Satisfies in multi-line const-declared object with generic type
/// (regression for jet #1535 — Cue `dictionaries satisfies Record<Locale, CueCopy>`).
#[test]
fn t28b_strip_satisfies_multiline_generic() {
    let source = "const dictionaries = {\n  'zh-TW': { hello: 'a' },\n  'en-US': { hello: 'b' },\n} satisfies Record<Locale, CueCopy>\n";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        result.code.contains("const dictionaries"),
        "must keep declaration: {}",
        result.code
    );
    assert!(
        result.code.contains("'zh-TW'") && result.code.contains("'en-US'"),
        "must keep object literal contents: {}",
        result.code
    );
    assert!(
        !result.code.contains("satisfies"),
        "must strip satisfies keyword: {}",
        result.code
    );
    assert!(
        !result.code.contains("Record<"),
        "must strip generic type after satisfies: {}",
        result.code
    );
    assert!(
        !result.code.contains("CueCopy"),
        "must strip type name argument: {}",
        result.code
    );
}

/// T29: Preserve `type` as JS Identifier
#[test]
fn t29_preserve_type_as_identifier() {
    let source = "const type = 'primary';\nif (type === 'primary') { run(); }";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        result.code.contains("const type = 'primary'"),
        "must preserve 'type' as variable name: {}",
        result.code
    );
    assert!(
        result.code.contains("type === 'primary'"),
        "must preserve 'type' usage in expression: {}",
        result.code
    );
}

/// T30: Strip `as const` Expression (TypeScript assertion, not valid JS)
///
/// `as const` is a TypeScript type assertion, same as `as Type`.
/// It is stripped during compilation — the expression values are preserved.
#[test]
fn t30_strip_as_const_expression() {
    let source = "const COLORS = ['red', 'blue'] as const;";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    // as_expression stripping: drops 'as const', keeps the expression values
    assert!(
        result.code.contains("'red'") && result.code.contains("'blue'"),
        "must preserve array contents after stripping 'as const': {}",
        result.code
    );
    assert!(
        !result.code.contains("as const"),
        "must strip 'as const' TypeScript assertion: {}",
        result.code
    );
}

/// T31: Strip Type Alias Declaration
#[test]
fn t31_strip_type_alias() {
    let source = "type UserId = string;\nconst id = '123';";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        !result.code.contains("type UserId"),
        "must strip type alias: {}",
        result.code
    );
    assert!(
        result.code.contains("const id = '123'"),
        "must preserve value code: {}",
        result.code
    );
}

/// T32: Mixed Import — Value and Type Specifiers
#[test]
fn t32_mixed_import_value_and_type() {
    let source = "import { useState, type Dispatch, useEffect, type SetStateAction } from 'react'";
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();
    assert!(
        result.code.contains("useState"),
        "must keep useState: {}",
        result.code
    );
    assert!(
        result.code.contains("useEffect"),
        "must keep useEffect: {}",
        result.code
    );
    assert!(
        !result.code.contains("Dispatch"),
        "must strip type Dispatch: {}",
        result.code
    );
    assert!(
        !result.code.contains("SetStateAction"),
        "must strip type SetStateAction: {}",
        result.code
    );
}

// ── HMR / React Fast Refresh Tests (T1, T2, T10–T15) ───────────────────

/// T1: import.meta.hot Injected in Dev Mode
///
/// Spec: "Given a .tsx file with no import.meta.hot usage, When transform_tsx()
/// processes it in dev mode, Then output contains import.meta.hot runtime setup code."
///
/// The import.meta.hot API is injected at serve time via `generate_hot_preamble()`,
/// not in the transform pass itself. React Fast Refresh injection (which is part of
/// the transform pass) is tested separately in T15. This test verifies both paths.
#[test]
fn t1_import_meta_hot_injected_in_dev_mode() {
    // Part 1: Verify generate_hot_preamble produces import.meta.hot setup
    let preamble = crate::dev_server::hmr_client::generate_hot_preamble("/src/App.tsx");
    assert!(
        preamble.contains("import.meta.hot"),
        "preamble must set up import.meta.hot: {}",
        preamble
    );
    assert!(
        preamble.contains("__jet_hmr_create_hot"),
        "preamble must call __jet_hmr_create_hot factory: {}",
        preamble
    );
    assert!(
        preamble.contains("/src/App.tsx"),
        "preamble must include the module URL: {}",
        preamble
    );

    // Part 2: Verify that transform_tsx in dev mode produces React Fast Refresh
    // instrumentation (which is the other half of HMR setup)
    let source = r#"export function App() { return <div>Hello</div>; }"#;
    let options = TransformOptions {
        dev_mode: true,
        ..TransformOptions::default()
    };
    let result = transform_tsx(source, &options).unwrap();

    assert!(
        result
            .code
            .contains("import RefreshRuntime from '/@react-refresh'"),
        "must inject RefreshRuntime import in dev mode: {}",
        result.code
    );
    assert!(
        result.code.contains("$RefreshReg$"),
        "must inject $RefreshReg$ in dev mode: {}",
        result.code
    );
}

/// T2: import.meta.hot Not Injected in Prod Build
#[test]
fn t2_import_meta_hot_not_injected_in_prod() {
    let source = r#"export function App() { return <div>Hello</div>; }"#;
    let options = TransformOptions {
        dev_mode: false,
        ..TransformOptions::default()
    };
    let result = transform_tsx(source, &options).unwrap();

    assert!(
        !result.code.contains("RefreshRuntime"),
        "must NOT inject RefreshRuntime in prod mode: {}",
        result.code
    );
    assert!(
        !result.code.contains("$RefreshReg$"),
        "must NOT inject $RefreshReg$ in prod mode: {}",
        result.code
    );
    assert!(
        !result.code.contains("$RefreshSig$"),
        "must NOT inject $RefreshSig$ in prod mode: {}",
        result.code
    );
    assert!(
        !result.code.contains("enqueueUpdate"),
        "must NOT inject enqueueUpdate in prod mode: {}",
        result.code
    );
}

/// T10: Component Declaration Gets $RefreshReg$
#[test]
fn t10_component_declaration_gets_refresh_reg() {
    let source = r#"export function App() { return <div/>; }"#;
    let options = TransformOptions {
        dev_mode: true,
        ..TransformOptions::default()
    };
    let result = transform_tsx(source, &options).unwrap();

    assert!(
        result.code.contains("$RefreshReg$(App, \"App\")"),
        "must inject $RefreshReg$ for App component: {}",
        result.code
    );
}

/// T11: Arrow Component Gets $RefreshReg$
#[test]
fn t11_arrow_component_gets_refresh_reg() {
    let source = r#"const App = () => <div/>;"#;
    let options = TransformOptions {
        dev_mode: true,
        ..TransformOptions::default()
    };
    let result = transform_tsx(source, &options).unwrap();

    assert!(
        result.code.contains("$RefreshReg$(App, \"App\")"),
        "must inject $RefreshReg$ for arrow App component: {}",
        result.code
    );
}

/// T12: Hook Usage Gets $RefreshSig$
#[test]
fn t12_hook_usage_gets_refresh_sig() {
    let source = r#"function Counter() {
  const [count, setCount] = useState(0);
  useEffect(() => {}, []);
  return <div>{count}</div>;
}"#;
    let options = TransformOptions {
        dev_mode: true,
        ..TransformOptions::default()
    };
    let result = transform_tsx(source, &options).unwrap();

    assert!(
        result.code.contains("$RefreshSig$()"),
        "must inject $RefreshSig$ call for hooks: {}",
        result.code
    );
    // Signature should include hook call order fingerprint
    assert!(
        result.code.contains("useState{}"),
        "hooks signature must include useState: {}",
        result.code
    );
    assert!(
        result.code.contains("useEffect{}"),
        "hooks signature must include useEffect: {}",
        result.code
    );
}

/// T13: Non-Component Function Skipped
#[test]
fn t13_non_component_function_skipped() {
    let source = r#"function calculateTotal(items: number[]): number { return items.reduce((a, b) => a + b, 0); }
export function App() { return <div>{calculateTotal([1,2,3])}</div>; }"#;
    let options = TransformOptions {
        dev_mode: true,
        ..TransformOptions::default()
    };
    let result = transform_tsx(source, &options).unwrap();

    // calculateTotal is lowercase — not a React component
    assert!(
        !result.code.contains("$RefreshReg$(calculateTotal"),
        "must NOT inject $RefreshReg$ for non-component function: {}",
        result.code
    );
    // App should still get registration
    assert!(
        result.code.contains("$RefreshReg$(App, \"App\")"),
        "must inject $RefreshReg$ for App: {}",
        result.code
    );
}

/// T14: React.memo Wrapped Component Detected
#[test]
fn t14_react_memo_wrapped_component() {
    let source = r#"const MemoApp = React.memo(function App() { return <div/>; });"#;
    let options = TransformOptions {
        dev_mode: true,
        ..TransformOptions::default()
    };
    let result = transform_tsx(source, &options).unwrap();

    assert!(
        result.code.contains("$RefreshReg$(MemoApp, \"MemoApp\")"),
        "must inject $RefreshReg$ for React.memo wrapped component: {}",
        result.code
    );
}

/// T15: Preamble and Footer Injected
#[test]
fn t15_preamble_and_footer_injected() {
    let source = r#"export function App() { return <div>Hello</div>; }"#;
    let options = TransformOptions {
        dev_mode: true,
        ..TransformOptions::default()
    };
    let result = transform_tsx(source, &options).unwrap();

    // Preamble: starts with RefreshRuntime import
    assert!(
        result
            .code
            .starts_with("import RefreshRuntime from '/@react-refresh'"),
        "output must start with RefreshRuntime import: {}",
        result.code
    );

    // Footer: ends with enqueueUpdate
    let trimmed = result.code.trim_end();
    assert!(
        trimmed.ends_with("RefreshRuntime.enqueueUpdate();"),
        "output must end with RefreshRuntime.enqueueUpdate(): {}",
        result.code
    );
}

/// Additional: No Fast Refresh for non-JSX .tsx file
#[test]
fn no_fast_refresh_for_non_jsx_file() {
    let source = r#"export const value: number = 42;
export function helper(x: number): number { return x * 2; }"#;
    let options = TransformOptions {
        dev_mode: true,
        ..TransformOptions::default()
    };
    let result = transform_tsx(source, &options).unwrap();

    assert!(
        !result.code.contains("RefreshRuntime"),
        "must NOT inject Fast Refresh for non-JSX file: {}",
        result.code
    );
}

#[test]
fn strips_exported_function_overload_signature() {
    let source = r#"export function withTheme<C>(Component: C): C;
export function withTheme(Component) { return Component; }"#;
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();

    assert!(
        !result.code.contains("withTheme<C>") && !result.code.contains("Component: C"),
        "must strip overload-only type syntax: {}",
        result.code
    );
    assert_eq!(
        result.code.matches("export function withTheme").count(),
        1,
        "must preserve only the implementation export: {}",
        result.code
    );
}

#[test]
fn strips_exported_type_namespace() {
    let source = r#"import { Interpolation } from '@emotion/serialize'
import { Theme } from './theming'
export namespace ReactJSX {
  export type ElementType = string
  export interface Element {}
}
export const value = 1"#;
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();

    assert!(
        !result.code.contains("namespace")
            && !result.code.contains("Interpolation")
            && !result.code.contains("Theme"),
        "must strip type-only namespace and imports: {}",
        result.code
    );
    assert!(
        result.code.contains("export const value = 1"),
        "must preserve value export: {}",
        result.code
    );
}

#[test]
fn drops_named_imports_that_are_type_only_after_strip() {
    let source = r#"import createCache, { EmotionCache } from '@emotion/cache'
import { Theme, ThemeContext } from './theming'
const cache = createCache()
export const value = ThemeContext"#;
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();

    assert!(
        result
            .code
            .contains("import createCache from '@emotion/cache';"),
        "must preserve default value import: {}",
        result.code
    );
    assert!(
        result
            .code
            .contains("import { ThemeContext } from './theming';"),
        "must preserve used named value import: {}",
        result.code
    );
    assert!(
        !result.code.contains("EmotionCache") && !result.code.contains("Theme,"),
        "must remove unused type-only named imports: {}",
        result.code
    );
}

#[test]
fn preserves_styled_components_value_imports_used_as_template_tags() {
    let source = r##"import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import styled, { createGlobalStyle, css } from "styled-components";

const GlobalStyle = createGlobalStyle`
  body { margin: 0; }
`;
const Matrix = styled.main`
  min-height: 100vh;
`;
const Button = styled.button`
  ${(props) => css`
    background: ${props.$accent || "#2563eb"};
  `}
`;

function App() {
  const [active] = useState(0);
  return <Matrix><GlobalStyle /><Button $accent="#2563eb">{active}</Button></Matrix>;
}

createRoot(document.getElementById("root")!).render(<App />);"##;
    let options = TransformOptions::default();
    let result = transform_tsx(source, &options).unwrap();

    assert!(
        result.code.contains("import { useState } from \"react\"")
            || result
                .code
                .contains("import React, { useState } from \"react\""),
        "must preserve React value import: {}",
        result.code
    );
    assert!(
        result
            .code
            .contains("import { createRoot } from \"react-dom/client\""),
        "must preserve createRoot value import: {}",
        result.code
    );
    assert!(
        result
            .code
            .contains("import styled, { createGlobalStyle, css } from \"styled-components\""),
        "must preserve styled-components default and named value imports: {}",
        result.code
    );
}
// CODEGEN-END
