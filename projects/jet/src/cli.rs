// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-src.md#schema
// CODEGEN-BEGIN
//! Jet CLI commands
//!
//! Commands for the Jet build tool: package management, dev server, and bundling.
//!
//! # Examples
//! ```bash
//! cclab jet install                   # Install dependencies
//! cclab jet add react                 # Add a dependency
//! cclab jet add react --dev           # Add as dev dependency
//! cclab jet remove react              # Remove a dependency
//! cclab jet dev -p 7099               # Start dev server on port 7099
//! cclab jet build                     # Build for production
//! cclab jet build --watch             # Build with watch mode
//! ```

use anyhow::{anyhow, Context, Result};
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// Build the jet CLI command tree
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn command() -> Command {
    Command::new("jet")
        .about("Jet - Rust-native build tool and package manager")
        .subcommand(
            Command::new("init")
                .about("Initialize a new project")
                .arg(Arg::new("name").help("Project name")),
        )
        .subcommand(
            Command::new("install")
                .about("Install dependencies from package.json")
                .arg(
                    Arg::new("packages")
                        .help("Specific packages to install")
                        .num_args(0..),
                )
                .arg(
                    Arg::new("frozen-lockfile")
                        .long("frozen-lockfile")
                        .conflicts_with("no-frozen-lockfile")
                        .action(ArgAction::SetTrue)
                        .help("Fail if lockfile drift detected (auto in CI)"),
                )
                .arg(
                    Arg::new("no-frozen-lockfile")
                        .long("no-frozen-lockfile")
                        .action(ArgAction::SetTrue)
                        .help("Disable CI auto-frozen lockfile for explicit bootstrap installs"),
                )
                .arg(
                    Arg::new("no-cache")
                        .long("no-cache")
                        .action(ArgAction::SetTrue)
                        .help(
                            "Skip disk metadata cache; always fetch fresh \
                             data from the registry (~/.cache/jet/metadata/)",
                        ),
                )
                .arg(
                    Arg::new("no-install")
                        .long("no-install")
                        .action(ArgAction::SetTrue)
                        .help(
                            "Resolve dependency graph and update jet-lock.yaml \
                             without downloading or extracting tarballs",
                        ),
                )
                .arg(
                    Arg::new("no-prebundle")
                        .long("no-prebundle")
                        .action(ArgAction::SetTrue)
                        .help("Skip dev-server prebundle after dependency installation"),
                )
                .arg(
                    Arg::new("nx")
                        .long("nx")
                        .action(ArgAction::SetTrue)
                        .help("Force Nx mode (override auto-detection)"),
                ),
        )
        .subcommand(
            Command::new("add")
                .about("Add one or more dependencies")
                .arg(
                    Arg::new("packages")
                        .required(true)
                        .num_args(1..)
                        .help(
                            "Package(s) to add (e.g. react, @mui/material, \
                             react@^18). Multiple packages may be added in \
                             one invocation.",
                        ),
                )
                .arg(
                    Arg::new("dev")
                        .short('D')
                        .long("dev")
                        .action(ArgAction::SetTrue)
                        .help("Add as dev dependency"),
                ),
        )
        .subcommand(
            Command::new("remove")
                .about("Remove a dependency")
                .arg(
                    Arg::new("package")
                        .required(true)
                        .help("Package to remove"),
                ),
        )
        .subcommand(
            Command::new("update")
                .about("Update dependencies")
                .arg(Arg::new("package").help("Specific package to update"))
                .arg(
                    Arg::new("latest")
                        .long("latest")
                        .action(ArgAction::SetTrue)
                        .help("Ignore semver range, update to absolute latest"),
                ),
        )
        .subcommand(
            Command::new("audit")
                .about("Check for known security vulnerabilities"),
        )
        .subcommand(
            Command::new("patch")
                .about("Create editable copy of installed package")
                .arg(
                    Arg::new("package")
                        .required(true)
                        .help("Package to patch"),
                ),
        )
        .subcommand(
            Command::new("patch-commit")
                .about("Generate .patch file from modified package")
                .arg(
                    Arg::new("package")
                        .required(true)
                        .help("Package to commit patch for"),
                ),
        )
        .subcommand(
            Command::new("publish")
                .about("Publish package to npm registry")
                .arg(
                    Arg::new("tag")
                        .long("tag")
                        .default_value("latest")
                        .help("Distribution tag"),
                )
                .arg(
                    Arg::new("access")
                        .long("access")
                        .help("Package access level (public/restricted)"),
                ),
        )
        .subcommand(Command::new("pack").about("Create tarball without publishing"))
        .subcommand(
            Command::new("store")
                .about("Store management commands")
                .subcommand(Command::new("prune").about("Remove unreferenced packages from global store")),
        )
        .subcommand(
            Command::new("dev")
                .about("Start development server with HMR")
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .help("Port to run on (default: from jet.toml or 3000)"),
                )
                .arg(
                    Arg::new("host")
                        .long("host")
                        .default_value("127.0.0.1")
                        .help("Host to bind to"),
                )
                .arg(
                    Arg::new("proxy")
                        .long("proxy")
                        .value_name("PATH=URL")
                        .action(ArgAction::Append)
                        .conflicts_with("wasm")
                        .help(
                            "Add a dev-server proxy rule, e.g. \
                             --proxy /api=http://localhost:3200. \
                             Repeatable; overrides matching [dev.proxy] config entries.",
                        ),
                )
                .arg(
                    // jet dev --wasm — one-command TSX→WASM dev loop.
                    // Runs wasm_build::build, serves dist/ via axum,
                    // watches src/ + jet.toml and rebuilds on
                    // change. Completely separate from the JS bundle
                    // dev server below.
                    Arg::new("wasm")
                        .long("wasm")
                        .action(ArgAction::SetTrue)
                        .help(
                            "WASM mode — build via [wasm] section of \
                             jet.toml, serve dist/, and rebuild \
                             on src/**/*.tsx change.",
                        ),
                )
                .arg(
                    // --debug: dev-profile wasm-pack build (keeps
                    // DWARF for Rust-source stepping) + turns on the
                    // jet-wasm `debug` feature so `window.__jet_debug`
                    // is live for `jet browser` to attach to.
                    Arg::new("debug")
                        .long("debug")
                        .action(ArgAction::SetTrue)
                        .help(
                            "Build the WASM app with debug info + \
                             runtime inspection hooks (window.__jet_debug). \
                             Only meaningful with --wasm.",
                        ),
                )
                .subcommand(
                    Command::new("shutdown")
                        .about("Request a running Jet dev server to shut down by host and port")
                        .arg(
                            Arg::new("port")
                                .short('p')
                                .long("port")
                                .required(true)
                                .help("Port of the running `jet dev` server"),
                        )
                        .arg(
                            Arg::new("host")
                                .long("host")
                                .default_value("127.0.0.1")
                                .help("Host of the running `jet dev` server"),
                        ),
                ),
        )
        .subcommand(serve_command())
        .subcommand(
            Command::new("build")
                .about("Build for production")
                .arg(
                    Arg::new("nx")
                        .long("nx")
                        .action(ArgAction::SetTrue)
                        .help(
                            "Force Nx mode: build all projects via the Nx \
                             project graph in topological order",
                        ),
                )
                .arg(
                    Arg::new("project")
                        .long("project")
                        .short('p')
                        .help(
                            "Nx mode only: build a single named project \
                             (and its dependencies)",
                        ),
                )
                .arg(
                    Arg::new("watch")
                        .short('w')
                        .long("watch")
                        .action(ArgAction::SetTrue)
                        .help("Watch mode"),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .default_value("dist")
                        .help("Output directory"),
                )
                .arg(
                    Arg::new("empty-out-dir")
                        .long("empty-out-dir")
                        .alias("clean")
                        .action(ArgAction::SetTrue)
                        .help(
                            "Empty the output directory before writing the new bundle. \
                             Safe by default: refuses filesystem root, project root, cwd, \
                             empty path, and unsafe symlink traversal. Targets outside the \
                             project root require --force.",
                        ),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .action(ArgAction::SetTrue)
                        .help(
                            "Allow --empty-out-dir to target a directory outside the \
                             project root. Required when -o points outside the repo \
                             (e.g. publishing into a sibling project's static dir).",
                        ),
                )
                .arg(
                    Arg::new("minify")
                        .long("minify")
                        .action(ArgAction::SetTrue)
                        .help(
                            "Enable minification (default; whitespace removal, comment stripping)",
                        ),
                )
                .arg(
                    Arg::new("no-minify")
                        .long("no-minify")
                        .action(ArgAction::SetTrue)
                        .help("Disable default minification"),
                )
                .arg(
                    Arg::new("sourcemap")
                        .long("sourcemap")
                        .default_value("external")
                        .help("Source map mode: external, inline, hidden, none"),
                )
                .arg(
                    Arg::new("splitting")
                        .long("splitting")
                        .action(ArgAction::SetTrue)
                        .help("Enable code splitting at dynamic import() boundaries"),
                )
                .arg(
                    Arg::new("define")
                        .long("define")
                        .action(ArgAction::Append)
                        .help("Define compile-time constants (e.g. --define KEY=VALUE)"),
                )
                .arg(
                    Arg::new("drop")
                        .long("drop")
                        .action(ArgAction::Append)
                        .help("Statements to drop: console, debugger"),
                )
                .arg(
                    Arg::new("target")
                        .long("target")
                        .value_parser(["web", "desktop", "tui"])
                        .help("Build target: web, desktop, or tui"),
                )
                .arg(
                    // jet build --wasm — compile the app's TSX entry through
                    // ts_to_rust → rustc (wasm32) → dist/app.wasm + boot loader
                    // + index.html, producing an openable static site that
                    // renders the root component on a WebGPU canvas surface.
                    Arg::new("wasm")
                        .long("wasm")
                        .action(ArgAction::SetTrue)
                        .help(
                            "WASM mode — compile TSX → Rust → wasm32 and emit \
                             dist/app.wasm + dist/boot.js + dist/index.html. \
                             Reads [wasm] section of jet.toml for \
                             entry + root_component + root_props.",
                        ),
                )
                .arg(
                    // --debug: dev-profile wasm-pack build (DWARF
                    // retained) + jet-wasm `debug` feature on.
                    Arg::new("debug")
                        .long("debug")
                        .action(ArgAction::SetTrue)
                        .help(
                            "Build the WASM app with debug info + \
                             runtime inspection hooks (window.__jet_debug). \
                             Only meaningful with --wasm.",
                        ),
                )
                .arg(
                    // jet build --lib — library mode: externalize npm deps,
                    // emit ESM (+ optional CJS), discover multiple entries
                    // from package.json `exports`. Reads [lib] section of
                    // jet.toml when present.
                    Arg::new("lib")
                        .long("lib")
                        .action(ArgAction::SetTrue)
                        .help(
                            "Library mode — build a publishable library: \
                             externalize npm dependencies/peerDependencies as \
                             real `import`/`require` statements, emit ESM (and \
                             CJS with --format), discover entries from \
                             package.json `exports`. Reads [lib] of jet.toml.",
                        ),
                )
                .arg(
                    Arg::new("format")
                        .long("format")
                        .help(
                            "Library output formats (comma-separated): esm, cjs. \
                             Only meaningful with --lib. Default: esm.",
                        ),
                ),
        )
        .subcommand(Command::new("check").about("Type check TypeScript files"))
        .subcommand(
            Command::new("config")
                .about("Inspect and lint jet.toml")
                .subcommand(
                    Command::new("lint")
                        .about(
                            "Validate jet.toml against the typed \
                             schema. Reports unknown keys (with did-you-mean \
                             suggestions), invalid values, and deprecated \
                             keys (with migration hints).",
                        )
                        .arg(
                            Arg::new("path")
                                .long("path")
                                .help(
                                    "Project root directory containing \
                                     jet.toml (default: current dir)",
                                ),
                        )
                        .arg(
                            Arg::new("format")
                                .long("format")
                                .value_parser(["human", "json"])
                                .default_value("human")
                                .help("Output format"),
                        )
                        .arg(
                            Arg::new("strict-warn")
                                .long("strict-warn")
                                .action(ArgAction::SetTrue)
                                .help(
                                    "Promote warnings to errors. Use in CI \
                                     to block deprecated keys from \
                                     accumulating silently.",
                                ),
                        ),
                )
                .subcommand(
                    Command::new("schema")
                        .about(
                            "Generate the JSON Schema for jet.toml. \
                             Default mode prints to stdout; --write updates \
                             schemas/jet.schema.json; --check exits \
                             non-zero on drift (CI gate).",
                        )
                        .arg(
                            Arg::new("path")
                                .long("path")
                                .help(
                                    "Workspace root directory the artifact is \
                                     written under (default: current dir).",
                                ),
                        )
                        .arg(
                            Arg::new("write")
                                .long("write")
                                .action(ArgAction::SetTrue)
                                .conflicts_with("check")
                                .help(
                                    "Write the schema to \
                                     <root>/schemas/jet.schema.json.",
                                ),
                        )
                        .arg(
                            Arg::new("check")
                                .long("check")
                                .action(ArgAction::SetTrue)
                                .help(
                                    "Compare against the on-disk artifact. \
                                     Exits 0 = match, 1 = drift, 2 = missing.",
                                ),
                        ),
                ),
        )
        .subcommand(
            Command::new("run")
                .about("Run a script, file, or task (no args = list scripts)")
                .arg(
                    Arg::new("target")
                        .help("Script name, file path, or task name"),
                )
                .arg(
                    Arg::new("args")
                        .num_args(0..)
                        .trailing_var_arg(true)
                        .help("Arguments to pass to the script"),
                )
                .arg(
                    Arg::new("watch")
                        .short('w')
                        .long("watch")
                        .action(ArgAction::SetTrue)
                        .help("Watch mode (re-run on file changes)"),
                )
                .arg(
                    Arg::new("filter")
                        .long("filter")
                        .help("Filter packages (task runner mode)"),
                )
                .arg(
                    Arg::new("dry")
                        .long("dry")
                        .action(ArgAction::SetTrue)
                        .help("Dry run: show what would execute"),
                ),
        )
        .subcommand(
            Command::new("exec")
                .about("Run a command with node_modules/.bin on PATH")
                .arg(
                    Arg::new("cmd")
                        .required(true)
                        .help("Command to execute"),
                )
                .arg(
                    Arg::new("args")
                        .num_args(0..)
                        .trailing_var_arg(true)
                        .help("Arguments"),
                ),
        )
        .subcommand(
            // @spec enhancement-html-reporter-for-native-test-runner-spec#R6
            Command::new("report")
                .about("Commands for managing HTML test reports")
                .subcommand(
                    Command::new("view")
                        .about("Open a report directory in the system default browser")
                        .arg(
                            Arg::new("dir")
                                .required(true)
                                .help("Path to a report directory containing index.html"),
                        )
                        .arg(
                            Arg::new("serve")
                                .long("serve")
                                .action(ArgAction::SetTrue)
                                .help("Serve the report on a local HTTP port instead of opening file:// URL"),
                        ),
                )
                .subcommand(
                    Command::new("merge")
                        .about("Merge N per-shard report directories into a single unified report")
                        .arg(
                            Arg::new("input")
                                .long("input")
                                .num_args(1..)
                                .required(true)
                                .help("Space-separated list of shard report directories"),
                        )
                        .arg(
                            Arg::new("output")
                                .long("output")
                                .required(true)
                                .help("Destination directory for the merged report"),
                        ),
                ),
        )
        .subcommand(
            Command::new("test")
                .about("Run frontend unit, component, and integration-style tests via the native jet test runner")
                .arg(
                    Arg::new("files")
                        .num_args(0..)
                        .help("Optional spec file(s) to run (otherwise auto-discover)"),
                )
                .arg(
                    Arg::new("grep")
                        .long("grep")
                        .short('g')
                        .help("Regex filter on full test name (suite > test)"),
                )
                .arg(
                    Arg::new("timeout")
                        .long("timeout")
                        .value_parser(clap::value_parser!(u64))
                        .help("Per-test timeout in ms (default 30000)"),
                )
                .arg(
                    // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
                    Arg::new("reporter")
                        .long("reporter")
                        .help("Comma-separated reporters: term, list, json, html (default: term,json)"),
                )
                .arg(
                    // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
                    Arg::new("report-dir")
                        .long("report-dir")
                        .help("Output directory for HTML report (default: test-results/report/)"),
                )
                .arg(
                    Arg::new("update-snapshots")
                        .long("update-snapshots")
                        .short('u')
                        .action(ArgAction::SetTrue)
                        .help("Overwrite snapshot files on mismatch"),
                )
                .arg(
                    // @spec enhancement-playwright-compat-shim-for-migration-window-spec#R1
                    Arg::new("playwright")
                        .long("playwright")
                        .action(ArgAction::SetTrue)
                        .help(
                            "[deprecated] Escape hatch: delegate to `npx playwright test` \
                             instead of the native runner. \
                             Deprecated in this minor release; removed in the second \
                             subsequent minor release. \
                             See projects/jet/docs/migration-from-playwright.md for the \
                             migration guide. \
                             Incompatible with --reporter, --trace, --workers, --shard, \
                             --report-dir.",
                        ),
                )
                .arg(
                    // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R1
                    Arg::new("trace")
                        .long("trace")
                        .value_name("MODE")
                        .default_value("off")
                        .help(
                            "Enable trace capture. \
                             on: capture and write trace for every test. \
                             retain-on-failure: capture for all tests but only \
                             write to disk for failed tests. \
                             off: no trace capture (zero overhead).",
                        ),
                )
                .arg(
                    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R1
                    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R9
                    Arg::new("workers")
                        .long("workers")
                        .value_name("N")
                        .value_parser(clap::value_parser!(usize))
                        .help(
                            "--workers=<N>  Number of concurrent workers \
                             [default: logical CPU count]. \
                             Use --workers=1 to force serial (debug) execution.",
                        ),
                )
                .arg(
                    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R3
                    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R9
                    Arg::new("shard")
                        .long("shard")
                        .value_name("i/N")
                        .help(
                            "--shard=<i/N>  Run only the i-th of N shards \
                             [e.g. --shard=2/4]. \
                             Distribute CI by running each shard independently.",
                        ),
                )
                .arg(
                    // @spec .aw/tech-design/projects/jet/logic/watch-mode.md#W1
                    Arg::new("watch")
                        .long("watch")
                        .action(ArgAction::SetTrue)
                        .help(
                            "Re-run the suite whenever any file under the project \
                             root changes. Ctrl-C to exit.",
                        ),
                )
                .arg(
                    // @spec .aw/tech-design/projects/jet/logic/inspector.md#I1
                    Arg::new("debug")
                        .long("debug")
                        .action(ArgAction::SetTrue)
                        .help(
                            "Debug mode: launch Chromium headed with a \
                             single worker so actions are visible. Combine \
                             with `page.pause()` in a spec to pause at a \
                             breakpoint (MVP: 30-minute hold).",
                        ),
                )
                .arg(
                    // @spec enhancement-define-component-and-browser-like-test-environment-boundary
                    Arg::new("env")
                        .long("env")
                        .value_name("KIND")
                        .help(
                            "Test environment: node (default), dom, component. \
                             `node` covers unit and pure-logic frontend tests. \
                             `dom` and `component` are reserved for browser-like \
                             environments and currently fail with a clear error \
                             pointing at `jet e2e` for product-flow cases.",
                        ),
                )
                .arg(
                    // @spec #2709
                    Arg::new("list-resolved")
                        .long("list-resolved")
                        .action(ArgAction::SetTrue)
                        .help(
                            "Resolve config + discovery inputs and print the manifest \
                             to stdout as JSON. Exits 0 on success, 2 on invalid config \
                             (with a single-line `discovery_config_error` JSON on stderr).",
                        ),
                ),
        )
        .subcommand(
            // @spec .aw/tech-design/projects/jet/specs/2385.md#cli
            Command::new("e2e")
                .about("Run or review product-flow E2E cases")
                .long_about(
                    "Run or review product-flow E2E cases. \
                     Use `jet test` for frontend unit, component, and integration-style tests.",
                )
                .allow_external_subcommands(true)
                .subcommand(
                    Command::new("run")
                        .about("Run product-flow E2E cases in Playwright-like automation mode")
                        .long_about(
                            "Run product-flow E2E cases in Playwright-like automation mode \
                             for CI, agents, and release gates. \
                             This mode does not launch the human desktop review app; \
                             evidence is written as JSON and JSONL.",
                        )
                        .arg(
                            Arg::new("cases")
                                .num_args(0..)
                                .help("Optional E2E case file(s) to run"),
                        )
                        .arg(
                            Arg::new("grep")
                                .long("grep")
                                .short('g')
                                .help("Regex filter on full case name"),
                        )
                        .arg(
                            Arg::new("timeout")
                                .long("timeout")
                                .value_parser(clap::value_parser!(u64))
                                .help("Per-case timeout in ms"),
                        )
                        .arg(
                            Arg::new("workers")
                                .long("workers")
                                .value_parser(clap::value_parser!(usize))
                                .help("Number of concurrent E2E workers"),
                        )
                        .arg(
                            Arg::new("serve")
                                .long("serve")
                                .value_name("MODE")
                                .default_value("off")
                                .value_parser(["off", "dev", "prod"])
                                .help(
                                    "Start an agent-first Jet server for this run: off, dev, prod. \
                                     Cannot be combined with --base-url.",
                                ),
                        )
                        .arg(
                            Arg::new("base-url")
                                .long("base-url")
                                .value_name("URL")
                                .help(
                                    "External AUT base URL for relative page.goto paths. \
                                     Cannot be combined with --serve dev/prod.",
                                ),
                        )
                        .arg(
                            Arg::new("trace")
                                .long("trace")
                                .value_name("MODE")
                                .default_value("retain-on-failure")
                                .help("Trace capture mode: off, on, retain-on-failure"),
                        )
                        .arg(
                            Arg::new("evidence-dir")
                                .long("evidence-dir")
                                .help("Directory for agent-readable E2E evidence"),
                        )
                        .arg(
                            Arg::new("json")
                                .long("json")
                                .action(ArgAction::SetTrue)
                                .help("Print the evidence bundle JSON to stdout"),
                        ),
                )
                .subcommand(
                    Command::new("open")
                        .about("Open the Cypress-like human runner for product-flow E2E cases")
                        .long_about(
                            "Open the Cypress-like human runner for product-flow E2E cases. \
                             Cypress-like describes the human review UX only; \
                             Jet does not use the Cypress runtime or Cypress Cloud. \
                             The runner launches a desktop-style review shell; \
                             the AUT runs in a separate visible controlled Jet Browser target, \
                             executes cases serially, and shows case timelines, pause/next/replay controls, \
                             selector context, assertion detail, screenshots, console, and network panels.",
                        )
                        .arg(
                            Arg::new("cases")
                                .num_args(0..)
                                .help("Optional E2E case file(s) to run and review"),
                        )
                        .arg(
                            Arg::new("grep")
                                .long("grep")
                                .short('g')
                                .help("Regex filter on full case name"),
                        )
                        .arg(
                            Arg::new("timeout")
                                .long("timeout")
                                .value_parser(clap::value_parser!(u64))
                                .help("Per-case timeout in ms"),
                        )
                        .arg(
                            Arg::new("evidence-dir")
                                .long("evidence-dir")
                                .help("Directory for review app state and evidence"),
                        )
                        .arg(
                            Arg::new("dry-run")
                                .long("dry-run")
                                .action(ArgAction::SetTrue)
                                .help("Launch or export the runner shell without running cases"),
                        )
                        .arg(
                            Arg::new("no-open")
                                .long("no-open")
                                .action(ArgAction::SetTrue)
                                .help("Export the runner shell/evidence without launching the desktop-style review shell"),
                        ),
                ),
        )
        .subcommand(
            // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R5
            Command::new("trace")
                .about("Work with jet trace archive files")
                .subcommand(
                    Command::new("view")
                        .about("Open a local HTTP trace viewer for a jet trace archive")
                        .arg(
                            Arg::new("file")
                                .required(true)
                                .help("Path to a trace zip archive"),
                        )
                        .arg(
                            Arg::new("port")
                                .long("port")
                                .short('p')
                                .value_parser(clap::value_parser!(u16))
                                .default_value("0")
                                .help("Port to bind (0 = free port)"),
                        )
                        .arg(
                            Arg::new("no-open")
                                .long("no-open")
                                .action(ArgAction::SetTrue)
                                .help("Skip automatic browser open; print URL only"),
                        ),
                )
                .subcommand(
                    Command::new("show")
                        .about("Print manifest summary for a trace archive")
                        .arg(
                            Arg::new("file")
                                .required(true)
                                .help("Path to a trace zip archive"),
                        ),
                )
                .subcommand(
                    Command::new("extract")
                        .about("Extract trace archive to a directory")
                        .arg(
                            Arg::new("file")
                                .required(true)
                                .help("Path to a trace zip archive"),
                        )
                        .arg(
                            Arg::new("dir")
                                .required(true)
                                .help("Output directory"),
                        ),
                ),
        )
        .subcommand(
            Command::new("jtx")
                .about("Download and execute a package (like npx)")
                .arg(
                    Arg::new("package")
                        .required(true)
                        .help("Package to execute"),
                )
                .arg(
                    Arg::new("args")
                        .num_args(0..)
                        .trailing_var_arg(true)
                        .help("Arguments"),
                ),
        )
        // @spec .aw/changes/enhancement-cclab-jet-browser-install-cli-pinned-chromium-down/specs/enhancement-cclab-jet-browser-install-cli-pinned-chromium-down-spec.md#R1
        // @spec .aw/changes/enhancement-cclab-jet-browser-install-cli-pinned-chromium-down/specs/enhancement-cclab-jet-browser-install-cli-pinned-chromium-down-spec.md#R6
        .subcommand(
            Command::new("browser")
                .about("Browser Bridge management + jet-wasm debugging commands")
                .subcommand(
                    Command::new("install")
                        .about("Download and cache a pinned browser binary")
                        .arg(
                            Arg::new("target")
                                .help("Browser to install (only 'chromium' is supported)")
                                .default_value("chromium"),
                        )
                        .arg(
                            Arg::new("revision")
                                .long("revision")
                                .short('r')
                                .default_value(crate::browser::DEFAULT_CHROMIUM_REVISION)
                                .help("Chromium snapshot revision number"),
                        )
                        .arg(
                            Arg::new("cache-dir")
                                .long("cache-dir")
                                .short('d')
                                .help("Root directory for cached browser installs (default: ~/.jet/browsers)"),
                        ),
                )
                .subcommand(
                    Command::new("launch")
                        .about("Launch Chromium, navigate to URL, and hold a foreground CDP session for other `jet browser` commands to reuse")
                        .arg(Arg::new("url").required(true).help("URL to open (typically the `jet dev --wasm --debug` address)")),
                )
                .subcommand(
                    Command::new("shutdown")
                        .about("Request the active `jet browser launch` session to close"),
                )
                .subcommand(
                    Command::new("tree")
                        .about("Print the element / layout / fiber tree from the attached session")
                        .arg(
                            Arg::new("which")
                                .default_value("element")
                                .help("element | layout | fiber"),
                        ),
                )
                .subcommand(
                    Command::new("pick")
                        .about("Arm a one-shot click listener; next click on the canvas prints the hit-tested node")
                        .arg(
                            Arg::new("timeout")
                                .long("timeout")
                                .default_value("30")
                                .help("Seconds to wait for the click"),
                        ),
                )
                .subcommand(
                    Command::new("hooks")
                        .about("Print hook values for a fiber")
                        .arg(Arg::new("fiber-id").required(true)),
                )
                .subcommand(
                    Command::new("highlight")
                        .about("Overlay a red rect on the given layout-node index (or clear when --clear)")
                        .arg(Arg::new("index").required(false))
                        .arg(
                            Arg::new("clear")
                                .long("clear")
                                .action(ArgAction::SetTrue)
                                .help("Clear the current highlight"),
                        ),
                )
                .subcommand(
                    Command::new("frame")
                        .about("Dump the last-frame paint ops"),
                )
                .subcommand(
                    Command::new("perf")
                        .about("Print a compact runtime performance snapshot without full capture"),
                )
                .subcommand(
                    Command::new("mouse")
                        .about("Dispatch one CDP mouse event into the attached Jet browser session")
                        .arg(
                            Arg::new("type")
                                .required(true)
                                .value_parser(["mouseMoved", "mousePressed", "mouseReleased"])
                                .help("CDP mouse event type"),
                        )
                        .arg(Arg::new("x").required(true).help("Viewport CSS-pixel x coordinate"))
                        .arg(Arg::new("y").required(true).help("Viewport CSS-pixel y coordinate"))
                        .arg(
                            Arg::new("button")
                                .long("button")
                                .value_parser(["left", "right", "middle", "none"])
                                .help("Mouse button for pressed/released events"),
                        )
                        .arg(
                            Arg::new("buttons")
                                .long("buttons")
                                .help("CDP buttons bitfield, e.g. 1 while dragging with the left button"),
                        )
                        .arg(
                            Arg::new("click-count")
                                .long("click-count")
                                .help("CDP clickCount for pressed/released events"),
                        ),
                )
                .subcommand(
                    Command::new("drag")
                        .about("Drag between two viewport coordinates using CDP mouse events")
                        .arg(Arg::new("from-x").required(true).help("Start x coordinate"))
                        .arg(Arg::new("from-y").required(true).help("Start y coordinate"))
                        .arg(Arg::new("to-x").required(true).help("End x coordinate"))
                        .arg(Arg::new("to-y").required(true).help("End y coordinate"))
                        .arg(
                            Arg::new("steps")
                                .long("steps")
                                .default_value("8")
                                .help("Interpolated mouseMoved events between press and release"),
                        ),
                )
                .subcommand(
                    Command::new("wheel")
                        .about("Dispatch one CDP mouse wheel event into the attached Jet browser session")
                        .arg(Arg::new("x").required(true).help("Viewport CSS-pixel x coordinate"))
                        .arg(Arg::new("y").required(true).help("Viewport CSS-pixel y coordinate"))
                        .arg(
                            Arg::new("delta-x")
                                .long("delta-x")
                                .default_value("0")
                                .help("Horizontal wheel delta in CSS pixels"),
                        )
                        .arg(
                            Arg::new("delta-y")
                                .long("delta-y")
                                .default_value("0")
                                .help("Vertical wheel delta in CSS pixels"),
                        ),
                )
                .subcommand(
                    Command::new("key")
                        .about("Press one key in the attached Jet browser session using CDP key events")
                        .arg(Arg::new("key").required(true).help("Key value such as c, Enter, or ArrowDown"))
                        .arg(
                            Arg::new("ctrl")
                                .long("ctrl")
                                .action(ArgAction::SetTrue)
                                .help("Hold Control while pressing the key"),
                        )
                        .arg(
                            Arg::new("meta")
                                .long("meta")
                                .action(ArgAction::SetTrue)
                                .help("Hold Meta/Command while pressing the key"),
                        )
                        .arg(
                            Arg::new("shift")
                                .long("shift")
                                .action(ArgAction::SetTrue)
                                .help("Hold Shift while pressing the key"),
                        )
                        .arg(
                            Arg::new("alt")
                                .long("alt")
                                .action(ArgAction::SetTrue)
                                .help("Hold Alt while pressing the key"),
                        ),
                )
                .subcommand(
                    Command::new("capture")
                        .about("Print a parity-ready JSON observation bundle from an attached wasm or DOM browser session")
                        .arg(
                            Arg::new("surface")
                                .long("surface")
                                .default_value("wasm")
                                .value_parser(["wasm", "dom"])
                                .help("Observation surface to capture: wasm debug bridge or live DOM"),
                        )
                        .arg(
                            Arg::new("root-selector")
                                .long("root-selector")
                                .default_value("body")
                                .help("DOM root selector for --surface dom"),
                        )
                        .arg(
                            Arg::new("out")
                                .long("out")
                                .short('o')
                                .alias("output")
                                .help("Output file (writes to stdout when omitted)"),
                        )
                        .arg(
                            Arg::new("pretty")
                                .long("pretty")
                                .action(ArgAction::SetTrue)
                                .help("Pretty-print JSON output"),
                        )
                        .arg(
                            Arg::new("hook")
                                .long("hook")
                                .action(ArgAction::Append)
                                .num_args(1)
                                .help("Fiber id whose hook values should be included; repeats. Defaults to fibers with hooks."),
                        ),
                )
                .subcommand(
                    Command::new("screenshot")
                        .about("Save a PNG of the current canvas")
                        .arg(
                            Arg::new("out")
                                .long("out")
                                .short('o')
                                .help("Output file (writes to stdout when omitted)"),
                        ),
                )
                .subcommand(
                    Command::new("eval")
                        .about("Runtime.evaluate escape hatch")
                        .arg(Arg::new("expr").required(true)),
                )
                .subcommand(
                    Command::new("debug")
                        .about("Open a foreground Browser Bridge session for human inspection")
                        .arg(Arg::new("url").required(true)),
                )
                .subcommand(
                    Command::new("tsx")
                        .about("Print TSX source locations for each component (reads dist/tsx-source-map.json — no live browser needed)")
                        .arg(
                            Arg::new("filter")
                                .help("Only print components whose name contains this substring"),
                        ),
                ),
        )
        .subcommand(browser_bridge_command())
}

