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

// Colocated unit tests for #4210: `mamba run <file>` executes on the
// project's own `.venv` interpreter by default, falling back to the `PATH`
// interpreter outside a synced project, with `--compile` as the explicit
// opt-in back to the Mamba compiler.
//
// These cover the rules observable only inside the implementation --
// interpreter selection per `Mode`, `PATH` resolution order, and the
// compiler/interpreter route decision -- as opposed to
// `apps/mamba/e2e/pkgmgr_run_file_venv.rs`, which judges the externally
// observable end-to-end `run` shape through the real built binary.
mod run_file {
    use std::path::{Path, PathBuf};

    use crate::pkgmanage::run::{
        resolve_path_interpreter, resolve_run_interpreter, routes_to_compiler, venv_python_path,
        Mode,
    };

    #[cfg(unix)]
    fn make_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).unwrap();
    }

    #[cfg(unix)]
    fn write_executable(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, "#!/bin/sh\n").unwrap();
        make_executable(&path);
        path
    }

    #[test]
    fn project_mode_selects_the_venv_interpreter() {
        let tmp = tempfile::tempdir().unwrap();
        let mode = Mode::Project {
            site_packages: tmp.path().join(".venv/lib/site-packages"),
        };
        let interpreter = resolve_run_interpreter(tmp.path(), &mode, None).unwrap();
        assert_eq!(interpreter, venv_python_path(tmp.path()));
        #[cfg(not(windows))]
        assert_eq!(interpreter, tmp.path().join(".venv/bin/python"));
        #[cfg(windows)]
        assert_eq!(interpreter, tmp.path().join(".venv/Scripts/python.exe"));
    }

    #[cfg(unix)]
    #[test]
    fn empty_lock_with_pyvenv_cfg_selects_the_venv_interpreter() {
        let tmp = tempfile::tempdir().unwrap();
        let venv = tmp.path().join(".venv");
        std::fs::create_dir_all(&venv).unwrap();
        std::fs::write(venv.join("pyvenv.cfg"), "").unwrap();

        let interpreter =
            resolve_run_interpreter(tmp.path(), &Mode::EmptyLock, None).unwrap();
        assert_eq!(interpreter, venv_python_path(tmp.path()));
    }

    #[cfg(unix)]
    #[test]
    fn empty_lock_without_pyvenv_cfg_falls_through_to_path() {
        let tmp = tempfile::tempdir().unwrap();
        let bin = tmp.path().join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let python3 = write_executable(&bin, "python3");
        let path_value = std::ffi::OsString::from(bin.to_str().unwrap());

        // No `.venv` at all in this project directory.
        let interpreter =
            resolve_run_interpreter(tmp.path(), &Mode::EmptyLock, Some(&path_value)).unwrap();
        assert_eq!(interpreter, python3);
    }

    #[cfg(unix)]
    #[test]
    fn legacy_mode_ignores_an_existing_venv_and_resolves_from_path() {
        let tmp = tempfile::tempdir().unwrap();
        let venv_bin = tmp.path().join(".venv").join("bin");
        std::fs::create_dir_all(&venv_bin).unwrap();
        write_executable(&venv_bin, "python");
        std::fs::write(tmp.path().join(".venv").join("pyvenv.cfg"), "").unwrap();

        let path_dir = tmp.path().join("path-bin");
        std::fs::create_dir_all(&path_dir).unwrap();
        let path_python3 = write_executable(&path_dir, "python3");
        let path_value = std::ffi::OsString::from(path_dir.to_str().unwrap());

        let interpreter =
            resolve_run_interpreter(tmp.path(), &Mode::Legacy, Some(&path_value)).unwrap();
        assert_eq!(interpreter, path_python3);
    }

    #[cfg(unix)]
    #[test]
    fn path_resolution_prefers_python3_over_python() {
        let tmp = tempfile::tempdir().unwrap();
        write_executable(tmp.path(), "python");
        let python3 = write_executable(tmp.path(), "python3");
        let path_value = std::ffi::OsString::from(tmp.path().to_str().unwrap());

        let resolved = resolve_path_interpreter(Some(&path_value)).unwrap();
        assert_eq!(resolved, python3);
    }

    #[test]
    fn path_resolution_with_no_candidate_names_both_python3_and_python() {
        let tmp = tempfile::tempdir().unwrap();
        let path_value = std::ffi::OsString::from(tmp.path().to_str().unwrap());

        let err = resolve_path_interpreter(Some(&path_value)).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("python3"), "{msg}");
        assert!(msg.contains("python"), "{msg}");
    }

    #[test]
    fn compile_flag_routes_to_the_compiler() {
        assert!(routes_to_compiler(true, "main.py"));
    }

    #[test]
    fn stdin_sentinel_routes_to_the_compiler() {
        assert!(routes_to_compiler(false, "-"));
    }

    #[test]
    fn plain_file_routes_to_the_interpreter() {
        assert!(!routes_to_compiler(false, "main.py"));
    }
}

