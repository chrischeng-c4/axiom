// Colocated unit tests for #4206: `mamba index build` writes
// `metadata.toml` beside each staged wheel, carrying the sha256 digest and
// the `Requires-Dist:` edges `mamba add --index` / `mamba lock --index`
// resolve the transitive closure against.
//
// These cover the rules that are observable only inside the implementation
// -- extra-marker filtering, marker stripping, and TOML string escaping --
// as opposed to `apps/mamba/e2e/pkgmgr_lock_frozen_transitive.rs`, which
// judges the externally observable `mamba.lock` shape end to end.

use crate::pkgmanage::index::{clean_requires, escape_toml_string, render_metadata_toml};

#[test]
fn clean_requires_drops_extra_marker_entries() {
    let raw = vec![
        "lib>=1".to_string(),
        "pytest>=7; extra == \"test\"".to_string(),
    ];
    let cleaned = clean_requires(&raw);
    assert_eq!(cleaned, vec!["lib>=1".to_string()]);
}

#[test]
fn clean_requires_strips_non_extra_markers_but_keeps_the_requirement() {
    let raw = vec!["lib>=1; python_version >= \"3.8\"".to_string()];
    let cleaned = clean_requires(&raw);
    assert_eq!(cleaned, vec!["lib>=1".to_string()]);
}

#[test]
fn clean_requires_keeps_plain_requirements_unchanged() {
    let raw = vec!["lib>=1".to_string(), "other==2.0".to_string()];
    let cleaned = clean_requires(&raw);
    assert_eq!(cleaned, vec!["lib>=1".to_string(), "other==2.0".to_string()]);
}

#[test]
fn render_metadata_toml_carries_name_version_filename_sha_and_requires() {
    let body = render_metadata_toml(
        "app",
        "1.0",
        "app-1.0-py3-none-any.whl",
        "deadbeef",
        &["lib>=1".to_string()],
    );
    assert_eq!(
        body,
        "name = \"app\"\n\
         version = \"1.0\"\n\
         filename = \"app-1.0-py3-none-any.whl\"\n\
         sha256 = \"deadbeef\"\n\
         requires = [\"lib>=1\"]\n"
    );
}

#[test]
fn render_metadata_toml_with_no_requires_writes_an_empty_array() {
    let body = render_metadata_toml("lib", "2.0", "lib-2.0-py3-none-any.whl", "cafebabe", &[]);
    assert_eq!(
        body,
        "name = \"lib\"\n\
         version = \"2.0\"\n\
         filename = \"lib-2.0-py3-none-any.whl\"\n\
         sha256 = \"cafebabe\"\n\
         requires = []\n"
    );
}

#[test]
fn escape_toml_string_escapes_backslash_and_double_quote() {
    assert_eq!(
        escape_toml_string(r#"C:\pkgs\a"b.whl"#),
        r#"C:\\pkgs\\a\"b.whl"#
    );
}