fn serve_command() -> Command {
    Command::new("serve")
        .about("Start an agent-first detached Jet server session")
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .help("Port to run on (default: from jet.toml or 3000; 0 asks the OS)"),
        )
        .arg(
            Arg::new("host")
                .long("host")
                .default_value("127.0.0.1")
                .help("Host to bind to"),
        )
        .arg(
            Arg::new("prod")
                .long("prod")
                .action(ArgAction::SetTrue)
                .help("Serve production dist artifacts from ./dist"),
        )
        .arg(
            Arg::new("wasm")
                .long("wasm")
                .action(ArgAction::SetTrue)
                .help("Serve the FE-on-WASM target through the detached serve session"),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .action(ArgAction::SetTrue)
                .help("Build the WASM app with debug info and runtime inspection hooks"),
        )
        .subcommand(
            Command::new("shutdown")
                .about("Request the active detached Jet serve session to shut down")
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .help("Port to shut down; defaults to .jet/serve-session.json"),
                )
                .arg(
                    Arg::new("host")
                        .long("host")
                        .default_value("127.0.0.1")
                        .help("Host to shut down when --port is provided"),
                ),
        )
}

fn browser_bridge_command() -> Command {
    Command::new("bb")
        .about("Agent-first Browser Bridge commands")
        .subcommand(
            Command::new("launch")
                .about("Launch headless Chromium as a detached Browser Bridge session")
                .arg(
                    Arg::new("url")
                        .required(true)
                        .help("URL to open (typically the `jet serve --wasm` address)"),
                ),
        )
        .subcommand(
            Command::new("shutdown").about("Request the active Browser Bridge session to close"),
        )
        .subcommand(
            Command::new("tree")
                .about("Print the element / layout / fiber tree from the attached session")
                .arg(
                    Arg::new("which")
                        .default_value("element")
                        .help("element | layout | fiber"),
                ),
        )
        .subcommand(
            Command::new("pick")
                .about("Arm a one-shot click listener; next click on the canvas prints the hit-tested node")
                .arg(
                    Arg::new("timeout")
                        .long("timeout")
                        .default_value("30")
                        .help("Seconds to wait for the click"),
                ),
        )
        .subcommand(
            Command::new("hooks")
                .about("Print hook values for a fiber")
                .arg(Arg::new("fiber-id").required(true)),
        )
        .subcommand(
            Command::new("highlight")
                .about("Overlay a red rect on the given layout-node index (or clear when --clear)")
                .arg(Arg::new("index").required(false))
                .arg(
                    Arg::new("clear")
                        .long("clear")
                        .action(ArgAction::SetTrue)
                        .help("Clear the current highlight"),
                ),
        )
        .subcommand(Command::new("frame").about("Dump the last-frame paint ops"))
        .subcommand(
            Command::new("perf")
                .about("Print a compact runtime performance snapshot without full capture"),
        )
        .subcommand(
            Command::new("mouse")
                .about("Dispatch one CDP mouse event into the attached Browser Bridge session")
                .arg(
                    Arg::new("type")
                        .required(true)
                        .value_parser(["mouseMoved", "mousePressed", "mouseReleased"])
                        .help("CDP mouse event type"),
                )
                .arg(Arg::new("x").required(true).help("Viewport CSS-pixel x coordinate"))
                .arg(Arg::new("y").required(true).help("Viewport CSS-pixel y coordinate"))
                .arg(
                    Arg::new("button")
                        .long("button")
                        .value_parser(["left", "right", "middle", "none"])
                        .help("Mouse button for pressed/released events"),
                )
                .arg(
                    Arg::new("buttons")
                        .long("buttons")
                        .help("CDP buttons bitfield, e.g. 1 while dragging with the left button"),
                )
                .arg(
                    Arg::new("click-count")
                        .long("click-count")
                        .help("CDP clickCount for pressed/released events"),
                ),
        )
        .subcommand(
            Command::new("drag")
                .about("Drag between two viewport coordinates using CDP mouse events")
                .arg(Arg::new("from-x").required(true).help("Start x coordinate"))
                .arg(Arg::new("from-y").required(true).help("Start y coordinate"))
                .arg(Arg::new("to-x").required(true).help("End x coordinate"))
                .arg(Arg::new("to-y").required(true).help("End y coordinate"))
                .arg(
                    Arg::new("steps")
                        .long("steps")
                        .default_value("8")
                        .help("Interpolated mouseMoved events between press and release"),
                ),
        )
        .subcommand(
            Command::new("wheel")
                .about("Dispatch one CDP mouse wheel event into the attached Browser Bridge session")
                .arg(Arg::new("x").required(true).help("Viewport CSS-pixel x coordinate"))
                .arg(Arg::new("y").required(true).help("Viewport CSS-pixel y coordinate"))
                .arg(
                    Arg::new("delta-x")
                        .long("delta-x")
                        .default_value("0")
                        .help("Horizontal wheel delta in CSS pixels"),
                )
                .arg(
                    Arg::new("delta-y")
                        .long("delta-y")
                        .default_value("0")
                        .help("Vertical wheel delta in CSS pixels"),
                ),
        )
        .subcommand(
            Command::new("key")
                .about("Press one key in the attached Browser Bridge session using CDP key events")
                .arg(Arg::new("key").required(true).help("Key value such as c, Enter, or ArrowDown"))
                .arg(
                    Arg::new("ctrl")
                        .long("ctrl")
                        .action(ArgAction::SetTrue)
                        .help("Hold Control while pressing the key"),
                )
                .arg(
                    Arg::new("meta")
                        .long("meta")
                        .action(ArgAction::SetTrue)
                        .help("Hold Meta/Command while pressing the key"),
                )
                .arg(
                    Arg::new("shift")
                        .long("shift")
                        .action(ArgAction::SetTrue)
                        .help("Hold Shift while pressing the key"),
                )
                .arg(
                    Arg::new("alt")
                        .long("alt")
                        .action(ArgAction::SetTrue)
                        .help("Hold Alt while pressing the key"),
                ),
        )
        .subcommand(
            Command::new("capture")
                .about("Print a parity-ready JSON observation bundle from an attached wasm or DOM browser session")
                .arg(
                    Arg::new("surface")
                        .long("surface")
                        .default_value("wasm")
                        .value_parser(["wasm", "dom"])
                        .help("Observation surface to capture: wasm debug bridge or live DOM"),
                )
                .arg(
                    Arg::new("root-selector")
                        .long("root-selector")
                        .default_value("body")
                        .help("DOM root selector for --surface dom"),
                )
                .arg(
                    Arg::new("out")
                        .long("out")
                        .short('o')
                        .alias("output")
                        .help("Output file (writes to stdout when omitted)"),
                )
                .arg(
                    Arg::new("pretty")
                        .long("pretty")
                        .action(ArgAction::SetTrue)
                        .help("Pretty-print JSON output"),
                )
                .arg(
                    Arg::new("hook")
                        .long("hook")
                        .action(ArgAction::Append)
                        .num_args(1)
                        .help("Fiber id whose hook values should be included; repeats. Defaults to fibers with hooks."),
                ),
        )
        .subcommand(
            Command::new("screenshot")
                .about("Save a PNG of the current canvas")
                .arg(
                    Arg::new("out")
                        .long("out")
                        .short('o')
                        .help("Output file (writes to stdout when omitted)"),
                ),
        )
        .subcommand(
            Command::new("eval")
                .about("Runtime.evaluate escape hatch")
                .arg(Arg::new("expr").required(true)),
        )
        .subcommand(
            Command::new("mcp")
                .about("Serve the Browser Bridge as an MCP stdio server for agents"),
        )
        .subcommand(
            Command::new("snapshot")
                .about("Print a ref-annotated semantic snapshot of the live DOM; refs (e1, e2, …) feed click/fill/type/hover/select")
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("Print the full JSON payload instead of the snapshot text"),
                ),
        )
        .subcommand(
            Command::new("click")
                .about("Click an element by snapshot ref (e12) or selector (CSS, text=…, role=…)")
                .arg(Arg::new("target").required(true).help("Snapshot ref (e12) or selector"))
                .arg(
                    Arg::new("dblclick")
                        .long("dblclick")
                        .action(ArgAction::SetTrue)
                        .help("Double-click instead of single click"),
                ),
        )
        .subcommand(
            Command::new("fill")
                .about("Replace an input/textarea value (native setter + input/change events)")
                .arg(Arg::new("target").required(true).help("Snapshot ref (e12) or selector"))
                .arg(Arg::new("text").required(true).help("New value")),
        )
        .subcommand(
            Command::new("type")
                .about("Focus an element and type text through the real CDP input pipeline (appends; use fill to replace)")
                .arg(Arg::new("target").required(true).help("Snapshot ref (e12) or selector"))
                .arg(Arg::new("text").required(true).help("Text to type")),
        )
        .subcommand(
            Command::new("hover")
                .about("Hover an element (mouseenter + mousemove at its center)")
                .arg(Arg::new("target").required(true).help("Snapshot ref (e12) or selector")),
        )
        .subcommand(
            Command::new("select")
                .about("Choose a <select> option by value or label")
                .arg(Arg::new("target").required(true).help("Snapshot ref (e12) or selector"))
                .arg(Arg::new("option").required(true).help("Option value or label")),
        )
        .subcommand(
            Command::new("check")
                .about("Check a checkbox (idempotent)")
                .arg(Arg::new("target").required(true).help("Snapshot ref (e12) or selector")),
        )
        .subcommand(
            Command::new("uncheck")
                .about("Uncheck a checkbox (idempotent)")
                .arg(Arg::new("target").required(true).help("Snapshot ref (e12) or selector")),
        )
        .subcommand(
            Command::new("goto")
                .about("Navigate the attached session to a URL and wait for load")
                .arg(Arg::new("url").required(true)),
        )
        .subcommand(Command::new("back").about("Go back one entry in session history"))
        .subcommand(Command::new("forward").about("Go forward one entry in session history"))
        .subcommand(Command::new("reload").about("Reload the current document"))
        .subcommand(
            Command::new("resize")
                .about("Resize the viewport (CDP device-metrics override)")
                .arg(Arg::new("width").required(true).help("Viewport width in CSS pixels"))
                .arg(Arg::new("height").required(true).help("Viewport height in CSS pixels")),
        )
        .subcommand(
            Command::new("wait")
                .about("Wait for a selector to attach, text to appear, or a fixed delay")
                .arg(
                    Arg::new("selector")
                        .long("selector")
                        .help("Selector (CSS, text=…, role=…) that must attach"),
                )
                .arg(Arg::new("text").long("text").help("Visible text that must appear"))
                .arg(
                    Arg::new("ms")
                        .long("ms")
                        .help("Sleep this many milliseconds instead of polling"),
                )
                .arg(
                    Arg::new("timeout")
                        .long("timeout")
                        .default_value("10000")
                        .help("Polling budget in milliseconds"),
                ),
        )
        .subcommand(
            Command::new("console")
                .about("Print console messages, page errors, and unhandled rejections captured since launch")
                .arg(
                    Arg::new("level")
                        .long("level")
                        .value_parser(["log", "info", "warn", "error", "debug"])
                        .help("Only entries of this level"),
                )
                .arg(
                    Arg::new("limit")
                        .long("limit")
                        .default_value("100")
                        .help("Most-recent entry cap"),
                )
                .arg(
                    Arg::new("clear")
                        .long("clear")
                        .action(ArgAction::SetTrue)
                        .help("Drain the buffer after reading"),
                ),
        )
        .subcommand(
            Command::new("requests")
                .about("Print fetch/XHR activity captured since launch")
                .arg(
                    Arg::new("limit")
                        .long("limit")
                        .default_value("100")
                        .help("Most-recent entry cap"),
                )
                .arg(
                    Arg::new("clear")
                        .long("clear")
                        .action(ArgAction::SetTrue)
                        .help("Drain the buffer after reading"),
                ),
        )
        .subcommand(
            Command::new("debug")
                .about("Open a foreground Browser Bridge session for human inspection")
                .arg(Arg::new("url").required(true)),
        )
        .subcommand(
            Command::new("tsx")
                .about("Print TSX source locations for each component (reads dist/tsx-source-map.json — no live browser needed)")
                .arg(
                    Arg::new("filter")
                        .help("Only print components whose name contains this substring"),
                ),
        )
}

/// Execute a jet CLI command
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn execute(matches: &ArgMatches) -> Result<()> {
    let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;
    rt.block_on(execute_async(matches))
}