// #4220: `resolve_via_pypi`'s pin must carry the (url, sha256) of the same
// artifact -- the wheel `pick_artifact_url`-equivalent selection picks --
// rather than the first sha256 listed on the release page. This exercises
// the pure selection seam (`lock::select_artifact`) with no network: a
// release whose first file is an sdist with a different digest than the
// host wheel that follows it.
mod lock_artifact_pairing {
    use crate::pkgmanage::lock::select_artifact;
    use crate::pkgmanage::pkgmgr::tags::TagSelector;
    use crate::pkgmanage::pkgmgr::types::{FileHash, ReleaseFile};

    fn release_file(filename: &str, url: &str, sha256: &str) -> ReleaseFile {
        ReleaseFile {
            filename: filename.to_string(),
            url: url.to_string(),
            hash: FileHash {
                algorithm: "sha256".to_string(),
                digest: sha256.to_string(),
            },
            requires_python: None,
            size: None,
            upload_time: None,
            yanked: false,
            yanked_reason: None,
            dist_info_metadata: serde_json::Value::Null,
            source: None,
        }
    }

    #[test]
    fn selected_pair_names_the_wheel_not_the_sdist_listed_first() {
        let sdist_digest = "a".repeat(64);
        let wheel_digest = "b".repeat(64);
        let files = vec![
            release_file(
                "demo-1.0.tar.gz",
                "https://example.test/demo-1.0.tar.gz",
                &sdist_digest,
            ),
            release_file(
                "demo-1.0-py3-none-any.whl",
                "https://example.test/demo-1.0-py3-none-any.whl",
                &wheel_digest,
            ),
        ];
        let selector = TagSelector::current_host();

        let (url, sha256) = select_artifact(&files, &selector)
            .expect("a compatible wheel is present in the release");

        assert_eq!(url, "https://example.test/demo-1.0-py3-none-any.whl");
        assert_eq!(sha256, wheel_digest);
    }
}

// #4221: `mamba add --index-url` and `mamba lock --index-url` must write the
// same lock body for the same manifest and registry, through the one seam
// `resolve_and_render_via_registry` -- rather than `add` handing the single
// `ResolvedDep` it fetched for the requested package to the single-package
// renderer, which drops every transitive edge the registry declared.
//
// This exercises the seam directly against a two-node registry graph
// (`gizmo` depends on `sprocket`), asserting the rendered `mamba.lock` body:
// both pins present, paired `sha256`/`url` for each, `direct = true` kept on
// the requested package, and its `dependencies` naming the transitive pin.
// The externally observable end-to-end shape (through the real binary,
// `sync`, and `run`) is judged by
// `apps/mamba/e2e/pkgmgr_add_registry_transitive.rs`.
mod add_registry_transitive {
    use crate::pkgmanage::add::ManifestState;
    use crate::pkgmanage::lock::resolve_and_render_via_registry;
    use crate::pkgmanage::pkgmgr::wheel_build::{compose_filename, CoreMetadata, WheelBuilder, WheelMetadata};
    use sha2::{Digest, Sha256};
    use std::collections::BTreeMap;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const ROOT_DIST: &str = "gizmo";
    const LEAF_DIST: &str = "sprocket";
    const VERSION: &str = "1.0";
    const ROOT_WHEEL_FILE: &str = "gizmo-1.0-py3-none-any.whl";
    const LEAF_WHEEL_FILE: &str = "sprocket-1.0-py3-none-any.whl";

    fn sha256_bytes(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }

