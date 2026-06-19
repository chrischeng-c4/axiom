---
id: projects-cap-tests-behavior-cap-command-replacement-parity-rs
summary: Lossless rust-source-unit coverage for `projects/cap/tests/behavior_cap_command_replacement_parity.rs`.
capability_refs:
  - id: agent-hook-installation
    role: primary
    claim: hook-payload-rewrite-adapters
    coverage: full
    rationale: "The parity test verifies cap hook command-string rewrite adapters and same-name replacement behavior against original system commands."
fill_sections: [overview, source, changes]
---

# Standardized projects/cap/tests/behavior_cap_command_replacement_parity.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/cap/tests/behavior_cap_command_replacement_parity.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use anyhow::{bail, Context, Result};

#[test]
fn active_replacements_match_success_and_error_behavior() -> Result<()> {
    let temp = tempfile::tempdir().context("create parity tempdir")?;
    let bin_dir = temp.path().join("bin");
    fs::create_dir(&bin_dir)?;
    let cap = build_cap_frontend(&bin_dir)?;
    let fixture = Fixture::create(temp.path())?;

    let success_cases = [
        Case::new(
            "ls",
            vec!["ls", "-1", fixture.list_dir()],
            "/bin/ls",
            vec!["-1", fixture.list_dir()],
        ),
        Case::new(
            "cat",
            vec!["cat", fixture.cat_file()],
            "/bin/cat",
            vec![fixture.cat_file()],
        ),
        Case::new(
            "uniq",
            vec!["uniq", fixture.uniq_file()],
            "/usr/bin/uniq",
            vec![fixture.uniq_file()],
        ),
        Case::new(
            "find",
            vec!["find", fixture.find_root(), "-type", "f", "-name", "*.txt"],
            "/usr/bin/find",
            vec![fixture.find_root(), "-type", "f", "-name", "*.txt"],
        ),
        Case::new(
            "du",
            vec!["du", "-sk", fixture.du_root()],
            "/usr/bin/du",
            vec!["-sk", fixture.du_root()],
        ),
        Case::new(
            "sort",
            vec!["sort", fixture.sort_file()],
            "/usr/bin/sort",
            vec![fixture.sort_file()],
        ),
        Case::new(
            "sed",
            vec!["sed", "-n", "2,4p", fixture.sed_file()],
            "/usr/bin/sed",
            vec!["-n", "2,4p", fixture.sed_file()],
        ),
        Case::new(
            "grep",
            vec!["grep", "-R", "NEEDLE", fixture.grep_root()],
            "/usr/bin/grep",
            vec!["-R", "NEEDLE", fixture.grep_root()],
        ),
    ];

    for case in success_cases {
        assert_success_parity(&cap, &case)?;
    }

    let run_success_cases = [
        (
            "run ls",
            format!("ls -1 {}", fixture.list_dir()),
            "/bin/ls",
            vec!["-1", fixture.list_dir()],
        ),
        (
            "run cat",
            format!("cat {}", fixture.cat_file()),
            "/bin/cat",
            vec![fixture.cat_file()],
        ),
        (
            "run uniq",
            format!("uniq {}", fixture.uniq_file()),
            "/usr/bin/uniq",
            vec![fixture.uniq_file()],
        ),
        (
            "run find",
            format!("find {} -type f -name '*.txt'", fixture.find_root()),
            "/usr/bin/find",
            vec![fixture.find_root(), "-type", "f", "-name", "*.txt"],
        ),
        (
            "run du",
            format!("du -sk {}", fixture.du_root()),
            "/usr/bin/du",
            vec!["-sk", fixture.du_root()],
        ),
        (
            "run sort",
            format!("sort {}", fixture.sort_file()),
            "/usr/bin/sort",
            vec![fixture.sort_file()],
        ),
        (
            "run sed",
            format!("sed -n 2,4p {}", fixture.sed_file()),
            "/usr/bin/sed",
            vec!["-n", "2,4p", fixture.sed_file()],
        ),
        (
            "run grep",
            format!("grep -R NEEDLE {}", fixture.grep_root()),
            "/usr/bin/grep",
            vec!["-R", "NEEDLE", fixture.grep_root()],
        ),
    ];

    for (name, command, original_program, original_args) in run_success_cases {
        assert_run_string_success_parity(&cap, name, &command, original_program, &original_args)?;
    }

    let missing = temp.path().join("missing-target").display().to_string();
    let error_cases = [
        Case::new(
            "ls",
            vec!["ls", missing.as_str()],
            "/bin/ls",
            vec![missing.as_str()],
        ),
        Case::new(
            "cat",
            vec!["cat", missing.as_str()],
            "/bin/cat",
            vec![missing.as_str()],
        ),
        Case::new(
            "uniq",
            vec!["uniq", missing.as_str()],
            "/usr/bin/uniq",
            vec![missing.as_str()],
        ),
        Case::new(
            "find",
            vec!["find", missing.as_str(), "-type", "f", "-name", "*.txt"],
            "/usr/bin/find",
            vec![missing.as_str(), "-type", "f", "-name", "*.txt"],
        ),
        Case::new(
            "du",
            vec!["du", "-sk", missing.as_str()],
            "/usr/bin/du",
            vec!["-sk", missing.as_str()],
        ),
        Case::new(
            "sort",
            vec!["sort", missing.as_str()],
            "/usr/bin/sort",
            vec![missing.as_str()],
        ),
        Case::new(
            "sed",
            vec!["sed", "-n", "1,2p", missing.as_str()],
            "/usr/bin/sed",
            vec!["-n", "1,2p", missing.as_str()],
        ),
        Case::new(
            "grep",
            vec!["grep", "-R", "NEEDLE", missing.as_str()],
            "/usr/bin/grep",
            vec!["-R", "NEEDLE", missing.as_str()],
        ),
    ];

    for case in error_cases {
        assert_error_parity(&cap, &case, &missing)?;
    }

    let run_error_cases = [
        (
            "run ls",
            format!("ls {}", missing),
            "/bin/ls",
            vec![missing.as_str()],
        ),
        (
            "run cat",
            format!("cat {}", missing),
            "/bin/cat",
            vec![missing.as_str()],
        ),
        (
            "run uniq",
            format!("uniq {}", missing),
            "/usr/bin/uniq",
            vec![missing.as_str()],
        ),
        (
            "run find",
            format!("find {} -type f -name '*.txt'", missing),
            "/usr/bin/find",
            vec![missing.as_str(), "-type", "f", "-name", "*.txt"],
        ),
        (
            "run du",
            format!("du -sk {}", missing),
            "/usr/bin/du",
            vec!["-sk", missing.as_str()],
        ),
        (
            "run sort",
            format!("sort {}", missing),
            "/usr/bin/sort",
            vec![missing.as_str()],
        ),
        (
            "run sed",
            format!("sed -n 1,2p {}", missing),
            "/usr/bin/sed",
            vec!["-n", "1,2p", missing.as_str()],
        ),
        (
            "run grep",
            format!("grep -R NEEDLE {}", missing),
            "/usr/bin/grep",
            vec!["-R", "NEEDLE", missing.as_str()],
        ),
    ];

    for (name, command, original_program, original_args) in run_error_cases {
        assert_run_string_error_parity(
            &cap,
            name,
            &command,
            original_program,
            &original_args,
            &missing,
        )?;
    }

    let no_match = Case::new(
        "grep",
        vec!["grep", "-R", "ABSENT", fixture.grep_root()],
        "/usr/bin/grep",
        vec!["-R", "ABSENT", fixture.grep_root()],
    );
    assert_quiet_nonzero_parity(&cap, &no_match)?;

    Ok(())
}

