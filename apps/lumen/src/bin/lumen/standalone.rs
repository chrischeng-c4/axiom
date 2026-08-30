//! Standalone deployment commands that do not require a running Lumen server.

use super::{
    StandaloneArgs, StandaloneBackupArgs, StandaloneCmd, StandaloneComposeCmd,
    StandaloneComposePatchArgs, StandaloneRestoreArgs,
};
use anyhow::{bail, Context, Result};
use serde_yaml::Value;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
#[cfg(all(feature = "backup", feature = "delegated-auth"))]
use std::process::{Command, Stdio};
#[cfg(all(feature = "backup", feature = "delegated-auth"))]
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

const IMAGE: &str = "ghcr.io/chrischeng-c4/lumen:0.4.30";
const MANAGED_LABEL: &str = "com.axiom.lumen.managed";
const MANAGED_LABEL_VALUE: &str = "com.axiom.lumen.managed=true";

#[path = "standalone/gke.rs"]
mod gke;

pub(crate) async fn run(args: StandaloneArgs) -> Result<()> {
    match args.cmd {
        StandaloneCmd::Compose(compose) => compose_patch(StandaloneArgs {
            cmd: StandaloneCmd::Compose(compose),
        }),
        StandaloneCmd::Gke(args) => gke::run(args),
        StandaloneCmd::Backup(args) => backup(args).await,
        StandaloneCmd::Restore(args) => restore(args).await,
    }
}

