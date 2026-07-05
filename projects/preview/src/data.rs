// <HANDWRITE gap="preview:data-lifecycle-local-contract" tracker="projects-preview-src-data-rs" reason="Data lifecycle planning and fake GCP provider state are hand-authored until Preview has provider adapter generator primitives.">
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPlanInput {
    pub mr: u32,
    pub app: String,
    pub base_namespace: String,
    pub target_namespace: String,
    pub route_target: String,
    pub provider: String,
    pub policy: String,
    pub source_instance: String,
    pub database: String,
    pub target_instance_prefix: String,
    pub ttl_hours: u32,
    pub secret_name: String,
    pub env_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataPlan {
    pub schema_version: u8,
    pub provider: String,
    pub policy: String,
    pub mr: u32,
    pub app: String,
    pub base_namespace: String,
    pub target_namespace: String,
    pub route_target: String,
    pub source: DataSource,
    pub target: DataTarget,
    pub ttl_hours: u32,
    pub actions: Vec<String>,
    pub guardrails: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSource {
    pub instance: String,
    pub database: String,
    pub access: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataTarget {
    pub instance: String,
    pub database: String,
    pub secret_name: String,
    pub env_name: String,
    pub connection_secret_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FakeDataState {
    pub schema_version: u8,
    pub resources: Vec<FakeDataResource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FakeDataResource {
    pub provider: String,
    pub policy: String,
    pub mr: u32,
    pub app: String,
    pub namespace: String,
    pub route_target: String,
    pub source_instance: String,
    pub source_database: String,
    pub target_instance: String,
    pub target_database: String,
    pub secret_name: String,
    pub status: String,
    pub ttl_hours: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FakeDataSummary {
    pub action: String,
    pub provider: String,
    pub target_instance: String,
    pub target_database: String,
    pub namespace: String,
    pub secret_name: String,
    pub state_path: String,
    pub changed: bool,
}

pub fn build_data_plan(input: DataPlanInput) -> Result<DataPlan> {
    validate_data_plan_input(&input)?;
    let target_instance = format!("{}{}", input.target_instance_prefix, input.mr);
    let target_database = input.database.clone();
    let guardrails = vec![
        format!(
            "target namespace must match uat-mr-* ({})",
            input.target_namespace
        ),
        format!("target instance must match preview-* ({target_instance})"),
        format!("source instance is read-only ({})", input.source_instance),
        format!("ttlHours is required ({})", input.ttl_hours),
        "cleanup may delete only fake/provider-owned preview resources".to_string(),
    ];
    let actions = vec![
        format!(
            "create {} preview data resource {}",
            input.provider, target_instance
        ),
        format!(
            "derive Kubernetes Secret {}/{}",
            input.target_namespace, input.secret_name
        ),
        format!(
            "rewrite workload env {} from Secret key DATABASE_URL",
            input.env_name
        ),
        "record provider ownership for guarded cleanup".to_string(),
    ];

    Ok(DataPlan {
        schema_version: 1,
        provider: input.provider,
        policy: input.policy,
        mr: input.mr,
        app: input.app,
        base_namespace: input.base_namespace,
        target_namespace: input.target_namespace,
        route_target: input.route_target,
        source: DataSource {
            instance: input.source_instance,
            database: input.database,
            access: "read-only".to_string(),
        },
        target: DataTarget {
            instance: target_instance,
            database: target_database,
            secret_name: input.secret_name,
            env_name: input.env_name,
            connection_secret_key: "DATABASE_URL".to_string(),
        },
        ttl_hours: input.ttl_hours,
        actions,
        guardrails,
    })
}

pub fn fake_apply_data_plan(plan: &DataPlan, state_path: &Path) -> Result<FakeDataSummary> {
    validate_fake_provider(plan)?;
    let mut state = read_fake_state(state_path)?;
    let resource = resource_from_plan(plan);
    let mut changed = false;
    if let Some(existing) = state
        .resources
        .iter_mut()
        .find(|item| item.target_instance == resource.target_instance)
    {
        if *existing != resource {
            *existing = resource;
            changed = true;
        }
    } else {
        state.resources.push(resource);
        changed = true;
    }
    write_fake_state(state_path, &state)?;
    Ok(summary_from_plan("apply", plan, state_path, changed))
}

pub fn fake_cleanup_data_plan(plan: &DataPlan, state_path: &Path) -> Result<FakeDataSummary> {
    validate_fake_provider(plan)?;
    validate_cleanup_target(plan)?;
    let mut state = read_fake_state(state_path)?;
    let before = state.resources.len();
    state
        .resources
        .retain(|item| item.target_instance != plan.target.instance);
    let changed = state.resources.len() != before;
    write_fake_state(state_path, &state)?;
    Ok(summary_from_plan("cleanup", plan, state_path, changed))
}

pub fn read_data_plan(path: &Path) -> Result<DataPlan> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("read data plan {}", path.display()))?;
    serde_json::from_str(&contents).with_context(|| format!("parse data plan {}", path.display()))
}

pub fn connection_string(plan: &DataPlan) -> String {
    format!(
        "postgres://preview:{}@{}.preview.local/{}",
        plan.route_target, plan.target.instance, plan.target.database
    )
}

fn validate_data_plan_input(input: &DataPlanInput) -> Result<()> {
    if input.ttl_hours == 0 {
        bail!("data preview ttl-hours must be greater than zero");
    }
    if !input.target_namespace.starts_with("uat-mr-") {
        bail!(
            "data target namespace must match uat-mr-*; got {}",
            input.target_namespace
        );
    }
    if !input.target_instance_prefix.starts_with("preview-") {
        bail!(
            "data target instance prefix must match preview-*; got {}",
            input.target_instance_prefix
        );
    }
    if input.source_instance.starts_with("preview-") {
        bail!(
            "data source instance must be a protected base/UAT instance, not {}",
            input.source_instance
        );
    }
    if input.database.trim().is_empty() {
        bail!("data database is required");
    }
    if input.provider.trim().is_empty() {
        bail!("data provider is required");
    }
    if input.policy.trim().is_empty() {
        bail!("data policy is required");
    }
    Ok(())
}

fn validate_fake_provider(plan: &DataPlan) -> Result<()> {
    if plan.provider != "fake-gcp-cloud-sql" {
        bail!(
            "local data apply supports provider fake-gcp-cloud-sql only; got {}",
            plan.provider
        );
    }
    Ok(())
}

fn validate_cleanup_target(plan: &DataPlan) -> Result<()> {
    if !plan.target_namespace.starts_with("uat-mr-") {
        bail!(
            "refusing data cleanup outside preview namespace {}",
            plan.target_namespace
        );
    }
    if !plan.target.instance.starts_with("preview-") {
        bail!(
            "refusing data cleanup for non-preview instance {}",
            plan.target.instance
        );
    }
    Ok(())
}

fn read_fake_state(path: &Path) -> Result<FakeDataState> {
    if !path.exists() {
        return Ok(FakeDataState {
            schema_version: 1,
            resources: Vec::new(),
        });
    }
    let contents = fs::read_to_string(path)
        .with_context(|| format!("read fake data state {}", path.display()))?;
    serde_json::from_str(&contents)
        .with_context(|| format!("parse fake data state {}", path.display()))
}

fn write_fake_state(path: &Path, state: &FakeDataState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create fake data state directory {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_string_pretty(state)? + "\n")
        .with_context(|| format!("write fake data state {}", path.display()))
}

fn resource_from_plan(plan: &DataPlan) -> FakeDataResource {
    FakeDataResource {
        provider: plan.provider.clone(),
        policy: plan.policy.clone(),
        mr: plan.mr,
        app: plan.app.clone(),
        namespace: plan.target_namespace.clone(),
        route_target: plan.route_target.clone(),
        source_instance: plan.source.instance.clone(),
        source_database: plan.source.database.clone(),
        target_instance: plan.target.instance.clone(),
        target_database: plan.target.database.clone(),
        secret_name: plan.target.secret_name.clone(),
        status: "provisioned".to_string(),
        ttl_hours: plan.ttl_hours,
    }
}

fn summary_from_plan(
    action: &str,
    plan: &DataPlan,
    state_path: &Path,
    changed: bool,
) -> FakeDataSummary {
    FakeDataSummary {
        action: action.to_string(),
        provider: plan.provider.clone(),
        target_instance: plan.target.instance.clone(),
        target_database: plan.target.database.clone(),
        namespace: plan.target_namespace.clone(),
        secret_name: plan.target.secret_name.clone(),
        state_path: state_path.display().to_string(),
        changed,
    }
}

// </HANDWRITE>