async fn execute_async(matches: &ArgMatches) -> Result<()> {
    let root_dir = std::env::current_dir().context("Failed to get current directory")?;

    match matches.subcommand() {
        Some(("init", m)) => {
            let name = m
                .get_one::<String>("name")
                .cloned()
                .unwrap_or_else(|| "my-app".to_string());
            println!("Initializing project: {}", name);
            println!("  This feature is under development");
            Ok(())
        }

        Some(("config", cm)) => match cm.subcommand() {
            Some(("lint", lm)) => {
                let project_root = lm
                    .get_one::<String>("path")
                    .map(PathBuf::from)
                    .unwrap_or(root_dir);
                let format = lm
                    .get_one::<String>("format")
                    .map(|s| s.as_str())
                    .unwrap_or("human");
                let strict_warn = lm.get_flag("strict-warn");
                let exit = crate::wasm_build::lint::run(&project_root, format, strict_warn);
                std::process::exit(exit);
            }
            Some(("schema", sm)) => {
                let workspace_root = sm
                    .get_one::<String>("path")
                    .map(PathBuf::from)
                    .unwrap_or(root_dir);
                let mode = if sm.get_flag("check") {
                    "check"
                } else if sm.get_flag("write") {
                    "write"
                } else {
                    "print"
                };
                let exit = crate::wasm_build::schema::run(&workspace_root, mode);
                std::process::exit(exit);
            }
            _ => {
                anyhow::bail!("Unknown config subcommand. Try one of: lint, schema.")
            }
        },

        Some(("install", m)) => {
            let frozen = m.get_flag("frozen-lockfile");
            let auto_ci_frozen = !m.get_flag("no-frozen-lockfile");
            let no_cache = m.get_flag("no-cache");
            let no_install = m.get_flag("no-install");
            let prebundle = !m.get_flag("no-prebundle");
            let force_nx = m.get_flag("nx");

            // Detect workspace mode (Nx > Jet > Single).
            let workspace_mode = if force_nx {
                let nx_mgr = crate::pkg_manager::nx::NxWorkspaceManager::discover(&root_dir)?
                    .context("--nx flag set but no nx.json found in current directory")?;
                crate::pkg_manager::workspace::WorkspaceMode::Nx(nx_mgr)
            } else {
                crate::pkg_manager::workspace::WorkspaceMode::detect(&root_dir)?
            };

            match workspace_mode {
                crate::pkg_manager::workspace::WorkspaceMode::Nx(nx_mgr) => {
                    println!("Nx workspace detected at {}", nx_mgr.root.display());
                    println!(
                        "Installing from root package.json \
                         (Nx manages workspace dependencies at root)"
                    );
                    let pm = crate::pkg_manager::PackageManager::new_with_flags(
                        nx_mgr.root.clone(),
                        no_cache,
                    )
                    .context("Failed to create package manager")?;
                    if no_install {
                        pm.install_lockfile_only().await
                    } else {
                        pm.install_with_ci_policy(frozen, auto_ci_frozen).await?;
                        if prebundle {
                            prebundle_after_install(nx_mgr.root).await?;
                        }
                        Ok(())
                    }
                }
                _ => {
                    let install_root = root_dir.clone();
                    let pm = crate::pkg_manager::PackageManager::new_with_flags(root_dir, no_cache)
                        .context("Failed to create package manager")?;
                    if no_install {
                        pm.install_lockfile_only().await
                    } else {
                        pm.install_with_ci_policy(frozen, auto_ci_frozen).await?;
                        if prebundle {
                            prebundle_after_install(install_root).await?;
                        }
                        Ok(())
                    }
                }
            }
        }

        Some(("add", m)) => {
            let packages: Vec<&str> = m
                .get_many::<String>("packages")
                .map(|vals| vals.map(|s| s.as_str()).collect())
                .unwrap_or_default();
            let dev = m.get_flag("dev");
            let pm = crate::pkg_manager::PackageManager::new(root_dir)
                .context("Failed to create package manager")?;
            pm.add_many(&packages, dev).await
        }

        Some(("remove", m)) => {
            let package = m.get_one::<String>("package").unwrap();
            let pm = crate::pkg_manager::PackageManager::new(root_dir)
                .context("Failed to create package manager")?;
            pm.remove(package).await
        }

        Some(("update", m)) => {
            let package = m.get_one::<String>("package").map(|s| s.as_str());
            let latest = m.get_flag("latest");
            let pm = crate::pkg_manager::PackageManager::new(root_dir)
                .context("Failed to create package manager")?;
            pm.update(package, latest).await
        }

        Some(("audit", _)) => {
            let pm = crate::pkg_manager::PackageManager::new(root_dir.clone())
                .context("Failed to create package manager")?;
            let report = pm.audit().await?;
            println!(
                "Vulnerabilities: {} critical, {} high, {} moderate, {} low ({} total)",
                report.summary.critical,
                report.summary.high,
                report.summary.moderate,
                report.summary.low,
                report.summary.total
            );
            if report.has_critical_or_high() {
                anyhow::bail!("Critical or high severity vulnerabilities found");
            }
            Ok(())
        }

        Some(("patch", m)) => {
            let package = m.get_one::<String>("package").unwrap();
            let pm = crate::pkg_manager::patch::PatchManager::new(root_dir);
            let path = pm.prepare_patch(package)?;
            println!("Patch directory: {:?}", path);
            println!("Edit files, then run 'jet patch-commit {}'", package);
            Ok(())
        }

        Some(("patch-commit", m)) => {
            let package = m.get_one::<String>("package").unwrap();
            let pm = crate::pkg_manager::patch::PatchManager::new(root_dir);
            let path = pm.commit_patch(package)?;
            println!("Patch file: {:?}", path);
            Ok(())
        }

        Some(("publish", m)) => {
            let tag = m
                .get_one::<String>("tag")
                .map(|s| s.as_str())
                .unwrap_or("latest");
            let access = m.get_one::<String>("access").map(|s| s.as_str());
            let publisher = crate::pkg_manager::publish::Publisher::new(root_dir);
            publisher.publish(tag, access).await
        }

        Some(("pack", _)) => {
            let publisher = crate::pkg_manager::publish::Publisher::new(root_dir);
            let path = publisher.pack()?;
            println!("Created: {:?}", path);
            Ok(())
        }

        Some(("store", m)) => match m.subcommand() {
            Some(("prune", _)) => {
                // GH #3596 — the prior `unwrap_or_else(|_| ".".to_string())`
                // silently re-pointed `jet store prune` at `./.jet-store`
                // when HOME was unset (daemon contexts, CI) or non-UTF-8.
                // The user expects ~/.jet-store; CWD-relative was wrong
                // and the GC pass was silently skipped (empty store →
                // "Pruned 0 packages" looks identical to a clean store).
                // Match VarError explicitly so the failure mode is loud.
                let store_path = resolve_store_path_from_home(std::env::var("HOME"))?;
                let gc = crate::pkg_manager::gc::StoreGc::new(store_path);
                let result = gc.prune(&[root_dir])?;
                println!(
                    "Pruned {} packages, reclaimed {} bytes",
                    result.removed, result.reclaimed_bytes
                );
                Ok(())
            }
            // GH #3721 — was `_ => { println!(...); Ok(()) }`. That wrote
            // "Unknown store subcommand" to STDOUT and returned exit 0, so
            // `jet store gc` (typo) silently succeeded under any CI step
            // that checks `$?`. Every other Unknown-subcommand branch in
            // cli.rs (config, report, trace, e2e, browser, top-level) uses
            // `anyhow::bail!` for stderr + non-zero exit. Match them.
            Some((other, _)) => {
                anyhow::bail!("{}", format_unknown_store_subcommand_err(Some(other)))
            }
            None => anyhow::bail!("{}", format_unknown_store_subcommand_err(None)),
        },

        Some(("serve", m)) => handle_serve_command(&root_dir, m).await,

        Some(("dev", m)) => {
            if let Some(("shutdown", sm)) = m.subcommand() {
                let port = sm
                    .get_one::<String>("port")
                    .and_then(|s| parse_cli_numeric_flag::<u16>("--port", s))
                    .context("--port is required and must be a number")?;
                let host = sm
                    .get_one::<String>("host")
                    .cloned()
                    .unwrap_or_else(|| "127.0.0.1".to_string());
                return shutdown_dev_server(&host, port).await;
            }

            // WASM mode short-circuits the JS-bundle dev path.
            if m.get_flag("wasm") {
                let cli_port = m
                    .get_one::<String>("port")
                    .and_then(|s| parse_cli_numeric_flag::<u16>("--port", s));
                let host = m
                    .get_one::<String>("host")
                    .cloned()
                    .unwrap_or_else(|| "127.0.0.1".to_string());
                return crate::wasm_dev::serve(
                    &root_dir,
                    crate::wasm_dev::DevOptions {
                        host,
                        port: cli_port.unwrap_or(3000),
                        debug: m.get_flag("debug"),
                    },
                )
                .await;
            }

            // Load jet.toml for dev settings (port, proxy)
            eprintln!(
                "[jet] Loading config from {}",
                root_dir.join("jet.toml").display()
            );
            let jet_config = crate::task_runner::config::JetConfig::load(&root_dir)?;
            let dev_proxy = merge_dev_proxy_rules(
                jet_config.dev.proxy.clone(),
                m.get_many::<String>("proxy")
                    .into_iter()
                    .flatten()
                    .map(String::as_str),
            )?;
            eprintln!(
                "[jet] Config loaded: dev.port={:?}, proxy keys={}",
                jet_config.dev.port,
                dev_proxy.len()
            );

            let cli_port = m
                .get_one::<String>("port")
                .and_then(|s| parse_cli_numeric_flag::<u16>("--port", s));
            // CLI --port overrides config dev.port, both override default 3000
            let port = cli_port.or(jet_config.dev.port).unwrap_or(3000);
            let host = m
                .get_one::<String>("host")
                .cloned()
                .unwrap_or_else(|| "127.0.0.1".to_string());

            let entry = find_entry_point(&root_dir)?;
            eprintln!("[jet] Resolved port={}", port);
            // @spec .aw/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R4
            let mut resolve_options = crate::resolver::ResolveOptions::default();
            if let Some(conds) = jet_config.resolve_conditions() {
                resolve_options.conditions = conds.to_vec();
            }

            let config = crate::dev_server::ServerConfig {
                host,
                port,
                root_dir: root_dir.clone(),
                public_dir: Some(root_dir.join("public")),
                entry,
                proxy: dev_proxy,
                aliases: jet_config.alias.clone(),
            };
            let bundle_opts = crate::bundler::BundleOptions {
                entry: config.entry.clone(),
                output_dir: root_dir.join("dist"),
                resolve_options,
                ..Default::default()
            };
            let bundler =
                crate::bundler::Bundler::new(bundle_opts).context("Failed to create bundler")?;
            let server = Arc::new(
                crate::dev_server::DevServer::new(bundler, config)
                    .context("Failed to create dev server")?,
            );
            server.start().await
        }

        Some(("build", m)) => {
            let output = m
                .get_one::<String>("output")
                .cloned()
                .unwrap_or_else(|| "dist".to_string());

            // @spec .aw/tech-design/projects/jet/logic/multi-target/build-targets.md
            // Resolve --target (defaults to web) and run the spec's
            // validation table before any expensive build step.
            let target_raw = m.get_one::<String>("target").map(|s| s.as_str());
            let build_target =
                crate::build_target::resolve(target_raw).map_err(|e| anyhow::anyhow!(e))?;
            let flag_snapshot = build_flag_snapshot_from_matches(m);
            crate::build_target::validate_combination(build_target, flag_snapshot)
                .map_err(|e| anyhow::anyhow!(e))?;
            // Spec validation table: print `info: target=<name>` on
            // every build (not just when --target is explicit) so
            // build logs are unambiguous about which target ran.
            eprintln!("info: target={}", build_target);

            // WASM build shares the frontend ingest shape with the normal
            // build (entry TS/TSX + HTML shell + CSS side-effect imports),
            // then switches backend from JS bundling to Rust/WASM lowering.
            if m.get_flag("wasm") {
                let profile = if m.get_flag("debug") {
                    crate::wasm_build::Profile::Dev
                } else {
                    crate::wasm_build::Profile::Release
                };
                let dist = root_dir.join(&output);
                crate::wasm_build::build_with_profile(
                    &root_dir,
                    std::path::Path::new(&output),
                    profile,
                    build_target,
                )?;
                let _ = dist;
                return Ok(());
            }

            // Library mode (`jet build --lib`, or a `[lib]` section in
            // jet.toml). Externalizes npm deps and emits a publishable
            // ESM (+ optional CJS) artifact instead of an app bundle. The
            // app-mode build path below is untouched.
            let lib_config = crate::task_runner::config::JetConfig::load(&root_dir)
                .ok()
                .and_then(|c| c.lib);
            if m.get_flag("lib") || lib_config.is_some() {
                return run_library_build(m, &root_dir, &output, lib_config.as_ref());
            }

            // GH #3708 — the prior `let _watch = m.get_flag("watch");`
            // silently discarded the flag. clap accepted `-w` / `--watch`,
            // the bundler ran once, printed "Build complete", exited 0.
            // The module docstring at the top of this file advertises
            // `cclab jet build --watch` as a working mode, and `jet test
            // --watch` / `jet dev --watch` both honour the flag, so the
            // asymmetric silent drop was a foot-gun. Surface the no-op
            // explicitly until a watch-build loop lands.
            if m.get_flag("watch") {
                eprintln!("{}", format_build_watch_not_implemented_warn());
            }
            let force_nx = m.get_flag("nx");
            let nx_project = m.get_one::<String>("project").cloned();

            // Detect workspace mode and dispatch to Nx build if appropriate.
            let workspace_mode = if force_nx {
                let nx_mgr = crate::pkg_manager::nx::NxWorkspaceManager::discover(&root_dir)?
                    .context("--nx flag set but no nx.json found in current directory")?;
                crate::pkg_manager::workspace::WorkspaceMode::Nx(nx_mgr)
            } else {
                crate::pkg_manager::workspace::WorkspaceMode::detect(&root_dir)?
            };

            if let crate::pkg_manager::workspace::WorkspaceMode::Nx(nx_mgr) = workspace_mode {
                return run_nx_build(&nx_mgr, &output, nx_project.as_deref()).await;
            }

            let entry = find_entry_point(&root_dir)?;
            let output_dir = root_dir.join(&output);

            let minify = build_minify_enabled_from_matches(m);
            let sourcemap_mode = match m
                .get_one::<String>("sourcemap")
                .map(|s| s.as_str())
                .unwrap_or("external")
            {
                "none" => crate::bundler::types::SourceMapOption::None,
                "inline" => crate::bundler::types::SourceMapOption::Inline,
                "hidden" => crate::bundler::types::SourceMapOption::Hidden,
                _ => crate::bundler::types::SourceMapOption::External,
            };
            let splitting = m.get_flag("splitting");

            // Parse --define KEY=VALUE pairs
            // GH #3712 — the prior `if let Some(...) = def.split_once('=') {
            // defines.insert(...); }` silently dropped any --define entry
            // that lacked an `=`, including the common typos
            // `--define VERSION` (forgot value) and `--define KEY:VALUE`
            // (used colon). Build said "complete"; the bundle shipped
            // with the constant unreplaced and the user only found out
            // in production. esbuild errors out on missing `=`; mirror
            // that behaviour by bailing with a tagged message so CI
            // catches the typo.
            let mut defines = crate::bundler::define::production_defines();
            let env_vars = crate::runner::env::scan_env_files(&root_dir, "production");
            defines.extend(crate::runner::env::import_meta_env_defines(
                &env_vars,
                "production",
            ));
            if let Some(defs) = m.get_many::<String>("define") {
                for def in defs {
                    let (key, value) = parse_define_arg(def).map_err(|msg| anyhow!("{}", msg))?;
                    defines.insert(key, value);
                }
            }

            // Parse --drop console|debugger
            let mut drops = Vec::new();
            if let Some(drop_args) = m.get_many::<String>("drop") {
                for d in drop_args {
                    match d.as_str() {
                        "console" => drops.push(crate::bundler::minify::DropStatement::Console),
                        "debugger" => drops.push(crate::bundler::minify::DropStatement::Debugger),
                        other => eprintln!("Warning: unknown drop target '{}'", other),
                    }
                }
            }

            let start = std::time::Instant::now();
            let frontend = crate::frontend::FrontendSources::load(&root_dir, entry.clone())
                .context("Failed to read frontend sources")?;

            // @spec .aw/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R4
            // Surface jet.toml parse errors instead of silently falling back to defaults.
            // Mirrors the jet dev path fix (PR #2940); GH #3061.
            let build_config = match crate::task_runner::config::JetConfig::load(&root_dir) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("[jet build] Failed to parse jet.toml: {e:#}");
                    eprintln!(
                        "[jet build] Continuing with built-in defaults; [resolve.conditions] / [alias] from the file will NOT take effect until the parse error is fixed."
                    );
                    crate::task_runner::config::JetConfig::default()
                }
            };
            let mut resolve_options = crate::resolver::ResolveOptions::for_browser_production();
            if let Some(conds) = build_config.resolve_conditions() {
                resolve_options.conditions = conds.to_vec();
            }

            let bundle_opts = crate::bundler::BundleOptions {
                entry: entry.clone(),
                output_dir: output_dir.clone(),
                minify,
                source_maps: sourcemap_mode != crate::bundler::types::SourceMapOption::None,
                resolve_options,
                // @spec .aw/tech-design/crates/jet/validate/add-production-jet-build-regression-coverage.md#changes
                // Web app production builds must emit a browser-bootable bundle,
                // not a "build complete" shell with bare React/MUI externals.
                externalize_all_packages: false,
                transform_options: crate::transform::TransformOptions {
                    dev_mode: false,
                    ..Default::default()
                },
                defines: defines.clone(),
                ..Default::default()
            };

            let bundler =
                crate::bundler::Bundler::new(bundle_opts).context("Failed to create bundler")?;
            let mut result = bundler
                .bundle(frontend.entry_path.clone())
                .await
                .context("Bundle failed")?;
            append_css_side_effect_assets(
                &root_dir,
                &frontend.entry_path,
                &frontend.css_side_effect_imports,
                minify,
                &mut result.assets,
            )
            .context("Failed to process CSS side-effect imports")?;

            // JET_BUNDLE_TIMING=1 extends the bundler's per-phase laps into
            // the post-process/minify tail so the whole wall-clock is
            // attributable from one run.
            let timing = std::env::var_os("JET_BUNDLE_TIMING").is_some();
            let mut last_lap = std::time::Instant::now();
            let mut lap = move |label: &str| {
                if timing {
                    eprintln!("[bundle-timing] post/{label}: {:?}", last_lap.elapsed());
                    last_lap = std::time::Instant::now();
                }
            };

            // Post-process: define replacement + syntax-aware static branch DCE.
            // The transform step already ran both per module (parallel, small
            // parses), so at bundle level replacement is a no-op unless a
            // define pattern only materialized across module glue. Gate the
            // DCE fixpoint (up to 8 full tree-sitter parses of the assembled
            // bundle — ~2s on the mui corpus) on the replacement actually
            // changing something.
            let replaced = crate::bundler::define::replace_defines(&result.code, &defines);
            lap("replace_defines");
            // Fold expression-level define guards UNCONDITIONALLY. Per-module
            // transforms already substitute `process.env.NODE_ENV` upstream, so
            // `replace_defines` here is often a no-op while the assembled bundle
            // still carries always-false `"production" !== "production" &&
            // console.warn(...)` guards (styled-components ships several).
            // fold_define_short_circuits collapses only literal-vs-literal
            // compares — pure define residue — and is internally gated, so
            // running it always is safe and cheap. eliminate_static_conditionals
            // then strips any statement-position `if (false) {}` the fold left.
            let oxc_minify_enabled = std::env::var_os("JET_DISABLE_OXC_MINIFY").is_none();
            let folded = crate::bundler::fold::fold_define_short_circuits(&replaced);
            let mut code = if folded == result.code || oxc_minify_enabled {
                folded
            } else {
                crate::bundler::dce::eliminate_static_conditionals_syntax(&folded)
            };
            lap("static_conditionals_dce");

            // Post-process: minify
            if minify {
                // JET_MINIFY_STAGE_DUMP=<dir> writes the bundle after each
                // minify stage so a runtime-only breakage (parses fine,
                // misbehaves in the browser) can be bisected to one pass.
                let stage_dump = std::env::var_os("JET_MINIFY_STAGE_DUMP").map(PathBuf::from);
                let dump_stage = |stage: &str, code: &str| {
                    if let Some(dir) = &stage_dump {
                        let _ = std::fs::create_dir_all(dir);
                        let _ = std::fs::write(dir.join(format!("{stage}.js")), code);
                    }
                };
                dump_stage("0-bundle", &code);
                code = crate::bundler::minify::minify_js(&code, &drops);
                lap("minify_js");
                dump_stage("1-minify-js", &code);
                code = crate::bundler::minify::replace_bool_literals(&code);
                lap("bool_literals");
                code = crate::bundler::minify::strip_use_client_directives(&code);
                lap("strip_directives");
                dump_stage("2-bool-literals", &code);
                let mut module_glue_base = None;
                {
                    let optimized =
                        crate::bundler::scope_hoist_opt::optimize_generated_module_glue(&code);
                    if optimized.len() < code.len() {
                        if oxc_minify_enabled {
                            module_glue_base = Some(code);
                            code = optimized;
                        } else if crate::bundler::dce::js_parses_without_errors(&optimized) {
                            code = optimized;
                        }
                    }
                }
                lap("module_glue");
                if std::env::var_os("JET_ENABLE_DIRECT_EXPORT_READS").is_some() {
                    let lowered = crate::bundler::scope_hoist_opt::lower_direct_export_reads(&code);
                    if lowered != code && crate::bundler::dce::js_parses_without_errors(&lowered) {
                        code = lowered;
                    }
                }
                lap("direct_export_reads");
                let mut oxc_applied = false;
                if oxc_minify_enabled {
                    if let Some(polished) = crate::bundler::minify::oxc_minify_js_candidate(&code) {
                        if polished.len() < code.len() {
                            code = polished;
                            oxc_applied = true;
                        }
                    } else if let Some(base) = module_glue_base.take() {
                        code = base;
                    }
                }
                lap("oxc_minify");
                // Optimistic guard scheme: run mangle → fold → semicolon
                // compaction unguarded, then parse ONCE. Parsing the full
                // bundle with tree-sitter is the third-largest build cost
                // (~0.5-0.9s across the corpus when done per stage); on the
                // happy path one parse certifies all three stages. Only when
                // that single parse fails do we re-run the chain with the
                // original per-stage guards to keep whichever stages are
                // individually sound — same output as the old scheme, the
                // extra parses are paid only on the rare corruption path.
                // Compute mangle -> fold -> semicolon-compaction ONCE, then
                // certify with as few full-bundle tree-sitter parses as
                // possible. The happy path is a single parse of the most
                // minified candidate. When that parse fails (a minify pass
                // produced output tree-sitter flags), we DON'T recompute the
                // expensive mangle: the already-computed `mangled`/`folded`
                // values ARE the stagewise fallback results (fold operates on
                // mangled, semicolon on folded), so we just probe them in
                // size order and keep the first that parses — byte-identical
                // to the old per-stage-guard scheme, minus the re-mangle and
                // the redundant parses.
                if !oxc_applied {
                    let mangled = crate::bundler::mangle::mangle_variables_with_root(&code);
                    lap("mangle");
                    let folded = crate::bundler::fold::fold_constants(&mangled);
                    lap("fold");
                    let compacted =
                        crate::bundler::minify::remove_semicolons_before_block_close_candidate(
                            &folded,
                        );
                    lap("semicolon_compaction");
                    let candidate = if compacted.len() < folded.len() {
                        &compacted
                    } else {
                        &folded
                    };
                    if crate::bundler::dce::js_parses_without_errors(candidate) {
                        code = candidate.clone();
                        lap("single_parse_guard");
                    } else if crate::bundler::dce::js_parses_without_errors(&folded) {
                        // semicolon-compaction is the culprit; keep folded.
                        code = folded;
                        lap("reuse_parse_guard");
                    } else if crate::bundler::dce::js_parses_without_errors(&mangled) {
                        // fold broke parse; keep mangled, still try its compaction.
                        let s =
                            crate::bundler::minify::remove_semicolons_before_block_close_candidate(
                                &mangled,
                            );
                        code = if s.len() < mangled.len()
                            && crate::bundler::dce::js_parses_without_errors(&s)
                        {
                            s
                        } else {
                            mangled
                        };
                        tracing::warn!("Constant folding skipped: optimized bundle did not parse");
                        lap("reuse_parse_guard");
                    } else {
                        // Mangle itself broke parse — fall all the way back to the
                        // pre-mangle bundle and fold/compact that.
                        tracing::warn!("Variable mangling skipped: optimized bundle did not parse");
                        let f = crate::bundler::fold::fold_constants(&code);
                        let base = if crate::bundler::dce::js_parses_without_errors(&f) {
                            f
                        } else {
                            code.clone()
                        };
                        let s =
                            crate::bundler::minify::remove_semicolons_before_block_close_candidate(
                                &base,
                            );
                        code = if s.len() < base.len()
                            && crate::bundler::dce::js_parses_without_errors(&s)
                        {
                            s
                        } else {
                            base
                        };
                        lap("remangle_fallback");
                    }
                    dump_stage("3-mangle", &code);
                    dump_stage("4-fold", &code);
                    // Post-mangle dead `var X=Y.exports;` alias removal — safe
                    // here (name assignment is fixed; removing it pre-mangle
                    // tripped a mangler collision). Parse-guarded; reverts on the
                    // rare shape it can't handle.
                    {
                        let pruned = crate::bundler::minify::remove_dead_exports_aliases(&code);
                        if pruned.len() < code.len()
                            && crate::bundler::dce::js_parses_without_errors(&pruned)
                        {
                            code = pruned;
                        }
                    }
                    lap("dead_exports_aliases");
                    if oxc_minify_enabled {
                        if let Some(polished) =
                            crate::bundler::minify::oxc_minify_js_candidate(&code)
                        {
                            if polished.len() < code.len() {
                                code = polished;
                            }
                        }
                    }
                    lap("oxc_minify_after_legacy");
                }
                // Extra polish (JET_EXTRA_MINIFY=1): residual-space squeeze,
                // block-level empty-statement removal, bracket-to-dot
                // properties. Worth ~0.3-2KB gzip per fixture but costs two
                // extra full-bundle passes plus a tree-sitter guard parse —
                // currently a net loss against the duration bar, so opt-in
                // until the passes pay for themselves.
                if std::env::var_os("JET_EXTRA_MINIFY").is_some() {
                    let polished = crate::bundler::minify::squeeze_residual_spaces(&code);
                    let polished =
                        crate::bundler::dce::remove_redundant_empty_statements(&polished);
                    let polished = crate::bundler::minify::bracket_to_dot_properties(&polished);
                    if polished.len() < code.len()
                        && crate::bundler::dce::js_parses_without_errors(&polished)
                    {
                        code = polished;
                    }
                }
                dump_stage("5-squeeze", &code);
            }

            // Content hash for filename
            let hash = content_hash_prefix(&code);
            let out_filename = format!("main.{hash}.js");
            lap("content_hash");

            std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

            // Source maps
            match &sourcemap_mode {
                crate::bundler::types::SourceMapOption::External => {
                    let map_filename = format!("{}.map", &out_filename);
                    let sources = vec![(
                        coerce_sourcemap_entry_path_or_warn(&entry),
                        result.code.clone(),
                    )];
                    let map = crate::bundler::sourcemap::generate_source_map(
                        &out_filename,
                        &sources,
                        &code,
                    );
                    code = crate::bundler::sourcemap::append_source_map_url(&code, &map_filename);
                    crate::bundler::sourcemap::write_external_map(
                        &output_dir,
                        &map_filename,
                        &map.json,
                    )?;
                }
                crate::bundler::types::SourceMapOption::Inline => {
                    let sources = vec![(
                        coerce_sourcemap_entry_path_or_warn(&entry),
                        result.code.clone(),
                    )];
                    let map = crate::bundler::sourcemap::generate_source_map(
                        &out_filename,
                        &sources,
                        &code,
                    );
                    code = crate::bundler::sourcemap::inline_source_map(&code, &map.json);
                }
                _ => {}
            }
            lap("sourcemap");

            // Write output
            std::fs::write(output_dir.join(&out_filename), &code)
                .context("Failed to write bundle")?;
            let css_filenames = write_bundle_assets(&output_dir, &result.assets)
                .context("Failed to write bundle assets")?;
            copy_public_dir(&root_dir, &output_dir).context("Failed to copy public assets")?;
            emit_build_index_html(
                &output_dir,
                &frontend.html_template,
                &frontend.entry,
                &out_filename,
                &css_filenames,
            )
            .context("Failed to write index.html")?;

            let duration = start.elapsed();
            let size_kb = code.len() as f64 / 1024.0;
            // GH #3705 — the prior `let _ = splitting; // TODO` silently
            // discarded the flag after clap parsing and validate_combination
            // both approved it. Build said "complete", but `--splitting`
            // never reached the chunk splitter, so dynamic-import bundles
            // landed as one file. Surface the no-op explicitly so users
            // know their flag is currently a TODO (tracked by #1089).
            if splitting {
                eprintln!("{}", format_splitting_not_implemented_warn());
            }

            println!(
                "Build complete in {:.0}ms: {}/{} ({:.1} KB)",
                duration.as_millis(),
                output,
                out_filename,
                size_kb,
            );
            Ok(())
        }

        Some(("check", _)) => Err(check_not_implemented_error()),

        Some(("run", m)) => {
            // No target → list available scripts (like `npm run`)
            let target = match m.get_one::<String>("target") {
                Some(t) => t,
                None => return list_scripts(&root_dir),
            };
            let args: Vec<String> = m
                .get_many::<String>("args")
                .map(|v| v.cloned().collect())
                .unwrap_or_default();
            let watch = m.get_flag("watch");
            let filter = m.get_one::<String>("filter").map(|s| s.as_str());
            let dry_run = m.get_flag("dry");

            handle_run(&root_dir, target, &args, watch, filter, dry_run).await
        }

        Some(("exec", m)) => {
            let cmd = m.get_one::<String>("cmd").unwrap();
            let args: Vec<String> = m
                .get_many::<String>("args")
                .map(|v| v.cloned().collect())
                .unwrap_or_default();

            let runner = crate::runner::ScriptRunner::new(root_dir);
            let result = runner.exec_command(cmd, &args).await?;
            print!("{}", result.stdout);
            eprint!("{}", result.stderr);
            std::process::exit(result.exit_code);
        }

        Some(("jtx", m)) => {
            let package = m.get_one::<String>("package").unwrap();
            let args: Vec<String> = m
                .get_many::<String>("args")
                .map(|v| v.cloned().collect())
                .unwrap_or_default();

            // Download package to temp, then execute its bin
            println!("Downloading {}...", package);
            let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
            let pm = crate::pkg_manager::PackageManager::new(temp_dir.path().to_path_buf())?;
            // Create minimal package.json
            let pkg = format!(
                r#"{{"name":"dlx-tmp","version":"0.0.0","dependencies":{{"{}":"latest"}}}}"#,
                package
            );
            std::fs::write(temp_dir.path().join("package.json"), pkg)?;
            pm.install().await?;

            let runner = crate::runner::ScriptRunner::new(temp_dir.path().to_path_buf());
            let result = runner.exec_command(package, &args).await?;
            print!("{}", result.stdout);
            eprint!("{}", result.stderr);
            std::process::exit(result.exit_code);
        }

        // @spec enhancement-html-reporter-for-native-test-runner-spec#R6
        // @spec enhancement-html-reporter-for-native-test-runner-spec#R7
        Some(("report", m)) => {
            match m.subcommand() {
                Some(("view", vm)) => {
                    let dir = PathBuf::from(vm.get_one::<String>("dir").unwrap());
                    let index_html = dir.join("index.html");
                    if !index_html.exists() {
                        anyhow::bail!(
                            "No index.html found in {}: run `jet test --reporter=html` first",
                            dir.display()
                        );
                    }
                    // Normalise to an absolute path for the file:// URL.
                    // GH #3604 — file:// URLs MUST be absolute; canonicalize
                    // can fail even when exists() passes, so we fall back to
                    // current_dir().join() before giving up.
                    let abs = match index_html.canonicalize() {
                        Ok(p) => p,
                        Err(canon_err) => {
                            absolutize_report_index(&index_html, std::env::current_dir())
                                .with_context(|| {
                                    format_report_view_abs_err(&index_html, &canon_err)
                                })?
                        }
                    };
                    let url = format!("file://{}", abs.display());
                    println!("Opening report: {url}");
                    if let Err(e) = open::that(&abs) {
                        eprintln!(
                            "Warning: could not open browser automatically ({e}). \
                             Open manually: {url}"
                        );
                    }
                    Ok(())
                }
                Some(("merge", vm)) => {
                    let inputs: Vec<PathBuf> = vm
                        .get_many::<String>("input")
                        .unwrap()
                        .map(PathBuf::from)
                        .collect();
                    let output = PathBuf::from(vm.get_one::<String>("output").unwrap());
                    crate::reporter::merge::merge_reports(&inputs, &output)
                }
                _ => {
                    anyhow::bail!(
                        "Unknown report subcommand. Try 'jet report view <dir>' \
                         or 'jet report merge --input <dirs...> --output <dir>'."
                    )
                }
            }
        }

        // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R5
        Some(("trace", m)) => match m.subcommand() {
            Some(("view", vm)) => {
                let file = PathBuf::from(vm.get_one::<String>("file").unwrap());
                let port = *vm.get_one::<u16>("port").unwrap_or(&0);
                let no_open = vm.get_flag("no-open");
                crate::trace::view::run(file, port, no_open).await
            }
            Some(("show", vm)) => {
                let file = PathBuf::from(vm.get_one::<String>("file").unwrap());
                crate::trace::view::show(&file)
            }
            Some(("extract", vm)) => {
                let file = PathBuf::from(vm.get_one::<String>("file").unwrap());
                let dir = PathBuf::from(vm.get_one::<String>("dir").unwrap());
                crate::trace::view::extract(&file, &dir)
            }
            _ => {
                anyhow::bail!("Unknown trace subcommand. Try 'jet trace view|show|extract'.")
            }
        },

        Some(("test", m)) => {
            // @spec enhancement-playwright-compat-shim-for-migration-window-spec#R1
            // @spec enhancement-playwright-compat-shim-for-migration-window-spec#R6
            if m.get_flag("playwright") {
                // R6: Reject native-only flags when --playwright is set.
                // Detect whether the user explicitly provided each native-only flag.
                let has_reporter = m.get_one::<String>("reporter").is_some();
                // trace has a default_value("off"), so we check if it was explicitly set
                let has_trace =
                    m.value_source("trace") == Some(clap::parser::ValueSource::CommandLine);
                let has_workers = m.get_one::<usize>("workers").is_some();
                let has_shard = m.get_one::<String>("shard").is_some();
                let has_report_dir = m.get_one::<String>("report-dir").is_some();

                if let Err((msg, code)) = crate::playwright_shim::validate_no_native_flags(
                    has_reporter,
                    has_trace,
                    has_workers,
                    has_shard,
                    has_report_dir,
                ) {
                    eprintln!("{}", msg);
                    std::process::exit(code);
                }

                // R1, R3, R4: delegate to playwright_shim::run which emits
                // the deprecation warning and spawns npx playwright test.
                let files: Vec<std::path::PathBuf> = m
                    .get_many::<String>("files")
                    .map(|v| v.map(std::path::PathBuf::from).collect())
                    .unwrap_or_default();
                let shim_args = crate::playwright_shim::PlaywrightArgs { files };
                let exit_code = crate::playwright_shim::run(&shim_args)?;
                std::process::exit(exit_code);
            }

            let mut cfg = crate::test_runner::RunnerConfig::default_for_root(&root_dir)
                .context("Failed to build test runner config")?;

            // Load [test.web_server] from jet.toml if present.
            // Surface parse errors instead of silently dropping [test.web_server].
            // Mirrors the jet dev (#2940) and jet build (#3061) fixes; GH #3065.
            // @spec .aw/tech-design/projects/jet/logic/web-server.md#W2
            match crate::task_runner::config::JetConfig::load(&root_dir) {
                Ok(jet_cfg) => {
                    cfg.web_server = jet_cfg.test.web_server.clone();
                }
                Err(e) => {
                    eprintln!("[jet test] Failed to parse jet.toml: {e:#}");
                    eprintln!(
                        "[jet test] Continuing without a preamble web server; [test.web_server] from the file will NOT take effect until the parse error is fixed."
                    );
                }
            }

            if let Some(files) = m.get_many::<String>("files") {
                cfg.only_files = files.map(PathBuf::from).collect();
            }
            if let Some(grep) = m.get_one::<String>("grep") {
                cfg.grep = Some(grep.clone());
            }
            if let Some(&timeout) = m.get_one::<u64>("timeout") {
                cfg.timeout_ms = timeout;
            }
            // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
            if let Some(reporter_str) = m.get_one::<String>("reporter") {
                cfg.reporters = crate::test_runner::config::Reporter::parse_list(reporter_str)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
            }
            if let Some(report_dir) = m.get_one::<String>("report-dir") {
                cfg.report_dir = PathBuf::from(report_dir);
            }
            cfg.update_snapshots = m.get_flag("update-snapshots");

            // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R1
            // GH #3094 — surface unknown `--trace` values instead of silently
            // coercing them to `off` (the opposite of what the user asked for).
            if let Some(trace_mode) = m.get_one::<String>("trace") {
                cfg.trace = match crate::test_runner::wire::WireTraceMode::from_str(trace_mode) {
                    Some(mode) => mode,
                    None => anyhow::bail!(
                        "Unknown --trace value '{trace_mode}'. Valid values: off, on, retain-on-failure"
                    ),
                };
            }

            // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R1
            if let Some(&n) = m.get_one::<usize>("workers") {
                if n < 1 {
                    anyhow::bail!("--workers must be >= 1");
                }
                cfg.workers = n;
            }

            // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R3
            if let Some(shard_str) = m.get_one::<String>("shard") {
                cfg.shard = Some(
                    crate::test_runner::parse_shard(shard_str)
                        .map_err(|e| anyhow::anyhow!("Invalid --shard value: {}", e))?,
                );
            }

            // @spec enhancement-define-component-and-browser-like-test-environment-boundary
            // When --list-resolved is set, defer `ensure_supported` so the
            // invalid-config path emits a JSON error envelope rather than a
            // bare `anyhow!` string.
            // @spec #2709
            let want_list_resolved = m.get_flag("list-resolved");
            if let Some(env_str) = m.get_one::<String>("env") {
                let env = crate::test_runner::config::TestEnvironment::parse(env_str)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                if !want_list_resolved {
                    env.ensure_supported()
                        .map_err(|e| anyhow::anyhow!("{}", e))?;
                }
                cfg.environment = env;
            }

            // @spec #2709
            // --list-resolved short-circuits before any execution side-effect.
            // Emits the resolved-discovery manifest to stdout (success) or
            // a stable JSON error line to stderr + exit code 2 (config invalid).
            if m.get_flag("list-resolved") {
                match crate::test_runner::discovery::resolve_discovery(&cfg) {
                    Ok(manifest) => {
                        let json = serde_json::to_string_pretty(&manifest)
                            .context("serialising resolved-discovery manifest")?;
                        println!("{json}");
                        std::process::exit(0);
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                        eprintln!("{}", err.to_json());
                        std::process::exit(2);
                    }
                }
            }

            // P4.2 --debug: headed + workers=1 for interactive inspection.
            // @spec .aw/tech-design/projects/jet/logic/inspector.md#I1
            if m.get_flag("debug") {
                cfg.headless = false;
                cfg.workers = 1;
                eprintln!(
                    "[jet debug] launching headed with workers=1. \
                     Insert `await page.pause()` in a spec to block until \
                     the test runner timeout (MVP: no interactive UI)."
                );
            }

            // P4.1: --watch mode loops on file changes.
            // @spec .aw/tech-design/projects/jet/logic/watch-mode.md#W1
            //
            // #2712: focused rerun skeleton. When the watcher reports
            // changed paths that match the test_match globs, we point
            // `cfg.only_files` at just those specs for the next run so
            // the suite shrinks to what the user actually touched. If
            // nothing test-shaped changed (e.g. a `src/util.ts` edit),
            // we clear the focus and fall back to a full rerun.
            if m.get_flag("watch") {
                loop {
                    let started = std::time::Instant::now();
                    let summary = crate::test_runner::run(cfg.clone()).await?;
                    let focused_label = if cfg.only_files.is_empty() {
                        "all".to_string()
                    } else {
                        format!("focused on {} spec(s)", cfg.only_files.len())
                    };
                    eprintln!(
                        "[jet watch] {} passed, {} failed ({}ms, {}). \
                         Watching for changes…",
                        summary.passed,
                        summary.failed,
                        started.elapsed().as_millis(),
                        focused_label,
                    );
                    // Fresh watcher per iteration — `wait_for_change` takes
                    // `&self`, but we move it into spawn_blocking. Rebinding
                    // keeps the borrow tidy without an Arc.
                    let root = root_dir.clone();
                    let changed = tokio::task::spawn_blocking(move || {
                        let w = crate::runner::watcher::DebouncedWatcher::new(&root, 300)?;
                        w.wait_for_change()
                    })
                    .await?
                    .context("watcher failed")?;
                    if changed.is_empty() {
                        continue;
                    }
                    // @spec #2712 — narrow the next run to changed specs only.
                    let focused =
                        crate::test_runner::discovery::pick_focused_specs(&cfg, &changed)?;
                    cfg.only_files = focused.clone();
                    if focused.is_empty() {
                        eprintln!(
                            "[jet watch] {} change(s) detected → re-running full suite",
                            changed.len(),
                        );
                    } else {
                        eprintln!(
                            "[jet watch] {} change(s) detected → re-running {} spec(s)",
                            changed.len(),
                            focused.len(),
                        );
                    }
                }
            }

            let summary = crate::test_runner::run(cfg).await?;
            if summary.failed > 0 {
                std::process::exit(1);
            }
            Ok(())
        }

        Some(("e2e", m)) => match m.subcommand() {
            // @spec .aw/tech-design/projects/jet/specs/2385.md#logic
            Some(("run", rm)) => {
                let evidence_dir = rm
                    .get_one::<String>("evidence-dir")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| {
                        crate::e2e::default_evidence_dir(&root_dir, crate::e2e::E2eMode::Run)
                    });
                let opts = crate::e2e::E2eRunOptions {
                    project_root: root_dir.clone(),
                    cases: rm
                        .get_many::<String>("cases")
                        .map(|v| v.map(PathBuf::from).collect())
                        .unwrap_or_default(),
                    grep: rm.get_one::<String>("grep").cloned(),
                    timeout_ms: rm.get_one::<u64>("timeout").copied(),
                    workers: crate::e2e::parse_workers(rm.get_one::<usize>("workers")),
                    trace: crate::e2e::parse_trace_mode(rm.get_one::<String>("trace"))?,
                    evidence_dir,
                    serve: crate::e2e::parse_serve_mode(rm.get_one::<String>("serve"))?,
                    base_url: rm.get_one::<String>("base-url").cloned(),
                    print_json: rm.get_flag("json"),
                };
                let result = crate::e2e::run_agent_mode(opts).await?;
                if crate::e2e::summary_exit_code(&result.bundle) != 0 {
                    std::process::exit(crate::e2e::summary_exit_code(&result.bundle));
                }
                Ok(())
            }
            // @spec .aw/tech-design/projects/jet/specs/2385.md#logic
            Some(("open", om)) => {
                let evidence_dir = om
                    .get_one::<String>("evidence-dir")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| {
                        crate::e2e::default_evidence_dir(&root_dir, crate::e2e::E2eMode::Open)
                    });
                let opts = crate::e2e::E2eOpenOptions {
                    project_root: root_dir.clone(),
                    cases: om
                        .get_many::<String>("cases")
                        .map(|v| v.map(PathBuf::from).collect())
                        .unwrap_or_default(),
                    grep: om.get_one::<String>("grep").cloned(),
                    timeout_ms: om.get_one::<u64>("timeout").copied(),
                    evidence_dir,
                    dry_run: om.get_flag("dry-run"),
                    no_open: om.get_flag("no-open"),
                };
                let result = crate::e2e::open_human_mode(opts).await?;
                if crate::e2e::summary_exit_code(&result.bundle) != 0 {
                    std::process::exit(crate::e2e::summary_exit_code(&result.bundle));
                }
                Ok(())
            }
            Some((other, _)) => {
                anyhow::bail!(
                    "Unknown e2e subcommand '{other}'. Try 'jet e2e run' or 'jet e2e open'."
                )
            }
            None => {
                anyhow::bail!("Unknown e2e subcommand. Try 'jet e2e run' or 'jet e2e open'.")
            }
        },

        // @spec .aw/changes/enhancement-cclab-jet-browser-install-cli-pinned-chromium-down/specs/enhancement-cclab-jet-browser-install-cli-pinned-chromium-down-spec.md#R1
        Some((browser_cmd @ ("browser" | "bb"), bm)) => match bm.subcommand() {
            Some(("install", im)) => {
                let target = im
                    .get_one::<String>("target")
                    .map(|s| s.as_str())
                    .unwrap_or("chromium");

                if target != "chromium" {
                    anyhow::bail!(
                        "{} is not yet supported; only chromium is available",
                        target
                    );
                }

                let revision = im
                    .get_one::<String>("revision")
                    .cloned()
                    .unwrap_or_else(|| crate::browser::DEFAULT_CHROMIUM_REVISION.to_string());

                // Resolve cache root: --cache-dir flag > ~/.jet/browsers/
                let cache_root: PathBuf = if let Some(dir) = im.get_one::<String>("cache-dir") {
                    PathBuf::from(dir)
                } else {
                    dirs::home_dir()
                        .context("Could not determine home directory")?
                        .join(".jet")
                        .join("browsers")
                };

                let binary_path =
                    crate::browser::install::install_chromium(&revision, &cache_root).await?;

                // Already-installed vs freshly-installed message is transparent to
                // the caller: both cases print the final path.
                println!(
                    "Installed chromium {} -> {}",
                    revision,
                    binary_path.display()
                );
                Ok(())
            }
            Some(("launch", lm)) => {
                let url = lm.get_one::<String>("url").expect("url required");
                if browser_cmd == "bb" {
                    crate::browser_cli::launch_detached(&root_dir, url).await
                } else {
                    crate::browser_cli::launch_foreground(&root_dir, url).await
                }
            }
            Some(("debug", lm)) => {
                let url = lm.get_one::<String>("url").expect("url required");
                crate::browser_cli::launch_foreground(&root_dir, url).await
            }
            Some(("mcp", _)) => crate::browser_cli::mcp::serve(&root_dir).await,
            Some(("shutdown", _)) => crate::browser_cli::shutdown(&root_dir).await,
            Some(("tree", tm)) => {
                let which = tm
                    .get_one::<String>("which")
                    .cloned()
                    .unwrap_or_else(|| "element".to_string());
                crate::browser_cli::tree(&root_dir, &which).await
            }
            Some(("pick", pm)) => {
                let timeout = pm
                    .get_one::<String>("timeout")
                    .and_then(|s| parse_cli_numeric_flag::<u64>("--timeout", s))
                    .unwrap_or(30);
                crate::browser_cli::pick(&root_dir, timeout).await
            }
            Some(("hooks", hm)) => {
                // GH #3651 — converted from `.parse::<u64>().ok()` to
                // `parse_cli_numeric_flag` so a malformed --fiber-id emits a
                // tagged warn via `jet::cli::flags` before the caller's
                // `.context()` fires. Sibling cleanup of the three sites
                // GH #3548 already converted (--port, --port, --timeout).
                let id = hm
                    .get_one::<String>("fiber-id")
                    .and_then(|s| parse_cli_numeric_flag::<u64>("--fiber-id", s))
                    .context("fiber-id must be a number")?;
                crate::browser_cli::hooks(&root_dir, id).await
            }
            Some(("highlight", hm)) => {
                let index = if hm.get_flag("clear") {
                    None
                } else {
                    // GH #3651 — same conversion for --index. `usize` already
                    // implements `FromStr<Err = ParseIntError>` so the
                    // existing helper covers it without an API change.
                    Some(
                        hm.get_one::<String>("index")
                            .and_then(|s| parse_cli_numeric_flag::<usize>("--index", s))
                            .context("highlight requires <index> or --clear")?,
                    )
                };
                crate::browser_cli::highlight(&root_dir, index).await
            }
            Some(("frame", _)) => crate::browser_cli::frame(&root_dir).await,
            Some(("perf", _)) => crate::browser_cli::perf(&root_dir).await,
            Some(("mouse", mm)) => {
                let event_type = mm.get_one::<String>("type").expect("type required");
                let x = mm
                    .get_one::<String>("x")
                    .expect("x required")
                    .parse::<f64>()
                    .context("x must be a number")?;
                let y = mm
                    .get_one::<String>("y")
                    .expect("y required")
                    .parse::<f64>()
                    .context("y must be a number")?;
                let button = mm.get_one::<String>("button").map(String::as_str);
                let buttons = mm
                    .get_one::<String>("buttons")
                    .and_then(|s| parse_cli_numeric_flag::<u64>("--buttons", s));
                let click_count = mm
                    .get_one::<String>("click-count")
                    .and_then(|s| parse_cli_numeric_flag::<u64>("--click-count", s));
                crate::browser_cli::mouse(&root_dir, event_type, x, y, button, buttons, click_count)
                    .await
            }
            Some(("drag", dm)) => {
                let from_x = dm
                    .get_one::<String>("from-x")
                    .expect("from-x required")
                    .parse::<f64>()
                    .context("from-x must be a number")?;
                let from_y = dm
                    .get_one::<String>("from-y")
                    .expect("from-y required")
                    .parse::<f64>()
                    .context("from-y must be a number")?;
                let to_x = dm
                    .get_one::<String>("to-x")
                    .expect("to-x required")
                    .parse::<f64>()
                    .context("to-x must be a number")?;
                let to_y = dm
                    .get_one::<String>("to-y")
                    .expect("to-y required")
                    .parse::<f64>()
                    .context("to-y must be a number")?;
                let steps = dm
                    .get_one::<String>("steps")
                    .and_then(|s| parse_cli_numeric_flag::<u64>("--steps", s))
                    .unwrap_or(8);
                crate::browser_cli::drag(&root_dir, from_x, from_y, to_x, to_y, steps).await
            }
            Some(("wheel", wm)) => {
                let x = wm
                    .get_one::<String>("x")
                    .expect("x required")
                    .parse::<f64>()
                    .context("x must be a number")?;
                let y = wm
                    .get_one::<String>("y")
                    .expect("y required")
                    .parse::<f64>()
                    .context("y must be a number")?;
                let delta_x = wm
                    .get_one::<String>("delta-x")
                    .expect("delta-x has default")
                    .parse::<f64>()
                    .context("--delta-x must be a number")?;
                let delta_y = wm
                    .get_one::<String>("delta-y")
                    .expect("delta-y has default")
                    .parse::<f64>()
                    .context("--delta-y must be a number")?;
                crate::browser_cli::wheel(&root_dir, x, y, delta_x, delta_y).await
            }
            Some(("key", km)) => {
                let key = km.get_one::<String>("key").expect("key required");
                let mut modifiers = 0;
                if km.get_flag("alt") {
                    modifiers |= 1;
                }
                if km.get_flag("ctrl") {
                    modifiers |= 2;
                }
                if km.get_flag("meta") {
                    modifiers |= 4;
                }
                if km.get_flag("shift") {
                    modifiers |= 8;
                }
                crate::browser_cli::key(&root_dir, key, modifiers).await
            }
            Some(("capture", cm)) => {
                let out = cm.get_one::<String>("out").map(PathBuf::from);
                let surface = cm
                    .get_one::<String>("surface")
                    .map(String::as_str)
                    .unwrap_or("wasm");
                let root_selector = cm
                    .get_one::<String>("root-selector")
                    .map(String::as_str)
                    .unwrap_or("body");
                let hooks = cm
                    .get_many::<String>("hook")
                    .into_iter()
                    .flatten()
                    .map(|raw| {
                        parse_cli_numeric_flag::<u64>("--hook", raw)
                            .context("--hook must be a fiber id")
                    })
                    .collect::<Result<Vec<_>>>()?;
                crate::browser_cli::capture(
                    &root_dir,
                    surface,
                    root_selector,
                    &hooks,
                    cm.get_flag("pretty"),
                    out.as_deref(),
                )
                .await
            }
            Some(("screenshot", sm)) => {
                let out = sm.get_one::<String>("out").map(PathBuf::from);
                crate::browser_cli::screenshot(&root_dir, out.as_deref()).await
            }
            Some(("eval", em)) => {
                let expr = em.get_one::<String>("expr").expect("expr required");
                crate::browser_cli::eval(&root_dir, expr).await
            }
            Some(("tsx", tm)) => {
                let filter = tm.get_one::<String>("filter").map(|s| s.as_str());
                crate::browser_cli::tsx(&root_dir, filter)
            }
            Some(("snapshot", sm)) => {
                let v = crate::browser_cli::interact::snapshot(&root_dir).await?;
                if sm.get_flag("json") {
                    println!("{}", serde_json::to_string_pretty(&v)?);
                } else {
                    let url = v["url"].as_str().unwrap_or("");
                    let title = v["title"].as_str().unwrap_or("");
                    println!("# {title} — {url}");
                    if v["truncated"].as_bool() == Some(true) {
                        println!("# (truncated at element cap — use a tighter page state)");
                    }
                    println!("{}", v["snapshot"].as_str().unwrap_or(""));
                }
                Ok(())
            }
            Some((verb @ ("click" | "hover" | "check" | "uncheck"), am)) => {
                let raw = am.get_one::<String>("target").expect("target required");
                let target = crate::browser_cli::interact::parse_target(raw)?;
                let v = match verb {
                    "click" => {
                        crate::browser_cli::interact::click(
                            &root_dir,
                            &target,
                            am.get_flag("dblclick"),
                        )
                        .await?
                    }
                    "hover" => crate::browser_cli::interact::hover(&root_dir, &target).await?,
                    "check" => {
                        crate::browser_cli::interact::set_checked(&root_dir, &target, true).await?
                    }
                    _ => {
                        crate::browser_cli::interact::set_checked(&root_dir, &target, false).await?
                    }
                };
                println!("{}", serde_json::to_string(&v)?);
                Ok(())
            }
            Some((verb @ ("fill" | "type"), am)) => {
                let raw = am.get_one::<String>("target").expect("target required");
                let text = am.get_one::<String>("text").expect("text required");
                let target = crate::browser_cli::interact::parse_target(raw)?;
                let v = if verb == "fill" {
                    crate::browser_cli::interact::fill(&root_dir, &target, text).await?
                } else {
                    crate::browser_cli::interact::type_text(&root_dir, &target, text).await?
                };
                println!("{}", serde_json::to_string(&v)?);
                Ok(())
            }
            Some(("select", am)) => {
                let raw = am.get_one::<String>("target").expect("target required");
                let option = am.get_one::<String>("option").expect("option required");
                let target = crate::browser_cli::interact::parse_target(raw)?;
                let v = crate::browser_cli::interact::select(&root_dir, &target, option).await?;
                println!("{}", serde_json::to_string(&v)?);
                Ok(())
            }
            Some(("goto", gm)) => {
                let url = gm.get_one::<String>("url").expect("url required");
                let v = crate::browser_cli::interact::goto(&root_dir, url).await?;
                println!("{}", serde_json::to_string(&v)?);
                Ok(())
            }
            Some(("back", _)) => {
                let v = crate::browser_cli::interact::history_step(&root_dir, -1).await?;
                println!("{}", serde_json::to_string(&v)?);
                Ok(())
            }
            Some(("forward", _)) => {
                let v = crate::browser_cli::interact::history_step(&root_dir, 1).await?;
                println!("{}", serde_json::to_string(&v)?);
                Ok(())
            }
            Some(("reload", _)) => {
                let v = crate::browser_cli::interact::reload(&root_dir).await?;
                println!("{}", serde_json::to_string(&v)?);
                Ok(())
            }
            Some(("resize", rm)) => {
                let width = rm
                    .get_one::<String>("width")
                    .and_then(|s| parse_cli_numeric_flag::<u64>("width", s))
                    .context("width must be a number")?;
                let height = rm
                    .get_one::<String>("height")
                    .and_then(|s| parse_cli_numeric_flag::<u64>("height", s))
                    .context("height must be a number")?;
                let v = crate::browser_cli::interact::resize(&root_dir, width, height).await?;
                println!("{}", serde_json::to_string(&v)?);
                Ok(())
            }
            Some(("wait", wm)) => {
                let selector = wm.get_one::<String>("selector").map(String::as_str);
                let text = wm.get_one::<String>("text").map(String::as_str);
                let ms = wm
                    .get_one::<String>("ms")
                    .and_then(|s| parse_cli_numeric_flag::<u64>("--ms", s));
                let timeout = wm
                    .get_one::<String>("timeout")
                    .and_then(|s| parse_cli_numeric_flag::<u64>("--timeout", s))
                    .unwrap_or(10_000);
                let v = crate::browser_cli::interact::wait(&root_dir, selector, text, ms, timeout)
                    .await?;
                println!("{}", serde_json::to_string(&v)?);
                Ok(())
            }
            Some(("console", cm)) => {
                let level = cm.get_one::<String>("level").map(String::as_str);
                let limit = cm
                    .get_one::<String>("limit")
                    .and_then(|s| parse_cli_numeric_flag::<usize>("--limit", s))
                    .unwrap_or(100);
                let v = crate::browser_cli::interact::console(
                    &root_dir,
                    level,
                    limit,
                    cm.get_flag("clear"),
                )
                .await?;
                println!("{}", serde_json::to_string_pretty(&v)?);
                Ok(())
            }
            Some(("requests", rm)) => {
                let limit = rm
                    .get_one::<String>("limit")
                    .and_then(|s| parse_cli_numeric_flag::<usize>("--limit", s))
                    .unwrap_or(100);
                let v =
                    crate::browser_cli::interact::requests(&root_dir, limit, rm.get_flag("clear"))
                        .await?;
                println!("{}", serde_json::to_string_pretty(&v)?);
                Ok(())
            }
            _ => {
                anyhow::bail!(
                    "Unknown browser subcommand. Try one of: install, launch, shutdown, debug, \
                     tree, pick, hooks, highlight, frame, perf, mouse, drag, key, capture, \
                     screenshot, eval, snapshot, click, fill, type, hover, select, check, \
                     uncheck, goto, back, forward, reload, resize, wait, console, requests."
                )
            }
        },

        _ => {
            anyhow::bail!("Unknown jet subcommand. Run 'jet --help' for usage.")
        }
    }
}