struct Case<'a> {
    name: &'a str,
    cap_args: Vec<&'a str>,
    original_program: &'a str,
    original_args: Vec<&'a str>,
}

impl<'a> Case<'a> {
    fn new(
        name: &'a str,
        cap_args: Vec<&'a str>,
        original_program: &'a str,
        original_args: Vec<&'a str>,
    ) -> Self {
        Self {
            name,
            cap_args,
            original_program,
            original_args,
        }
    }
}

fn assert_success_parity(cap: &Path, case: &Case<'_>) -> Result<()> {
    let cap_out = run(cap, &case.cap_args)?;
    let original_out = run(Path::new(case.original_program), &case.original_args)?;
    assert_eq!(
        exit_code(&cap_out),
        exit_code(&original_out),
        "{} exit",
        case.name
    );
    assert_eq!(cap_out.stdout, original_out.stdout, "{} stdout", case.name);
    assert_eq!(cap_out.stderr, original_out.stderr, "{} stderr", case.name);
    Ok(())
}

fn assert_error_parity(cap: &Path, case: &Case<'_>, missing: &str) -> Result<()> {
    let cap_out = run(cap, &case.cap_args)?;
    let original_out = run(Path::new(case.original_program), &case.original_args)?;
    assert_ne!(
        exit_code(&original_out),
        Some(0),
        "{} original must fail",
        case.name
    );
    assert_eq!(
        exit_code(&cap_out),
        exit_code(&original_out),
        "{} exit",
        case.name
    );
    assert_eq!(cap_out.stdout, original_out.stdout, "{} stdout", case.name);
    assert!(
        !cap_out.stderr.is_empty(),
        "{} cap stderr should explain the failure",
        case.name
    );
    let cap_stderr = String::from_utf8_lossy(&cap_out.stderr);
    assert!(
        cap_stderr.contains(case.name),
        "{} stderr should name the command: {cap_stderr}",
        case.name
    );
    if case.name != "sort" {
        assert!(
            cap_stderr.contains(missing),
            "{} stderr should name the failed path: {cap_stderr}",
            case.name
        );
    }
    Ok(())
}

