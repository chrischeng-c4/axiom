use anyhow::{Context, Result};
use clap::{Arg, ArgAction, ArgMatches, Command};
use mamba::bench::{BenchRunner, BenchSuite, print_report, run_suite};
use mamba::conformance::{ConformanceOptions, run_suite as run_conformance_suite};
use mamba::driver::{Backend, CompilerConfig, CompilerSession, EmitMode, MambaConfig};
use mamba::pkgmanage::add as pkg_add;
use mamba::pkgmanage::builder as pkg_builder;
use mamba::pkgmanage::cache as pkg_cache;
use mamba::pkgmanage::export as pkg_export;
use mamba::pkgmanage::hash as pkg_hash;
use mamba::pkgmanage::index as pkg_index;
use mamba::pkgmanage::init as pkg_init;
use mamba::pkgmanage::install as pkg_install;
use mamba::pkgmanage::lock as pkg_lock;
use mamba::pkgmanage::pip as pkg_pip;
use mamba::pkgmanage::remove as pkg_remove;
use mamba::pkgmanage::sync as pkg_sync;
use mamba::pkgmanage::tree as pkg_tree;
use mamba::pkgmanage::validate as pkg_validate;
use mamba::pkgmanage::version as pkg_version;

// Force-link Mamba native binding crates so their #[distributed_slice(MAMBA_MODULES)]
// entries are included in the binary.  Without these, `mamba run` cannot resolve
// imports like `from mambalibs.pg import connect`.
#[cfg(feature = "native-modules")]
use agentkit_binding as _;
#[cfg(feature = "native-modules")]
use cclab_log_mamba as _;
#[cfg(feature = "native-modules")]
use cclab_mcp_mamba as _;
#[cfg(feature = "native-modules")]
use cclab_qc_mamba as _;
#[cfg(feature = "native-modules")]
use mambalibs_http_binding as _;
#[cfg(feature = "native-modules")]
use pgkit_binding as _;

