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

// Colocated unit tests for #4207: `mamba sync` installs the locked wheels
// into the venv's own standard site-packages (the `purelib` its own
// interpreter reports), and `run`/`pip` read that same layout, replacing
// the offline stub install and the `PYTHONPATH` injection it required.
//
// These cover rules observable only inside the implementation -- as
// opposed to `apps/mamba/e2e/pkgmgr_sync_real_install.rs`, which judges the
// externally observable end-to-end `sync` / `run` / `sync --check` shape.

mod sync_pip_run_real_install {
    use crate::pkgmanage::pkgmgr::installer::{InstallMode, InstallRequest, Installer};
    use crate::pkgmanage::pkgmgr::venv::{
        create_venv, first_python_on_path, VenvCreationOutcome, VenvLayout, VenvOptions,
    };
    use crate::pkgmanage::pkgmgr::wheel_build::{
        compose_filename, CoreMetadata, WheelBuilder, WheelMetadata,
    };
    use crate::pkgmanage::pkgmgr::toolchain::PythonVersion;
    use crate::pkgmanage::pip::resolve_default_site_packages;
    use crate::pkgmanage::run::{configure_command_environment, Mode};
    use crate::pkgmanage::sync::{resolve_artifact_path, resolve_site_packages, LockedPkg};

    fn empty_locked_pkg(name: &str) -> LockedPkg {
        LockedPkg {
            name: name.to_string(),
            version: "1.0".to_string(),
            url: String::new(),
            sha256: String::new(),
            source_kind: "index".to_string(),
            path: String::new(),
            provider: String::new(),
            provides: Vec::new(),
            compatibility: String::new(),
            maturity: String::new(),
        }
    }

    /// Build a real, importable wheel directly in `dir` -- a bare wheel
    /// file the way a direct-file/path lock entry names one on disk. Built
    /// through the product's own `WheelBuilder` (see
    /// `apps/mamba/tests/pkgmgr/fixtures.rs::build_wheel_file`), not
    /// imported from it -- this crate's unit tests can't reach `tests/`.
    fn build_wheel_file(dir: &std::path::Path, name: &str, version: &str) -> std::path::PathBuf {
        std::fs::create_dir_all(dir).unwrap();
        let filename = compose_filename(name, version, "py3", "none", "any");
        let mut wheel_meta = WheelMetadata::new("mamba-pkgmgr-test");
        wheel_meta.tags.push("py3-none-any".into());
        let core_meta = CoreMetadata::new(name, version);
        let mut builder = WheelBuilder::new(filename, wheel_meta, core_meta);
        builder.add_file(format!("{name}/__init__.py"), "answer = 42\n".to_string());
        builder.build_to_dir(dir).unwrap()
    }

    #[test]
    fn resolve_site_packages_uses_the_venv_layout_not_a_flat_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let venv_dir = tmp.path().join(".venv");
        std::fs::create_dir_all(&venv_dir).unwrap();
        std::fs::write(venv_dir.join("pyvenv.cfg"), "version = 3.11.9\n").unwrap();

        let expected =
            VenvLayout::for_current_platform(&venv_dir, &PythonVersion::new(3, 11, 9))
                .site_packages;
        let got = resolve_site_packages(&venv_dir);