fn assert_quiet_nonzero_parity(cap: &Path, case: &Case<'_>) -> Result<()> {
    let cap_out = run(cap, &case.cap_args)?;
    let original_out = run(Path::new(case.original_program), &case.original_args)?;
    assert_eq!(
        exit_code(&cap_out),
        exit_code(&original_out),
        "{} exit",
        case.name
    );
    assert_eq!(cap_out.stdout, original_out.stdout, "{} stdout", case.name);
    assert_eq!(cap_out.stderr, original_out.stderr, "{} stderr", case.name);
    Ok(())
}

fn assert_run_string_success_parity(
    cap: &Path,
    name: &str,
    command: &str,
    original_program: &str,
    original_args: &[&str],
) -> Result<()> {
    let cap_out = run(cap, &["run", command])?;
    let original_out = run(Path::new(original_program), original_args)?;
    assert_eq!(exit_code(&cap_out), exit_code(&original_out), "{name} exit");
    assert_eq!(cap_out.stdout, original_out.stdout, "{name} stdout");
    assert_eq!(cap_out.stderr, original_out.stderr, "{name} stderr");
    Ok(())
}

fn assert_run_string_error_parity(
    cap: &Path,
    name: &str,
    command: &str,
    original_program: &str,
    original_args: &[&str],
    missing: &str,
) -> Result<()> {
    let cap_out = run(cap, &["run", command])?;
    let original_out = run(Path::new(original_program), original_args)?;
    assert_ne!(
        exit_code(&original_out),
        Some(0),
        "{name} original must fail"
    );
    assert_eq!(exit_code(&cap_out), exit_code(&original_out), "{name} exit");
    assert_eq!(cap_out.stdout, original_out.stdout, "{name} stdout");
    assert!(
        !cap_out.stderr.is_empty(),
        "{name} cap stderr should explain the failure"
    );
    let cap_stderr = String::from_utf8_lossy(&cap_out.stderr);
    let command_name = name.strip_prefix("run ").unwrap_or(name);
    assert!(
        cap_stderr.contains(command_name),
        "{name} stderr should name the command: {cap_stderr}",
    );
    if command_name != "sort" {
        assert!(
            cap_stderr.contains(missing),
            "{name} stderr should name the failed path: {cap_stderr}",
        );
    }
    Ok(())
}

fn run(program: &Path, args: &[&str]) -> Result<Output> {
    Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("run {} {}", program.display(), args.join(" ")))
}

fn exit_code(output: &Output) -> Option<i32> {
    output.status.code()
}

fn build_cap_frontend(bin_dir: &Path) -> Result<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cap = bin_dir.join(format!("cap{}", std::env::consts::EXE_SUFFIX));
    let fast = bin_dir.join(format!("cap-fast{}", std::env::consts::EXE_SUFFIX));
    let full = bin_dir.join(format!("cap-full{}", std::env::consts::EXE_SUFFIX));
    fs::copy(cap_full_binary()?, &full).context("copy cap-full sibling")?;

    let strip_flag = if cfg!(target_os = "macos") {
        "-Wl,-dead_strip"
    } else {
        "-Wl,--gc-sections"
    };
    let c_flags = [
        "-Oz",
        "-ffunction-sections",
        "-fdata-sections",
        "-fno-stack-protector",
        "-fno-unwind-tables",
        "-fno-asynchronous-unwind-tables",
        strip_flag,
    ];
    compile_c(&manifest.join("src/cap_fast_frontend.c"), &fast, &c_flags)?;

    let mut frontend_flags = c_flags.to_vec();
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        frontend_flags.extend([
            "-ffreestanding",
            "-fno-builtin",
            "-nostartfiles",
            "-Wl,-e,_start",
        ]);
    }
    compile_c(&manifest.join("src/cap_frontend.c"), &cap, &frontend_flags)?;

    if cfg!(target_os = "macos") {
        let _ = Command::new("codesign")
            .args(["-s", "-", "-f", "--options", "runtime"])
            .arg(&cap)
            .status();
        let _ = Command::new("codesign")
            .args(["-s", "-", "-f", "--options", "runtime"])
            .arg(&fast)
            .status();
        let _ = Command::new("codesign")
            .args(["-s", "-", "-f"])
            .arg(&full)
            .status();
    }

    Ok(cap)
}