fn cli() -> Command {
    Command::new("mamba")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Mamba - Force-typed Python compiler with native code generation")
        .subcommand(
            Command::new("build")
                .about("Compile a Mamba source file or project")
                .arg(Arg::new("file").help("Source file (.py/.tp); omit to use entry_point from mamba.toml"))
                .arg(Arg::new("config").short('c').long("config").value_name("PATH").help("Path to mamba.toml"))
                .arg(Arg::new("backend").short('b').long("backend").default_value("cranelift").help("Codegen backend: cranelift, llvm, wasm"))
                .arg(Arg::new("emit").long("emit").help("Dump intermediate: ast, hir, mir"))
                .arg(Arg::new("output").short('o').long("output").help("Output file path")),
        )
        .subcommand(
            Command::new("check")
                .about("Type-check a Mamba source file (no codegen)")
                .arg(Arg::new("file").required(true).help("Source file (.tp)")),
        )
        .subcommand(
            Command::new("run")
                .about("Compile and execute a Mamba source file or project")
                .arg(Arg::new("file").help("Source file (.py/.tp); omit to use entry_point from mamba.toml"))
                .arg(Arg::new("config").short('c').long("config").value_name("PATH").help("Path to mamba.toml")),
        )
        .subcommand(
            Command::new("bench")
                .about("Run the Mamba benchmark suite")
                .arg(Arg::new("compare").long("compare").value_name("ENGINE").help("Compare against: cpython"))
                .arg(Arg::new("filter").long("filter").value_name("KIND").help("Filter: numeric, recursion, workload"))
                .arg(Arg::new("file").value_name("FILE").help("Benchmark a single file"))
                .arg(Arg::new("fixtures").long("fixtures").value_name("DIR").default_missing_value(format!("{}/core/bench", mamba::conformance::FIXTURES_ROOT)).num_args(0..=1).help("Run fixture-based benchmarks"))
                .arg(Arg::new("json").long("json").value_name("PATH").help("Write timings + ratios to a JSON file (baseline format)"))
                .arg(Arg::new("check").long("check").value_name("PATH").help("Compare current run against the baseline JSON; fail on regression beyond --threshold")),
        )
        .subcommand(
            Command::new("test")
                .about("Run Mamba tests — Python-perspective, CPython-style runner")
                .arg(Arg::new("path").help("Test file or directory containing test_*.py; if omitted, requires --conformance or --regen-golden"))
                .arg(Arg::new("conformance").long("conformance").action(ArgAction::SetTrue).help("Run CPython 3.12 conformance suite"))
                .arg(Arg::new("category").long("category").value_name("NAME").help("Filter conformance category"))
                .arg(Arg::new("dir").long("dir").value_name("PATH").help("Conformance fixtures directory"))
                .arg(Arg::new("regen-golden").long("regen-golden").action(ArgAction::SetTrue).help("Regenerate .expected golden files"))
                .arg(Arg::new("timeout").long("timeout").value_name("SECS").default_value("60").help("Per-test timeout in seconds (path mode)"))
                .arg(Arg::new("jobs").long("jobs").short('j').value_name("N").default_value("4").help("Max parallel subprocesses (path mode)")),
        )
        .subcommand(
            Command::new("test-batch")
                .about("Zygote-fork batch conformance runner: init the runtime ONCE, fork a worker per fixture (isolated, init amortized via COW). Reports per-path PASS/FAIL/TIMEOUT/CRASH.")
                .arg(Arg::new("paths").long("paths").value_name("FILE").help("File with one fixture path per line; omit to read paths from stdin"))
                .arg(Arg::new("jobs").long("jobs").short('j').value_name("N").default_value("8").help("Max concurrent worker forks"))
                .arg(Arg::new("timeout").long("timeout").value_name("SECS").default_value("10").help("Per-fixture timeout in seconds"))
                .arg(Arg::new("json").long("json").value_name("PATH").help("Write per-path results as JSON lines (path\\tverdict)")),
        )
        .subcommand(
            Command::new("surface-report")
                .about("Compare mamba's registered Python-level surface for a stdlib/3p package against the typeshed stub")
                .arg(Arg::new("package").long("package").short('p').required(true).value_name("NAME").help("Package name (e.g. os, ssl, idna)"))
                .arg(Arg::new("typeshed").long("typeshed").value_name("PATH").help("Path to typeshed checkout (default: vendor/typeshed)"))
                .arg(Arg::new("mamba-src").long("mamba-src").value_name("PATH").help("Path to projects/mamba/src (default: projects/mamba/src)")),
        )
        .subcommand(
            Command::new("pytest")
                .about("Run pytest-under-mamba on a CPython Lib/test-style file or directory")
                .arg(Arg::new("path").required(true).help("Test file or directory containing test_*.py"))
                .arg(Arg::new("timeout").long("timeout").value_name("SECS").default_value("60").help("Per-test timeout in seconds"))
                .arg(Arg::new("jobs").long("jobs").short('j').value_name("N").default_value("4").help("Max parallel subprocesses")),
        )
        .subcommand(
            Command::new("init")
                .about("Scaffold a new mamba project (mamba.toml, .python-version, .gitignore, README.md, src/__init__.py)")
                .arg(Arg::new("path").help("Project directory; defaults to current working directory")),
        )
        .subcommand(
            Command::new("add")
                .about("Add a dependency to mamba.toml and update mamba.lock")
                .arg(Arg::new("spec").required(true).help("Dependency spec or local wheel path, e.g. foo==1.2.3 or ./wheels/foo-1.2.3-py3-none-any.whl (bare names require --index or explicit --index-url)"))
                .arg(Arg::new("index").long("index").value_name("DIR").help("Frozen local index directory (overrides $MAMBA_FROZEN_INDEX)"))
                .arg(Arg::new("index-url").long("index-url").value_name("URL").help("Explicit PyPI-compatible registry base URL (overrides $MAMBA_INDEX_URL)"))
                .arg(Arg::new("offline").long("offline").action(ArgAction::SetTrue).help("Disallow network; require pinned version or local index")),
        )
        .subcommand(
            Command::new("remove")
                .about("Remove a dependency from mamba.toml and update mamba.lock")
                .arg(Arg::new("name").required(true).help("Dependency name (no version pin)")),
        )
        .subcommand(
            Command::new("lock")
                .about("Regenerate mamba.lock from mamba.toml; resolves transitive deps via a frozen index or explicit registry URL")
                .arg(Arg::new("index").long("index").value_name("DIR").help("Frozen local index directory (overrides $MAMBA_FROZEN_INDEX)"))
                .arg(Arg::new("index-url").long("index-url").value_name("URL").help("Explicit PyPI-compatible registry base URL (overrides $MAMBA_INDEX_URL)"))
                .arg(Arg::new("offline").long("offline").action(ArgAction::SetTrue).help("Disallow network; require frozen index")),
        )
        .subcommand(
            Command::new("export")
                .about("Export mamba.lock to requirements.txt or pylock.toml")
                .arg(Arg::new("format").long("format").value_name("FORMAT").default_value("requirements-txt").help("requirements-txt | pylock.toml"))
                .arg(Arg::new("output-file").long("output-file").short('o').value_name("PATH").help("Write output to PATH; omit or use - for stdout"))
                .arg(Arg::new("no-hashes").long("no-hashes").action(ArgAction::SetTrue).help("Do not emit --hash continuations in requirements-txt output"))
                .arg(Arg::new("no-header").long("no-header").action(ArgAction::SetTrue).help("Do not emit the generated header in requirements-txt output"))
                .arg(Arg::new("no-emit-package").long("no-emit-package").value_name("NAME").action(ArgAction::Append).help("Exclude a package from requirements-txt output"))
                .arg(Arg::new("marker").long("marker").value_name("PEP508").help("Append a global PEP 508 environment marker to requirements-txt pins"))
                .arg(Arg::new("annotate").long("annotate").action(ArgAction::SetTrue).help("Annotate requirements pins with reverse dependency comments"))
                .arg(Arg::new("requires-python").long("requires-python").value_name("SPEC").help("Set requires-python in pylock.toml output"))
                .arg(Arg::new("environment").long("environment").value_name("MARKER").action(ArgAction::Append).help("Add an environment marker to pylock.toml output")),
        )
        .subcommand(
            Command::new("tree")
                .about("Display the dependency tree from mamba.lock")
                .arg(Arg::new("depth").long("depth").value_name("N").help("Maximum tree depth"))
                .arg(Arg::new("package").long("package").short('p').value_name("NAME").help("Render only the subtree rooted at NAME"))
                .arg(Arg::new("invert").long("invert").action(ArgAction::SetTrue).help("Show reverse dependency tree"))
                .arg(Arg::new("prune").long("prune").value_name("NAME").action(ArgAction::Append).help("Skip a dependency subtree"))
                .arg(Arg::new("no-dedupe").long("no-dedupe").action(ArgAction::SetTrue).help("Render repeated subtrees instead of marking duplicates")),
        )
        .subcommand(
            Command::new("version")
                .about("Read, set, or bump the PEP 621 [project].version in pyproject.toml")
                .arg(Arg::new("version").value_name("VERSION").help("Explicit PEP 440 version to set"))
                .arg(Arg::new("bump").long("bump").value_name("KIND").help("major | minor | patch | alpha | beta | rc | post | dev | release"))
                .arg(Arg::new("dry-run").long("dry-run").action(ArgAction::SetTrue).help("Print the next version without writing pyproject.toml")),
        )
        .subcommand(
            Command::new("pip")
                .about("pip-compatible installed-environment inspection")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("list")
                        .about("List installed distributions from site-packages")
                        .arg(Arg::new("site-packages").long("site-packages").value_name("DIR").help("site-packages directory; defaults to .venv/site-packages"))
                        .arg(Arg::new("format").long("format").value_name("FORMAT").default_value("columns").help("columns | freeze"))
                        .arg(Arg::new("no-header").long("no-header").action(ArgAction::SetTrue).help("Omit column headers"))
                        .arg(Arg::new("sort-by-version").long("sort-by-version").action(ArgAction::SetTrue).help("Sort rows by version instead of package name")),
                )
                .subcommand(
                    Command::new("freeze")
                        .about("Emit installed distributions as requirements pins")
                        .arg(Arg::new("site-packages").long("site-packages").value_name("DIR").help("site-packages directory; defaults to .venv/site-packages")),
                )
                .subcommand(
                    Command::new("show")
                        .about("Show metadata for one installed distribution")
                        .arg(Arg::new("name").required(true).help("Package name"))
                        .arg(Arg::new("site-packages").long("site-packages").value_name("DIR").help("site-packages directory; defaults to .venv/site-packages")),
                )
                .subcommand(
                    Command::new("check")
                        .about("Check installed distribution requirements")
                        .arg(Arg::new("site-packages").long("site-packages").value_name("DIR").help("site-packages directory; defaults to .venv/site-packages")),
                ),
        )
        .subcommand(
            Command::new("index")
                .about("Build frozen local package indexes from wheel artifacts")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("build")
                        .about("Build a frozen local index from wheel files or directories")
                        .arg(Arg::new("out").long("out").short('o').required(true).value_name("DIR").help("Output frozen index directory"))
                        .arg(Arg::new("paths").required(true).num_args(1..).value_name("WHEEL_OR_DIR").help("Wheel file(s) or directories to scan recursively")),
                ),
        )
        .subcommand(
            Command::new("sync")
                .about("Converge `.venv`/site-packages to mamba.lock (idempotent; second run is a no-op)")
                .arg(Arg::new("jobs").long("jobs").short('j').value_name("N").help("Max concurrent downloads (overrides $MAMBA_JOBS; default 8)")),
        )
        .subcommand(
            Command::new("pkgmgr-validate")
                .about("Drive the 9 release-blocking pkgmgr workflow families and emit a summary")
                .arg(Arg::new("include-live-network").long("include-live-network").action(ArgAction::SetTrue).help("Also walk opt-in live-network workflows (never blocks)"))
                .arg(Arg::new("json").long("json").action(ArgAction::SetTrue).help("Emit machine-readable JSON to stdout")),
        )
        .subcommand(
            Command::new("install")
                .about("Install a tool from the frozen local index into $MAMBA_TOOLS_DIR")
                .arg(Arg::new("name").help("Package name (omit when using --list)"))
                .arg(Arg::new("version").long("version").value_name("X.Y.Z").help("Pin a specific version (default: latest in index)"))
                .arg(Arg::new("index").long("index").value_name("DIR").help("Frozen local index directory (overrides $MAMBA_FROZEN_INDEX)"))
                .arg(Arg::new("list").long("list").action(ArgAction::SetTrue).help("List installed tools"))
                .arg(Arg::new("uninstall").long("uninstall").value_name("NAME").help("Uninstall a tool")),
        )
        .subcommand(
            Command::new("hash")
                .about("Compute a content-addressed digest of one or more files")
                .arg(Arg::new("path").required(true).num_args(1..).help("File(s) to hash"))
                .arg(Arg::new("algorithm").long("algorithm").short('a').value_name("ALGO").help("sha256 (default) | sha384 | sha512")),
        )
        .subcommand(
            Command::new("cache")
                .about("Inspect and maintain the package cache root (respects $MAMBA_CACHE_DIR)")
                .subcommand(Command::new("dir").about("Print the resolved cache root"))
                .subcommand(Command::new("clean").about("Remove every entry under the cache root"))
                .subcommand(Command::new("prune").about("Remove stale cache entries (currently equivalent to clean)")),
        )
}

