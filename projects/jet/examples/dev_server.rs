// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-examples.md#schema
// CODEGEN-BEGIN
//! Jet Dev Server Example
//!
//! Starts a dev server for the Todo app at `examples/jet/`.
//! Open http://127.0.0.1:5000 in your browser.
//!
//! Run: cargo run -p jet --example dev_server

use jet::asset::AssetOptions;
use jet::bundler::BundleOptions;
use jet::dev_server::{DevServer, ServerConfig};
use jet::resolver::ResolveOptions;
use jet::transform::{TransformOptions, TypeScriptTarget};

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

fn find_example_root() -> PathBuf {
    let candidates = [
        PathBuf::from("examples/jet"),
        PathBuf::from("../../examples/jet"),
    ];
    for c in &candidates {
        if c.join("src/index.tsx").exists() {
            return fs::canonicalize(c).unwrap();
        }
    }
    panic!("Cannot find examples/jet/ — run from workspace root");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let root = find_example_root();

    println!();
    println!("  Jet Dev Server");
    println!("  Project: {}", root.display());
    println!();

    let bundle_options = BundleOptions {
        entry: root.join("src/index.tsx"),
        output_dir: root.join("dist"),
        source_maps: false,
        minify: false,
        resolve_options: ResolveOptions {
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
        },
        transform_options: TransformOptions {
            jsx_automatic: true,
            ts_target: TypeScriptTarget::ES2020,
            source_maps: false,
            ..Default::default()
        },
        asset_options: AssetOptions::default(),
        css_bundle: false,
        externals: HashSet::new(),
        externalize_all_packages: false,
        defines: std::collections::HashMap::new(),
    };

    let bundler = jet::bundler::Bundler::new(bundle_options)?;

    let server_config = ServerConfig {
        host: "127.0.0.1".into(),
        port: 5000,
        root_dir: root.clone(),
        public_dir: Some(root.join("public")),
        entry: PathBuf::from("src/index.tsx"),
        proxy: std::collections::HashMap::new(),
        aliases: std::collections::HashMap::new(),
    };

    println!("  http://{}:{}", server_config.host, server_config.port);
    println!();

    let server = Arc::new(DevServer::new(bundler, server_config)?);
    server.start().await?;

    Ok(())
}
// CODEGEN-END
