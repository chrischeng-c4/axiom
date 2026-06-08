// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
use super::*;

/// T34: Detect Builtin Import via require()
#[test]
fn t34_detect_builtin_import_via_require() {
    let mut sources = HashMap::new();
    sources.insert(
        "axios".to_string(),
        "var url = require('url');\nmodule.exports = {};".to_string(),
    );
    let result = detect_builtin_imports(&sources);
    assert!(
        result.contains_key("url"),
        "must detect 'url' builtin: {:?}",
        result
    );
    assert!(
        result["url"].contains("axios"),
        "must track importing package: {:?}",
        result["url"]
    );
}

/// T35: Detect node: Prefixed Builtin Import
#[test]
fn t35_detect_node_prefixed_builtin() {
    let mut sources = HashMap::new();
    sources.insert(
        "some-lib".to_string(),
        "const c = require('node:crypto');".to_string(),
    );
    let result = detect_builtin_imports(&sources);
    assert!(
        result.contains_key("crypto"),
        "must detect 'crypto' (prefix stripped): {:?}",
        result
    );
}

/// T36: Crypto Polyfill Exports Web Crypto API
#[test]
fn t36_crypto_polyfill_exports_web_crypto() {
    let output = generate_polyfill("crypto");
    assert!(!output.is_empty(), "must generate non-empty polyfill");
    assert!(
        output.contains("globalThis.crypto"),
        "must use globalThis.crypto: {}",
        output
    );
    assert!(
        output.contains("randomUUID"),
        "must include randomUUID: {}",
        output
    );
    assert!(
        output.contains("export"),
        "must be valid ESM (has export): {}",
        output
    );
}

/// T37: Buffer Polyfill Exports Uint8Array-Based Implementation
#[test]
fn t37_buffer_polyfill_uint8array() {
    let output = generate_polyfill("buffer");
    assert!(!output.is_empty(), "must generate non-empty polyfill");
    assert!(
        output.contains("class Buffer") || output.contains("Buffer"),
        "must export Buffer class: {}",
        output
    );
    assert!(
        output.contains("Buffer.from") || output.contains("static from"),
        "must have Buffer.from: {}",
        output
    );
    assert!(
        output.contains("Buffer.alloc") || output.contains("static alloc"),
        "must have Buffer.alloc: {}",
        output
    );
}

/// T38: Process Polyfill Contains NODE_ENV
#[test]
fn t38_process_polyfill_node_env() {
    let output = generate_polyfill("process");
    assert!(!output.is_empty(), "must generate non-empty polyfill");
    assert!(
        output.contains("NODE_ENV"),
        "must contain NODE_ENV: {}",
        output
    );
    assert!(
        output.contains("'development'"),
        "must set NODE_ENV to 'development': {}",
        output
    );
    assert!(
        output.contains("browser: true") || output.contains("browser"),
        "must set browser flag: {}",
        output
    );
}

/// T39: Path Polyfill Exports POSIX Path Functions
#[test]
fn t39_path_polyfill_posix_functions() {
    let output = generate_polyfill("path");
    assert!(!output.is_empty(), "must generate non-empty polyfill");
    for func in &["join", "resolve", "dirname", "basename", "extname"] {
        assert!(
            output.contains(&format!("export function {}", func))
                || output.contains(&format!("export var {}", func)),
            "must export {}: {}",
            func,
            output
        );
    }
}

/// T40: Events Polyfill Exports EventEmitter
#[test]
fn t40_events_polyfill_eventemitter() {
    let output = generate_polyfill("events");
    assert!(!output.is_empty(), "must generate non-empty polyfill");
    assert!(
        output.contains("class EventEmitter") || output.contains("EventEmitter"),
        "must export EventEmitter class: {}",
        output
    );
    for method in &["on", "emit", "removeListener"] {
        assert!(
            output.contains(method),
            "must have {} method: {}",
            method,
            output
        );
    }
}

/// T41: Url Polyfill Exports Browser-Native URL
#[test]
fn t41_url_polyfill_browser_native() {
    let output = generate_polyfill("url");
    assert!(!output.is_empty(), "must generate non-empty polyfill");
    assert!(
        output.contains("URL") && output.contains("URLSearchParams"),
        "must re-export URL and URLSearchParams: {}",
        output
    );
}

/// T42: Stub Builtin Exports Empty Object
#[test]
fn t42_stub_builtin_exports_empty_object() {
    let output = generate_stub("fs", "some-lib");
    assert!(
        output.contains("export default"),
        "must have default export: {}",
        output
    );
    assert!(
        output.contains("export {}") || output.contains("export default stub"),
        "must export empty: {}",
        output
    );
    // Verify it's valid ESM by checking for export keyword
    assert!(output.contains("export"), "must be valid ESM");
}