fn main() -> Result<()> {
    // Install the ObjectOps callback table before any binding code can fire.
    // Bindings that depend only on cclab-mamba-registry call through
    // `registry::ops()` to allocate / inspect mamba objects and to raise
    // exceptions; the table must be populated before the first such call.
    mamba::runtime::registry_bridge::install();
    // Trip loudly if a force-link `use … as _;` was dropped — linkme misses
    // are otherwise silent and only surface as `ModuleNotFoundError` deep
    // inside user code.
    pkg_builder::assert_all_registered();

    // Python-style direct invocation, intercepted before clap parses
    // subcommands:
    //   mamba -c "<code>"   — run an inline program (CPython `python -c`)
    //   mamba <file>.py     — run a script (CPython `python file.py`)
    // `sys.executable` points at this binary, so CPython-conformance
    // fixtures spawn `[sys.executable, "-c", ...]` and expect both forms.
    {
        let argv: Vec<String> = std::env::args().collect();
        if argv.len() >= 2 {
            if argv[1] == "-c" {
                let Some(code) = argv.get(2) else {
                    eprintln!("Argument expected for the -c option");
                    std::process::exit(2);
                };
                return run_inline_source(code, "<string>");
            }
            if argv[1].ends_with(".py") && std::path::Path::new(&argv[1]).exists() {
                return run_script_path(&argv[1]);
            }
        }
    }

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("build", sub)) => pkg_builder::cmd_build(sub),
        Some(("check", sub)) => cmd_check(sub),
        Some(("run", sub)) => cmd_run(sub),
        Some(("bench", sub)) => cmd_bench(sub),
        Some(("test", sub)) => cmd_test(sub),
        Some(("test-batch", sub)) => cmd_test_batch(sub),
        Some(("surface-report", sub)) => cmd_surface_report(sub),
        Some(("pytest", sub)) => cmd_pytest(sub),
        Some(("init", sub)) => pkg_init::cmd_init(sub),
        Some(("add", sub)) => pkg_add::cmd_add(sub),
        Some(("remove", sub)) => pkg_remove::cmd_remove(sub),
        Some(("lock", sub)) => pkg_lock::cmd_lock(sub),
        Some(("export", sub)) => pkg_export::cmd_export(sub),
        Some(("tree", sub)) => pkg_tree::cmd_tree(sub),
        Some(("version", sub)) => pkg_version::cmd_version(sub),
        Some(("pip", sub)) => pkg_pip::cmd_pip(sub),
        Some(("index", sub)) => pkg_index::cmd_index(sub),
        Some(("sync", sub)) => pkg_sync::cmd_sync(sub),
        Some(("cache", sub)) => pkg_cache::cmd_cache(sub),
        Some(("hash", sub)) => pkg_hash::cmd_hash(sub),
        Some(("install", sub)) => pkg_install::cmd_install(sub),
        Some(("pkgmgr-validate", sub)) => pkg_validate::cmd_validate(sub),
        _ => {
            cli().print_help().ok();
            Ok(())
        }
    }
}

