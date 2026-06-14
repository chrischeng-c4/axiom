// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Request-capable effect lowering for WASM.

use jet::tsx_to_rust::transpile;

#[test]
fn use_effect_fetch_lowers_to_wasm_host_bridge() {
    let out = transpile(
        r#"
import { useEffect } from 'react';

interface AppProps {}

export function App({}: AppProps) {
  useEffect(() => {
    fetch('/api/projects');
  }, []);

  return <div id="root">loading</div>;
}
"#,
    )
    .expect("useEffect fetch should lower");

    assert!(out.contains("jet_wasm::react::use_effect_once(|| {"));
    assert!(out.contains("wasm_bindgen_futures::spawn_local(async move {"));
    assert!(out.contains("jet_wasm::host::fetch_json(\"/api/projects\").await"));
    assert!(out.contains("jet_wasm::host::console_error"));
}

#[test]
fn use_effect_non_empty_deps_fail_loudly() {
    let err = transpile(
        r#"
import { useEffect } from 'react';

interface AppProps {
  id: string;
}

export function App({ id }: AppProps) {
  useEffect(() => {
    fetch('/api/projects');
  }, [id]);

  return <div id="root">loading</div>;
}
"#,
    )
    .unwrap_err();
    let msg = format!("{err:#}");

    assert!(
        msg.contains("useEffect lowering supports only an empty dependency array"),
        "unexpected diagnostic: {msg}"
    );
}
// CODEGEN-END
