// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-examples.md#schema
// CODEGEN-BEGIN
//! Complete Jet (Talos) Pipeline Example
//!
//! Builds the sample Todo app at `examples/jet/` through the full pipeline:
//!   1. Resolver  - Node.js-compatible module resolution
//!   2. Transform - JSX / TypeScript / CSS code transformation
//!   3. Bundler   - Dependency graph + parallel compilation + bundle output
//!   4. DevServer - Configuration overview (not started)
//!
//! Run: cargo run -p jet --example full_pipeline

use jet::asset::AssetOptions;
use jet::bundler::{BundleOptions, Bundler};
use jet::dev_server::ServerConfig;
use jet::resolver::{ModuleResolver, ResolveOptions};
use jet::transform::{TransformOptions, Transformer, TypeScriptTarget};

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Locate `examples/jet/` relative to workspace root.
fn find_example_root() -> PathBuf {
    // When run via `cargo run -p jet --example`, cwd is workspace root
    let candidates = [
        PathBuf::from("examples/jet"),
        PathBuf::from("../../examples/jet"), // from crate dir
    ];
    for c in &candidates {
        if c.join("src/index.tsx").exists() {
            return fs::canonicalize(c).unwrap();
        }
    }
    panic!("Cannot find examples/jet/ — run from workspace root");
}

// ─── Shared resolve options ──────────────────────────────────────────────────

fn resolve_options() -> ResolveOptions {
    ResolveOptions {
        extensions: vec![
            "ts".into(),
            "tsx".into(),
            "js".into(),
            "jsx".into(),
            "json".into(),
            "css".into(),
        ],
        resolve_index: true,
        ..Default::default()
    }
}

fn transform_options() -> TransformOptions {
    TransformOptions {
        jsx_automatic: true,
        ts_target: TypeScriptTarget::ES2020,
        source_maps: false,
        ..Default::default()
    }
}

// ─── Part 1: Resolver ────────────────────────────────────────────────────────

fn demo_resolver(root: &Path) {
    println!("=== Part 1: Module Resolver ===\n");

    let resolver = ModuleResolver::new(resolve_options()).unwrap();
    let from = root.join("src/index.tsx");

    let cases: &[(&str, &str)] = &[
        ("./App", "relative .tsx"),
        ("./utils/store", "relative .ts"),
        ("./utils/id", "relative .ts (transitive)"),
        ("./styles/reset.css", "relative .css"),
        ("./styles/app.css", "relative .css"),
        ("./utils", "directory -> index.ts"),
    ];

    for (specifier, label) in cases {
        match resolver.resolve(specifier, &from) {
            Ok(resolved) => {
                let short = resolved.path.strip_prefix(root).unwrap_or(&resolved.path);
                println!("  {:<25} -> {} ({})", specifier, short.display(), label);
            }
            Err(e) => println!("  {:<25} -> ERROR: {} ({})", specifier, e, label),
        }
    }
    println!();
}

// ─── Part 2: Transformer ────────────────────────────────────────────────────

fn demo_transformer(root: &Path) {
    println!("=== Part 2: Code Transformer ===\n");

    let transformer = Transformer::new(transform_options());

    // TSX -> JS (type stripping + JSX)
    let tsx_src = fs::read_to_string(root.join("src/components/Header.tsx")).unwrap();
    let tsx_out = transformer
        .transform_js(&tsx_src, Path::new("Header.tsx"))
        .unwrap();
    println!(
        "  [TSX] Header.tsx  {} -> {} bytes",
        tsx_src.len(),
        tsx_out.code.len()
    );
    for line in tsx_out.code.lines().take(4) {
        println!("    {}", line);
    }
    println!();

    // TS -> JS (type stripping only)
    let ts_src = fs::read_to_string(root.join("src/utils/time.ts")).unwrap();
    let ts_out = transformer
        .transform_js(&ts_src, Path::new("time.ts"))
        .unwrap();
    println!(
        "  [TS]  time.ts  {} -> {} bytes",
        ts_src.len(),
        ts_out.code.len()
    );
    for line in ts_out.code.lines().take(4) {
        println!("    {}", line);
    }
    println!();

    // CSS -> JS injection
    let css_src = fs::read_to_string(root.join("src/styles/app.css")).unwrap();
    let css_out = transformer.transform_css(&css_src).unwrap();
    println!(
        "  [CSS] app.css -> JS injection  {} bytes",
        css_out.code.len()
    );
    for line in css_out.code.lines().take(3) {
        println!("    {}", line);
    }
    println!();
}