/// T43: Stub Builtin Emits Warning Log
#[test]
fn t43_stub_builtin_emits_warning() {
    let output = generate_stub("fs", "some-lib");
    assert!(
        output.contains("console.warn"),
        "must include console.warn: {}",
        output
    );
    assert!(
        output.contains("[jet] Warning: 'fs' imported by 'some-lib'"),
        "must include specific warning: {}",
        output
    );
    assert!(
        output.contains("stubbed (no browser equivalent)"),
        "must note stubbed: {}",
        output
    );
}

/// T44: Unused Builtin Not Detected
#[test]
fn t44_unused_builtin_not_detected() {
    let mut sources = HashMap::new();
    sources.insert(
        "my-lib".to_string(),
        "import something from 'lodash';".to_string(),
    );
    let result = detect_builtin_imports(&sources);
    assert!(
        !result.contains_key("dgram"),
        "dgram must not be detected: {:?}",
        result
    );
    assert!(
        !result.contains_key("stream"),
        "stream must not be detected when unused: {:?}",
        result
    );
}

/// Test: has_polyfill returns true for polyfill builtins
#[test]
fn test_has_polyfill_true_for_known() {
    assert!(has_polyfill("crypto"));
    assert!(has_polyfill("url"));
    assert!(has_polyfill("buffer"));
    assert!(has_polyfill("path"));
    assert!(has_polyfill("events"));
    assert!(has_polyfill("process"));
}

/// Test: has_polyfill returns false for stub builtins
#[test]
fn test_has_polyfill_false_for_stubs() {
    assert!(!has_polyfill("fs"));
    assert!(!has_polyfill("child_process"));
    assert!(!has_polyfill("net"));
}

/// Test: find_require_imports extracts correctly
#[test]
fn test_find_require_imports() {
    let source = r#"
var url = require('url');
var fs = require("fs");
var x = require('lodash');
"#;
    let imports = find_require_imports(source);
    assert!(imports.contains(&"url".to_string()));
    assert!(imports.contains(&"fs".to_string()));
    assert!(imports.contains(&"lodash".to_string()));
}

/// Test: find_from_imports extracts correctly
#[test]
fn test_find_from_imports() {
    let source = r#"
import x from 'crypto';
export { y } from "node:url";
"#;
    let imports = find_from_imports(source);
    assert!(imports.contains(&"crypto".to_string()));
    assert!(imports.contains(&"node:url".to_string()));
}

/// Test: detect handles both require and from imports together
#[test]
fn test_detect_mixed_import_styles() {
    let mut sources = HashMap::new();
    sources.insert("pkg-a".to_string(), "var p = require('path');".to_string());
    sources.insert(
        "pkg-b".to_string(),
        "import { URL } from 'url';".to_string(),
    );
    let result = detect_builtin_imports(&sources);
    assert!(result.contains_key("path"), "must detect path");
    assert!(result.contains_key("url"), "must detect url");
}

/// Test: generate_polyfill returns empty for unknown builtin
#[test]
fn test_generate_polyfill_unknown() {
    let output = generate_polyfill("unknown_module");
    assert!(output.is_empty(), "unknown builtin should return empty");
}

/// Test: stream polyfill auto-generates events polyfill dependency
///
/// When stream is detected as imported but events is not, write_polyfills()
/// must also generate polyfill-events.mjs because the stream polyfill
/// imports from it.
#[test]
fn test_stream_polyfill_ensures_events() {
    let dir = tempfile::tempdir().unwrap();
    let jet_dir = dir.path();

    let mut detected: HashMap<String, HashSet<String>> = HashMap::new();
    let mut importers = HashSet::new();
    importers.insert("some-lib".to_string());
    detected.insert("stream".to_string(), importers);
    // events is NOT in detected

    let written = write_polyfills(&detected, jet_dir);

    assert!(
        written.contains(&"stream".to_string()),
        "stream polyfill must be written: {:?}",
        written
    );
    assert!(
        written.contains(&"events".to_string()),
        "events polyfill must be auto-generated when stream is present: {:?}",
        written
    );
    assert!(
        jet_dir.join("polyfill-events.mjs").exists(),
        "polyfill-events.mjs must exist on disk"
    );
    assert!(
        jet_dir.join("polyfill-stream.mjs").exists(),
        "polyfill-stream.mjs must exist on disk"
    );
}
// CODEGEN-END