/// Run ONE fixture in the current (forked child) process and return an exit
/// code: 0 = PASS, 1 = FAIL. The child captures its own stdout to a temp file,
/// runs the full compile+execute pipeline under `catch_unwind`, and decides
/// PASS = (run returned Ok) AND stdout contains "<stem> OK" (the fixture
/// self-check convention). A hard crash (SIGSEGV/abort/stack-overflow) never
/// reaches here — the kernel kills the child and the parent records CRASH.
fn run_one_fixture(path: &str) -> i32 {
    let stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    // Redirect stdout -> per-child temp file, stderr -> /dev/null, so the
    // fixture's prints are captured and compiler diagnostics don't pollute the
    // parent's terminal. Use the raw fds (we never return; _exit follows).
    let pid = unsafe { libc::getpid() };
    let tmp = format!("/tmp/mb_testbatch_{pid}.out");
    unsafe {
        if let Ok(c_tmp) = std::ffi::CString::new(tmp.as_str()) {
            let fd = libc::open(
                c_tmp.as_ptr(),
                libc::O_CREAT | libc::O_RDWR | libc::O_TRUNC,
                0o600,
            );
            if fd >= 0 {
                libc::dup2(fd, 1);
                libc::close(fd);
            }
        }
        if let Ok(devnull) = std::ffi::CString::new("/dev/null") {
            let nfd = libc::open(devnull.as_ptr(), libc::O_WRONLY);
            if nfd >= 0 {
                libc::dup2(nfd, 2);
                libc::close(nfd);
            }
        }
    }

    let owned = path.to_string();
    let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let config = CompilerConfig {
            backend: Backend::CraneliftJit,
            project_config: None,
            ..Default::default()
        };
        let mut session = CompilerSession::new(config);
        session.run(&owned)
    }));

    // Flush the captured stdout then read it back to check the OK marker.
    use std::io::Write as _;
    std::io::stdout().flush().ok();
    let captured = std::fs::read_to_string(&tmp).unwrap_or_default();
    let _ = std::fs::remove_file(&tmp);

    let ok = matches!(ran, Ok(Ok(()))) && captured.contains(&format!("{stem} OK"));
    if ok { 0 } else { 1 }
}