// ─── Part 3: Bundler ─────────────────────────────────────────────────────────

async fn demo_bundler(root: &Path) {
    println!("=== Part 3: Bundler (full pipeline) ===\n");

    let entry = root.join("src/index.tsx");
    let output_dir = root.join("dist");
    fs::create_dir_all(&output_dir).ok();

    let options = BundleOptions {
        entry: entry.clone(),
        output_dir: output_dir.clone(),
        source_maps: false,
        minify: false,
        resolve_options: resolve_options(),
        transform_options: transform_options(),
        asset_options: AssetOptions::default(),
        externals: HashSet::new(),
        externalize_all_packages: false,
        defines: std::collections::HashMap::new(),
    };

    let bundler = Bundler::new(options).unwrap();

    match bundler.bundle(entry).await {
        Ok(output) => {
            let lines = output.code.lines().count();
            let size = output.code.len();

            println!("  Bundle OK");
            println!("  Size:   {} bytes  |  Lines: {}", size, lines);
            println!("  Assets: {}", output.assets.len());

            let out_path = output_dir.join("bundle.js");
            fs::write(&out_path, &output.code).unwrap();
            println!("  Output: {}\n", out_path.display());

            // Count modules in bundle (each __talos__.define call)
            let module_count = output.code.matches("__talos__.define(").count();
            println!("  Modules bundled: {}", module_count);
            println!();

            println!("  ---- bundle preview (first 35 lines) ----");
            for (i, line) in output.code.lines().enumerate().take(35) {
                println!("  {:>3}: {}", i + 1, line);
            }
            if lines > 35 {
                println!("  ... ({} more lines)", lines - 35);
            }
        }
        Err(e) => {
            eprintln!("  Bundle FAILED: {}", e);
        }
    }
    println!();
}

// ─── Part 4: DevServer config ────────────────────────────────────────────────

fn demo_dev_server_config(root: &Path) {
    println!("=== Part 4: Dev Server Configuration ===\n");

    let config = ServerConfig {
        host: "127.0.0.1".into(),
        port: 5000,
        root_dir: root.to_path_buf(),
        public_dir: Some(root.join("public")),
        entry: PathBuf::from("src/index.tsx"),
        proxy: std::collections::HashMap::new(),
        aliases: std::collections::HashMap::new(),
    };

    println!("  Host:   {}:{}", config.host, config.port);
    println!("  Root:   {}", config.root_dir.display());
    println!("  Public: {:?}\n", config.public_dir);
    println!("  Routes:");
    println!("    /              -> index.html (with HMR client)");
    println!("    /bundle.js     -> on-the-fly bundle");
    println!("    /static/*      -> public/ directory");
    println!("    /__talos_hmr   -> WebSocket HMR\n");
    println!("  (Not started — use `ob talos dev`)\n");
}

// ─── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    let root = find_example_root();

    println!();
    println!("========================================");
    println!("  Jet (Talos) — Full Pipeline Example");
    println!("========================================");
    println!();
    println!("  Project: {}\n", root.display());

    demo_resolver(&root);
    demo_transformer(&root);
    demo_bundler(&root).await;
    demo_dev_server_config(&root);

    println!("========================================");
    println!("  Done!");
    println!("========================================");

    Ok(())
}
// CODEGEN-END
