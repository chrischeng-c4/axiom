// HANDWRITE-BEGIN gap="missing-generator:unit-test:f612e56c" tracker="#2159" reason="Execute tape spec gen for TypeScript, Python, and Rust and inspect emitted route scope. generator gap: missing-generator:test:generated-client-journey (#2159)."
// @spec apps/tape/tech-design/logic/eliminate-production-ec-false-green-paths.md#unit-test

use std::fs;
use std::path::Path;
use std::process::Command;

fn tape_bin() -> &'static str {
    env!("CARGO_BIN_EXE_tape")
}

fn emitted_source(root: &Path) -> String {
    let mut paths = fs::read_dir(root)
        .expect("generated client output directory")
        .map(|entry| entry.expect("generated client entry").path())
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    paths.sort();
    assert!(paths.len() >= 3, "expected a multi-file client in {root:?}");

    paths
        .into_iter()
        .map(|path| {
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn spec_gen_emits_three_language_clients_with_tape_route_scope() {
    let temp = tempfile::tempdir().expect("client generation tempdir");

    for language in ["ts", "py", "rust"] {
        let out = temp.path().join(language);
        let output = Command::new(tape_bin())
            .args(["spec", "gen", "--lang", language, "--out"])
            .arg(&out)
            .output()
            .unwrap_or_else(|error| panic!("run tape spec gen for {language}: {error}"));
        assert!(
            output.status.success(),
            "tape spec gen {language} failed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let source = emitted_source(&out);
        for route_fragment in [
            "/topics/",
            "append",
            "replay",
            "checkpoint",
            "/admin/backup",
        ] {
            assert!(
                source.contains(route_fragment),
                "{language} client must include Tape route fragment {route_fragment}"
            );
        }
    }
}
// HANDWRITE-END