/// Zygote-fork batch conformance runner. The fixed ~0.16s per-process init
/// (Cranelift ISA + eager stdlib registration) is the dominant cost of a
/// process-per-fixture sweep; here we pay it ONCE in this (parent) process,
/// then `fork()` a worker per fixture. Each child COW-inherits the warm
/// runtime (skipping stdlib re-registration) and runs in an isolated address
/// space — so per-fixture isolation is sound (no cross-fixture state leakage)
/// and a crashing fixture only kills its own child, never the pool.
fn cmd_test_batch(sub: &ArgMatches) -> Result<()> {
    use std::collections::HashMap;
    use std::io::{Read as _, Write as _};
    use std::time::Instant;

    let paths: Vec<String> = {
        let text = if let Some(pf) = sub.get_one::<String>("paths") {
            std::fs::read_to_string(pf).with_context(|| format!("reading {pf}"))?
        } else {
            let mut s = String::new();
            std::io::stdin()
                .read_to_string(&mut s)
                .context("reading stdin")?;
            s
        };
        text.lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    };
    let jobs: usize = sub
        .get_one::<String>("jobs")
        .and_then(|s| s.parse().ok())
        .unwrap_or(8)
        .max(1);
    let timeout_secs: u64 = sub
        .get_one::<String>("timeout")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    let total = paths.len();
    if total == 0 {
        println!("test-batch: 0 fixtures");
        return Ok(());
    }

    // WARM the zygote: register stdlib + search paths into THIS process's
    // thread-locals so forked children inherit them (COW) and skip the
    // ~40-60ms re-registration. Critically, we do NOT compile/run any fixture
    // in the parent — so JIT_LOCK is never held and no user/async state exists
    // at any fork point, keeping the children's inherited locks clean.
    mamba::runtime::module::mb_register_native_modules();
    mamba::runtime::module::mb_register_builtins();
    mamba::runtime::module::mb_init_search_paths();
    // Build the Cranelift ISA + runtime symbol-table caches in the PARENT so
    // every forked worker COW-inherits them read-only instead of each rebuilding
    // them (fork-per-fixture = one compile per child = zero LazyLock amortization
    // unless the caches are warm before the fork).
    mamba::codegen::cranelift::jit::warm_jit_caches();

    // Flush parent buffers before forking so nothing is duplicated in children.
    std::io::stdout().flush().ok();
    std::io::stderr().flush().ok();

    let start = Instant::now();
    // verdict: 1=pass 2=fail 3=timeout 4=crash
    let mut results: Vec<u8> = vec![0; total];
    let mut inflight: HashMap<i32, (usize, Instant)> = HashMap::new();
    let mut next = 0usize;
    let (mut pass, mut fail, mut timeout_n, mut crash) = (0usize, 0usize, 0usize, 0usize);

    loop {
        // Fill the pool.
        while inflight.len() < jobs && next < total {
            let idx = next;
            next += 1;
            let path = paths[idx].clone();
            let pid = unsafe { libc::fork() };
            if pid == 0 {
                // CHILD — run one fixture, never return.
                let code = run_one_fixture(&path);
                unsafe { libc::_exit(code) };
            } else if pid > 0 {
                inflight.insert(pid, (idx, Instant::now()));
            } else {
                // fork() failed — treat as crash and keep going.
                results[idx] = 4;
                crash += 1;
            }
        }
        if inflight.is_empty() && next >= total {
            break;
        }

        // Reap one finished child without blocking; otherwise check timeouts.
        let mut status: libc::c_int = 0;
        let r = unsafe { libc::waitpid(-1, &mut status, libc::WNOHANG) };
        if r > 0 {
            if let Some((idx, _)) = inflight.remove(&r) {
                let exited = libc::WIFEXITED(status);
                let code = if exited {
                    libc::WEXITSTATUS(status)
                } else {
                    -1
                };
                match (exited, code) {
                    (true, 0) => {
                        results[idx] = 1;
                        pass += 1;
                    }
                    (true, 1) => {
                        results[idx] = 2;
                        fail += 1;
                    }
                    _ => {
                        // killed by signal / abnormal exit = hard crash
                        results[idx] = 4;
                        crash += 1;
                    }
                }
            }
        } else {
            // Nothing ready: enforce timeouts, then yield briefly.
            let now = Instant::now();
            let killed: Vec<(i32, usize)> = inflight
                .iter()
                .filter(|(_, (_, st))| now.duration_since(*st).as_secs() >= timeout_secs)
                .map(|(pid, (idx, _))| (*pid, *idx))
                .collect();
            for (pid, idx) in killed {
                unsafe {
                    libc::kill(pid, libc::SIGKILL);
                    let mut s2: libc::c_int = 0;
                    libc::waitpid(pid, &mut s2, 0);
                }
                inflight.remove(&pid);
                results[idx] = 3;
                timeout_n += 1;
            }
            std::thread::sleep(std::time::Duration::from_micros(500));
        }
    }

    let elapsed = start.elapsed();

    if let Some(jpath) = sub.get_one::<String>("json") {
        let mut out = String::new();
        for (i, p) in paths.iter().enumerate() {
            let v = match results[i] {
                1 => "PASS",
                2 => "FAIL",
                3 => "TIMEOUT",
                4 => "CRASH",
                _ => "UNKNOWN",
            };
            out.push_str(p);
            out.push('\t');
            out.push_str(v);
            out.push('\n');
        }
        std::fs::write(jpath, out).with_context(|| format!("writing {jpath}"))?;
    }

    println!(
        "test-batch: {total} fixtures in {:.2}s ({:.1}/s, jobs={jobs})\n  PASS={pass} FAIL={fail} TIMEOUT={timeout_n} CRASH={crash}",
        elapsed.as_secs_f64(),
        total as f64 / elapsed.as_secs_f64().max(1e-9),
    );
    Ok(())
}

fn cmd_surface_report(sub: &ArgMatches) -> Result<()> {
    let package = sub
        .get_one::<String>("package")
        .expect("clap enforces required");
    let typeshed = sub
        .get_one::<String>("typeshed")
        .cloned()
        .unwrap_or_else(|| {
            // Default: probe project-local checkout, fall back to a repo-root one.
            for p in ["projects/mamba/vendor/typeshed", "vendor/typeshed"] {
                if std::path::Path::new(p).is_dir() {
                    return p.to_string();
                }
            }
            "projects/mamba/vendor/typeshed".to_string()
        });
    let mamba_src = sub
        .get_one::<String>("mamba-src")
        .cloned()
        .unwrap_or_else(|| "projects/mamba/src".to_string());

    let report = mamba::surface::build_report(
        package,
        std::path::Path::new(&typeshed),
        std::path::Path::new(&mamba_src),
    )
    .map_err(|e| anyhow::anyhow!(e))
    .with_context(|| format!("surface report for `{}`", package))?;

    print!("{}", report.render());
    Ok(())
}