/// Execute a build across an Nx workspace in topological dependency order.
///
/// 1. Fetches the project graph via `nx graph --json`.
/// 2. Computes the topological build order.
/// 3. Optionally filters to a single `project` (and its transitive deps).
/// 4. For each project that has a resolvable entry point, invokes the bundler.
async fn run_nx_build(
    nx_mgr: &crate::pkg_manager::nx::NxWorkspaceManager,
    output: &str,
    project: Option<&str>,
) -> Result<()> {
    println!("Nx workspace detected at {}", nx_mgr.root.display());

    // Prefer standalone project.json scanning over `nx graph --json` CLI.
    // Falls back to nx CLI if file-based scan fails.
    let graph = nx_mgr
        .build_project_graph_from_files()
        .or_else(|_| nx_mgr.get_project_graph())
        .context("Failed to retrieve Nx project graph")?;

    let order = graph.topological_sort();

    // Filter: if a project name is given, only keep that project.
    let targets: Vec<String> = if let Some(name) = project {
        if !graph.graph.nodes.contains_key(name) {
            anyhow::bail!("Project '{}' not found in the Nx project graph", name);
        }
        vec![name.to_string()]
    } else {
        order
    };

    println!("Building {} project(s) in dependency order…", targets.len());

    let start = std::time::Instant::now();
    let mut built = 0usize;
    let mut skipped = 0usize;

    for project_name in &targets {
        // Derive project root from the graph; fall back to project name.
        let project_root = graph
            .project_root(project_name)
            .map(|r| nx_mgr.root.join(r))
            .unwrap_or_else(|| nx_mgr.root.join(project_name));

        // Skip projects without a detectable entry point.
        let entry = match find_entry_point(&project_root) {
            Ok(e) => e,
            Err(_) => {
                println!("  skip  {} (no entry point)", project_name);
                skipped += 1;
                continue;
            }
        };

        let output_dir = project_root.join(output);
        let frontend = crate::frontend::FrontendSources::load(&project_root, entry.clone())
            .with_context(|| format!("Failed to read frontend sources for '{}'", project_name))?;

        // Auto-detect lib vs app: libs externalize all node_modules deps.
        // Check: 1) project.json projectType, 2) graph node type, 3) path heuristic
        let is_lib = {
            // 1. Read project.json directly for projectType
            let project_json_path = project_root.join("project.json");
            let from_project_json = if project_json_path.exists() {
                std::fs::read_to_string(&project_json_path)
                    .ok()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                    .and_then(|v| {
                        v.get("projectType")
                            .and_then(|t| t.as_str().map(String::from))
                    })
                    .map(|t| t == "library" || t == "lib")
                    .unwrap_or(false)
            } else {
                false
            };
            // 2. Graph node type
            let from_graph = graph
                .graph
                .nodes
                .get(project_name.as_str())
                .and_then(|n| n.project_type.as_deref())
                .map(|t| t == "library" || t == "lib")
                .unwrap_or(false);
            // 3. Path heuristic
            let from_path = project_root.to_string_lossy().contains("/libs/");
            from_project_json || from_graph || from_path
        };

        let bundle_opts = crate::bundler::BundleOptions {
            entry: entry.clone(),
            output_dir: output_dir.clone(),
            minify: false,
            source_maps: true,
            externalize_all_packages: is_lib,
            ..Default::default()
        };

        let bundler = crate::bundler::Bundler::new(bundle_opts)
            .with_context(|| format!("Failed to create bundler for '{}'", project_name))?;

        // Build with 30s timeout to prevent hanging on huge bundles
        let bundle_future = bundler.bundle(frontend.entry_path.clone());
        let result =
            match tokio::time::timeout(std::time::Duration::from_secs(30), bundle_future).await {
                Ok(Ok(r)) => r,
                Ok(Err(e)) => {
                    println!("  ERROR {} ({})", project_name, e);
                    skipped += 1;
                    continue;
                }
                Err(_) => {
                    println!("  TIMEOUT {} (>30s)", project_name);
                    skipped += 1;
                    continue;
                }
            };

        let code = crate::bundler::define::replace_defines(
            &result.code,
            &crate::bundler::define::production_defines(),
        );
        let code = crate::bundler::dce::eliminate_static_conditionals_syntax(&code);

        // Content hash for output filename.
        let hash = content_hash_prefix(&code);
        let out_filename = format!("main.{hash}.js");

        std::fs::create_dir_all(&output_dir)
            .with_context(|| format!("Failed to create output dir for '{}'", project_name))?;
        std::fs::write(output_dir.join(&out_filename), &code)
            .with_context(|| format!("Failed to write bundle for '{}'", project_name))?;
        let css_filenames = write_bundle_assets(&output_dir, &result.assets)
            .with_context(|| format!("Failed to write bundle assets for '{}'", project_name))?;
        copy_public_dir(&project_root, &output_dir)
            .with_context(|| format!("Failed to copy public assets for '{}'", project_name))?;
        emit_build_index_html(
            &output_dir,
            &frontend.html_template,
            &frontend.entry,
            &out_filename,
            &css_filenames,
        )
        .with_context(|| format!("Failed to write index.html for '{}'", project_name))?;

        let size_kb = code.len() as f64 / 1024.0;
        println!(
            "  build {} → {}/{} ({:.1} KB)",
            project_name, output, out_filename, size_kb,
        );
        built += 1;
    }

    let duration = start.elapsed();
    println!(
        "\nNx build complete in {:.0}ms: {} built, {} skipped",
        duration.as_millis(),
        built,
        skipped,
    );
    Ok(())
}

