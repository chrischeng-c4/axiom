use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::MambaError;

/// Run cbindgen on a Rust crate to produce a C header (#255).
pub fn run_cbindgen(
    crate_dir: &Path,
    output_header: &Path,
) -> crate::error::Result<PathBuf> {
    // Write cbindgen.toml config
    let config_path = crate_dir.join("cbindgen.toml");
    std::fs::write(&config_path, CBINDGEN_CONFIG)?;

    let output = Command::new("cbindgen")
        .args([
            "--config", config_path.to_str().unwrap_or("cbindgen.toml"),
            "--crate", &crate_name_from_dir(crate_dir),
            "--output", output_header.to_str().unwrap_or("output.h"),
        ])
        .current_dir(crate_dir)
        .output()
        .map_err(|e| MambaError::Other(format!(
            "failed to run cbindgen: {e}. Install with: cargo install cbindgen"
        )))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MambaError::Other(format!("cbindgen failed:\n{stderr}")));
    }

    Ok(output_header.to_path_buf())
}

fn crate_name_from_dir(dir: &Path) -> String {
    dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .replace('-', "_")
}

const CBINDGEN_CONFIG: &str = r#"
language = "C"
include_guard = "_MAMBA_FFI_H"
style = "Both"
tab_width = 4

[export]
prefix = ""

[fn]
args = "Auto"
"#;

/// Orchestrate the full FFI pipeline: cbindgen → parse → map types → generate stubs.
pub fn generate_ffi_bindings(
    crate_dir: &Path,
    output_dir: &Path,
) -> crate::error::Result<PathBuf> {
    std::fs::create_dir_all(output_dir)?;

    let header_path = output_dir.join("bindings.h");
    run_cbindgen(crate_dir, &header_path)?;

    let header_content = std::fs::read_to_string(&header_path)?;
    let parsed = super::c_parser::parse_c_header(&header_content);

    let crate_name = crate_name_from_dir(crate_dir);
    let stub = super::stub_gen::generate_tpi_stub(&parsed, &crate_name);
    let stub_path = output_dir.join(format!("{crate_name}.tpi"));
    std::fs::write(&stub_path, stub)?;

    Ok(stub_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_name_from_dir_simple() {
        let p = Path::new("/home/user/my_crate");
        assert_eq!(crate_name_from_dir(p), "my_crate");
    }

    #[test]
    fn test_crate_name_from_dir_with_hyphens() {
        let p = Path::new("/home/user/my-cool-crate");
        assert_eq!(crate_name_from_dir(p), "my_cool_crate");
    }

    #[test]
    fn test_crate_name_from_dir_root() {
        // Root path has no file_name component
        let p = Path::new("/");
        assert_eq!(crate_name_from_dir(p), "unknown");
    }

    #[test]
    fn test_crate_name_from_dir_nested() {
        let p = Path::new("/projects/cclab/projects/mamba");
        assert_eq!(crate_name_from_dir(p), "mamba");
    }

    #[test]
    fn test_cbindgen_config_contains_language() {
        assert!(CBINDGEN_CONFIG.contains("language = \"C\""));
    }

    #[test]
    fn test_cbindgen_config_has_include_guard() {
        assert!(CBINDGEN_CONFIG.contains("include_guard = \"_MAMBA_FFI_H\""));
    }

    #[test]
    fn test_cbindgen_config_has_export_section() {
        assert!(CBINDGEN_CONFIG.contains("[export]"));
        assert!(CBINDGEN_CONFIG.contains("[fn]"));
    }

    #[test]
    fn test_crate_name_from_dir_single_component() {
        let p = Path::new("mamba");
        assert_eq!(crate_name_from_dir(p), "mamba");
    }

    #[test]
    fn test_crate_name_from_dir_no_hyphens() {
        let p = Path::new("/foo/bar/hello");
        assert_eq!(crate_name_from_dir(p), "hello");
    }

    #[test]
    fn test_crate_name_from_dir_multiple_hyphens() {
        let p = Path::new("a-b-c-d");
        assert_eq!(crate_name_from_dir(p), "a_b_c_d");
    }

    #[test]
    fn test_run_cbindgen_missing_tool() {
        // cbindgen is likely not installed in CI — verify error path
        let tmp = std::env::temp_dir().join("mamba_cbindgen_test");
        std::fs::create_dir_all(&tmp).unwrap();
        let header = tmp.join("out.h");
        let result = run_cbindgen(&tmp, &header);
        // Either cbindgen is missing (Err) or the crate dir is invalid (Err)
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("cbindgen") || err_msg.contains("failed"),
            "unexpected error: {err_msg}"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