    /// Build one minimal but real wheel through the product's own wheel
    /// builder, so the digest asserted below is the digest of exactly the
    /// bytes served at that entry's own url.
    fn build_wheel(dist: &str, expected_file: &str) -> Vec<u8> {
        let dir = tempfile::tempdir().expect("fixture: create a temp dir for the wheel");
        let filename = compose_filename(dist, VERSION, "py3", "none", "any");
        let mut wheel_meta = WheelMetadata::new("mamba-unit-add-registry-transitive");
        wheel_meta.tags.push("py3-none-any".into());
        let core_meta = CoreMetadata::new(dist, VERSION);
        let mut builder = WheelBuilder::new(filename, wheel_meta, core_meta);
        builder.add_file(format!("{dist}/__init__.py"), "ORIGIN = 1\n".to_string());
        let wheel = builder
            .build_to_dir(dir.path())
            .unwrap_or_else(|e| panic!("fixture: build wheel {dist}-{VERSION}: {e:?}"));
        assert_eq!(
            wheel.file_name().and_then(|n| n.to_str()),
            Some(expected_file)
        );
        std::fs::read(&wheel).unwrap_or_else(|e| panic!("read {}: {e}", wheel.display()))
    }

    fn simple_page(dist: &str, wheel_file: &str, wheel_url: &str, digest: &str) -> String {
        format!(
            "<!DOCTYPE html>\n\
             <html><head><title>Links for {dist}</title></head>\n\
             <body>\n\
             <a href=\"{wheel_url}#sha256={digest}\">{wheel_file}</a><br/>\n\
             </body></html>\n"
        )
    }

    fn manifest_state(deps: &[&str]) -> ManifestState {
        ManifestState {
            project_name: "unit-fixture".to_string(),
            project_version: "0.1.0".to_string(),
            python_requires: ">=3.12".to_string(),
            dependencies: deps.iter().map(|d| d.to_string()).collect(),
            dev_dependencies: Vec::new(),
            source_overrides: BTreeMap::new(),
        }
    }

    struct Pin {
        name: String,
        version: String,
        sha256: String,
        url: String,
        direct: Option<bool>,
        dependencies: Vec<String>,
    }

    fn parse_lock(body: &str) -> Vec<Pin> {
        let doc: toml::Value = body
            .parse()
            .unwrap_or_else(|e| panic!("parse lock body: {e}\n--- body ---\n{body}"));
        let packages = doc
            .get("package")
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("lock body has no [[package]] array\n--- body ---\n{body}"));
        let string_at = |t: &toml::Value, key: &str| -> String {
            t.get(key)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string()
        };
        packages
            .iter()
            .map(|t| Pin {
                name: string_at(t, "name"),
                version: string_at(t, "version"),
                sha256: string_at(t, "sha256"),
                url: string_at(t, "url"),
                direct: t.get("direct").and_then(|v| v.as_bool()),
                dependencies: t
                    .get("dependencies")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default(),
            })
            .collect()
    }

    fn find<'a>(pins: &'a [Pin], name: &str) -> &'a Pin {
        pins.iter()
            .find(|p| p.name == name)
            .unwrap_or_else(|| panic!("no pin named `{name}` in {:?}", pins.iter().map(|p| &p.name).collect::<Vec<_>>()))
    }

    #[test]
    fn add_and_lock_seam_pins_the_transitive_closure_with_paired_digests() {
        // Build the two wheels and their pages/routes on a multi-thread
        // tokio runtime, kept alive for the whole test; the seam under test
        // builds its own runtime internally and must be called outside any
        // `block_on` of this one.
        let rt = tokio::runtime::Runtime::new().expect("fixture: build a runtime for the registry");

        let root_bytes = build_wheel(ROOT_DIST, ROOT_WHEEL_FILE);
        let leaf_bytes = build_wheel(LEAF_DIST, LEAF_WHEEL_FILE);
        let root_digest = sha256_bytes(&root_bytes);
        let leaf_digest = sha256_bytes(&leaf_bytes);
        assert_ne!(root_digest, leaf_digest);

        let (server, root_url, leaf_url) = rt.block_on(async {
            let server = MockServer::start().await;
            let base = server.uri();
            let root_wheel_route = format!("/files/{ROOT_WHEEL_FILE}");
            let leaf_wheel_route = format!("/files/{LEAF_WHEEL_FILE}");
            let root_url = format!("{base}{root_wheel_route}");
            let leaf_url = format!("{base}{leaf_wheel_route}");

            for route in [
                format!("/pypi/{ROOT_DIST}/json"),
                format!("/pypi/{LEAF_DIST}/json"),
                format!("/pypi/{LEAF_DIST}/{VERSION}/json"),
            ] {
                Mock::given(method("GET"))
                    .and(path(route))
                    .respond_with(ResponseTemplate::new(404))
                    .mount(&server)
                    .await;
            }

            Mock::given(method("GET"))
                .and(path(format!("/pypi/{ROOT_DIST}/{VERSION}/json")))
                .respond_with(ResponseTemplate::new(200).set_body_raw(
                    format!("{{\"info\":{{\"requires_dist\":[\"{LEAF_DIST}\"]}}}}").into_bytes(),
                    "application/json",
                ))
                .mount(&server)
                .await;

            for (route, page) in [
                (
                    format!("/simple/{ROOT_DIST}/"),
                    simple_page(ROOT_DIST, ROOT_WHEEL_FILE, &root_url, &root_digest),
                ),
                (
                    format!("/simple/{LEAF_DIST}/"),
                    simple_page(LEAF_DIST, LEAF_WHEEL_FILE, &leaf_url, &leaf_digest),
                ),
            ] {
                Mock::given(method("GET"))
                    .and(path(route))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .set_body_raw(page.into_bytes(), "text/html; charset=utf-8"),
                    )
                    .mount(&server)
                    .await;
            }

            for (route, bytes) in [
                (root_wheel_route.clone(), root_bytes.clone()),
                (leaf_wheel_route.clone(), leaf_bytes.clone()),
            ] {
                Mock::given(method("GET"))
                    .and(path(route))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .set_body_raw(bytes, "application/octet-stream"),
                    )
                    .mount(&server)
                    .await;
            }

            (server, root_url, leaf_url)
        });

        let index_url = format!("{}/simple", server.uri());
        let state = manifest_state(&[&format!("{ROOT_DIST}=={VERSION}")]);

        // Called synchronously, outside any `block_on` of `rt`: the seam
        // builds its own runtime internally.
        let body = resolve_and_render_via_registry(&state, &index_url)
            .unwrap_or_else(|e| panic!("resolve_and_render_via_registry: {e}\n--- body so far: n/a ---"));

        let pins = parse_lock(&body);

        let root = find(&pins, ROOT_DIST);
        assert_eq!(root.version, VERSION);
        assert_eq!(root.direct, Some(true));
        assert_eq!(root.dependencies, vec![format!("{LEAF_DIST}=={VERSION}")]);
        assert!(!root.sha256.is_empty() && !root.url.is_empty());
        assert_eq!(root.url, root_url);
        assert_eq!(root.sha256, root_digest);

        let leaf = find(&pins, LEAF_DIST);
        assert_eq!(leaf.version, VERSION);
        assert_eq!(leaf.direct, Some(false));
        assert!(leaf.dependencies.is_empty());
        assert!(!leaf.sha256.is_empty() && !leaf.url.is_empty());
        assert_eq!(leaf.url, leaf_url);
        assert_eq!(leaf.sha256, leaf_digest);

        drop(server);
    }
}