pub(crate) fn compose_patch(args: StandaloneArgs) -> Result<()> {
    let StandaloneCmd::Compose(compose) = args.cmd else {
        bail!("invalid standalone command")
    };
    let StandaloneComposeCmd::Patch(patch) = compose.cmd;
    patch_compose(patch)
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 63
        && name.as_bytes()[0].is_ascii_alphanumeric()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

fn managed_labels(value: &Value) -> bool {
    match value {
        Value::Mapping(labels) => labels.iter().any(|(key, value)| {
            key.as_str() == Some(MANAGED_LABEL)
                && (value.as_str() == Some("true") || value.as_bool() == Some(true))
        }),
        Value::Sequence(labels) => labels
            .iter()
            .any(|label| label.as_str() == Some(MANAGED_LABEL_VALUE)),
        _ => false,
    }
}

fn temporary_file(parent: &Path, target: &Path) -> Result<(PathBuf, File)> {
    let filename = target
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("compose path must name a file"))?
        .to_string_lossy();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    for attempt in 0..100u32 {
        let path = parent.join(format!(
            ".{filename}.lumen.tmp-{}-{stamp}-{attempt}",
            std::process::id()
        ));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    bail!("could not allocate a unique compose temporary file")
}

fn patch_compose(args: StandaloneComposePatchArgs) -> Result<()> {
    if !valid_name(&args.name) {
        bail!("invalid standalone name")
    }
    let parent = args
        .file
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut root: Value = if args.file.exists() {
        let source = fs::read_to_string(&args.file).context("read compose file")?;
        if source.trim().is_empty() {
            Value::Mapping(Default::default())
        } else {
            serde_yaml::from_str(&source).context("parse compose YAML")?
        }
    } else {
        Value::Mapping(Default::default())
    };
    let map = root
        .as_mapping_mut()
        .ok_or_else(|| anyhow::anyhow!("compose root must be a mapping"))?;
    let services = map
        .entry(Value::String("services".into()))
        .or_insert_with(|| Value::Mapping(Default::default()));
    let services = services
        .as_mapping_mut()
        .ok_or_else(|| anyhow::anyhow!("compose services must be a mapping"))?;
    let key = Value::String(args.name.clone());
    if let Some(existing) = services.get(&key) {
        if !existing
            .as_mapping()
            .and_then(|m| m.get(Value::String("labels".into())))
            .is_some_and(managed_labels)
        {
            bail!("service already exists and is not managed by Lumen")
        }
    }
    let service: Value = serde_yaml::from_str(&format!(
        "image: {IMAGE}\nports:\n  - '127.0.0.1:7373:7373'\nvolumes:\n  - '{}-data:/var/lib/lumen/data'\nenvironment:\n  LUMEN_AUTH: off\nlabels:\n  com.axiom.lumen.managed: 'true'\n",
        args.name
    ))?;
    services.insert(key, service);
    let volumes = map
        .entry(Value::String("volumes".into()))
        .or_insert_with(|| Value::Mapping(Default::default()));
    let volumes = volumes
        .as_mapping_mut()
        .ok_or_else(|| anyhow::anyhow!("compose volumes must be a mapping"))?;
    volumes
        .entry(Value::String(format!("{}-data", args.name)))
        .or_insert(Value::Mapping(Default::default()));
    let bytes = serde_yaml::to_string(&root)
        .context("serialize compose")?
        .into_bytes();
    let (tmp, mut f) = temporary_file(parent, &args.file).context("create temporary compose")?;
    let result = (|| -> Result<()> {
        f.write_all(&bytes)
            .context("write temporary compose file")?;
        f.sync_all().context("fsync temporary compose file")?;
        fs::rename(&tmp, &args.file).context("replace compose file")?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&tmp);
        return Err(error);
    }
    File::open(parent)
        .context("open compose parent directory")?
        .sync_all()
        .context("fsync compose parent directory")?;
    Ok(())
}

#[cfg(feature = "backup")]
fn compose_base_url(path: &Path, name: Option<&str>) -> Result<String> {
    let name = name.unwrap_or("lumen");
    if !valid_name(name) {
        bail!("invalid standalone name")
    }
    let source = fs::read_to_string(path).context("read compose file")?;
    let root: Value = serde_yaml::from_str(&source).context("parse compose YAML")?;
    let service = root
        .get("services")
        .and_then(Value::as_mapping)
        .and_then(|services| services.get(Value::String(name.to_string())))
        .ok_or_else(|| anyhow::anyhow!("managed Compose service `{name}` was not found"))?;
    let managed = service
        .as_mapping()
        .and_then(|mapping| mapping.get(Value::String("labels".into())))
        .is_some_and(managed_labels);
    if !managed {
        bail!("Compose service `{name}` is not managed by Lumen")
    }
    if service.get("image").and_then(Value::as_str) != Some(IMAGE) {
        bail!("Compose service `{name}` does not use the Lumen 0.4.30 image")
    }
    let ports = service
        .get("ports")
        .and_then(Value::as_sequence)
        .ok_or_else(|| anyhow::anyhow!("Compose service `{name}` has no declared port"))?;
    if ports.len() != 1 || ports[0].as_str() != Some("127.0.0.1:7373:7373") {
        bail!("Compose service `{name}` must declare only 127.0.0.1:7373:7373")
    }
    Ok("http://127.0.0.1:7373".into())
}

#[cfg(feature = "backup")]
fn validate_snapshot(bytes: &[u8]) -> Result<()> {
    let snapshot: lumen::storage::SnapshotV1 =
        serde_json::from_slice(bytes).context("decode SnapshotV1")?;
    lumen::storage::Engine::new()
        .restore(snapshot)
        .context("validate SnapshotV1")?;
    Ok(())
}

#[cfg(feature = "backup")]
fn write_backup(path: &Path, bytes: &[u8]) -> Result<()> {
    storage_durable::atomic_write_strict(path, bytes).context("write backup output")?;
    Ok(())
}

#[cfg(not(feature = "backup"))]
async fn backup(_args: StandaloneBackupArgs) -> Result<()> {
    bail!("standalone backup requires the `backup` feature")
}

#[cfg(feature = "backup")]
async fn backup(args: StandaloneBackupArgs) -> Result<()> {
    if args.gke.is_some() && args.name.is_some() {
        bail!("--name is valid only with --compose")
    }
    if let Some(path) = args.compose {
        #[cfg(feature = "backup")]
        {
            let url = compose_base_url(&path, args.name.as_deref())?;
            let transport = service_backup::AdminSnapshotTransport::new()
                .context("build admin snapshot transport")?;
            let bytes = transport.fetch_exact(&url, None).await?;
            validate_snapshot(&bytes)?;
            write_backup(&args.out, &bytes)?;
            println!("backup complete: {}", args.out.display());
            return Ok(());
        }
    }
    if args.name.is_some() {
        bail!("--name is valid only with --compose")
    }
    let path = args.gke.context("exactly one backup target is required")?;
    gke_backup(path, args.out).await
}

#[cfg(not(feature = "backup"))]
async fn restore(_args: StandaloneRestoreArgs) -> Result<()> {
    bail!("standalone restore requires the `backup` feature")
}

#[cfg(feature = "backup")]
async fn restore(args: StandaloneRestoreArgs) -> Result<()> {
    if !args.replace {
        bail!("--replace is required")
    }
    if args.gke.is_some() && args.name.is_some() {
        bail!("--name is valid only with --compose")
    }
    #[cfg(not(feature = "delegated-auth"))]
    if args.gke.is_some() {
        bail!("GKE restore requires the `delegated-auth` feature")
    }
    let bytes = fs::read(&args.file).context("read backup file")?;
    validate_snapshot(&bytes)?;
    if let Some(path) = args.compose {
        #[cfg(feature = "backup")]
        {
            let url = compose_base_url(&path, args.name.as_deref())?;
            let transport = service_backup::AdminSnapshotTransport::new()
                .context("build admin snapshot transport")?;
            transport.restore_exact(&url, None, &bytes).await?;
            println!("restore complete");
            return Ok(());
        }
    }
    if args.name.is_some() {
        bail!("--name is valid only with --compose")
    }
    let path = args.gke.context("exactly one restore target is required")?;
    gke_restore(path, bytes).await
}

#[cfg(feature = "backup")]
async fn gke_backup(path: PathBuf, out: PathBuf) -> Result<()> {
    #[cfg(feature = "delegated-auth")]
    {
        let target = gke::load_target(&path)?;
        let port = cli_std::connect::free_local_port()?;
        let mut command = Command::new("kubectl");
        command.args([
            "port-forward",
            "-n",
            &target.namespace,
            &format!("svc/{}", target.name),
            &format!("{port}:7373"),
        ]);
        command.stdout(Stdio::null()).stderr(Stdio::null());
        let _guard = cli_std::connect::ChildGuard::spawn(&mut command)?;
        let result = tokio::select! {
            result = async {
                wait_for_local_port_ready_async(port, Duration::from_secs(30)).await?;
                let minter = service_auth::k8s::KubeTokenMinter::from_ambient_config().await?;
                let source = service_auth::k8s::TokenSource::new(
                    std::sync::Arc::new(minter),
                    service_auth::k8s::TokenRequestTarget::kubernetes_default(
                        &target.namespace,
                        format!("{}-admin", target.name),
                    )?,
                );
                let token = source.token().await?;
                let transport = service_backup::AdminSnapshotTransport::new()
                    .context("build admin snapshot transport")?;
                let bytes = transport
                    .fetch_exact(&format!("http://127.0.0.1:{port}"), Some(token.expose()))
                    .await?;
                validate_snapshot(&bytes)?;
                write_backup(&out, &bytes)
            } => result,
            _ = tokio::signal::ctrl_c() => bail!("interrupted"),
        };
        result?;
        println!("backup complete: {}", out.display());
        return Ok(());
    }
    #[cfg(not(feature = "delegated-auth"))]
    let _ = (path, out);
    #[cfg(not(feature = "delegated-auth"))]
    bail!("GKE backup requires the `delegated-auth` feature")
}

#[cfg(feature = "backup")]
async fn gke_restore(path: PathBuf, bytes: Vec<u8>) -> Result<()> {
    #[cfg(feature = "delegated-auth")]
    {
        let target = gke::load_target(&path)?;
        let port = cli_std::connect::free_local_port()?;
        let mut command = Command::new("kubectl");
        command.args([
            "port-forward",
            "-n",
            &target.namespace,
            &format!("svc/{}", target.name),
            &format!("{port}:7373"),
        ]);
        command.stdout(Stdio::null()).stderr(Stdio::null());
        let _guard = cli_std::connect::ChildGuard::spawn(&mut command)?;
        let result = tokio::select! {
            result = async {
                wait_for_local_port_ready_async(port, Duration::from_secs(30)).await?;
                let minter = service_auth::k8s::KubeTokenMinter::from_ambient_config().await?;
                let source = service_auth::k8s::TokenSource::new(
                    std::sync::Arc::new(minter),
                    service_auth::k8s::TokenRequestTarget::kubernetes_default(
                        &target.namespace,
                        format!("{}-admin", target.name),
                    )?,
                );
                let token = source.token().await?;
                let transport = service_backup::AdminSnapshotTransport::new()
                    .map_err(anyhow::Error::new)
                    .context("build admin snapshot transport")?;
                transport
                    .restore_exact(
                        &format!("http://127.0.0.1:{port}"),
                        Some(token.expose()),
                        &bytes,
                    )
                    .await
                    .map_err(anyhow::Error::new)
            } => result,
            _ = tokio::signal::ctrl_c() => bail!("interrupted"),
        };
        result?;
        println!("restore complete");
        return Ok(());
    }
    #[cfg(not(feature = "delegated-auth"))]
    let _ = (path, bytes);
    #[cfg(not(feature = "delegated-auth"))]
    bail!("GKE restore requires the `delegated-auth` feature")
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
async fn wait_for_local_port_ready_async(port: u16, timeout: Duration) -> Result<()> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return Ok(());
        }
        let now = tokio::time::Instant::now();
        if now >= deadline {
            bail!("timed out waiting for local port {port}")
        }
        tokio::time::sleep((deadline - now).min(Duration::from_millis(100))).await;
    }
}