        assert_ne!(
            got,
            venv_dir.join("site-packages"),
            "must not resolve the flat .venv/site-packages layout"
        );
        assert_eq!(got, expected);
    }

    #[test]
    fn resolve_artifact_path_fails_naming_the_package_when_neither_path_nor_url_is_set() {
        let pkg = empty_locked_pkg("nopath-nourl-pkg");

        let err = resolve_artifact_path(&pkg).expect_err("must fail: neither path nor url");
        let msg = err.to_string();
        assert!(
            msg.contains("nopath-nourl-pkg"),
            "error must name the package, got: {msg}"
        );
    }

    #[test]
    fn sync_installs_a_direct_file_wheel_leaving_a_record_and_no_stub_markers() {
        let Some(python) = first_python_on_path() else {
            eprintln!("skip: no python3/python on PATH");
            return;
        };
        let tmp = tempfile::tempdir().unwrap();
        let venv_dir = tmp.path().join(".venv");
        let opts = VenvOptions::new(python, &venv_dir);
        let layout = match create_venv(&opts) {
            Ok(VenvCreationOutcome::Created { layout, .. }) => layout,
            Ok(VenvCreationOutcome::Refused { reason }) => {
                panic!("venv creation refused: {reason}")
            }
            Err(e) => panic!("venv creation failed: {e}"),
        };

        let wheel_dir = tmp.path().join("wheels");
        let wheel_path = build_wheel_file(&wheel_dir, "libanswer", "1.0");

        let mut pkg = empty_locked_pkg("libanswer");
        pkg.source_kind = "direct_file".to_string();
        pkg.path = wheel_path.to_string_lossy().into_owned();

        let artifact = resolve_artifact_path(&pkg).expect("resolve artifact path");
        let installer = Installer::new();
        installer
            .install(InstallRequest {
                artifact_path: artifact,
                site_packages: layout.site_packages.clone(),
                python_executable: layout.python_executable.clone(),
                mode: InstallMode::Purelib,
            })
            .expect("install");

        let dist_info_entries: Vec<_> = std::fs::read_dir(&layout.site_packages)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".dist-info"))
            .collect();
        assert_eq!(
            dist_info_entries.len(),
            1,
            "expected exactly one *.dist-info directory under {}",
            layout.site_packages.display()
        );
        let record = dist_info_entries[0].path().join("RECORD");
        assert!(record.is_file(), "{} must exist", record.display());

        let module_dir = layout.site_packages.join("libanswer");
        assert!(
            !module_dir.join("INSTALLER").exists(),
            "no stub INSTALLER marker must be written"
        );
        assert!(
            !module_dir.join("VERSION").exists(),
            "no stub VERSION marker must be written"
        );
    }

    #[test]
    fn run_command_environment_sets_no_pythonpath_and_puts_venv_bin_first_on_path() {
        let tmp = tempfile::tempdir().unwrap();
        let project_dir = tmp.path();
        let venv = project_dir.join(".venv");
        std::fs::create_dir_all(venv.join("bin")).unwrap();
        std::fs::write(venv.join("pyvenv.cfg"), "version = 3.11.9\n").unwrap();

        let mode = Mode::Project {
            site_packages: venv.join("lib").join("python3.11").join("site-packages"),
        };
        let mut command = std::process::Command::new("true");
        configure_command_environment(&mut command, project_dir, &mode);

        let mut saw_pythonpath = false;
        let mut path_value: Option<std::ffi::OsString> = None;
        for (k, v) in command.get_envs() {
            if k == "PYTHONPATH" {
                saw_pythonpath = true;
            }
            if k == "PATH" {
                path_value = v.map(|v| v.to_os_string());
            }
        }
        assert!(!saw_pythonpath, "PYTHONPATH must never be set by run");

        let path_value = path_value.expect("PATH must be set when a venv exists");
        let bin_dir = venv.join("bin");
        let first_entry = std::env::split_paths(&path_value).next();
        assert_eq!(
            first_entry.as_deref(),
            Some(bin_dir.as_path()),
            "the venv's own bin dir must be first on PATH"
        );
    }

    #[test]
    fn pip_default_site_packages_uses_the_venv_layout_not_a_flat_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let cwd = tmp.path();
        let venv_dir = cwd.join(".venv");
        std::fs::create_dir_all(&venv_dir).unwrap();
        std::fs::write(venv_dir.join("pyvenv.cfg"), "version = 3.11.9\n").unwrap();

        let expected =
            VenvLayout::for_current_platform(&venv_dir, &PythonVersion::new(3, 11, 9))
                .site_packages;
        let got = resolve_default_site_packages(cwd);

        assert_ne!(
            got,
            venv_dir.join("site-packages"),
            "must not default to the flat .venv/site-packages layout"
        );
        assert_eq!(got, expected);
    }
}