/// Drive a `jet build --lib` library build.
///
/// Translates CLI flags (`--format`, `--output`) and the optional `[lib]`
/// section of `jet.toml` into [`crate::bundler::LibBuildOptions`], runs
/// [`crate::bundler::build_library`], and prints the emitted files. App-mode
/// build is unaffected — this is only reached when `--lib` is passed or a
/// `[lib]` section is present.
/// @issue #170
fn run_library_build(
    m: &ArgMatches,
    root_dir: &Path,
    output: &str,
    lib_config: Option<&crate::task_runner::config::LibConfig>,
) -> Result<()> {
    use crate::bundler::types::OutputFormat;

    // Formats: --format overrides [lib] formats overrides default [esm].
    let format_strings: Vec<String> = if let Some(arg) = m.get_one::<String>("format") {
        arg.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else if let Some(formats) = lib_config.and_then(|c| c.formats.clone()) {
        formats
    } else {
        vec!["esm".to_string()]
    };

    let mut formats = Vec::new();
    for f in &format_strings {
        match f.to_lowercase().as_str() {
            "esm" | "es" | "module" => formats.push(OutputFormat::Esm),
            "cjs" | "commonjs" => formats.push(OutputFormat::Cjs),
            other => anyhow::bail!(
                "jet build --lib: unknown format '{}' (expected esm or cjs)",
                other
            ),
        }
    }
    if formats.is_empty() {
        formats.push(OutputFormat::Esm);
    }

    // out_dir: --output (defaults to "dist") wins; else [lib] out_dir.
    let out_rel = if output != "dist" {
        output.to_string()
    } else {
        lib_config
            .and_then(|c| c.out_dir.clone())
            .unwrap_or_else(|| output.to_string())
    };
    let out_dir = root_dir.join(&out_rel);

    // Resolve conditions: prefer the build-mode default (import/require/...).
    let conditions = crate::resolver::ResolveOptions::for_browser_production()
        .conditions
        .clone();

    let preserve_modules = lib_config
        .and_then(|c| c.preserve_modules)
        .unwrap_or(false);

    let options = crate::bundler::LibBuildOptions {
        project_root: root_dir.to_path_buf(),
        out_dir,
        formats,
        conditions,
        extra_externals: std::collections::HashSet::new(),
        preserve_modules,
    };

    let start = std::time::Instant::now();
    let result = crate::bundler::build_library(options).context("Library build failed")?;

    println!(
        "Library build complete in {:.0}ms: {} file(s)",
        start.elapsed().as_millis(),
        result.entries.len(),
    );
    for entry in &result.entries {
        let size_kb = entry.code.len() as f64 / 1024.0;
        let rel = entry
            .path
            .strip_prefix(root_dir)
            .unwrap_or(&entry.path)
            .display();
        println!("  {} ({:?}) → {} ({:.1} KB)", entry.subpath, entry.format, rel, size_kb);
    }

    Ok(())
}

/// GH #3248 — Classify a project as library based on `project.json`'s
/// `projectType` field. The prior implementation collapsed read, parse,
/// and field-shape errors into `false` via chained `.ok()`, which made
/// every malformed `project.json` produce the wrong bundle output
/// (libraries got their deps bundled, apps got them externalised) with
/// no log trail. This helper distinguishes:
///
/// Format the warning emitted when a CLI numeric flag's value fails
/// to parse, before the caller falls back to a built-in default. Names
/// the flag, the rejected literal, the expected type, and the
/// GH #3596 — resolve `$HOME/.jet-store` without silently collapsing
/// `std::env::VarError` discriminants to `./.jet-store` (CWD), which
/// would silently make `jet store prune` operate on the wrong path.
///
/// Cases:
/// - `Ok(home)` → `Ok(<home>/.jet-store)`.
/// - `Err(NotPresent)` → `Err` with an actionable message naming HOME +
///   suggesting `--store-path` or setting HOME. The user genuinely has
///   no HOME (daemon, fresh shell).
/// - `Err(NotUnicode(_))` → `Err` with a message DISTINGUISHING the
///   misconfiguration from NotPresent.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub(crate) fn resolve_store_path_from_home(
    current: Result<String, std::env::VarError>,
) -> Result<PathBuf> {
    match current {
        Ok(home) => Ok(PathBuf::from(&home).join(".jet-store")),
        Err(std::env::VarError::NotPresent) => {
            anyhow::bail!("{}", format_store_home_err("not-present"));
        }
        Err(std::env::VarError::NotUnicode(_)) => {
            anyhow::bail!("{}", format_store_home_err("not-unicode"));
        }
    }
}

/// GH #3596 — build the error message for a failed HOME lookup during
/// `jet store prune` store-path resolution. Extracted so the wording
/// (tag + kind + actionable suggestion) is unit-testable.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub(crate) fn format_store_home_err(observed_kind: &str) -> String {
    format!(
        "GH #3596 jet store prune: HOME observed as {observed_kind}; \
         cannot resolve ~/.jet-store. The prior implementation silently \
         fell back to ./.jet-store (CWD-relative) which is almost certainly \
         not the package-store directory the user intended to prune. \
         Set HOME to your home directory, or extend this command with a \
         --store-path flag to pass the store location explicitly."
    )
}

/// GH #3721 — build the error message for an unknown / missing
/// `jet store <subcommand>`. Extracted so the wording is unit-testable
/// and distinct between "typo'd subcommand" and "bare `jet store`".
///
/// The prior implementation wrote "Unknown store subcommand" to STDOUT
/// and returned `Ok(())` → exit 0, so a CI step running `jet store gc`
/// (typo) silently succeeded. Every other Unknown-subcommand branch
/// (config, report, trace, e2e, browser) bails via stderr + exit 1.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub(crate) fn format_unknown_store_subcommand_err(other: Option<&str>) -> String {
    match other {
        Some(name) => format!(
            "GH #3721 jet store: unknown subcommand '{name}'. The only \
             supported `jet store` subcommand right now is `prune` (GC the \
             ~/.jet-store package store). The prior code printed this \
             message to STDOUT and returned exit 0 — a `jet store gc` typo \
             in a CI step then silently succeeded. Try `jet store prune`."
        ),
        None => "GH #3721 jet store: missing subcommand. The only \
                 supported `jet store` subcommand right now is `prune` \
                 (GC the ~/.jet-store package store). Try `jet store \
                 prune`."
            .to_string(),
    }
}

/// GH #3604 — `jet report view` produced an invalid relative `file://`
/// URL when canonicalize() failed (e.g., restricted parent dir). Recover
/// by joining the index path onto CWD; if both fail, bail.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub(crate) fn absolutize_report_index(
    index_html: &Path,
    cwd_result: std::io::Result<PathBuf>,
) -> Result<PathBuf> {
    if index_html.is_absolute() {
        return Ok(index_html.to_path_buf());
    }
    let cwd = cwd_result.with_context(|| {
        "GH #3604 jet report view: canonicalize() failed and current_dir() \
         also failed — cannot construct an absolute file:// URL for the \
         report path"
    })?;
    Ok(cwd.join(index_html))
}

/// GH #3604 — build the contextual error for an absolutize failure so the
/// user sees both the original canonicalize error and the GH issue tag.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub(crate) fn format_report_view_abs_err(index_html: &Path, canon_err: &std::io::Error) -> String {
    format!(
        "GH #3604 jet report view: canonicalize({}) failed ({}); the prior \
         implementation silently produced a malformed file:// URL with a \
         relative path",
        index_html.display(),
        canon_err
    )
}

/// underlying `ParseIntError`, and tags `GH #3548` so users grepping
/// "my --port was ignored" or "jet dev port=3000 even though I asked
/// for X" can land on this line. Extracted for unit-test pinning.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub(crate) fn format_cli_flag_parse_warn(
    flag: &str,
    value: &str,
    type_name: &str,
    err: &std::num::ParseIntError,
) -> String {
    format!(
        "GH #3548 CLI flag {flag}={value:?} could not be parsed as {type_name}: {err}; the flag will be IGNORED and the built-in default will be used instead. Re-run with a numeric value (e.g. {flag}=8080)."
    )
}

/// Parse a CLI flag's string argument as a numeric type, surfacing
/// `ParseIntError` via [`format_cli_flag_parse_warn`] before returning
/// `None`. Designed as a drop-in for the prior `.parse::<T>().ok()`
/// pattern — the caller's existing `.unwrap_or(default)` continues to
/// apply, but operators now see a `tracing::warn!` tagged `GH #3548`
/// when their value is rejected. GH #3548.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub(crate) fn parse_cli_numeric_flag<T>(flag: &str, value: &str) -> Option<T>
where
    T: std::str::FromStr<Err = std::num::ParseIntError>,
{
    match value.parse::<T>() {
        Ok(v) => Some(v),
        Err(err) => {
            tracing::warn!(
                target: "jet::cli::flags",
                flag = flag,
                value = value,
                expected = std::any::type_name::<T>(),
                error = %err,
                "{}",
                format_cli_flag_parse_warn(flag, value, std::any::type_name::<T>(), &err)
            );
            None
        }
    }
}

fn merge_dev_proxy_rules<'a>(
    mut config_proxy: HashMap<String, String>,
    cli_rules: impl IntoIterator<Item = &'a str>,
) -> Result<HashMap<String, String>> {
    for raw in cli_rules {
        let (prefix, target) = parse_dev_proxy_rule(raw)?;
        config_proxy.insert(prefix, target);
    }
    Ok(config_proxy)
}

fn parse_dev_proxy_rule(raw: &str) -> Result<(String, String)> {
    let Some((prefix_raw, target_raw)) = raw.split_once('=') else {
        anyhow::bail!(
            "invalid --proxy value {raw:?}; expected PATH=URL, e.g. --proxy /api=http://localhost:3200"
        );
    };
    let prefix_trimmed = prefix_raw.trim();
    let prefix = if prefix_trimmed == "/" {
        "/".to_string()
    } else {
        prefix_trimmed.trim_end_matches('/').to_string()
    };
    let target = target_raw.trim().trim_end_matches('/').to_string();

    if prefix.is_empty() || !prefix.starts_with('/') {
        anyhow::bail!(
            "invalid --proxy path {prefix_raw:?}; proxy paths must start with '/', e.g. /api"
        );
    }
    if target.is_empty() || !(target.starts_with("http://") || target.starts_with("https://")) {
        anyhow::bail!(
            "invalid --proxy target {target_raw:?}; proxy targets must start with http:// or https://"
        );
    }

    Ok((prefix, target))
}

async fn handle_serve_command(root_dir: &PathBuf, m: &ArgMatches) -> Result<()> {
    if let Some(("shutdown", sm)) = m.subcommand() {
        if let Some(port) = sm
            .get_one::<String>("port")
            .and_then(|s| parse_cli_numeric_flag::<u16>("--port", s))
        {
            let host = sm
                .get_one::<String>("host")
                .cloned()
                .unwrap_or_else(|| "127.0.0.1".to_string());
            shutdown_dev_server(&host, port).await?;
            return Ok(());
        }

        let (session, body) = crate::dev_server::serve_process::shutdown_active(root_dir).await?;
        print_shutdown_result(&session.host, session.port, body);
        return Ok(());
    }

    let jet_config = crate::task_runner::config::JetConfig::load(root_dir).unwrap_or_default();
    let cli_port = m
        .get_one::<String>("port")
        .and_then(|s| parse_cli_numeric_flag::<u16>("--port", s));
    let port = cli_port.or(jet_config.dev.port).unwrap_or(3000);
    let host = m
        .get_one::<String>("host")
        .cloned()
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let prod = m.get_flag("prod");
    let wasm = m.get_flag("wasm");
    if std::env::var_os("JET_SERVE_CHILD").is_some() {
        if !prod {
            anyhow::bail!("JET_SERVE_CHILD is only valid for `jet serve --prod`");
        }
        return crate::dev_server::prod_static::serve(
            root_dir,
            crate::dev_server::prod_static::ProdOptions {
                host,
                port,
                target: crate::dev_server::serve_process::serve_session_target(prod, wasm),
            },
        )
        .await;
    }

    launch_detached_serve(root_dir, &host, port, prod, wasm, m.get_flag("debug")).await
}

