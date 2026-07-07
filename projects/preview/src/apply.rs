// <HANDWRITE gap="issue-1109:local-apply-gitops-execution" tracker="projects-preview-src-apply-rs" reason="Local apply and GitOps execution adapters are hand-authored until Preview has generator primitives for command execution surfaces.">
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::PreviewEnvironment;

const BASE_APPLY_ORDER: [&str; 9] = [
    "k8s/namespace.yaml",
    "k8s/service-account.yaml",
    "k8s/resource-quota.yaml",
    "k8s/limit-range.yaml",
    "k8s/workload-role.yaml",
    "k8s/workload-role-binding.yaml",
    "k8s/deployment.yaml",
    "k8s/service.yaml",
    "router/route-binding.yaml",
];

const DATA_SECRET_PATH: &str = "k8s/data-secret.yaml";

const PROTECTED_NAMESPACE_NAMES: [&str; 4] =
    ["default", "kube-system", "preview-system", "uat-base"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestInventory {
    pub schema_version: u8,
    pub namespace: String,
    pub route_target: String,
    pub entries: Vec<ManifestInventoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestInventoryEntry {
    pub order: usize,
    pub path: String,
    pub api_version: String,
    pub kind: String,
    pub namespace: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyOptions {
    pub dir: PathBuf,
    pub context: Option<String>,
    pub dry_run: bool,
    pub allow_non_kind: bool,
    pub plan_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplySummary {
    pub context: String,
    pub dry_run: bool,
    pub plan_only: bool,
    pub inventory: ManifestInventory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitopsBundleFile {
    pub path: String,
    pub contents: String,
}

pub fn apply_manifest_paths() -> Vec<&'static str> {
    BASE_APPLY_ORDER.to_vec()
}

pub fn manifest_inventory_for_env(env: &PreviewEnvironment) -> Result<ManifestInventory> {
    let root = Path::new("");
    let order = apply_order_for_env(env);
    let entries = order
        .iter()
        .enumerate()
        .map(|(order, path)| {
            let object = rendered_object_for_path(env, path)?;
            inventory_entry(order, path, &object)
        })
        .collect::<Result<Vec<_>>>()?;

    let inventory = ManifestInventory {
        schema_version: 1,
        namespace: env.spec.namespace.clone(),
        route_target: env.spec.route.target.clone(),
        entries,
    };
    validate_inventory(&inventory, root)?;
    Ok(inventory)
}

pub fn manifest_inventory_from_dir(dir: &Path) -> Result<ManifestInventory> {
    let order = apply_order_for_dir(dir);
    let entries = order
        .iter()
        .enumerate()
        .map(|(order, relative)| {
            let path = dir.join(relative);
            let contents =
                fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
            let object: Value = serde_yaml::from_str(&contents)
                .with_context(|| format!("parse manifest {}", path.display()))?;
            inventory_entry(order, relative, &object)
        })
        .collect::<Result<Vec<_>>>()?;

    let namespace = entries
        .iter()
        .find(|entry| entry.kind == "Namespace")
        .map(|entry| entry.name.clone())
        .ok_or_else(|| anyhow!("manifest inventory missing Namespace"))?;
    let route_target = route_target_from_dir(dir)?;
    let inventory = ManifestInventory {
        schema_version: 1,
        namespace,
        route_target,
        entries,
    };
    validate_inventory(&inventory, dir)?;
    Ok(inventory)
}

pub fn apply_rendered_manifests(options: &ApplyOptions) -> Result<ApplySummary> {
    let inventory = manifest_inventory_from_dir(&options.dir)?;
    let context = match &options.context {
        Some(context) => context.clone(),
        None if options.plan_only => "plan-only".to_string(),
        None => current_kubectl_context()?,
    };
    if !options.plan_only {
        assert_kind_context(&context, options.allow_non_kind)?;
        for entry in &inventory.entries {
            let path = options.dir.join(&entry.path);
            kubectl_apply(&path, options.context.as_deref(), options.dry_run)?;
        }
    }

    Ok(ApplySummary {
        context,
        dry_run: options.dry_run,
        plan_only: options.plan_only,
        inventory,
    })
}

pub fn render_gitops_bundle(dir: &Path) -> Result<Vec<GitopsBundleFile>> {
    let inventory = manifest_inventory_from_dir(dir)?;
    let mut files = Vec::new();
    let mut kustomization_resources = Vec::new();

    for entry in &inventory.entries {
        let source = dir.join(&entry.path);
        let contents =
            fs::read_to_string(&source).with_context(|| format!("read {}", source.display()))?;
        let target = format!(
            "manifests/{:02}-{}",
            entry.order,
            entry
                .path
                .rsplit('/')
                .next()
                .ok_or_else(|| anyhow!("invalid manifest path {}", entry.path))?
        );
        kustomization_resources.push(target.clone());
        files.push(GitopsBundleFile {
            path: target,
            contents,
        });
    }

    files.push(GitopsBundleFile {
        path: "manifest-inventory.json".to_string(),
        contents: serde_json::to_string_pretty(&inventory)? + "\n",
    });
    files.push(GitopsBundleFile {
        path: "kustomization.yaml".to_string(),
        contents: render_kustomization(&kustomization_resources),
    });
    Ok(files)
}

pub fn apply_summary_markdown(summary: &ApplySummary) -> String {
    let mode = if summary.plan_only {
        "plan-only"
    } else if summary.dry_run {
        "server-dry-run"
    } else {
        "apply"
    };
    let mut output = format!(
        "### Preview Apply Summary\n\nMode: `{mode}`\nContext: `{}`\nNamespace: `{}`\nRoute target: `{}`\n\nObjects:\n",
        summary.context, summary.inventory.namespace, summary.inventory.route_target
    );
    for entry in &summary.inventory.entries {
        let namespace = entry.namespace.as_deref().unwrap_or("<cluster>");
        output.push_str(&format!(
            "- {:02} `{}` `{}/{}` from `{}`\n",
            entry.order, entry.kind, namespace, entry.name, entry.path
        ));
    }
    output
}

fn rendered_object_for_path(env: &PreviewEnvironment, path: &str) -> Result<Value> {
    let contents = crate::render::render_single_manifest(env, path)?;
    serde_yaml::from_str(&contents).with_context(|| format!("parse rendered manifest {path}"))
}

fn inventory_entry(order: usize, path: &str, object: &Value) -> Result<ManifestInventoryEntry> {
    let api_version = object
        .get("apiVersion")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("{path} missing apiVersion"))?
        .to_string();
    let kind = object
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("{path} missing kind"))?
        .to_string();
    let name = object
        .pointer("/metadata/name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("{path} missing metadata.name"))?
        .to_string();
    let namespace = object
        .pointer("/metadata/namespace")
        .and_then(Value::as_str)
        .map(str::to_string);
    Ok(ManifestInventoryEntry {
        order,
        path: path.to_string(),
        api_version,
        kind,
        namespace,
        name,
    })
}

fn validate_inventory(inventory: &ManifestInventory, root: &Path) -> Result<()> {
    let expected_order = if inventory
        .entries
        .iter()
        .any(|entry| entry.path == DATA_SECRET_PATH)
    {
        apply_order_with_data()
    } else {
        BASE_APPLY_ORDER.to_vec()
    };
    if inventory.entries.len() != expected_order.len() {
        bail!("manifest inventory has an unexpected number of entries");
    }
    for (order, expected) in expected_order.iter().enumerate() {
        let Some(entry) = inventory.entries.get(order) else {
            bail!("manifest inventory missing order {order}");
        };
        if entry.order != order || entry.path != *expected {
            bail!(
                "manifest inventory order mismatch at {order}: expected {expected}, got {}",
                entry.path
            );
        }
        if entry.kind == "Namespace" && PROTECTED_NAMESPACE_NAMES.contains(&entry.name.as_str()) {
            bail!("refusing to apply protected namespace {}", entry.name);
        }
        if Path::new(&entry.path).is_absolute() || entry.path.contains("..") {
            bail!(
                "manifest inventory path must be relative and bounded: {}",
                entry.path
            );
        }
        if !root.as_os_str().is_empty() && !root.join(&entry.path).is_file() {
            bail!("manifest inventory path does not exist: {}", entry.path);
        }
    }
    Ok(())
}

fn apply_order_for_env(env: &PreviewEnvironment) -> Vec<&'static str> {
    if env.spec.data.is_some() {
        apply_order_with_data()
    } else {
        BASE_APPLY_ORDER.to_vec()
    }
}

fn apply_order_for_dir(dir: &Path) -> Vec<&'static str> {
    if dir.join(DATA_SECRET_PATH).is_file() {
        apply_order_with_data()
    } else {
        BASE_APPLY_ORDER.to_vec()
    }
}

fn apply_order_with_data() -> Vec<&'static str> {
    let mut order = BASE_APPLY_ORDER.to_vec();
    order.insert(6, DATA_SECRET_PATH);
    order
}