// #4222: `pkgmgr-validate`'s `auth` family probe must accept the PEP 503
// canonical name the registry resolver locks (`auth_demo` -> `auth-demo`),
// not the literal underscored request string.
mod lock_pins_package_tests {
    use crate::pkgmanage::validate::lock_pins_package;

    const SINGLE_ENTRY_LOCK: &str = r#"
[[package]]
name = "auth-demo"
version = "1.0.0"
"#;

    const TWO_ENTRY_LOCK: &str = r#"
[[package]]
name = "auth-demo"
version = "1.0.0"

[[package]]
name = "other-pkg"
version = "2.0.0"
"#;

    #[test]
    fn matches_canonical_name_at_requested_version() {
        assert!(lock_pins_package(SINGLE_ENTRY_LOCK, "auth_demo", "1.0.0"));
    }

    #[test]
    fn rejects_wrong_version_on_matching_package() {
        assert!(!lock_pins_package(SINGLE_ENTRY_LOCK, "auth_demo", "2.0.0"));
    }

    #[test]
    fn rejects_missing_package() {
        let lock = r#"
[[package]]
name = "other-pkg"
version = "1.0.0"
"#;
        assert!(!lock_pins_package(lock, "auth_demo", "1.0.0"));
    }

    #[test]
    fn does_not_pair_name_from_one_entry_with_version_from_another() {
        // `auth-demo` is pinned at 1.0.0, `other-pkg` at 2.0.0: a name from
        // one entry must never satisfy a version borrowed from the other.
        assert!(!lock_pins_package(TWO_ENTRY_LOCK, "auth_demo", "2.0.0"));
        assert!(!lock_pins_package(TWO_ENTRY_LOCK, "other_pkg", "1.0.0"));
        // Sanity: each entry still matches its own true pin.
        assert!(lock_pins_package(TWO_ENTRY_LOCK, "auth_demo", "1.0.0"));
        assert!(lock_pins_package(TWO_ENTRY_LOCK, "other_pkg", "2.0.0"));
    }
}
