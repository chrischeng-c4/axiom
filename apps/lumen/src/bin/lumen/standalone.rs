//! Standalone deployment commands that do not require a running Lumen server.

use super::{StandaloneArgs, StandaloneCmd, StandaloneComposeCmd, StandaloneComposePatchArgs};
use anyhow::{bail, Context, Result};
use serde_yaml::Value;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const IMAGE: &str = "ghcr.io/chrischeng-c4/lumen:0.4.29";
const MANAGED_LABEL: &str = "com.axiom.lumen.managed";
const MANAGED_LABEL_VALUE: &str = "com.axiom.lumen.managed=true";

pub(crate) fn compose_patch(args: StandaloneArgs) -> Result<()> {
    let StandaloneCmd::Compose(compose) = args.cmd;
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