async fn launch_detached_serve(
    root_dir: &Path,
    host: &str,
    port: u16,
    prod: bool,
    wasm: bool,
    debug: bool,
) -> Result<()> {
    let launch = crate::dev_server::serve_process::launch_detached(
        crate::dev_server::serve_process::ServeProcessOptions {
            root_dir: root_dir.to_path_buf(),
            host: host.to_string(),
            port,
            prod,
            wasm,
            debug,
            ready_timeout: Duration::from_secs(30),
        },
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&launch.payload())?);
    Ok(())
}

async fn shutdown_dev_server(host: &str, port: u16) -> Result<()> {
    let body = crate::dev_server::serve_process::shutdown_host_port(host, port).await?;
    print_shutdown_result(host, port, body);
    Ok(())
}

fn print_shutdown_result(host: &str, port: u16, body: String) {
    let url = crate::dev_server::serve_process::shutdown_url(host, port);
    println!("[jet] shutdown requested: {url}");
    let body = body.trim();
    if !body.is_empty() {
        println!("{body}");
    }
}

/// - `NotFound` → silent `false` (the call site already gates on
///   `exists()`, but the race window remains)
/// - other IO / parse / field-shape errors → `tracing::warn!` then
///   `false` (fall through to graph + heuristic)
/// - valid `"library"` or `"lib"` → `true`
/// - any other valid value → `false`
fn read_project_type_is_lib(path: &Path) -> bool {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return false,
        Err(err) => {
            tracing::warn!(
                target: "jet::cli::build",
                path = %path.display(),
                error = %err,
                "GH #3248 failed to read project.json; falling back to graph/heuristic classifier"
            );
            return false;
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(err) => {
            tracing::warn!(
                target: "jet::cli::build",
                path = %path.display(),
                error = %err,
                "GH #3248 failed to parse project.json; falling back to graph/heuristic classifier"
            );
            return false;
        }
    };
    match parsed.get("projectType") {
        Some(serde_json::Value::String(s)) => s == "library" || s == "lib",
        Some(other) => {
            tracing::warn!(
                target: "jet::cli::build",
                path = %path.display(),
                value = ?other,
                "GH #3248 project.json projectType is not a string; falling back to graph/heuristic classifier"
            );
            false
        }
        None => false,
    }
}

/// JSON-value-kind label for non-string `package.json` `scripts` entries.
/// Exposed `pub(crate)` so the gh3789 tests can pin both the kind names
/// and the rename surface.
// @spec gh3789 — silent non-string script-value fallback in list_scripts
pub(crate) fn package_script_value_kind(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// Format the warn string emitted when `list_scripts` sees a non-string
/// `scripts.<name>` value in `package.json`. Pinned in tests to keep the
/// helper name discoverable across renames.
// @spec gh3789
pub(crate) fn format_cli_run_non_string_script_value_warn(name: &str, kind: &str) -> String {
    format!(
        "gh3789: package.json scripts.{name:?} has non-string value of \
         JSON kind {kind:?}; npm/yarn/pnpm reject this — rendering \
         sentinel in `jet run --list`"
    )
}

/// Sentinel surfaced for non-string `scripts.<name>` values. The
/// trailing `<kind>` keeps the gap visible to a human reading
/// `jet run --list`; the accompanying warn is the audit trail.
// @spec gh3789
pub(crate) fn format_non_string_script_value_sentinel(kind: &str) -> String {
    format!("<non-string script value: {kind}>")
}

/// Coerce a `package.json` `scripts.<name>` value into the display string
/// used by `list_scripts`. The legacy code did
/// `cmd.as_str().unwrap_or("")`, which silently rendered every non-string
/// value as a blank column — indistinguishable from an empty string.
// @spec gh3789
pub(crate) fn coerce_script_command_or_warn(name: &str, v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        other => {
            let kind = package_script_value_kind(other);
            tracing::warn!(
                "{}",
                format_cli_run_non_string_script_value_warn(name, kind)
            );
            format_non_string_script_value_sentinel(kind)
        }
    }
}

/// GH #3799 — warn message shown when `--sourcemap <raw>` carries a value
/// outside the documented set (`none|inline|hidden|external`). The prior
/// wildcard `_ =>` arm silently coerced typos onto External and the
/// operator only discovered the mismatch after shipping.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub(crate) fn format_cli_build_sourcemap_unknown_warn(raw: &str) -> String {
    format!(
        "gh3799: jet build saw unknown --sourcemap value {:?}; \
         expected one of \"none\", \"inline\", \"hidden\", \"external\"; \
         falling back to External — the shipped bundle may not match \
         the operator's intended sourcemap policy",
        raw
    )
}

/// GH #3799 — coerce the CLI `--sourcemap` flag into a typed
/// [`crate::bundler::types::SourceMapOption`].
///
/// - `None` (flag omitted) → External (silent default).
/// - `Some("none"|"inline"|"hidden"|"external")` → matching variant (silent).
/// - `Some(other)` → emit a `tracing::warn!` and fall back to External so
///   the existing wildcard behaviour is preserved on the legacy fallback arm.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub(crate) fn coerce_sourcemap_mode_or_warn(
    value: Option<&str>,
) -> crate::bundler::types::SourceMapOption {
    use crate::bundler::types::SourceMapOption;
    match value {
        None | Some("external") => SourceMapOption::External,
        Some("none") => SourceMapOption::None,
        Some("inline") => SourceMapOption::Inline,
        Some("hidden") => SourceMapOption::Hidden,
        Some(other) => {
            tracing::warn!("{}", format_cli_build_sourcemap_unknown_warn(other));
            SourceMapOption::External
        }
    }
}

/// GH #3803 — warn shown when the resolved entry path for a build's
/// source-map carries non-UTF-8 bytes. The prior `to_string_lossy()`
/// silently substituted U+FFFD so two entry paths differing only in
/// non-UTF-8 bytes (e.g. per-locale build entries on a non-UTF-8
/// filesystem) lossy onto the same `sources[]` entry — devtools then
/// open a file the operator never edited.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub(crate) fn format_cli_build_sourcemap_non_utf8_entry_warn(
    entry: &std::path::Path,
    lossy: &str,
) -> String {
    format!(
        "gh3803: jet build sourcemap saw non-UTF-8 entry path entry={:?}; \
         lossy form is {:?}; two entries differing only in non-UTF-8 bytes \
         lossy onto the same sources[] string so devtools may open the \
         wrong file from a stack trace",
        entry, lossy
    )
}

/// GH #3803 — coerce an entry path into a UTF-8 string for the
/// source-map `sources[]` array. UTF-8 paths pass through silently;
/// non-UTF-8 paths still render (via `to_string_lossy`) but emit a
/// `tracing::warn!` first so operators can spot lossy-collisions in
/// devtools / Sentry breakpoint targets.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub(crate) fn coerce_sourcemap_entry_path_or_warn(entry: &std::path::Path) -> String {
    match entry.to_str() {
        Some(s) => s.to_string(),
        None => {
            let lossy = entry.to_string_lossy().into_owned();
            tracing::warn!(
                target: "jet::cli::build",
                entry = %entry.display(),
                lossy = %lossy,
                "{}",
                format_cli_build_sourcemap_non_utf8_entry_warn(entry, &lossy)
            );
            lossy
        }
    }
}

/// List all available scripts from package.json and jet.toml pipeline.
/// Equivalent to `npm run` (no arguments).
fn list_scripts(root_dir: &PathBuf) -> Result<()> {
    // package.json scripts
    let pkg_path = root_dir.join("package.json");
    if pkg_path.exists() {
        let content = std::fs::read_to_string(&pkg_path)?;
        // GH #3218 — the prior `if let Ok(pkg) = ...` silently swallowed
        // malformed package.json. `jet run` then displayed a clean "no
        // scripts found" result (no header) when the actual cause was a
        // parse error, indistinguishable from a project with zero
        // scripts. Mirrors the jet dev (#2940), jet build (#3061),
        // jet test (#3065), and ScriptRunner::load_pkg_scripts (#3170)
        // fixes — surface the parse error to stderr and continue to the
        // pipeline half so partial output still renders.
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(pkg) => {
                if let Some(scripts) = pkg.get("scripts").and_then(|s| s.as_object()) {
                    if !scripts.is_empty() {
                        println!("Scripts available via `jet run`:\n");
                        let max_len = scripts.keys().map(|k| k.len()).max().unwrap_or(0);
                        for (name, cmd) in scripts {
                            let cmd_str = coerce_script_command_or_warn(name, cmd);
                            println!("  {:<width$}  {}", name, cmd_str, width = max_len);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[jet run] Failed to parse {}: {e}", pkg_path.display());
                eprintln!(
                    "[jet run] Scripts from package.json will NOT be listed until the parse error is fixed."
                );
            }
        }
    }

    // jet.toml pipeline tasks
    // Surface parse errors instead of silently dropping the pipeline listing.
    // Mirrors the jet dev (#2940), jet build (#3061), jet test (#3065) fixes; GH #3069.
    match crate::task_runner::config::JetConfig::load(root_dir) {
        Ok(config) => {
            if !config.pipeline.is_empty() {
                println!("\nPipeline tasks (jet.toml):\n");
                for (name, def) in &config.pipeline {
                    let deps = if def.depends_on.is_empty() {
                        String::new()
                    } else {
                        format!(" (deps: {})", def.depends_on.join(", "))
                    };
                    println!("  {}{}", name, deps);
                }
            }
        }
        Err(e) => {
            eprintln!("[jet run] Failed to parse jet.toml: {e:#}");
            eprintln!(
                "[jet run] Pipeline tasks from the file will NOT be listed until the parse error is fixed."
            );
        }
    }

    Ok(())
}

/// Handle `jet run <target>`: resolve as script, file, or task.
async fn handle_run(
    root_dir: &PathBuf,
    target: &str,
    args: &[String],
    watch: bool,
    filter: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    let runner = crate::runner::ScriptRunner::new(root_dir.clone());

    // 1. Check package.json scripts
    if runner.has_script(target) {
        let result = runner.run_script(target, args).await?;
        print!("{}", result.stdout);
        eprint!("{}", result.stderr);
        if result.exit_code != 0 {
            anyhow::bail!("Script '{}' exited with code {}", target, result.exit_code);
        }
        return Ok(());
    }

    // 2. Check if it's a file
    if runner.is_file(target) {
        let path = std::path::Path::new(target);
        let result = runner.run_file(path, args, watch).await?;
        print!("{}", result.stdout);
        eprint!("{}", result.stderr);
        if result.exit_code != 0 {
            anyhow::bail!("File '{}' exited with code {}", target, result.exit_code);
        }
        return Ok(());
    }

    // 3. Check task runner (jet.toml pipeline)
    match crate::task_runner::TaskRunner::new(root_dir) {
        Ok(tr) => {
            if tr.has_task(target) {
                let results = tr.run(target, filter, dry_run).await?;
                crate::task_runner::TaskRunner::print_summary(&results);
                let any_failed = results
                    .iter()
                    .any(|r| r.status == crate::task_runner::TaskStatus::Failed);
                if any_failed {
                    anyhow::bail!("Some tasks failed");
                }
                return Ok(());
            }
        }
        Err(err) if root_dir.join("jet.toml").exists() => {
            return Err(err).context("Failed to load task runner");
        }
        Err(_) => {}
    }

    anyhow::bail!(
        "Target '{}' not found as a script, file, or task. \
         Check package.json scripts or jet.toml pipeline.",
        target
    )
}

/// Find the project entry point by checking common locations
fn find_entry_point(root_dir: &PathBuf) -> Result<PathBuf> {
    let candidates = [
        "src/main.ts",
        "src/main.tsx",
        "src/index.ts",
        "src/index.tsx",
        "src/index.js",
        "src/main.js",
    ];

    for candidate in &candidates {
        let path = root_dir.join(candidate);
        if path.exists() {
            return Ok(PathBuf::from(candidate));
        }
    }

    anyhow::bail!("No entry point found. Tried: {}", candidates.join(", "))
}

fn content_hash_prefix(content: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let hex = format!("{:x}", hasher.finalize());
    hex[..8].to_string()
}

fn write_bundle_assets(
    output_dir: &Path,
    assets: &[crate::bundler::types::Asset],
) -> Result<Vec<String>> {
    let mut css_filenames = Vec::new();

    for asset in assets {
        let output_path = output_dir.join(&asset.filename);
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&output_path, &asset.content)?;

        if asset.asset_type == crate::bundler::types::AssetType::Css {
            if !css_filenames.contains(&asset.filename) {
                css_filenames.push(asset.filename.clone());
            }
        }
    }

    Ok(css_filenames)
}

fn append_css_side_effect_assets(
    root_dir: &Path,
    entry_path: &Path,
    specifiers: &[String],
    minify: bool,
    assets: &mut Vec<crate::bundler::types::Asset>,
) -> Result<()> {
    if specifiers.is_empty() {
        return Ok(());
    }

    let config = match crate::css::TailwindConfig::load(root_dir) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("[jet build] Failed to parse Tailwind config: {e:#}");
            eprintln!("[jet build] Continuing with built-in Tailwind defaults; your tailwind.config.js / [css.tailwind] settings will NOT take effect until the parse error is fixed.");
            crate::css::TailwindConfig::default()
        }
    };
    let pipeline = crate::css::CssPipeline::new(root_dir.to_path_buf(), config, minify);
    let mut seen_paths = std::collections::HashSet::new();

    for specifier in specifiers {
        let css_path = resolve_css_side_effect_import_path(root_dir, entry_path, specifier)?;
        let css_path = std::fs::canonicalize(&css_path)
            .with_context(|| format!("resolving CSS side-effect import `{specifier}`"))?;
        if !seen_paths.insert(css_path.clone()) {
            continue;
        }

        let output = pipeline
            .process(&css_path)
            .with_context(|| format!("processing CSS side-effect import `{specifier}`"))?;
        let stem = css_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("style");
        let filename = format!("{}.{}.css", stem, output.hash);
        if assets.iter().any(|asset| asset.filename == filename) {
            continue;
        }
        assets.push(crate::bundler::types::Asset {
            filename,
            content: output.css.into_bytes(),
            asset_type: crate::bundler::types::AssetType::Css,
        });
    }

    Ok(())
}

fn resolve_css_side_effect_import_path(
    root_dir: &Path,
    entry_path: &Path,
    specifier: &str,
) -> Result<PathBuf> {
    let path = if let Some(root_relative) = specifier.strip_prefix('/') {
        root_dir.join(root_relative)
    } else if specifier.starts_with("./") || specifier.starts_with("../") {
        entry_path.parent().unwrap_or(root_dir).join(specifier)
    } else {
        anyhow::bail!(
            "CSS side-effect import `{specifier}` is not yet supported by jet build; use a relative or root-relative CSS path"
        );
    };

    if path.is_file() {
        return Ok(path);
    }

    anyhow::bail!(
        "CSS side-effect import `{specifier}` from {} could not be resolved",
        entry_path.display()
    )
}

fn copy_public_dir(root_dir: &Path, output_dir: &Path) -> Result<()> {
    let public_dir = root_dir.join("public");
    if !public_dir.is_dir() {
        return Ok(());
    }

    copy_public_dir_contents(&public_dir, &public_dir, output_dir)
}

fn copy_public_dir_contents(base_dir: &Path, current_dir: &Path, output_dir: &Path) -> Result<()> {
    for entry in std::fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let relative_path = path.strip_prefix(base_dir)?;
        let output_path = output_dir.join(relative_path);

        if file_type.is_dir() {
            std::fs::create_dir_all(&output_path)?;
            copy_public_dir_contents(base_dir, &path, output_dir)?;
        } else if file_type.is_file() {
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&path, &output_path)?;
        }
    }

    Ok(())
}

fn emit_build_index_html(
    output_dir: &Path,
    template: &str,
    entry: &Path,
    js_filename: &str,
    css_filenames: &[String],
) -> Result<()> {
    let html = render_build_index_html(&template, entry, js_filename, css_filenames);
    std::fs::write(output_dir.join("index.html"), html)?;
    Ok(())
}

fn render_build_index_html(
    template: &str,
    entry: &Path,
    js_filename: &str,
    css_filenames: &[String],
) -> String {
    crate::frontend::render_js_index_html(template, entry, js_filename, css_filenames)
}

/// Build a `FlagSnapshot` from clap matches for the `build` subcommand.
/// Pulled into its own helper so the validation-table tests can share
/// the exact production logic.
///
/// @spec .aw/tech-design/projects/jet/logic/multi-target/build-targets.md
/// (Slice 4 — validation table coverage).
fn build_flag_snapshot_from_matches(m: &ArgMatches) -> crate::build_target::FlagSnapshot {
    crate::build_target::FlagSnapshot {
        wasm: m.get_flag("wasm"),
        minify: m.get_flag("minify"),
        // `--sourcemap` defaults to `external`. Only count it as
        // explicitly "set" if the user picked something else.
        sourcemap_set: m
            .get_one::<String>("sourcemap")
            .map(|s| s.as_str())
            .map(|s| s != "external")
            .unwrap_or(false),
        splitting: m.get_flag("splitting"),
        drop_set: m
            .get_many::<String>("drop")
            .map(|v| v.count() > 0)
            .unwrap_or(false),
    }
}

fn build_minify_enabled_from_matches(m: &ArgMatches) -> bool {
    !m.get_flag("no-minify")
}

/// Produce the typed error returned by `jet check` until TypeScript
/// type checking actually lands. Extracted so the diagnostic message
/// is unit-testable without spawning the binary.
///
/// @spec projects/jet/docs/check-exits-non-zero-while-unimplemented.md#interface
/// @issue #1316
fn check_not_implemented_error() -> anyhow::Error {
    anyhow::anyhow!(
        "`jet check` is not yet implemented (TypeScript type checking is on \
         the roadmap — tracked by #1316). Until it lands, run `tsc --noEmit` \
         directly. Exiting non-zero so this stub does not silently mask \
         frontend validation failures in CI."
    )
}

/// GH #3712 — `jet build --define KEY` (missing `=VALUE`, or typo'd
/// with `:` instead of `=`) previously silently dropped the entry via
/// `if let Some((k, v)) = def.split_once('=')`. The bundle shipped
/// without the constant replaced; the build log said "complete" and
/// the user only noticed in production. esbuild errors out on missing
/// `=`; mirror that. Extracted so the wording is unit-testable.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub(crate) fn parse_define_arg(arg: &str) -> std::result::Result<(String, String), String> {
    match arg.split_once('=') {
        Some((key, value)) if !key.is_empty() => Ok((key.to_string(), value.to_string())),
        Some((_empty_key, _)) => Err(format!(
            "GH #3712 jet build --define {arg:?}: KEY is empty. \
             Expected --define KEY=VALUE (e.g. --define VERSION=1.2.3). \
             The bundle would silently ship without the constant replaced."
        )),
        None => Err(format!(
            "GH #3712 jet build --define {arg:?}: missing '=' separator. \
             Expected --define KEY=VALUE (e.g. --define VERSION=1.2.3). \
             Common typos: `--define KEY` (forgot value) and \
             `--define KEY:VALUE` (used colon instead of equals). \
             The prior silent drop shipped bundles with the constant \
             unreplaced; CI never caught the typo. Mirrors esbuild's \
             behaviour of erroring on malformed --define."
        )),
    }
}

/// GH #3708 — `jet build --watch` previously hit
/// `let _watch = m.get_flag("watch");`, which silently discarded
/// the flag. clap accepted `-w` / `--watch`, the bundler ran once,
/// printed "Build complete", and exited 0 — contradicting the module
/// docstring which advertises `cclab jet build --watch` as a working
/// mode, and asymmetric with `jet test --watch` / `jet dev --watch`
/// which both honour the flag.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub(crate) fn format_build_watch_not_implemented_warn() -> String {
    "warning [GH #3708]: jet build --watch is currently a no-op — the \
     bundler runs once and exits. For continuous rebuilds use \
     `jet dev` (HMR + dev server) or `jet test --watch` (re-runs the \
     test suite on file change). `jet build --watch` will be wired in \
     once #3708 lands; until then the flag is parsed and ignored."
        .to_string()
}

/// GH #3705 — `jet build --splitting` previously hit
/// `let _ = splitting; // TODO: integrate with chunk splitter`, which
/// silently discarded the flag. Build said "complete" and the user
/// only noticed when they inspected `dist/` and found one un-split
/// bundle where they expected chunks at dynamic-import boundaries.
/// Surface the no-op explicitly with a tagged warn naming both
/// the issue tag and the downstream tracker (#1089). Once the
/// chunk splitter is wired in, this warn deletes itself.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub(crate) fn format_splitting_not_implemented_warn() -> String {
    "warning [GH #3705]: jet build --splitting is currently a no-op — \
     the chunk splitter is not yet wired into the bundler pipeline \
     (tracked by #1089). The build produced ONE file, not chunks at \
     dynamic-import boundaries. Drop --splitting until #1089 lands, \
     or expect first-load size budgets, dynamic-import semantics, and \
     bundle-size CI gates that assume splitting is on to misbehave."
        .to_string()
}

async fn prebundle_after_install(root_dir: PathBuf) -> Result<()> {
    crate::dev_server::prebundle::PreBundler::new(root_dir)
        .prebundle_deps()
        .await
        .map(|_| ())
}

#[cfg(test)]
mod build_index_html_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn build_index_rewrites_vite_entry_script_to_hashed_bundle() {
        let template = r#"<!doctype html>
<html>
  <head></head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>"#;

        let html =
            render_build_index_html(template, Path::new("src/main.tsx"), "main.12345678.js", &[]);