fn cmd_pytest(sub: &ArgMatches) -> Result<()> {
    use mamba::conformance::pytest_runner::{self, PytestOptions};

    let path = sub
        .get_one::<String>("path")
        .expect("clap enforces required");
    let timeout_secs: u64 = sub
        .get_one::<String>("timeout")
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    let jobs: usize = sub
        .get_one::<String>("jobs")
        .and_then(|s| s.parse().ok())
        .unwrap_or(4)
        .max(1);
    let path_buf = std::path::PathBuf::from(path);
    if !path_buf.exists() {
        anyhow::bail!("path does not exist: {}", path_buf.display());
    }
    let mut opts = PytestOptions::new(path_buf);
    opts.timeout_secs = timeout_secs;
    opts.jobs = jobs;
    let summary = pytest_runner::run(&opts);
    if !summary.success() {
        std::process::exit(1);
    }
    Ok(())
}

fn cmd_check(sub: &ArgMatches) -> Result<()> {
    let file = sub.get_one::<String>("file").unwrap();
    let config = CompilerConfig::default();
    let mut session = CompilerSession::new(config);
    match session.check(file) {
        Ok(()) => println!("Type check passed"),
        Err(e) => {
            eprintln!("{}", session.render_error(&e));
            std::process::exit(1);
        }
    }
    Ok(())
}

/// Map a failed program run to a CPython-style process exit. `SystemExit`
/// carries the exit status (no argument → 0, int → that code, anything
/// else → message on stderr + 1); every other uncaught exception renders
/// and exits 1.
fn exit_like_cpython(session: &CompilerSession, err: &mamba::error::MambaError) -> ! {
    let rendered = session.render_error(err);
    let stripped = rendered.strip_prefix("error: ").unwrap_or(&rendered);
    // A program with no executable top-level statements (`pass`, comments
    // only) produces no JIT entry point; CPython exits 0 on such input.
    if stripped.contains("no entry point found") {
        std::process::exit(0);
    }
    if let Some(rest) = stripped.strip_prefix("SystemExit") {
        let payload = rest.trim_start_matches(':').trim();
        if payload.is_empty() || payload == "None" {
            std::process::exit(0);
        }
        if let Ok(code) = payload.parse::<i32>() {
            std::process::exit(code);
        }
        eprintln!("{payload}");
        std::process::exit(1);
    }
    eprintln!("{rendered}");
    std::process::exit(1);
}

/// `mamba -c "<code>"` — compile and run an inline program, mirroring
/// CPython's `python -c`. Exit code follows the program (SystemExit /
/// uncaught exception → non-zero), no package-manager preflight.
fn run_inline_source(code: &str, name: &str) -> Result<()> {
    let config = CompilerConfig {
        backend: Backend::CraneliftJit,
        ..Default::default()
    };
    let mut session = CompilerSession::new(config);
    match session.run_source(code, name) {
        Ok(()) => Ok(()),
        Err(e) => exit_like_cpython(&session, &e),
    }
}

/// `mamba <file>.py` — run a script directly, mirroring `python file.py`.
fn run_script_path(path: &str) -> Result<()> {
    let config = CompilerConfig {
        backend: Backend::CraneliftJit,
        ..Default::default()
    };
    let mut session = CompilerSession::new(config);
    match session.run(path) {
        Ok(()) => Ok(()),
        Err(e) => exit_like_cpython(&session, &e),
    }
}