fn route_target_from_dir(dir: &Path) -> Result<String> {
    let path = dir.join("router/route-binding.yaml");
    let contents = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let object: Value =
        serde_yaml::from_str(&contents).with_context(|| format!("parse {}", path.display()))?;
    object
        .pointer("/data/target")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("route binding missing data.target"))
}

fn current_kubectl_context() -> Result<String> {
    let output = Command::new("kubectl")
        .args(["config", "current-context"])
        .output()
        .context("start kubectl config current-context")?;
    if !output.status.success() {
        bail!(
            "kubectl config current-context failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn assert_kind_context(context: &str, allow_non_kind: bool) -> Result<()> {
    if context.starts_with("kind-") || allow_non_kind {
        return Ok(());
    }
    bail!(
        "refusing to apply outside a kind context; context={context}; pass --allow-non-kind to override"
    )
}

fn kubectl_apply(path: &Path, context: Option<&str>, dry_run: bool) -> Result<()> {
    let mut command = Command::new("kubectl");
    if let Some(context) = context {
        command.args(["--context", context]);
    }
    command.args(["apply"]);
    if dry_run {
        command.arg("--dry-run=server");
    }
    let output = command
        .args(["-f"])
        .arg(path)
        .output()
        .with_context(|| format!("start kubectl apply {}", path.display()))?;
    if !output.status.success() {
        bail!(
            "kubectl apply {} failed: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn render_kustomization(resources: &[String]) -> String {
    let mut output =
        "apiVersion: kustomize.config.k8s.io/v1beta1\nkind: Kustomization\nresources:\n"
            .to_string();
    for resource in resources {
        output.push_str(&format!("- {resource}\n"));
    }
    output
}

// </HANDWRITE>