fn compile_c(source: &Path, out: &Path, flags: &[&str]) -> Result<()> {
    let cc = std::env::var("CC").unwrap_or_else(|_| "/usr/bin/cc".to_string());
    let status = Command::new(cc)
        .args(flags)
        .arg(source)
        .arg("-o")
        .arg(out)
        .status()
        .with_context(|| format!("compile {}", source.display()))?;
    if !status.success() {
        bail!("compile {} failed with {status}", source.display());
    }
    Ok(())
}

fn cap_full_binary() -> Result<PathBuf> {
    if let Some(path) = option_env!("CARGO_BIN_EXE_cap-full") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }

    let current = std::env::current_exe().context("resolve test executable")?;
    let deps = current
        .parent()
        .context("test executable has no deps directory")?;
    let profile = deps
        .parent()
        .context("test executable has no profile directory")?;
    let candidate = profile.join(format!("cap-full{}", std::env::consts::EXE_SUFFIX));
    if candidate.is_file() {
        return Ok(candidate);
    }
    bail!("could not locate cap-full binary next to test profile")
}

struct Fixture {
    list_dir: String,
    cat_file: String,
    uniq_file: String,
    find_root: String,
    du_root: String,
    sort_file: String,
    sed_file: String,
    grep_root: String,
}

impl Fixture {
    fn create(root: &Path) -> Result<Self> {
        let data = root.join("data");
        fs::create_dir(&data)?;

        let list_dir = data.join("list");
        fs::create_dir(&list_dir)?;
        fs::write(list_dir.join("b.txt"), b"b\n")?;
        fs::write(list_dir.join("a.txt"), b"a\n")?;

        let cat_file = data.join("cat.txt");
        fs::write(&cat_file, b"alpha\nbeta\n")?;

        let uniq_file = data.join("uniq.txt");
        fs::write(&uniq_file, b"same\nsame\nnext\nnext\nsame\n")?;

        let find_root = data.join("find");
        fs::create_dir(&find_root)?;
        fs::write(find_root.join("only.txt"), b"found\n")?;

        let du_root = data.join("du");
        fs::create_dir(&du_root)?;
        fs::write(du_root.join("payload.bin"), vec![b'x'; 16 * 1024])?;

        let sort_file = data.join("sort.txt");
        let mut sort = fs::File::create(&sort_file)?;
        for idx in (0..120_000).rev() {
            writeln!(sort, "line-{idx:06}")?;
        }

        let sed_file = data.join("sed.txt");
        fs::write(&sed_file, b"one\ntwo\nthree\nfour\nfive\n")?;

        let grep_root = data.join("grep");
        fs::create_dir(&grep_root)?;
        fs::write(grep_root.join("match.txt"), b"plain\nNEEDLE here\n")?;

        Ok(Self {
            list_dir: path_string(&list_dir),
            cat_file: path_string(&cat_file),
            uniq_file: path_string(&uniq_file),
            find_root: path_string(&find_root),
            du_root: path_string(&du_root),
            sort_file: path_string(&sort_file),
            sed_file: path_string(&sed_file),
            grep_root: path_string(&grep_root),
        })
    }

    fn list_dir(&self) -> &str {
        &self.list_dir
    }
    fn cat_file(&self) -> &str {
        &self.cat_file
    }
    fn uniq_file(&self) -> &str {
        &self.uniq_file
    }
    fn find_root(&self) -> &str {
        &self.find_root
    }
    fn du_root(&self) -> &str {
        &self.du_root
    }
    fn sort_file(&self) -> &str {
        &self.sort_file
    }
    fn sed_file(&self) -> &str {
        &self.sed_file
    }
    fn grep_root(&self) -> &str {
        &self.grep_root
    }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/cap/tests/behavior_cap_command_replacement_parity.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Lossless rust-source-unit ownership for the command replacement parity integration test.
```