fn cmd_run(sub: &ArgMatches) -> Result<()> {
    use std::io::Read as _;

    // Package-manager preflight: when invoked inside a mamba project
    // with a populated lockfile, require `.venv` to be in sync before
    // running anything. See projects/mamba/src/pkgmanage/run.rs.
    let cwd_for_preflight = std::env::current_dir().context("getcwd")?;
    if let Err(e) = mamba::pkgmanage::run::preflight(&cwd_for_preflight) {
        eprintln!("{e}");
        std::process::exit(1);
    }

    let project_config: Option<MambaConfig> =
        if let Some(cfg_path) = sub.get_one::<String>("config") {
            Some(MambaConfig::from_file(std::path::Path::new(cfg_path))?)
        } else {
            let cwd = std::env::current_dir().context("getcwd")?;
            MambaConfig::discover(&cwd).map(|(cfg, _)| cfg)
        };

    let file_arg = sub.get_one::<String>("file");
    let entry_from_config = project_config
        .as_ref()
        .and_then(|c| c.entry_point().map(|s| s.to_string()));
    let file: String = match (file_arg, entry_from_config) {
        (Some(f), _) => f.clone(),
        (None, Some(ep)) => ep,
        (None, None) => anyhow::bail!(
            "no source file specified and no mamba.toml found; pass a file or cd into a project directory"
        ),
    };

    let config = CompilerConfig {
        backend: Backend::CraneliftJit,
        project_config,
        ..Default::default()
    };
    let mut session = CompilerSession::new(config);

    // REQ: R1 — when the argument is the stdin sentinel "-", read from stdin.
    if file == "-" {
        // REQ: R3 — reject when stdin is a TTY (no piped input provided).
        use std::io::IsTerminal as _;
        if std::io::stdin().is_terminal() {
            eprintln!("error: stdin is a tty, no script provided");
            std::process::exit(1);
        }

        let mut source = String::new();
        std::io::stdin()
            .read_to_string(&mut source)
            .context("reading stdin")?;

        // REQ: R2 — exit behaviour mirrors `mamba run <file.py>`.
        match session.run_source(&source, "<stdin>") {
            Ok(()) => {}
            Err(e) => {
                eprintln!("{}", session.render_error(&e));
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // REQ: R4 — existing file-path behaviour is preserved.
    match session.run(&file) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{}", session.render_error(&e));
            std::process::exit(1);
        }
    }
    Ok(())
}

fn cmd_bench(sub: &ArgMatches) -> Result<()> {
    if sub.contains_id("fixtures")
        && sub.value_source("fixtures") != Some(clap::parser::ValueSource::DefaultValue)
    {
        let dir = sub
            .get_one::<String>("fixtures")
            .map(|s| std::path::PathBuf::from(s))
            .unwrap_or_else(|| {
                std::path::PathBuf::from(format!(
                    "{}/core/bench",
                    mamba::conformance::FIXTURES_ROOT
                ))
            });
        let mamba_bin =
            std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("mamba"));
        let fixtures = mamba::bench::discover_fixtures(&dir);
        if fixtures.is_empty() {
            anyhow::bail!("no .py fixtures found in {}", dir.display());
        }
        println!(
            "Running {} fixture benchmarks from {}...",
            fixtures.len(),
            dir.display()
        );
        let rows = mamba::bench::run_fixture_suite(&fixtures, &mamba_bin);
        mamba::bench::print_fixture_report(&rows);
        let ok = rows
            .iter()
            .filter(|r| r.mamba.as_ref().map_or(false, |m| m.correct))
            .count();
        let fail = rows.len() - ok;
        println!("{ok} correct, {fail} failed.");
        return Ok(());
    }

    let compare = sub.get_one::<String>("compare").map(|s| s.as_str());
    let filter = sub.get_one::<String>("filter").map(|s| s.as_str());
    let file = sub.get_one::<String>("file");

    let runner = if matches!(compare, Some("cpython")) {
        BenchRunner::default()
    } else {
        BenchRunner::mamba_only()
    };

    let suite = if let Some(path) = file {
        let source =
            std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!("read {path}: {e}"))?;
        let leaked: &'static str = Box::leak(source.into_boxed_str());
        let name: &'static str = Box::leak(path.to_string().into_boxed_str());
        BenchSuite {
            benchmarks: vec![mamba::bench::Benchmark {
                name,
                source: leaked,
                kind: mamba::bench::BenchKind::Workload,
                iters: 10,
            }],
        }
    } else {
        BenchSuite::builtin()
    };

    let filtered_suite = if let Some(kind_str) = filter {
        use mamba::bench::BenchKind;
        let kind = match kind_str {
            "numeric" => BenchKind::Numeric,
            "recursion" => BenchKind::Recursion,
            "workload" => BenchKind::Workload,
            other => anyhow::bail!("unknown kind {other:?}; use: numeric, recursion, workload"),
        };
        BenchSuite {
            benchmarks: suite
                .benchmarks
                .into_iter()
                .filter(|b| b.kind == kind)
                .collect(),
        }
    } else {
        suite
    };

    println!(
        "Running Mamba benchmark suite ({} benchmarks)...",
        filtered_suite.benchmarks.len()
    );
    let rows = run_suite(&filtered_suite, &runner);
    print_report(&rows);
    let ok = rows.iter().filter(|r| r.mamba_ns_mean.is_some()).count();
    let err = rows.len() - ok;
    println!("{ok} succeeded, {err} failed.");

    // --json <path>: emit a baseline-friendly JSON snapshot of the run.
    if let Some(path) = sub.get_one::<String>("json") {
        write_baseline_json(path, &rows)
            .map_err(|e| anyhow::anyhow!("write baseline {path}: {e}"))?;
        println!("baseline written to {path}");
    }

    // --check <baseline>: regression gate. Compares mamba_ns_mean against the
    // committed baseline; exits non-zero if any benchmark is more than 10%
    // slower than the baseline median (CI hook).
    if let Some(path) = sub.get_one::<String>("check") {
        let regressions = check_against_baseline(path, &rows, 1.10)
            .map_err(|e| anyhow::anyhow!("read baseline {path}: {e}"))?;
        if !regressions.is_empty() {
            for line in &regressions {
                eprintln!("regression: {line}");
            }
            anyhow::bail!("{} benchmark regression(s) vs {}", regressions.len(), path);
        }
        println!("no regression vs {path}");
    }

    Ok(())
}

