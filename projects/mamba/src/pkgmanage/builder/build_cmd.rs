// `mamba build` subcommand body. Lifted verbatim from `main.rs` so that
// pkgmanage owns end-to-end build orchestration (manifest discovery →
// CompilerSession → linker invocation).

use anyhow::Result;
use clap::ArgMatches;

use crate::driver::{Backend, CompilerConfig, CompilerSession, EmitMode, MambaConfig};

pub fn cmd_build(sub: &ArgMatches) -> Result<()> {
    let project_config: Option<MambaConfig> =
        if let Some(cfg_path) = sub.get_one::<String>("config") {
            Some(MambaConfig::from_file(std::path::Path::new(cfg_path))?)
        } else {
            None
        };

    let file_arg = sub.get_one::<String>("file");
    let entry_from_config = project_config
        .as_ref()
        .and_then(|c| c.entry_point().map(|s| s.to_string()));
    let file: String = match (file_arg, entry_from_config) {
        (Some(f), _) => f.clone(),
        (None, Some(ep)) => ep,
        (None, None) => anyhow::bail!(
            "no source file specified and no mamba.toml found; pass a file or use --config"
        ),
    };

    let backend = match sub.get_one::<String>("backend").map(|s| s.as_str()) {
        Some("cranelift") | None => Backend::Cranelift,
        Some("llvm") => Backend::Llvm,
        Some("wasm") => Backend::Wasm,
        Some(other) => anyhow::bail!("unknown backend: {other}"),
    };
    let emit = sub.get_one::<String>("emit").map(|s| match s.as_str() {
        "ast" => EmitMode::Ast,
        "hir" => EmitMode::Hir,
        "mir" => EmitMode::Mir,
        _ => EmitMode::Ast,
    });
    let output = sub.get_one::<String>("output").map(|s| s.as_str());

    let config = CompilerConfig {
        backend,
        emit,
        project_config,
        ..Default::default()
    };
    let mut session = CompilerSession::new(config);
    match session.build(&file, output) {
        Ok(bytes) => {
            if bytes.is_empty() {
                return Ok(());
            }
            if let Some(exe_path) = output {
                let tmp_obj = format!("{exe_path}.o");
                std::fs::write(&tmp_obj, &bytes)
                    .map_err(|e| anyhow::anyhow!("write {tmp_obj}: {e}"))?;
                let status = std::process::Command::new("cc")
                    .args([&tmp_obj, "-o", exe_path])
                    .status()
                    .map_err(|e| anyhow::anyhow!("invoke linker: {e}"))?;
                let _ = std::fs::remove_file(&tmp_obj);
                if !status.success() {
                    anyhow::bail!("linker failed with {status}");
                }
                println!("Built executable: {exe_path}");
            } else {
                let obj_path = std::path::Path::new(&file)
                    .with_extension("o")
                    .to_string_lossy()
                    .to_string();
                std::fs::write(&obj_path, &bytes)
                    .map_err(|e| anyhow::anyhow!("write {obj_path}: {e}"))?;
                println!("Wrote object file: {obj_path}");
            }
        }
        Err(e) => {
            eprintln!("{}", session.render_error(&e));
            std::process::exit(1);
        }
    }
    Ok(())
}