        assert!(html.contains(r#"<script type="module" src="./main.12345678.js"></script>"#));
        assert!(!html.contains("/src/main.tsx"));
    }

    #[test]
    fn build_index_injects_script_when_template_has_no_module_script() {
        let template = r#"<!doctype html><html><body><div id="root"></div></body></html>"#;

        let html =
            render_build_index_html(template, Path::new("src/main.ts"), "main.abcdef12.js", &[]);

        assert!(html.contains(r#"<script type="module" src="./main.abcdef12.js"></script>"#));
        assert!(html.find("./main.abcdef12.js").unwrap() < html.find("</body>").unwrap());
    }

    #[test]
    fn build_index_links_css_assets_in_head() {
        let template = r#"<!doctype html>
<html>
  <head><title>App</title></head>
  <body><script type="module" src="./src/main.tsx"></script></body>
</html>"#;
        let css = vec!["main.deadbeef.css".to_string()];

        let html = render_build_index_html(
            template,
            Path::new("src/main.tsx"),
            "main.cafe1234.js",
            &css,
        );

        assert!(html.contains(r#"<link rel="stylesheet" href="./main.deadbeef.css" />"#));
        assert!(html.find("./main.deadbeef.css").unwrap() < html.find("</head>").unwrap());
    }

    #[test]
    fn css_side_effect_imports_emit_build_assets_once() {
        let root = TempDir::new().unwrap();
        let src = root.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        let entry = src.join("main.tsx");
        let css = src.join("main.css");
        std::fs::write(&entry, "import './main.css';\n").unwrap();
        std::fs::write(&css, ".status { color: rgb(20, 80, 120); }\n").unwrap();
        let mut assets = Vec::new();

        append_css_side_effect_assets(
            root.path(),
            &entry,
            &["./main.css".to_string(), "./main.css".to_string()],
            false,
            &mut assets,
        )
        .unwrap();

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].asset_type, crate::bundler::types::AssetType::Css);
        assert!(assets[0].filename.starts_with("main."));
        assert!(assets[0].filename.ends_with(".css"));

        let out = root.path().join("dist");
        let css_filenames = write_bundle_assets(&out, &assets).unwrap();
        assert_eq!(css_filenames, vec![assets[0].filename.clone()]);
        let written = std::fs::read_to_string(out.join(&assets[0].filename)).unwrap();
        assert!(written.contains("status"));
    }

    #[test]
    fn content_hash_prefix_is_stable_sha256_prefix() {
        assert_eq!(
            content_hash_prefix("console.log('basic');"),
            content_hash_prefix("console.log('basic');")
        );
        assert_eq!(
            content_hash_prefix("console.log('basic');"),
            "77561beb".to_string()
        );
        assert_ne!(
            content_hash_prefix("console.log('basic');"),
            content_hash_prefix("console.log('changed');")
        );
    }

    #[test]
    fn copy_public_dir_preserves_nested_assets() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join("app");
        let output = temp.path().join("dist");
        let nested = root.join("public/icons");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(root.join("public/favicon.svg"), "<svg />").unwrap();
        std::fs::write(nested.join("logo.svg"), "<svg id=\"logo\" />").unwrap();

        copy_public_dir(&root, &output).unwrap();

        assert_eq!(
            std::fs::read_to_string(output.join("favicon.svg")).unwrap(),
            "<svg />"
        );
        assert_eq!(
            std::fs::read_to_string(output.join("icons/logo.svg")).unwrap(),
            "<svg id=\"logo\" />"
        );
    }
}

#[cfg(test)]
mod check_handler_tests {
    //! Regression for #1316: `jet check` must surface a non-zero
    //! "not yet implemented" diagnostic instead of printing
    //! "under development" and exiting 0.
    //!
    //! Spec: projects/jet/docs/check-exits-non-zero-while-unimplemented.md
    use super::*;

    #[test]
    fn check_not_implemented_error_mentions_issue_link() {
        let err = check_not_implemented_error();
        let msg = format!("{err}");
        assert!(
            msg.contains("not yet implemented"),
            "diagnostic must say `not yet implemented`, got: {msg}"
        );
        assert!(
            msg.contains("#1316"),
            "diagnostic must mention tracking issue #1316, got: {msg}"
        );
    }

    #[test]
    fn check_not_implemented_error_is_non_empty_and_not_panicking() {
        let err = check_not_implemented_error();
        let msg = format!("{err}");
        assert!(
            !msg.trim().is_empty(),
            "diagnostic must be non-empty so users see why the exit was non-zero"
        );
    }
}

#[cfg(test)]
mod e2e_command_contract_tests {
    //! Parser contract for `.aw/tech-design/projects/jet/specs/2385.md`.

    use super::*;
    use clap::error::ErrorKind;

    fn help_text(args: &[&str]) -> String {
        let err = command().try_get_matches_from(args).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::DisplayHelp);
        err.to_string()
    }

    // @spec .aw/tech-design/projects/jet/specs/2385.md#test-plan
    #[test]
    fn top_level_help_lists_product_flow_e2e_namespace() {
        let help = help_text(&["jet", "--help"]);
        assert!(help.contains("e2e"), "top-level help must list e2e: {help}");
        assert!(
            help.contains("product-flow E2E"),
            "top-level help must describe product-flow E2E: {help}"
        );
    }

    // @spec .aw/tech-design/projects/jet/specs/2385.md#test-plan
    #[test]
    fn e2e_run_help_describes_agent_ci_mode() {
        let help = help_text(&["jet", "e2e", "run", "--help"]);
        assert!(
            help.contains("Playwright-like automation mode"),
            "run help must name Playwright-like mode: {help}"
        );
        assert!(help.contains("CI"), "run help must mention CI: {help}");
        assert!(
            help.contains("agents"),
            "run help must mention agents: {help}"
        );
        assert!(
            help.contains("--serve") && help.contains("--base-url"),
            "run help must expose AUT serve/base-url controls: {help}"
        );
        assert!(
            !help.contains("desktop review app; implementation is tracked by #2390"),
            "run help must not route users to desktop open-mode implementation: {help}"
        );
    }

    // @spec .aw/tech-design/projects/jet/specs/2385.md#test-plan
    #[test]
    fn e2e_open_help_describes_desktop_shell_and_visible_jet_browser_without_cypress_runtime() {
        let help = help_text(&["jet", "e2e", "open", "--help"]);
        assert!(
            help.contains("desktop-style review shell"),
            "open help must name the desktop-style review shell: {help}"
        );
        assert!(
            help.contains("visible controlled Jet Browser"),
            "open help must name the visible controlled Jet Browser: {help}"
        );
        assert!(
            help.contains("does not use the Cypress runtime or Cypress Cloud"),
            "open help must reject Cypress runtime/cloud dependency: {help}"
        );
    }

    // @spec .aw/tech-design/projects/jet/specs/2385.md#test-plan
    #[test]
    fn test_help_keeps_frontend_test_boundary() {
        let help = help_text(&["jet", "test", "--help"]);
        assert!(
            help.contains("frontend unit, component, and integration-style tests"),
            "test help must keep frontend-test boundary: {help}"
        );
    }

    #[test]
    fn serve_command_exposes_agent_first_session_surface() {
        let help = help_text(&["jet", "serve", "--help"]);
        assert!(
            help.contains("detached") && help.contains("--wasm") && help.contains("--prod"),
            "serve help must expose detached agent mode plus wasm/prod target flags: {help}"
        );
        assert!(
            help.contains("shutdown"),
            "serve help must expose the session shutdown control surface: {help}"
        );
    }

    #[test]
    fn serve_command_is_observable_to_dispatch() {
        let matches = command()
            .try_get_matches_from(["jet", "serve", "--port", "0"])
            .expect("serve parses");
        let (name, sm) = matches.subcommand().expect("top-level subcommand");
        assert_eq!(
            name, "serve",
            "dispatch must distinguish `jet serve` from legacy `jet dev`"
        );
        assert_eq!(sm.get_one::<String>("port").map(String::as_str), Some("0"));
    }

    #[test]
    fn serve_shutdown_can_use_session_file_without_port() {
        let matches = command()
            .try_get_matches_from(["jet", "serve", "shutdown"])
            .expect("serve shutdown parses without --port");
        let (name, sm) = matches.subcommand().expect("top-level subcommand");
        assert_eq!(name, "serve");
        let (_, shutdown) = sm.subcommand().expect("serve subcommand");
        assert!(
            shutdown.get_one::<String>("port").is_none(),
            "serve shutdown should allow session-file based shutdown"
        );
    }

    #[test]
    fn dev_help_exposes_proxy_rules() {
        let help = help_text(&["jet", "dev", "--help"]);
        assert!(
            help.contains("--proxy") && help.contains("PATH=URL") && help.contains("[dev.proxy]"),
            "dev help must expose CLI proxy rules and config override behavior: {help}"
        );
    }

    #[test]
    fn dev_proxy_cli_rules_merge_over_config_rules() {
        let mut config = std::collections::HashMap::new();
        config.insert("/api".to_string(), "http://localhost:3000".to_string());
        config.insert("/auth".to_string(), "http://localhost:3001".to_string());

        let merged = merge_dev_proxy_rules(
            config,
            [
                "/api=http://localhost:4200",
                "/events/=https://example.test/events/",
            ],
        )
        .expect("valid proxy rules must merge");

        assert_eq!(
            merged.get("/api").map(String::as_str),
            Some("http://localhost:4200"),
            "CLI proxy rule must override the same config prefix"
        );
        assert_eq!(
            merged.get("/auth").map(String::as_str),
            Some("http://localhost:3001"),
            "unrelated config proxy rule must be preserved"
        );
        assert_eq!(
            merged.get("/events").map(String::as_str),
            Some("https://example.test/events"),
            "trailing slash normalization keeps segment-aware matching stable"
        );
    }

    #[test]
    fn dev_proxy_cli_rule_rejects_ambiguous_shapes() {
        assert!(parse_dev_proxy_rule("api=http://localhost:3200").is_err());
        assert!(parse_dev_proxy_rule("/api=localhost:3200").is_err());
        assert!(parse_dev_proxy_rule("/api").is_err());
    }

    #[test]
    fn dev_proxy_is_not_silently_accepted_in_wasm_mode() {
        let err = command()
            .try_get_matches_from([
                "jet",
                "dev",
                "--wasm",
                "--proxy",
                "/api=http://localhost:3200",
            ])
            .expect_err("wasm dev does not currently wire proxy rules");
        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }

    // @spec .aw/tech-design/projects/jet/specs/3941.md#unit-test
    #[test]
    fn browser_capture_help_describes_dom_and_wasm_surfaces() {
        let help = help_text(&["jet", "browser", "capture", "--help"]);
        assert!(
            help.contains("--surface"),
            "capture help must advertise --surface: {help}"
        );
        assert!(
            help.contains("wasm") && help.contains("dom"),
            "capture help must enumerate wasm/dom surfaces: {help}"
        );
        assert!(
            help.contains("--root-selector"),
            "capture help must advertise DOM root selector: {help}"
        );
    }

    #[test]
    fn bb_alias_exposes_browser_bridge_capture_surface() {
        let help = help_text(&["jet", "bb", "capture", "--help"]);
        assert!(
            help.contains("--surface") && help.contains("wasm") && help.contains("dom"),
            "bb capture help must expose the same DOM/WASM surface contract: {help}"
        );
        assert!(
            help.contains("--root-selector"),
            "bb capture help must expose DOM root selector: {help}"
        );
    }

    #[test]
    fn browser_bridge_launch_is_agent_first_and_debug_is_human_foreground() {
        let launch_help = help_text(&["jet", "bb", "launch", "--help"]);
        assert!(
            launch_help.contains("headless") && launch_help.contains("detached"),
            "bb launch help must advertise headless detached agent mode: {launch_help}"
        );

        let debug_help = help_text(&["jet", "bb", "debug", "--help"]);
        assert!(
            debug_help.contains("foreground") && debug_help.contains("human inspection"),
            "bb debug help must advertise foreground human inspection mode: {debug_help}"
        );
    }

    #[test]
    fn bb_command_is_observable_to_dispatch() {
        let matches = command()
            .try_get_matches_from(["jet", "bb", "launch", "http://127.0.0.1:3000/"])
            .expect("bb launch parses");
        let (name, _) = matches.subcommand().expect("top-level subcommand");
        assert_eq!(
            name, "bb",
            "dispatch must distinguish `jet bb launch` from legacy `jet browser launch`"
        );
    }

    #[test]
    fn browser_bridge_help_lists_lightweight_perf_probe() {
        let bb_help = help_text(&["jet", "bb", "--help"]);
        assert!(
            bb_help.contains("perf"),
            "bb help must expose the lightweight perf snapshot command: {bb_help}"
        );

        let perf_help = help_text(&["jet", "bb", "perf", "--help"]);
        assert!(
            perf_help.contains("performance") && perf_help.contains("without full capture"),
            "bb perf help must distinguish itself from heavy capture: {perf_help}"
        );
    }

    #[test]
    fn browser_input_help_describes_mouse_drag_wheel_and_key_commands() {
        let browser_help = help_text(&["jet", "browser", "--help"]);
        assert!(
            browser_help.contains("mouse")
                && browser_help.contains("drag")
                && browser_help.contains("wheel")
                && browser_help.contains("key"),
            "browser help must list input-driving commands: {browser_help}"
        );

        let drag_help = help_text(&["jet", "browser", "drag", "--help"]);
        assert!(
            drag_help.contains("CDP mouse events") && drag_help.contains("--steps"),
            "drag help must describe CDP mouse input and steps: {drag_help}"
        );

        let wheel_help = help_text(&["jet", "browser", "wheel", "--help"]);
        assert!(
            wheel_help.contains("CDP mouse wheel event")
                && wheel_help.contains("--delta-x")
                && wheel_help.contains("--delta-y"),
            "wheel help must describe CDP wheel input and deltas: {wheel_help}"
        );

        let key_help = help_text(&["jet", "browser", "key", "--help"]);
        assert!(
            key_help.contains("--ctrl") && key_help.contains("--meta"),
            "key help must advertise shortcut modifiers: {key_help}"
        );
    }

    // @spec enhancement-define-component-and-browser-like-test-environment-boundary
    #[test]
    fn test_help_describes_env_flag_and_boundary() {
        let help = help_text(&["jet", "test", "--help"]);
        assert!(
            help.contains("--env"),
            "test help must advertise --env: {help}"
        );
        assert!(
            help.contains("node") && help.contains("dom") && help.contains("component"),
            "test help must enumerate env kinds: {help}"
        );
        assert!(
            help.contains("jet e2e"),
            "test help must point at jet e2e for product-flow cases: {help}"
        );
    }

    // @spec enhancement-define-component-and-browser-like-test-environment-boundary
    #[test]
    fn test_accepts_env_flag_at_parser_level() {
        let m = command()
            .try_get_matches_from(["jet", "test", "--env", "node"])
            .unwrap();
        let (sub, sm) = m.subcommand().unwrap();
        assert_eq!(sub, "test");
        assert_eq!(
            sm.get_one::<String>("env").map(String::as_str),
            Some("node")
        );
    }

    // @spec .aw/tech-design/projects/jet/specs/2385.md#test-plan
    #[test]
    fn unknown_e2e_subcommand_has_e2e_specific_diagnostic() {
        let matches = command()
            .try_get_matches_from(["jet", "e2e", "preview"])
            .unwrap();
        let err = execute(&matches).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("Unknown e2e subcommand 'preview'"),
            "unknown diagnostic must be e2e-specific: {msg}"
        );
        assert!(
            msg.contains("jet e2e run") && msg.contains("jet e2e open"),
            "unknown diagnostic must suggest the valid modes: {msg}"
        );
    }

    // @spec .aw/tech-design/projects/jet/specs/2385.md#test-plan
    #[test]
    fn run_and_open_parse_mode_specific_options() {
        let run_matches = command()
            .try_get_matches_from([
                "jet",
                "e2e",
                "run",
                "--evidence-dir",
                "out/evidence",
                "--serve",
                "dev",
                "--json",
                "flows/cue-artifact-studio.spec.ts",
            ])
            .unwrap();
        let (_, e2e) = run_matches.subcommand().unwrap();
        let (_, run) = e2e.subcommand().unwrap();
        assert_eq!(
            run.get_one::<String>("evidence-dir").map(String::as_str),
            Some("out/evidence")
        );
        assert_eq!(
            run.get_one::<String>("serve").map(String::as_str),
            Some("dev")
        );
        assert!(run.get_flag("json"));

        let base_url_matches = command()
            .try_get_matches_from([
                "jet",
                "e2e",
                "run",
                "--base-url",
                "http://127.0.0.1:43127/",
                "flows/cue-artifact-studio.spec.ts",
            ])
            .unwrap();
        let (_, e2e) = base_url_matches.subcommand().unwrap();
        let (_, run) = e2e.subcommand().unwrap();
        assert_eq!(
            run.get_one::<String>("base-url").map(String::as_str),
            Some("http://127.0.0.1:43127/")
        );

        let open_matches = command()
            .try_get_matches_from(["jet", "e2e", "open", "--dry-run", "--no-open"])
            .unwrap();
        let (_, e2e) = open_matches.subcommand().unwrap();
        let (_, open) = e2e.subcommand().unwrap();
        assert!(open.get_flag("dry-run"));
        assert!(open.get_flag("no-open"));
    }
}

#[cfg(test)]
mod build_target_validation_table_tests {
    //! Slice 4 of #1239 — every row of the validation table in
    //! `.aw/tech-design/projects/jet/logic/multi-target/build-targets.md`
    //! exercised through the real `cli::command()` parser.
    //!
    //! The workspace doesn't pull in `assert_cmd`. The next-best
    //! layer is `try_get_matches_from()` against the same `Command`
    //! the binary uses, then re-applying the production resolve +
    //! validate + flag_snapshot helpers — so any drift between
    //! parser and validator surfaces here.
    use super::*;
    use crate::build_target::{self, BuildTarget};
    use clap::error::ErrorKind;
    use std::path::Path;
    use tempfile::TempDir;

    fn run_build(args: &[&str]) -> Result<ArgMatches, clap::Error> {
        let mut argv = vec!["jet", "build"];
        argv.extend_from_slice(args);
        command().try_get_matches_from(argv).and_then(|m| {
            // unwrap the build subcommand
            let (sub, sm) = m.subcommand().unwrap();
            assert_eq!(sub, "build");
            Ok(sm.clone())
        })
    }

    fn resolve_target(m: &ArgMatches) -> BuildTarget {
        build_target::resolve(m.get_one::<String>("target").map(|s| s.as_str())).unwrap()
    }

    // #2395 — `--empty-out-dir` and `--force` are accepted by the parser
    #[test]
    fn build_accepts_empty_out_dir_and_force_flags() {
        let m = run_build(&["--empty-out-dir"]).unwrap();
        assert!(m.get_flag("empty-out-dir"));
        assert!(!m.get_flag("force"));

        let m = run_build(&["--empty-out-dir", "--force"]).unwrap();
        assert!(m.get_flag("empty-out-dir"));
        assert!(m.get_flag("force"));

        // --clean is an alias for --empty-out-dir
        let m = run_build(&["--clean"]).unwrap();
        assert!(m.get_flag("empty-out-dir"));
    }

    // ── validation-table row 1: no --target defaults to web ─────────
    #[test]
    fn no_target_flag_defaults_to_web() {
        let m = run_build(&[]).unwrap();
        assert_eq!(resolve_target(&m), BuildTarget::Web);
    }

    #[test]
    fn explicit_web_target_parses_to_web_variant() {
        let m = run_build(&["--target", "web"]).unwrap();
        assert_eq!(resolve_target(&m), BuildTarget::Web);
    }

    // ── --target web (no --wasm) is legal ──
    #[test]
    fn target_web_without_wasm_is_legal() {
        let m = run_build(&["--target", "web"]).unwrap();
        let t = resolve_target(&m);
        let snap = build_flag_snapshot_from_matches(&m);
        build_target::validate_combination(t, snap).unwrap();
    }

    // ── clap rejects unknown --target X ────
    #[test]
    fn unknown_target_rejected_by_clap_with_closed_enum() {
        let err = run_build(&["--target", "atari2600"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidValue);
        let msg = err.to_string();
        for v in BuildTarget::ALL {
            assert!(msg.contains(v), "expected {v} in error message: {msg}");
        }
    }

    // ── web accepts the full bundler flag set ──────────────────
    #[test]
    fn target_web_accepts_full_bundler_flags() {
        let m = run_build(&[
            "--target",
            "web",
            "--minify",
            "--splitting",
            "--sourcemap",
            "inline",
            "--drop",
            "console",
            "--wasm",
        ])
        .unwrap();
        let t = resolve_target(&m);
        let snap = build_flag_snapshot_from_matches(&m);
        build_target::validate_combination(t, snap).unwrap();
    }

    // ── flag_snapshot helper round-trip ────────────────────────────
    #[test]
    fn flag_snapshot_picks_up_each_flag() {
        let m = run_build(&[
            "--minify",
            "--splitting",
            "--sourcemap",
            "hidden",
            "--drop",
            "console",
            "--wasm",
        ])
        .unwrap();
        let snap = build_flag_snapshot_from_matches(&m);
        assert!(snap.wasm);
        assert!(snap.minify);
        assert!(snap.sourcemap_set);
        assert!(snap.splitting);
        assert!(snap.drop_set);
    }

    #[test]
    fn build_minify_is_default_and_no_minify_is_escape_hatch() {
        let default_build = run_build(&[]).unwrap();
        assert!(
            build_minify_enabled_from_matches(&default_build),
            "jet build should minify by default"
        );

        let no_minify = run_build(&["--no-minify"]).unwrap();
        assert!(
            !build_minify_enabled_from_matches(&no_minify),
            "--no-minify must disable default minification"
        );

        let explicit_then_disabled = run_build(&["--minify", "--no-minify"]).unwrap();
        assert!(
            !build_minify_enabled_from_matches(&explicit_then_disabled),
            "--no-minify must win when both flags are present"
        );
    }

    #[test]
    fn flag_snapshot_treats_default_sourcemap_as_unset() {
        let m = run_build(&["--sourcemap", "external"]).unwrap();
        let snap = build_flag_snapshot_from_matches(&m);
        assert!(
            !snap.sourcemap_set,
            "external is the default; should not count as set"
        );
    }

    // ── tempdir suppression: parser does not touch the filesystem
    // for any of these rows. Sanity check: an unrelated tempdir works.
    #[test]
    fn parser_is_pure_and_does_not_touch_disk() {
        let _tmp = TempDir::new().unwrap();
        let m = run_build(&["--target", "web", "--wasm"]).unwrap();
        assert_eq!(resolve_target(&m), BuildTarget::Web);
        assert!(!Path::new("/this/path/should/not/exist/anywhere").exists());
    }
}

#[cfg(test)]
mod list_scripts_tests {
    //! GH #3218 — `list_scripts` (the `jet run` bare-command lister)
    //! previously used `if let Ok(pkg) = serde_json::from_str(...)`,
    //! silently swallowing malformed package.json and showing a clean
    //! "no scripts found" result indistinguishable from a project with
    //! zero scripts. These tests pin the new contract: parse failure is
    //! surfaced but the function still returns Ok so the pipeline half
    //! of the listing can render.
    use tempfile::TempDir;

    /// Malformed package.json must NOT cause `list_scripts` to bubble
    /// an error back to the CLI runner — the pipeline section after it
    /// still needs to run. The pre-fix code returned Ok by accident
    /// (silent swallow); the post-fix code returns Ok by design after
    /// surfacing the parse error to stderr.
    #[test]
    fn list_scripts_returns_ok_when_package_json_is_malformed() {
        let dir = TempDir::new().unwrap();
        // Trailing comma — invalid JSON.
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name":"broken","scripts":{"dev":"jet dev",}}"#,
        )
        .unwrap();
        let root = dir.path().to_path_buf();
        let result = super::list_scripts(&root);
        assert!(
            result.is_ok(),
            "malformed package.json must not propagate; the pipeline section still needs to run; got {result:?}"
        );
    }

    /// Valid package.json with `scripts` must still produce Ok.
    /// Captures the happy-path contract.
    #[test]
    fn list_scripts_returns_ok_for_valid_package_json_with_scripts() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name":"ok","scripts":{"dev":"jet dev"}}"#,
        )
        .unwrap();
        let root = dir.path().to_path_buf();
        let result = super::list_scripts(&root);
        assert!(
            result.is_ok(),
            "valid package.json must produce Ok; got {result:?}"
        );
    }

    /// Missing package.json must produce Ok (pkg_path.exists() guard).
    /// Pins the contract that a script-less project still gets its
    /// pipeline tasks listed.
    #[test]
    fn list_scripts_returns_ok_when_package_json_is_missing() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().to_path_buf();
        let result = super::list_scripts(&root);
        assert!(
            result.is_ok(),
            "missing package.json must produce Ok; got {result:?}"
        );
    }
}

#[cfg(test)]
mod gh3789_non_string_script_value_warn_tests {
    //! GH #3789 — `list_scripts` rendered every non-string
    //! `package.json` `scripts.<name>` value as a blank command column
    //! via `cmd.as_str().unwrap_or("")`. Distinguishes "wrong JSON
    //! shape" from "empty string", warns the operator, and substitutes
    //! a sentinel that keeps the gap visible.
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn string_value_renders_unchanged() {
        // 1) String passes through untouched (legacy happy path).
        let v = json!("jet dev");
        let s = super::coerce_script_command_or_warn("dev", &v);
        assert_eq!(s, "jet dev");
    }

    #[test]
    fn number_value_emits_sentinel() {
        // 2) Number → sentinel labelled `number`.
        let v = json!(42);
        let s = super::coerce_script_command_or_warn("test", &v);
        assert_eq!(s, "<non-string script value: number>");
    }

    #[test]
    fn bool_value_emits_sentinel() {
        // 3) Boolean → sentinel labelled `bool`.
        let v = json!(true);
        let s = super::coerce_script_command_or_warn("ci", &v);
        assert_eq!(s, "<non-string script value: bool>");
    }

    #[test]
    fn null_value_emits_sentinel() {
        // 4) Null → sentinel labelled `null`. Distinct from a string
        // that happens to be the literal `"null"`.
        let v = json!(null);
        let s = super::coerce_script_command_or_warn("prepublish", &v);
        assert_eq!(s, "<non-string script value: null>");
        let literal = super::coerce_script_command_or_warn("x", &json!("null"));
        assert_eq!(literal, "null");
        assert_ne!(s, literal);
    }

    #[test]
    fn array_value_emits_sentinel() {
        // 5) Array → sentinel labelled `array`.
        let v = json!(["jet", "dev"]);
        let s = super::coerce_script_command_or_warn("dev", &v);
        assert_eq!(s, "<non-string script value: array>");
    }

    #[test]
    fn object_value_emits_sentinel() {
        // 6) Object → sentinel labelled `object`.
        let v = json!({"cmd": "jet dev"});
        let s = super::coerce_script_command_or_warn("dev", &v);
        assert_eq!(s, "<non-string script value: object>");
    }

    #[test]
    fn value_kind_returns_expected_tokens() {
        // 7) The kind classifier returns each expected token.
        assert_eq!(super::package_script_value_kind(&json!(null)), "null");
        assert_eq!(super::package_script_value_kind(&json!(true)), "bool");
        assert_eq!(super::package_script_value_kind(&json!(1)), "number");
        assert_eq!(super::package_script_value_kind(&json!("x")), "string");
        assert_eq!(super::package_script_value_kind(&json!([1])), "array");
        assert_eq!(super::package_script_value_kind(&json!({"a":1})), "object");
    }

