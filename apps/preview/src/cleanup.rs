// <HANDWRITE gap="issue-1111:guarded-cleanup-janitor" tracker="projects-preview-src-cleanup-rs" reason="Guarded janitor planning and kubectl cleanup execution are hand-authored until Preview has generator primitives for lifecycle janitors.">
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::model::CleanupAction;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JanitorPlan {
    pub mr: u32,
    pub namespace: String,
    pub route_target: String,
    pub control_namespace: String,
    pub protected_namespaces: Vec<String>,
    pub action: CleanupAction,
    pub reason: String,
    pub delete_namespace: bool,
    pub delete_route_binding: bool,
    pub skipped: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JanitorInput {
    pub mr: u32,
    pub namespace: String,
    pub route_target: String,
    pub control_namespace: String,
    pub protected_namespaces: Vec<String>,
    pub mr_closed: bool,
    pub ttl_expired: bool,
    pub namespace_exists: bool,
    pub route_binding_exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupApplyOptions {
    pub context: Option<String>,
    pub allow_non_kind: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupApplySummary {
    pub action: CleanupAction,
    pub namespace: String,
    pub route_target: String,
    pub deleted_namespace: bool,
    pub deleted_route_binding: bool,
    pub skipped: Vec<String>,
}

pub fn plan_guarded_cleanup(input: JanitorInput) -> JanitorPlan {
    let mut skipped = Vec::new();
    let protected = input.protected_namespaces.contains(&input.namespace);
    let selector_ok = input.namespace.starts_with("uat-mr-");
    if protected {
        skipped.push(format!("protected namespace {}", input.namespace));
    }
    if !selector_ok {
        skipped.push(format!(
            "namespace {} does not match preview selector uat-mr-*",
            input.namespace
        ));
    }

    let (action, reason, mut delete_namespace, delete_route_binding) = if protected || !selector_ok
    {
        (
            CleanupAction::Keep,
            "guardrail rejected namespace cleanup".to_string(),
            false,
            false,
        )
    } else if input.mr_closed {
        (
            CleanupAction::Delete,
            "MR is closed or merged".to_string(),
            input.namespace_exists,
            input.route_binding_exists,
        )
    } else if input.namespace_exists && !input.route_binding_exists {
        (
            CleanupAction::Delete,
            "preview namespace exists without matching route target".to_string(),
            true,
            false,
        )
    } else if !input.namespace_exists && input.route_binding_exists {
        (
            CleanupAction::Delete,
            "route target exists without preview namespace".to_string(),
            false,
            true,
        )
    } else if input.ttl_expired {
        (
            CleanupAction::Drain,
            "preview TTL expired; remove route binding before namespace deletion".to_string(),
            false,
            input.route_binding_exists,
        )
    } else {
        (
            CleanupAction::Keep,
            "MR remains active and preview is within TTL".to_string(),
            false,
            false,
        )
    };

    if delete_namespace && !input.namespace_exists {
        delete_namespace = false;
    }

    JanitorPlan {
        mr: input.mr,
        namespace: input.namespace,
        route_target: input.route_target,
        control_namespace: input.control_namespace,
        protected_namespaces: input.protected_namespaces,
        action,
        reason,
        delete_namespace,
        delete_route_binding,
        skipped,
    }
}

pub fn apply_guarded_cleanup(
    plan: &JanitorPlan,
    options: &CleanupApplyOptions,
) -> Result<CleanupApplySummary> {
    validate_plan(plan)?;
    if !options.allow_non_kind {
        let context = match &options.context {
            Some(context) => context.clone(),
            None => current_kubectl_context().context("resolve kubectl context")?,
        };
        if !context.starts_with("kind-") {
            bail!("refusing cleanup outside kind context {context}; pass --allow-non-kind to override");
        }
    }

    let mut deleted_namespace = false;
    let mut deleted_route_binding = false;
    if plan.delete_route_binding {
        kubectl_delete(
            "configmap",
            &format!("routebinding-{}", plan.route_target),
            Some(&plan.control_namespace),
            options.context.as_deref(),
        )?;
        deleted_route_binding = true;
    }
    if plan.delete_namespace {
        kubectl_delete(
            "namespace",
            &plan.namespace,
            None,
            options.context.as_deref(),
        )?;
        deleted_namespace = true;
    }

    Ok(CleanupApplySummary {
        action: plan.action,
        namespace: plan.namespace.clone(),
        route_target: plan.route_target.clone(),
        deleted_namespace,
        deleted_route_binding,
        skipped: plan.skipped.clone(),
    })
}

pub fn read_janitor_plan(path: &Path) -> Result<JanitorPlan> {
    let contents =
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&contents).with_context(|| format!("parse {}", path.display()))
}

fn validate_plan(plan: &JanitorPlan) -> Result<()> {
    if plan.protected_namespaces.contains(&plan.namespace) {
        bail!("refusing to clean protected namespace {}", plan.namespace);
    }
    if !plan.namespace.starts_with("uat-mr-") {
        bail!(
            "refusing broad namespace cleanup for {}; expected uat-mr-*",
            plan.namespace
        );
    }
    Ok(())
}

fn kubectl_delete(
    kind: &str,
    name: &str,
    namespace: Option<&str>,
    context: Option<&str>,
) -> Result<()> {
    let mut command = Command::new("kubectl");
    if let Some(context) = context {
        command.args(["--context", context]);
    }
    command.args(["delete", kind, name, "--ignore-not-found=true"]);
    if let Some(namespace) = namespace {
        command.args(["-n", namespace]);
    }
    let output = command
        .output()
        .with_context(|| format!("start kubectl delete {kind} {name}"))?;
    if !output.status.success() {
        bail!(
            "kubectl delete {kind} {name} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
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

// </HANDWRITE>
