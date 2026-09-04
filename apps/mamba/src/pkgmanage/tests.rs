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

// Colocated unit tests for #4208: `mamba sync` must uninstall distributions
// the lock no longer pins, and `mamba sync --check` must report them as
// extraneous instead of calling the environment synchronized.
//
// These cover the rule that is observable only inside the implementation --
// enumerating a venv's own `*.dist-info` directories and comparing each
// `(normalized name, version)` against `mamba.lock`, and pruning exactly
// what a distribution's own RECORD (plus the bytecode cache `import`
// derives from it) owns -- as opposed to
// `apps/mamba/e2e/pkgmgr_sync_prune.rs`, which judges the externally
// observable `sync` / `sync --check` / `run` shape end to end.
mod sync_prune {
    use std::path::Path;

    use crate::pkgmanage::pkgmgr::installer::Installer;
    use crate::pkgmanage::sync::{plan_extraneous, plan_install, prune_distribution, LockedPkg};

    fn locked_pkg(name: &str, version: &str) -> LockedPkg {
        LockedPkg {
            name: name.to_string(),
            version: version.to_string(),
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

    fn write_dist_info(site: &Path, dist: &str, version: &str) {
        let dir = site.join(format!("{dist}-{version}.dist-info"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("RECORD"), "").unwrap();
    }

    #[test]
    fn plan_extraneous_names_the_dist_info_the_lock_no_longer_pins() {
        let tmp = tempfile::tempdir().unwrap();
        let site = tmp.path().join("site-packages");
        std::fs::create_dir_all(&site).unwrap();
        write_dist_info(&site, "a", "1.0");
        write_dist_info(&site, "b", "1.0");

        let packages = vec![locked_pkg("a", "1.0")];
        let extraneous = plan_extraneous(&packages, &site);

        assert_eq!(extraneous, vec![("b".to_string(), "1.0".to_string())]);
        let report = extraneous
            .iter()
            .map(|(n, v)| format!("{n}=={v}"))
            .collect::<Vec<_>>()
            .join(", ");
        assert_eq!(report, "b==1.0");
    }

    #[test]
    fn plan_extraneous_and_plan_install_are_both_empty_when_synchronized() {
        let tmp = tempfile::tempdir().unwrap();
        let site = tmp.path().join("site-packages");
        std::fs::create_dir_all(&site).unwrap();
        write_dist_info(&site, "a", "1.0");

        let packages = vec![locked_pkg("a", "1.0")];
        assert!(plan_extraneous(&packages, &site).is_empty());
        assert!(plan_install(&packages, Some(&site)).is_empty());
    }

    #[test]
    fn plan_extraneous_ignores_a_provider_directory_without_dist_info() {
        let tmp = tempfile::tempdir().unwrap();
        let site = tmp.path().join("site-packages");
        std::fs::create_dir_all(site.join("mamba_provider_pkg")).unwrap();
        std::fs::write(site.join("mamba_provider_pkg").join("INSTALLER"), "mamba\n").unwrap();

        let extraneous = plan_extraneous(&[], &site);
        assert!(
            extraneous.is_empty(),
            "a provider directory without dist-info must not be reported: {extraneous:?}"
        );
    }

    #[test]
    fn sync_uninstalls_the_extraneous_distribution_and_leaves_the_kept_one_intact() {
        let tmp = tempfile::tempdir().unwrap();
        let site = tmp.path().join("site-packages");
        std::fs::create_dir_all(&site).unwrap();

        // `a` stays locked and installed.
        std::fs::create_dir_all(site.join("a-1.0.dist-info")).unwrap();
        std::fs::write(
            site.join("a-1.0.dist-info").join("RECORD"),
            "a/__init__.py,,\na-1.0.dist-info/RECORD,,\n",
        )
        .unwrap();
        std::fs::create_dir_all(site.join("a")).unwrap();
        std::fs::write(site.join("a").join("__init__.py"), "answer = 1\n").unwrap();

        // `b` is installed but the lock no longer pins it.
        std::fs::create_dir_all(site.join("b-1.0.dist-info")).unwrap();
        std::fs::write(
            site.join("b-1.0.dist-info").join("RECORD"),
            "b/__init__.py,,\nb-1.0.dist-info/RECORD,,\n",
        )
        .unwrap();
        std::fs::create_dir_all(site.join("b")).unwrap();
        std::fs::write(site.join("b").join("__init__.py"), "answer = 2\n").unwrap();

        let packages = vec![locked_pkg("a", "1.0")];
        let extraneous = plan_extraneous(&packages, &site);
        assert_eq!(extraneous, vec![("b".to_string(), "1.0".to_string())]);

        let installer = Installer::new();
        for (name, _version) in &extraneous {
            installer.uninstall(name, &site).expect("uninstall b");
        }

        assert!(!site.join("b-1.0.dist-info").exists());
        assert!(!site.join("b").exists());
        assert!(site.join("a-1.0.dist-info").is_dir());
        assert!(site.join("a").join("__init__.py").is_file());
    }

    #[test]
    fn prune_removes_the_bytecode_cache_and_the_emptied_package_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let site = tmp.path().join("site-packages");
        std::fs::create_dir_all(site.join("b")).unwrap();
        std::fs::write(site.join("b").join("__init__.py"), "answer = 2\n").unwrap();
        std::fs::create_dir_all(site.join("b").join("__pycache__")).unwrap();
        std::fs::write(
            site.join("b")
                .join("__pycache__")
                .join("__init__.cpython-312.pyc"),
            b"\0asm\0\0\0\0",
        )
        .unwrap();

        let dist_info = site.join("b-1.0.dist-info");
        std::fs::create_dir_all(&dist_info).unwrap();
        std::fs::write(dist_info.join("METADATA"), "Name: b\nVersion: 1.0\n").unwrap();
        std::fs::write(
            dist_info.join("RECORD"),
            "b/__init__.py,,\n\
             b-1.0.dist-info/METADATA,,\n\
             b-1.0.dist-info/RECORD,,\n",
        )
        .unwrap();

        let installer = Installer::new();
        prune_distribution(&installer, &site, "b").expect("prune b");

        assert!(!site.join("b").exists(), "b/ must be gone after pruning");
        assert!(
            !site.join("b-1.0.dist-info").exists(),
            "b-1.0.dist-info must be gone after pruning"
        );
    }

    #[test]
    fn prune_keeps_a_namespace_portion_owned_by_a_surviving_distribution() {
        let tmp = tempfile::tempdir().unwrap();
        let site = tmp.path().join("site-packages");
        std::fs::create_dir_all(site.join("ns")).unwrap();
        std::fs::write(site.join("ns").join("one.py"), "one = 1\n").unwrap();
        std::fs::write(site.join("ns").join("two.py"), "two = 2\n").unwrap();
        std::fs::create_dir_all(site.join("ns").join("__pycache__")).unwrap();
        std::fs::write(
            site.join("ns")
                .join("__pycache__")
                .join("one.cpython-312.pyc"),
            b"\0asm\0\0\0\0",
        )
        .unwrap();
        std::fs::write(
            site.join("ns")
                .join("__pycache__")
                .join("two.cpython-312.pyc"),
            b"\0asm\0\0\0\0",
        )
        .unwrap();

        let one_dist_info = site.join("one-1.0.dist-info");
        std::fs::create_dir_all(&one_dist_info).unwrap();
        std::fs::write(one_dist_info.join("METADATA"), "Name: one\nVersion: 1.0\n").unwrap();
        std::fs::write(
            one_dist_info.join("RECORD"),
            "ns/one.py,,\n\
             one-1.0.dist-info/METADATA,,\n\
             one-1.0.dist-info/RECORD,,\n",
        )
        .unwrap();

        let two_dist_info = site.join("two-1.0.dist-info");
        std::fs::create_dir_all(&two_dist_info).unwrap();
        std::fs::write(two_dist_info.join("METADATA"), "Name: two\nVersion: 1.0\n").unwrap();
        std::fs::write(
            two_dist_info.join("RECORD"),
            "ns/two.py,,\n\
             two-1.0.dist-info/METADATA,,\n\
             two-1.0.dist-info/RECORD,,\n",
        )
        .unwrap();

        let installer = Installer::new();
        prune_distribution(&installer, &site, "one").expect("prune one");

        assert!(
            !site.join("ns").join("one.py").exists(),
            "ns/one.py must be gone after pruning `one`"
        );
        assert!(
            !site
                .join("ns")
                .join("__pycache__")
                .join("one.cpython-312.pyc")
                .exists(),
            "ns/__pycache__/one.cpython-312.pyc must be gone after pruning `one`"
        );
        assert!(
            !site.join("one-1.0.dist-info").exists(),
            "one-1.0.dist-info must be gone after pruning `one`"
        );

        assert!(
            site.join("ns").join("two.py").is_file(),
            "ns/two.py belongs to the surviving distribution `two` and must survive"
        );
        assert!(
            site.join("ns")
                .join("__pycache__")
                .join("two.cpython-312.pyc")
                .is_file(),
            "ns/__pycache__/two.cpython-312.pyc belongs to `two` and must survive"
        );
        assert!(
            site.join("ns").is_dir(),
            "ns/ is still owned by the surviving distribution `two` and must survive"
        );
    }
}

// Colocated unit tests for #4209: `--index-url` must accept the PEP 503
// `.../simple` URL a `uv` user already has in their fingers, not only the
// registry base URL `mamba` builds its own paths from.
//
// These cover the normalisation rule itself -- trim trailing slashes, strip
// at most one trailing `/simple`, never touch anything else -- as opposed to
// `apps/mamba/e2e/pkgmgr_index_url_simple.rs`, which judges the externally
// observable `mamba add`/`mamba.lock` behavior end to end.
mod index_url_simple {
    use crate::pkgmanage::pkgmgr::http::{index_client_for_url, normalize_index_url};

    #[test]
    fn normalize_index_url_strips_one_trailing_simple() {
        assert_eq!(normalize_index_url("http://h:1/simple"), "http://h:1");
    }

    #[test]
    fn normalize_index_url_strips_trailing_slash_then_simple() {
        assert_eq!(normalize_index_url("http://h:1/simple/"), "http://h:1");
        assert_eq!(normalize_index_url("http://h:1/simple//"), "http://h:1");
    }

    #[test]
    fn normalize_index_url_keeps_a_bare_base_and_drops_only_its_trailing_slash() {
        assert_eq!(normalize_index_url("http://h:1/"), "http://h:1");
        assert_eq!(normalize_index_url("http://h:1"), "http://h:1");
    }

    #[test]
    fn normalize_index_url_strips_at_most_one_simple() {
        assert_eq!(
            normalize_index_url("http://h:1/simple/simple"),
            "http://h:1/simple"
        );
    }

    #[test]
    fn normalize_index_url_is_case_sensitive_and_never_rewrites_the_host_or_path() {
        assert_eq!(normalize_index_url("http://h:1/Simple"), "http://h:1/Simple");
        assert_eq!(
            normalize_index_url("https://pypi.org/simple"),
            "https://pypi.org"
        );
        assert_eq!(
            normalize_index_url("http://h:1/mirror/simple"),
            "http://h:1/mirror"
        );
    }

    /// `add.rs::resolve_with_pypi` and `lock.rs::resolve_via_pypi` both build
    /// their `IndexClient` through `index_client_for_url` rather than a
    /// struct literal, so both entry points -- and `$MAMBA_INDEX_URL`, which
    /// flows through the same `resolve_index_url` into these sites -- get the
    /// same normalisation for free.
    #[test]
    fn both_pypi_entry_points_construct_the_client_through_the_normaliser() {
        let client = index_client_for_url(
            "http://h:1/simple/",
            String::new(),
            8,
            30,
            3,
            None,
        );
        assert_eq!(client.index_url, "http://h:1");
    }
}