    #[test]
    fn warn_helper_name_pinned_for_discoverability() {
        // 8) Pin both helper names so a rename without grep breaks tests.
        let w = super::format_cli_run_non_string_script_value_warn("dev", "number");
        assert!(
            !w.is_empty(),
            "format_cli_run_non_string_script_value_warn must produce non-empty text",
        );
        let sentinel = super::format_non_string_script_value_sentinel("array");
        assert!(
            sentinel.contains("array"),
            "sentinel must carry the kind: {sentinel}"
        );
    }

    #[test]
    fn warn_string_includes_gh3789_tag_and_script_name() {
        // 9) gh3789 tag + script name preserved for log triage.
        let w = super::format_cli_run_non_string_script_value_warn("my-script", "number");
        assert!(w.contains("gh3789"), "missing gh3789 tag: {w}");
        assert!(w.contains("my-script"), "missing script name: {w}");
    }

    #[test]
    fn warn_string_distinct_from_prior_silent_fallback_families() {
        // 10) Sibling-distinctness vs every prior warn family — log
        // filtering must split them.
        let w = super::format_cli_run_non_string_script_value_warn("dev", "number");
        for prior in [
            "gh3763", "gh3765", "gh3768", "gh3770", "gh3772", "gh3774", "gh3776", "gh3787",
        ] {
            assert!(!w.contains(prior), "must not overlap {prior}: {w}");
        }
    }

    #[test]
    fn list_scripts_does_not_propagate_when_value_is_non_string() {
        // Integration: a package.json with a non-string script value
        // does not bubble an error out of `list_scripts` — the
        // pipeline section after the loop still needs to render.
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name":"weird","scripts":{"dev":"jet dev","broken":42,"empty":null}}"#,
        )
        .unwrap();
        let root = dir.path().to_path_buf();
        let result = super::list_scripts(&root);
        assert!(
            result.is_ok(),
            "non-string script values must not propagate; got {result:?}"
        );
    }
}

#[cfg(test)]
mod gh3509_handle_run_tests {
    //! GH #3509 — `handle_run` previously used
    //! `if let Ok(tr) = TaskRunner::new(root_dir)` and silently dropped
    //! every TaskRunner-construction error. Since `JetConfig::load`
    //! returns Ok(default) for a missing `jet.toml`, the only
    //! way `new` returns Err is if the file exists and contains a real
    //! config error (TOML parse, TaskGraph cycle, undefined task ref,
    //! cache-dir IO failure). The user saw a misleading "target not
    //! found" instead of the parser diagnostic.
    use tempfile::TempDir;

    /// No `jet.toml` and no matching script/file: the function
    /// must fall through to the "target not found" bail — the absence
    /// of a config is NOT a config error.
    #[tokio::test]
    async fn gh3509_no_jet_config_falls_through_to_target_not_found() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().to_path_buf();
        let err = super::handle_run(&root, "nonexistent", &[], false, None, false)
            .await
            .expect_err("nonexistent target with no config must Err");
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("not found as a script, file, or task"),
            "missing config must fall through to target-not-found, got: {msg}"
        );
    }

    /// Malformed `jet.toml`: the function must surface the
    /// TaskRunner-load error (which includes the parser-classified
    /// diagnostic) instead of silently fall-through to "target not found".
    #[tokio::test]
    async fn gh3509_malformed_jet_config_surfaces_load_error() {
        let dir = TempDir::new().unwrap();
        // Unknown top-level key — `classify_jet_toml_error` produces a
        // structured "unknown field" diagnostic.
        std::fs::write(
            dir.path().join("jet.toml"),
            "[piepline.build]\ncommand = \"echo hi\"\n",
        )
        .unwrap();
        let root = dir.path().to_path_buf();

        let err = super::handle_run(&root, "build", &[], false, None, false)
            .await
            .expect_err("malformed jet.toml must Err");
        let msg = format!("{:#}", err);

        assert!(
            msg.contains("Failed to load task runner"),
            "error must surface the TaskRunner-load context, got: {msg}"
        );
        assert!(
            !msg.contains("not found as a script, file, or task"),
            "malformed config must NOT fall through to the misleading \
             \"target not found\" message, got: {msg}"
        );
    }
}

#[cfg(test)]
mod gh3712_parse_define_arg_tests {
    //! GH #3712 — `jet build --define KEY` (missing `=VALUE`, or
    //! typo'd with `:` instead of `=`) previously silently dropped the
    //! entry via `if let Some(...) = def.split_once('=')`. The bundle
    //! shipped with the constant unreplaced; the build log said
    //! "complete"; the user only noticed in production. esbuild
    //! errors out on missing `=`; mirror that.
    use super::*;

    #[test]
    fn happy_key_equals_value_parses() {
        let (k, v) = parse_define_arg("VERSION=1.2.3").unwrap();
        assert_eq!(k, "VERSION");
        assert_eq!(v, "1.2.3");
    }

    #[test]
    fn empty_value_is_ok_user_may_intentionally_set_empty() {
        // `--define DEBUG=` is a legitimate way to set DEBUG to an
        // empty string. esbuild accepts it; mirror that.
        let (k, v) = parse_define_arg("DEBUG=").unwrap();
        assert_eq!(k, "DEBUG");
        assert_eq!(v, "");
    }

    #[test]
    fn value_with_embedded_equals_keeps_only_first_split() {
        // `--define EXPR=a=b` should set EXPR to "a=b", not bail.
        let (k, v) = parse_define_arg("EXPR=a=b").unwrap();
        assert_eq!(k, "EXPR");
        assert_eq!(v, "a=b");
    }

    #[test]
    fn missing_equals_returns_tagged_err() {
        let err = parse_define_arg("VERSION").expect_err("missing = must error");
        assert!(err.contains("GH #3712"), "err: {err}");
        assert!(err.contains("VERSION"), "err: {err}");
    }

    #[test]
    fn colon_typo_returns_tagged_err_with_typo_hint() {
        let err = parse_define_arg("KEY:VALUE").expect_err("colon must error");
        assert!(err.contains("GH #3712"), "err: {err}");
        assert!(
            err.contains("colon") || err.contains("KEY:VALUE"),
            "err must name the typo shape: {err}"
        );
    }

    #[test]
    fn empty_key_returns_tagged_err() {
        let err = parse_define_arg("=value").expect_err("empty key must error");
        assert!(err.contains("GH #3712"), "err: {err}");
        assert!(err.contains("empty"), "err: {err}");
    }

    #[test]
    fn err_mentions_expected_format_so_user_can_fix() {
        let err = parse_define_arg("oops").expect_err("missing = must error");
        assert!(err.contains("KEY=VALUE"), "err: {err}");
    }

    #[test]
    fn err_mentions_esbuild_so_users_know_intent() {
        let err = parse_define_arg("oops").expect_err("missing = must error");
        assert!(err.contains("esbuild"), "err: {err}");
    }
}

#[cfg(test)]
mod gh3708_build_watch_not_implemented_warn_tests {
    //! GH #3708 — `jet build --watch` previously hit
    //! `let _watch = m.get_flag("watch");` which silently discarded
    //! the flag. clap accepted `-w` / `--watch`, the bundler ran once,
    //! printed "Build complete", and exited 0 — contradicting the
    //! module docstring and asymmetric with `jet test --watch` /
    //! `jet dev --watch` which both honour the flag.
    use super::*;

    #[test]
    fn warn_tags_gh_issue_so_breadcrumb_is_searchable() {
        let msg = format_build_watch_not_implemented_warn();
        assert!(msg.contains("GH #3708"), "msg: {msg}");
    }

    #[test]
    fn warn_names_the_flag_so_users_know_what_was_ignored() {
        let msg = format_build_watch_not_implemented_warn();
        assert!(msg.contains("--watch"), "msg: {msg}");
        assert!(msg.contains("no-op"), "msg: {msg}");
    }

    #[test]
    fn warn_names_the_observable_symptom_runs_once_and_exits() {
        let msg = format_build_watch_not_implemented_warn();
        assert!(
            msg.contains("runs once") || msg.contains("exits"),
            "warn must name the observable symptom: {msg}"
        );
    }

    #[test]
    fn warn_points_at_the_working_alternatives() {
        let msg = format_build_watch_not_implemented_warn();
        assert!(
            msg.contains("jet dev") && msg.contains("jet test"),
            "warn must redirect users to the working watch-mode siblings: {msg}"
        );
    }

    #[test]
    fn warn_is_deterministic() {
        let a = format_build_watch_not_implemented_warn();
        let b = format_build_watch_not_implemented_warn();
        assert_eq!(a, b);
    }

    #[test]
    fn warn_starts_with_warning_keyword_so_log_filters_catch_it() {
        let msg = format_build_watch_not_implemented_warn();
        assert!(
            msg.to_ascii_lowercase().starts_with("warning"),
            "warn must announce itself as a warning: {msg}"
        );
    }

    #[test]
    fn warn_distinct_from_splitting_warn() {
        // GH #3705 and GH #3708 are sibling silent-no-op fixes; the
        // warns must be clearly distinguishable so a user who sees one
        // doesn't think the other was meant.
        let watch = format_build_watch_not_implemented_warn();
        let split = format_splitting_not_implemented_warn();
        assert_ne!(watch, split);
        assert!(watch.contains("GH #3708"));
        assert!(split.contains("GH #3705"));
    }
}

#[cfg(test)]
mod gh3705_splitting_not_implemented_warn_tests {
    //! GH #3705 — `jet build --splitting` previously hit
    //! `let _ = splitting; // TODO: integrate with chunk splitter`.
    //! Build said "complete", the user assumed splitting happened,
    //! and only noticed when `dist/` held one un-split bundle.
    //! The warn helper exists so we can assert the exact wording
    //! without spawning the binary or running an end-to-end build.
    use super::*;

    #[test]
    fn warn_tags_gh_issue_so_breadcrumb_is_searchable() {
        let msg = format_splitting_not_implemented_warn();
        assert!(msg.contains("GH #3705"), "msg: {msg}");
    }

    #[test]
    fn warn_names_the_no_op_so_users_know_their_flag_is_ignored() {
        let msg = format_splitting_not_implemented_warn();
        assert!(msg.contains("no-op"), "msg: {msg}");
        assert!(msg.contains("--splitting"), "msg: {msg}");
    }

    #[test]
    fn warn_points_at_downstream_tracker_for_fix_eta() {
        let msg = format_splitting_not_implemented_warn();
        assert!(msg.contains("#1089"), "msg: {msg}");
    }

    #[test]
    fn warn_names_the_observable_symptom_one_file_not_chunks() {
        let msg = format_splitting_not_implemented_warn();
        assert!(
            msg.contains("ONE file") || msg.contains("one file"),
            "warn must name the observable symptom: {msg}"
        );
    }

    #[test]
    fn warn_names_downstream_consumers_so_users_know_what_else_breaks() {
        let msg = format_splitting_not_implemented_warn();
        assert!(
            msg.contains("first-load") || msg.contains("dynamic-import") || msg.contains("CI"),
            "warn must name at least one downstream consumer: {msg}"
        );
    }

    #[test]
    fn warn_is_deterministic() {
        let a = format_splitting_not_implemented_warn();
        let b = format_splitting_not_implemented_warn();
        assert_eq!(a, b);
    }

    #[test]
    fn warn_starts_with_warning_keyword_so_it_renders_as_a_warning() {
        let msg = format_splitting_not_implemented_warn();
        assert!(
            msg.to_ascii_lowercase().starts_with("warning"),
            "warn must announce itself as a warning so log filters catch it: {msg}"
        );
    }
}

#[cfg(test)]
mod gh3721_unknown_store_subcommand_err_tests {
    //! GH #3721 — `jet store <unknown>` previously printed
    //! "Unknown store subcommand. Try 'jet store prune'." to STDOUT
    //! and returned Ok(()) → exit 0. A `jet store gc` typo in a CI step
    //! that checks `$?` would silently succeed. Every sibling
    //! Unknown-subcommand branch (config, report, trace, e2e, browser)
    //! uses anyhow::bail! → stderr + exit 1. This test module pins:
    //! the helper exists, the wording is GH-tagged, named-subcommand
    //! and missing-subcommand variants produce distinct messages, and
    //! the named-subcommand variant echoes the typo verbatim so the
    //! user can spot it without re-reading docs.
    use super::*;

    #[test]
    fn err_tags_gh_issue() {
        let msg = format_unknown_store_subcommand_err(Some("gc"));
        assert!(msg.contains("GH #3721"), "msg: {msg}");
    }

    #[test]
    fn err_echoes_unknown_subcommand_verbatim() {
        let msg = format_unknown_store_subcommand_err(Some("gc"));
        assert!(
            msg.contains("'gc'"),
            "err must echo the typo verbatim so user can spot it: {msg}"
        );
    }

    #[test]
    fn err_names_only_supported_subcommand() {
        let msg = format_unknown_store_subcommand_err(Some("list"));
        assert!(
            msg.contains("prune"),
            "err must name the only valid subcommand: {msg}"
        );
    }

    #[test]
    fn missing_and_unknown_produce_distinct_messages() {
        let unknown = format_unknown_store_subcommand_err(Some("gc"));
        let missing = format_unknown_store_subcommand_err(None);
        assert_ne!(
            unknown, missing,
            "unknown vs missing subcommand must produce distinct wording"
        );
    }

    #[test]
    fn missing_variant_does_not_quote_any_name() {
        let msg = format_unknown_store_subcommand_err(None);
        // No quoted name because there isn't one.
        assert!(
            !msg.contains("'gc'") && !msg.contains("''"),
            "bare `jet store` must not invent a quoted subcommand name: {msg}"
        );
        assert!(msg.contains("prune"), "msg: {msg}");
    }

    #[test]
    fn err_names_silent_success_root_cause() {
        let msg = format_unknown_store_subcommand_err(Some("gc"));
        assert!(
            msg.contains("STDOUT") || msg.contains("exit 0") || msg.contains("silently succeeded"),
            "err must name the root cause so future readers don't re-introduce it: {msg}"
        );
    }

    #[test]
    fn err_is_deterministic_for_fixed_inputs() {
        let a = format_unknown_store_subcommand_err(Some("foo"));
        let b = format_unknown_store_subcommand_err(Some("foo"));
        assert_eq!(a, b);
        let na = format_unknown_store_subcommand_err(None);
        let nb = format_unknown_store_subcommand_err(None);
        assert_eq!(na, nb);
    }

    #[test]
    fn err_distinct_from_store_home_err() {
        // GH #3596 sibling: the store-home err and the unknown-subcommand
        // err must NOT collapse onto each other. If a future refactor
        // accidentally re-uses one wording for the other, downstream
        // diagnostics confuse "HOME unset" with "typo'd subcommand".
        let unknown = format_unknown_store_subcommand_err(Some("gc"));
        let home_not_present = format_store_home_err("not-present");
        assert_ne!(unknown, home_not_present);
    }
}

#[cfg(test)]
mod gh3799_sourcemap_unknown_warn_tests {
    use super::{coerce_sourcemap_mode_or_warn, format_cli_build_sourcemap_unknown_warn};
    use crate::bundler::types::SourceMapOption;

    #[test]
    fn absent_flag_falls_back_to_external_silently() {
        assert_eq!(
            coerce_sourcemap_mode_or_warn(None),
            SourceMapOption::External
        );
    }

    #[test]
    fn each_canonical_value_maps_silently() {
        assert_eq!(
            coerce_sourcemap_mode_or_warn(Some("none")),
            SourceMapOption::None
        );
        assert_eq!(
            coerce_sourcemap_mode_or_warn(Some("inline")),
            SourceMapOption::Inline
        );
        assert_eq!(
            coerce_sourcemap_mode_or_warn(Some("hidden")),
            SourceMapOption::Hidden
        );
        assert_eq!(
            coerce_sourcemap_mode_or_warn(Some("external")),
            SourceMapOption::External
        );
    }

    #[test]
    fn unknown_value_warns_and_falls_back_to_external() {
        let mode = coerce_sourcemap_mode_or_warn(Some("externl"));
        assert_eq!(mode, SourceMapOption::External);
    }

    #[test]
    fn warn_helpers_pinned_for_discoverability() {
        let src = include_str!("cli.rs");
        assert!(src.contains("fn format_cli_build_sourcemap_unknown_warn"));
        assert!(src.contains("fn coerce_sourcemap_mode_or_warn"));
    }

    #[test]
    fn each_warn_string_carries_gh3799_tag() {
        let s = format_cli_build_sourcemap_unknown_warn("externl");
        assert!(s.starts_with("gh3799:"), "missing gh3799 tag: {s:?}");
        assert!(s.contains("--sourcemap"));
        assert!(s.contains("externl"));
    }

    #[test]
    fn warn_distinct_from_prior_silent_fallback_families() {
        let s = format_cli_build_sourcemap_unknown_warn("nope");
        for tag in [
            "gh3763", "gh3765", "gh3768", "gh3770", "gh3772", "gh3774", "gh3776", "gh3787",
            "gh3789", "gh3791", "gh3793", "gh3795", "gh3797",
        ] {
            assert!(!s.contains(tag), "gh3799 warn must not carry {tag}: {s:?}");
        }
    }

    #[test]
    fn warn_names_full_canonical_set() {
        let s = format_cli_build_sourcemap_unknown_warn("hiden");
        assert!(s.contains("\"none\""));
        assert!(s.contains("\"inline\""));
        assert!(s.contains("\"hidden\""));
        assert!(s.contains("\"external\""));
    }

    #[test]
    fn warn_explains_consequence_to_operator() {
        let s = format_cli_build_sourcemap_unknown_warn("ext");
        assert!(
            s.contains("sourcemap policy") || s.contains("shipped bundle"),
            "warn should explain consequence: {s:?}"
        );
    }

    #[test]
    fn two_distinct_typos_share_external_fallback_but_emit_distinct_warns() {
        // Pins the documented behaviour: ALL unknowns fall back to External
        // (so behaviour is preserved vs the legacy wildcard arm) while the
        // warn carries the raw typo so operators can fix it.
        assert_eq!(
            coerce_sourcemap_mode_or_warn(Some("externl")),
            SourceMapOption::External
        );
        assert_eq!(
            coerce_sourcemap_mode_or_warn(Some("hidde")),
            SourceMapOption::External
        );
        let w1 = format_cli_build_sourcemap_unknown_warn("externl");
        let w2 = format_cli_build_sourcemap_unknown_warn("hidde");
        assert_ne!(
            w1, w2,
            "warns must carry the raw typo so operators can spot which run hit which mistake"
        );
    }

    #[test]
    fn empty_string_value_is_treated_as_unknown_not_default() {
        // An operator that passes `--sourcemap ""` is making a typo'd
        // explicit choice, not omitting the flag. We surface a warn rather
        // than treating it like the absent-flag default.
        let mode = coerce_sourcemap_mode_or_warn(Some(""));
        assert_eq!(mode, SourceMapOption::External);
        let s = format_cli_build_sourcemap_unknown_warn("");
        assert!(s.contains("\"\""));
    }
}

#[cfg(test)]
mod gh3803_sourcemap_entry_path_warn_tests {
    use super::{
        coerce_sourcemap_entry_path_or_warn, format_cli_build_sourcemap_non_utf8_entry_warn,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn utf8_entry_path_passes_through_silently() {
        let s = coerce_sourcemap_entry_path_or_warn(Path::new("/proj/src/index.ts"));
        assert_eq!(s, "/proj/src/index.ts");
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_entry_produces_lossy_form_not_empty() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let p = PathBuf::from(OsStr::from_bytes(b"/proj/\xFFnonutf8.ts"));
        let s = coerce_sourcemap_entry_path_or_warn(&p);
        assert!(
            !s.is_empty(),
            "non-UTF-8 entry path must produce a lossy form, not empty"
        );
        assert!(
            s.contains("\u{FFFD}"),
            "lossy form should contain U+FFFD substitute: {s:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn two_distinct_non_utf8_entries_lossy_onto_same_string_is_warned() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        // 0xFF and 0xFE both lossy onto U+FFFD — without the warn this
        // collision would silently produce a misdirected source map.
        let p1 = PathBuf::from(OsStr::from_bytes(b"/proj/\xFFa.ts"));
        let p2 = PathBuf::from(OsStr::from_bytes(b"/proj/\xFEa.ts"));
        let s1 = coerce_sourcemap_entry_path_or_warn(&p1);
        let s2 = coerce_sourcemap_entry_path_or_warn(&p2);
        // The lossy forms collide (this is the bug the warn surfaces).
        // The warn message is what operators read to spot the issue:
        // each warn carries the original Path's Debug form which preserves
        // the distinct byte sequences.
        let w1 = format_cli_build_sourcemap_non_utf8_entry_warn(&p1, &s1);
        let w2 = format_cli_build_sourcemap_non_utf8_entry_warn(&p2, &s2);
        assert_ne!(
            w1, w2,
            "warns must distinguish the two original paths even if the lossy forms collide"
        );
    }

    #[test]
    fn warn_helpers_pinned_for_discoverability() {
        let src = include_str!("cli.rs");
        assert!(src.contains("fn format_cli_build_sourcemap_non_utf8_entry_warn"));
        assert!(src.contains("fn coerce_sourcemap_entry_path_or_warn"));
    }

    #[test]
    fn each_warn_string_carries_gh3803_tag() {
        let s =
            format_cli_build_sourcemap_non_utf8_entry_warn(Path::new("/proj/x.ts"), "lossy-form");
        assert!(s.starts_with("gh3803:"), "missing gh3803 tag: {s:?}");
        assert!(s.contains("non-UTF-8 entry path"));
    }

    #[test]
    fn warn_distinct_from_prior_silent_fallback_families() {
        let s = format_cli_build_sourcemap_non_utf8_entry_warn(Path::new("/x"), "lossy");
        for tag in [
            "gh3763", "gh3765", "gh3768", "gh3770", "gh3772", "gh3774", "gh3776", "gh3787",
            "gh3789", "gh3791", "gh3793", "gh3795", "gh3797", "gh3799", "gh3801",
        ] {
            assert!(!s.contains(tag), "gh3803 warn must not carry {tag}: {s:?}");
        }
    }

    #[test]
    fn warn_explains_devtools_consequence() {
        let s = format_cli_build_sourcemap_non_utf8_entry_warn(Path::new("/x"), "lossy");
        assert!(
            s.contains("devtools") || s.contains("stack trace"),
            "warn should explain the devtools/stack-trace consequence: {s:?}"
        );
    }

    #[test]
    fn both_call_sites_route_through_helper() {
        // GH #3803 sibling: both the External and Inline source-map arms
        // must build their `sources[]` via the helper, not via inline
        // `to_string_lossy()`. A drive-by refactor that re-inlines the
        // lossy call on either arm would re-open the silent-collision
        // bug for that mode.
        let src = include_str!("cli.rs");
        let body_end = src.find("#[cfg(test)]").unwrap_or(src.len());
        let body = &src[..body_end];
        // No inline `entry.to_string_lossy().to_string()` should appear
        // in the build subcommand's source-map block any more.
        let inline_marker = "entry.to_string_lossy().to_string()";
        let occurrences = body.matches(inline_marker).count();
        assert_eq!(
            occurrences, 0,
            "found {occurrences} inline `entry.to_string_lossy().to_string()` calls; expected 0 — both arms must use coerce_sourcemap_entry_path_or_warn"
        );
        // Both External and Inline arms should mention the helper.
        let helper_calls = body
            .matches("coerce_sourcemap_entry_path_or_warn(&entry)")
            .count();
        assert!(
            helper_calls >= 2,
            "expected helper called from both External and Inline arms (>= 2 hits); got {helper_calls}"
        );
    }

    #[test]
    fn happy_path_relative_entry_passes_through() {
        let s = coerce_sourcemap_entry_path_or_warn(Path::new("src/index.ts"));
        assert_eq!(s, "src/index.ts");
    }

    #[cfg(unix)]
    #[test]
    fn warn_carries_original_path_debug_form() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let p = PathBuf::from(OsStr::from_bytes(b"/proj/\xFFnonutf8.ts"));
        let lossy = p.to_string_lossy().into_owned();
        let s = format_cli_build_sourcemap_non_utf8_entry_warn(&p, &lossy);
        assert!(
            s.contains("entry="),
            "warn should label the original path with entry=: {s:?}"
        );
        assert!(
            s.contains("lossy form is"),
            "warn should label the lossy form: {s:?}"
        );
    }
}
// CODEGEN-END