/// JSON writer for `mamba bench --json baseline.json`.
///
/// Format (kept intentionally simple so a future CI gate can diff easily):
/// ```json
/// {
///   "version": 1,
///   "benchmarks": [
///     {"name": "fib30", "kind": "Recursion",
///      "mamba_ns": 12345, "cpython_ns": 67890,
///      "speedup_vs_cpython": 5.5}
///   ]
/// }
/// ```
fn write_baseline_json(path: &str, rows: &[mamba::bench::ReportRow]) -> std::io::Result<()> {
    use std::fmt::Write as _;
    let mut s = String::from("{\n  \"version\": 1,\n  \"benchmarks\": [");
    for (i, r) in rows.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str("\n    {");
        let _ = write!(s, "\"name\": {:?}, ", r.name);
        let _ = write!(s, "\"kind\": {:?}", format!("{:?}", r.kind));
        if let Some(v) = r.mamba_ns_mean {
            let _ = write!(s, ", \"mamba_ns\": {v}");
        }
        if let Some(v) = r.cpython_ns_mean {
            let _ = write!(s, ", \"cpython_ns\": {v}");
        }
        if let Some(v) = r.pypy_ns_mean {
            let _ = write!(s, ", \"pypy_ns\": {v}");
        }
        if let Some(sp) = r.speedup() {
            let _ = write!(s, ", \"speedup_vs_cpython\": {sp:.4}");
        }
        if let Some(err) = r.mamba_error.as_deref() {
            let _ = write!(s, ", \"error\": {err:?}");
        }
        s.push('}');
    }
    s.push_str("\n  ]\n}\n");
    std::fs::write(path, s)
}

/// Compare current results against a committed baseline JSON. Returns a list
/// of human-readable regression descriptions; an empty list means no
/// benchmark exceeded the regression threshold.
///
/// `threshold` is the multiplier on baseline mean time before a slower run
/// counts as a regression (e.g. 1.10 = "fail if 10% slower than baseline").
fn check_against_baseline(
    path: &str,
    rows: &[mamba::bench::ReportRow],
    threshold: f64,
) -> std::io::Result<Vec<String>> {
    let raw = std::fs::read_to_string(path)?;
    // Tiny ad-hoc parser to avoid pulling in serde_json: extract
    // "name" → mamba_ns pairs from the simple shape we emit above.
    let mut baseline: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    for chunk in raw.split('{').skip(2) {
        let name = match chunk.find("\"name\":") {
            Some(start) => {
                let after = &chunk[start + 7..];
                let q = match after.find('"') {
                    Some(q) => q,
                    None => continue,
                };
                let after_q = &after[q + 1..];
                let end = match after_q.find('"') {
                    Some(e) => e,
                    None => continue,
                };
                after_q[..end].to_string()
            }
            None => continue,
        };
        let mamba_ns = match chunk.find("\"mamba_ns\":") {
            Some(start) => {
                let after = chunk[start + 11..].trim_start();
                let end = after
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(after.len());
                after[..end].parse::<u64>().ok()
            }
            None => None,
        };
        if let Some(ns) = mamba_ns {
            baseline.insert(name, ns);
        }
    }

    let mut out = Vec::new();
    for r in rows {
        let now = match r.mamba_ns_mean {
            Some(v) => v,
            None => continue,
        };
        if let Some(&base) = baseline.get(&r.name) {
            let ratio = now as f64 / base.max(1) as f64;
            if ratio > threshold {
                out.push(format!(
                    "{} now {} ns/op vs baseline {} ns/op ({:.2}× slower)",
                    r.name, now, base, ratio
                ));
            }
        }
    }
    Ok(out)
}

fn cmd_test(sub: &ArgMatches) -> Result<()> {
    let regen = sub.get_flag("regen-golden");
    let conformance = sub.get_flag("conformance");
    let path = sub.get_one::<String>("path");

    if let Some(p) = path {
        if conformance || regen {
            anyhow::bail!(
                "`mamba test <path>` is mutually exclusive with --conformance and --regen-golden"
            );
        }
        use mamba::conformance::pytest_runner::{self, PytestOptions};
        let path_buf = std::path::PathBuf::from(p);
        if !path_buf.exists() {
            anyhow::bail!("path does not exist: {}", path_buf.display());
        }
        let timeout_secs: u64 = sub
            .get_one::<String>("timeout")
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);
        let jobs: usize = sub
            .get_one::<String>("jobs")
            .and_then(|s| s.parse().ok())
            .unwrap_or(4)
            .max(1);
        let mut opts = PytestOptions::new(path_buf);
        opts.timeout_secs = timeout_secs;
        opts.jobs = jobs;
        let summary = pytest_runner::run(&opts);
        if !summary.success() {
            std::process::exit(1);
        }
        return Ok(());
    }

    if regen {
        let dir = sub
            .get_one::<String>("dir")
            .map(|s| s.as_str())
            .unwrap_or(mamba::conformance::FIXTURES_ROOT);
        let status = std::process::Command::new("python3")
            .args(["tests/regen_golden.py", dir])
            .status()
            .map_err(|e| anyhow::anyhow!("failed to invoke python3: {e}"))?;
        if !status.success() {
            anyhow::bail!("regen_golden.py exited with {status}");
        }
        return Ok(());
    }

    if conformance {
        let dir = sub
            .get_one::<String>("dir")
            .map(|s| std::path::PathBuf::from(s))
            .unwrap_or_else(|| std::path::PathBuf::from(mamba::conformance::FIXTURES_ROOT));
        let category = sub.get_one::<String>("category").cloned();

        let opts = ConformanceOptions {
            conformance_dir: dir,
            category,
            ..Default::default()
        };
        println!("Running Mamba conformance suite...");
        if let Some(cat) = &opts.category {
            println!("  category filter: {cat}");
        }

        let (_, summary) = run_conformance_suite(&opts);
        println!();
        println!(
            "Results: {} total, {} passed, {} failed, {} xfailed, {} errors",
            summary.total, summary.passed, summary.failed, summary.xfailed, summary.errors
        );
        if summary.failed > 0 || summary.errors > 0 {
            std::process::exit(1);
        }
        return Ok(());
    }

    println!("Use 'mamba test --help' for usage information");
    Ok(())
}
