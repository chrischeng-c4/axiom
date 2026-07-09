// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-commands-plan-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `vat plan` — project the configured run topology without executing it.

use std::collections::{BTreeSet, HashSet};
use std::process::ExitCode;

use anyhow::{bail, Result};
use serde::Serialize;

use crate::config::{
    self, ClusterBackend, PortSpec, RetentionPolicy, RunnerConfig, ScenarioConfig,
    ScenarioNetworkMode, ServiceConfig, ServicePreset, ServiceRuntime, VatConfig,
};
use crate::spec::EgressPolicy;
use crate::state::ConfigRef;

#[derive(Debug, Clone)]
pub enum PlanTarget {
    Runner { runner_ids: Vec<String> },
    Scenario { scenario_id: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct RunPlan {
    pub config: ConfigRef,
    pub selection: PlannedSelection,
    pub workspace: PlannedWorkspace,
    pub network: PlannedNetwork,
    pub services: Vec<PlannedService>,
    pub env: PlannedEnv,
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedSelection {
    pub kind: String,
    pub runner_id: String,
    pub runners: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scenario: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedWorkspace {
    pub base: String,
    pub workdir: String,
    pub keep: RetentionPolicy,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedNetwork {
    pub egress: String,
    pub hermetic: bool,
    pub routes: Vec<PlannedRoute>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedRoute {
    pub host: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedService {
    pub id: String,
    pub backing: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    pub owned_by_vat: bool,
    pub requires: Vec<String>,
    pub exported_env: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external: Option<PlannedEndpoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster: Option<String>,
    pub readiness: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedEndpoint {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedEnv {
    pub base: Vec<String>,
    pub services: Vec<String>,
}

pub fn exec(target: PlanTarget, json: bool) -> Result<ExitCode> {
    let plan = build(target)?;
    if json {
        crate::commands::print_json(&plan, false)?;
    } else {
        print_human(&plan);
    }
    Ok(ExitCode::SUCCESS)
}

pub fn build(target: PlanTarget) -> Result<RunPlan> {
    let cwd = std::env::current_dir()?;
    let cfg = config::load_nearest(&cwd)?;
    build_from_config(&cfg, target)
}

pub fn build_from_config(cfg: &VatConfig, target: PlanTarget) -> Result<RunPlan> {
    let planned = match target {
        PlanTarget::Runner { runner_ids } => select_runners(cfg, runner_ids)?,
        PlanTarget::Scenario { scenario_id } => select_scenario(cfg, &scenario_id)?,
    };
    let service_refs = ordered_required_services(
        cfg,
        &planned
            .service_ids
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    )?;
    let services = service_refs
        .iter()
        .map(|service| planned_service(service))
        .collect::<Vec<_>>();
    let service_env = services
        .iter()
        .flat_map(|service| service.exported_env.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let artifacts = planned
        .runners
        .iter()
        .flat_map(|runner| runner.artifacts.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let routes = planned_routes(cfg, &services);
    let egress = if planned.hermetic {
        "localhost-only".to_string()
    } else {
        cfg.network
            .as_ref()
            .map(|network| egress_name(network.egress).to_string())
            .unwrap_or_else(|| "open".to_string())
    };

    Ok(RunPlan {
        config: ConfigRef {
            path: cfg.path.to_string_lossy().into_owned(),
            digest: cfg.digest.clone(),
        },
        selection: PlannedSelection {
            kind: planned.kind,
            runner_id: planned.runner_id,
            runners: planned
                .runners
                .iter()
                .map(|runner| runner.id.clone())
                .collect(),
            scenario: planned.scenario_id,
            reason: planned.reason,
        },
        workspace: PlannedWorkspace {
            base: cfg.base_dir().to_string_lossy().into_owned(),
            workdir: cfg.workspace.workdir.to_string_lossy().into_owned(),
            keep: cfg.workspace.keep,
        },
        network: PlannedNetwork {
            egress,
            hermetic: planned.hermetic,
            routes,
        },
        services,
        env: PlannedEnv {
            base: vec![
                "VAT_CONFIG_ROOT".to_string(),
                "VAT_WORKSPACE_BASE".to_string(),
            ],
            services: service_env,
        },
        artifacts,
    })
}

struct Selection {
    kind: String,
    runner_id: String,
    runners: Vec<RunnerConfig>,
    scenario_id: Option<String>,
    reason: String,
    service_ids: Vec<String>,
    hermetic: bool,
}

fn select_runners(cfg: &VatConfig, runner_ids: Vec<String>) -> Result<Selection> {
    let runners = if runner_ids.len() > 1 {
        let mut seen = HashSet::new();
        let mut selected = Vec::new();
        for id in &runner_ids {
            if !seen.insert(id.clone()) {
                bail!("runner `{id}` listed twice");
            }
            selected.push(cfg.runner(id)?.clone());
        }
        selected
    } else {
        vec![cfg
            .select_runner(runner_ids.first().map(String::as_str))?
            .0
            .clone()]
    };
    let reason = if runner_ids.len() > 1 {
        "explicit_concurrent"
    } else if runner_ids.len() == 1 {
        "explicit"
    } else if cfg.default_runner.is_some() {
        "default_runner"
    } else {
        "single_runner"
    };
    let mut service_ids = Vec::new();
    for runner in &runners {
        for service_id in &runner.requires {
            if !service_ids.contains(service_id) {
                service_ids.push(service_id.clone());
            }
        }
    }
    Ok(Selection {
        kind: "runner".to_string(),
        runner_id: runners
            .iter()
            .map(|runner| runner.id.as_str())
            .collect::<Vec<_>>()
            .join("+"),
        runners,
        scenario_id: None,
        reason: reason.to_string(),
        service_ids,
        hermetic: false,
    })
}

fn select_scenario(cfg: &VatConfig, scenario_id: &str) -> Result<Selection> {
    let scenario = cfg.scenario(scenario_id)?.clone();
    let runner = cfg.runner(&scenario.runner)?.clone();
    let service_ids = scenario_service_ids(cfg, &scenario, &runner)?;
    if scenario.network == ScenarioNetworkMode::Hermetic
        && !service_set_has_http_mock(cfg, &service_ids)
    {
        bail!(
            "scenario `{}` network = hermetic requires a participating `preset = \"http-mock\"` service",
            scenario.id
        );
    }
    Ok(Selection {
        kind: "scenario".to_string(),
        runner_id: runner.id.clone(),
        runners: vec![runner],
        scenario_id: Some(scenario.id),
        reason: "scenario".to_string(),
        service_ids,
        hermetic: scenario.network == ScenarioNetworkMode::Hermetic,
    })
}

fn scenario_service_ids(
    cfg: &VatConfig,
    scenario: &ScenarioConfig,
    runner: &RunnerConfig,
) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    for id in std::iter::once(&scenario.app)
        .chain(scenario.requires.iter())
        .chain(runner.requires.iter())
    {
        if !ids.contains(id) {
            ids.push(id.clone());
        }
    }
    Ok(
        ordered_required_services(cfg, &ids.iter().map(String::as_str).collect::<Vec<_>>())?
            .into_iter()
            .map(|service| service.id.clone())
            .collect(),
    )
}

fn ordered_required_services<'a>(
    cfg: &'a VatConfig,
    service_ids: &[&str],
) -> Result<Vec<&'a ServiceConfig>> {
    let mut ordered = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for service_id in service_ids {
        visit_required_service(cfg, service_id, &mut visiting, &mut visited, &mut ordered)?;
    }
    Ok(ordered)
}

fn visit_required_service<'a>(
    cfg: &'a VatConfig,
    service_id: &str,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
    ordered: &mut Vec<&'a ServiceConfig>,
) -> Result<()> {
    if visited.contains(service_id) {
        return Ok(());
    }
    if !visiting.insert(service_id.to_string()) {
        bail!("service dependency cycle includes `{service_id}`");
    }
    let service = cfg.service(service_id)?;
    for required in &service.requires {
        visit_required_service(cfg, required, visiting, visited, ordered)?;
    }
    visiting.remove(service_id);
    visited.insert(service_id.to_string());
    ordered.push(service);
    Ok(())
}

fn planned_service(service: &ServiceConfig) -> PlannedService {
    let backing = service_backing(service);
    PlannedService {
        id: service.id.clone(),
        backing: backing.to_string(),
        preset: service.preset.map(service_preset_name).map(str::to_string),
        runtime: service_runtime(service),
        owned_by_vat: service.external.is_none(),
        requires: service.requires.clone(),
        exported_env: exported_env_keys(service),
        image: service.image.clone(),
        external: service.external.as_ref().map(|external| PlannedEndpoint {
            host: external.host.clone(),
            port: external.port,
        }),
        cluster: service
            .cluster
            .map(cluster_backend_name)
            .map(str::to_string),
        readiness: readiness_name(service).to_string(),
    }
}

fn service_backing(service: &ServiceConfig) -> &'static str {
    if service.cluster.is_some() {
        "cluster"
    } else if service.image.is_some() {
        "image"
    } else if service.external.is_some() {
        "external"
    } else if service.preset.is_some() {
        "preset"
    } else {
        "cmd"
    }
}

fn service_runtime(service: &ServiceConfig) -> Option<String> {
    if let Some(preset) = service.preset {
        if preset.is_builtin() {
            return Some("builtin".to_string());
        }
        return Some(
            match service.runtime {
                ServiceRuntime::Auto => "auto",
                ServiceRuntime::Native => "native",
                ServiceRuntime::Docker => "docker",
            }
            .to_string(),
        );
    }
    service.image.as_ref().map(|_| "docker".to_string())
}

fn exported_env_keys(service: &ServiceConfig) -> Vec<String> {
    let mut keys = BTreeSet::new();
    if let Some(preset) = service.preset {
        if service.export.is_empty() {
            if let Some(default) = default_preset_env(preset) {
                keys.insert(default.to_string());
            }
        } else {
            for (key, template) in &service.export {
                if template.contains("{host}") || template.contains("{port}") {
                    keys.insert(key.clone());
                } else {
                    keys.insert(template.clone());
                }
            }
        }
        add_endpoint_env_keys(&mut keys, &service.id);
    } else if service.external.is_some() || service.image.is_some() {
        keys.extend(service.export.keys().cloned());
        add_endpoint_env_keys(&mut keys, &service.id);
    } else if service.cluster.is_some() {
        keys.insert("KUBECONFIG".to_string());
        keys.extend(service.export.keys().cloned());
        let upper = service.id.to_uppercase().replace(['-', '.'], "_");
        keys.insert(format!("VAT_SERVICE_{upper}_KUBECONFIG"));
    } else {
        if service.ready_http.is_some() && service.export.is_empty() {
            let upper = service.id.to_uppercase().replace(['-', '.'], "_");
            keys.insert(format!("VAT_SERVICE_{upper}_URL"));
        }
        for (key, template) in &service.export {
            if template.contains("{host}") || template.contains("{port}") {
                keys.insert(key.clone());
            } else if service.ready_http.is_some() {
                keys.insert(template.clone());
            }
        }
        if command_service_needs_port(service) {
            add_endpoint_env_keys(&mut keys, &service.id);
        }
    }
    keys.into_iter().collect()
}

fn add_endpoint_env_keys(keys: &mut BTreeSet<String>, service_id: &str) {
    let upper = service_id.to_uppercase().replace(['-', '.'], "_");
    keys.insert(format!("VAT_SERVICE_{upper}_HOST"));
    keys.insert(format!("VAT_SERVICE_{upper}_PORT"));
}

fn command_service_needs_port(service: &ServiceConfig) -> bool {
    service.cmd.iter().any(|value| value.contains("{port}"))
        || service
            .ready_http
            .as_deref()
            .map(|value| value.contains("{port}"))
            .unwrap_or(false)
        || service
            .ready_cmd
            .iter()
            .any(|value| value.contains("{port}"))
        || service
            .export
            .values()
            .any(|value| value.contains("{port}") || value.contains("{host}"))
        || matches!(service.port, PortSpec::Fixed(_))
}

fn default_preset_env(preset: ServicePreset) -> Option<&'static str> {
    Some(match preset {
        ServicePreset::Postgres | ServicePreset::Mysql => "DATABASE_URL",
        ServicePreset::Redis => "REDIS_URL",
        ServicePreset::Nats => "NATS_URL",
        ServicePreset::Rabbitmq => "AMQP_URL",
        ServicePreset::Mongo => "MONGODB_URI",
        ServicePreset::Opensearch => "OPENSEARCH_URL",
        ServicePreset::Firestore => "FIRESTORE_EMULATOR_HOST",
        ServicePreset::Pubsub => "PUBSUB_EMULATOR_HOST",
        ServicePreset::Datastore => "DATASTORE_EMULATOR_HOST",
        ServicePreset::Bigtable => "BIGTABLE_EMULATOR_HOST",
        ServicePreset::Spanner => "SPANNER_EMULATOR_HOST",
        ServicePreset::FirebaseAuth => "FIREBASE_AUTH_EMULATOR_HOST",
        ServicePreset::CloudTasks => "CLOUD_TASKS_EMULATOR_HOST",
        ServicePreset::CloudScheduler => "CLOUD_SCHEDULER_EMULATOR_HOST",
        ServicePreset::CloudWorkflows => "CLOUD_WORKFLOWS_EMULATOR_HOST",
        ServicePreset::CloudStorage => "STORAGE_EMULATOR_HOST",
        ServicePreset::HttpMock => "VAT_HTTP_MOCK_HOST",
        ServicePreset::Openapi => "OPENAPI_MOCK_HOST",
        ServicePreset::Firebase => return None,
    })
}

fn readiness_name(service: &ServiceConfig) -> &'static str {
    if !service.ready_cmd.is_empty() {
        "cmd"
    } else if service.ready_http.is_some() {
        "http"
    } else if service.external.is_some() || service.preset.is_some() || service.cluster.is_some() {
        "tcp"
    } else {
        "none"
    }
}

fn planned_routes(cfg: &VatConfig, services: &[PlannedService]) -> Vec<PlannedRoute> {
    let mut routes = Vec::new();
    let mut explicit_hosts = BTreeSet::new();
    if let Some(network) = &cfg.network {
        for route in &network.routes {
            explicit_hosts.insert(route.host.clone());
            routes.push(PlannedRoute {
                host: route.host.clone(),
                target: Some(route.target.clone()),
                source: "explicit".to_string(),
            });
        }
    }
    for service in services {
        let Some(preset) = service.preset.as_deref().and_then(service_preset_by_name) else {
            continue;
        };
        let Some(host) = preset.preset_gcp_host() else {
            continue;
        };
        if explicit_hosts.contains(host) {
            continue;
        }
        routes.push(PlannedRoute {
            host: host.to_string(),
            target: None,
            source: format!("preset:{}", service.id),
        });
    }
    routes
}

fn service_set_has_http_mock(cfg: &VatConfig, service_ids: &[String]) -> bool {
    service_ids.iter().any(|id| {
        cfg.service(id)
            .map(|service| service.preset == Some(ServicePreset::HttpMock))
            .unwrap_or(false)
    })
}

fn service_preset_name(preset: ServicePreset) -> &'static str {
    match preset {
        ServicePreset::Postgres => "postgres",
        ServicePreset::Redis => "redis",
        ServicePreset::Nats => "nats",
        ServicePreset::Rabbitmq => "rabbitmq",
        ServicePreset::Mysql => "mysql",
        ServicePreset::Mongo => "mongo",
        ServicePreset::Opensearch => "opensearch",
        ServicePreset::Firestore => "firestore",
        ServicePreset::Pubsub => "pubsub",
        ServicePreset::Datastore => "datastore",
        ServicePreset::Bigtable => "bigtable",
        ServicePreset::Spanner => "spanner",
        ServicePreset::Firebase => "firebase",
        ServicePreset::FirebaseAuth => "firebase-auth",
        ServicePreset::CloudTasks => "cloud-tasks",
        ServicePreset::CloudScheduler => "cloud-scheduler",
        ServicePreset::CloudWorkflows => "cloud-workflows",
        ServicePreset::CloudStorage => "cloud-storage",
        ServicePreset::HttpMock => "http-mock",
        ServicePreset::Openapi => "openapi",
    }
}

fn service_preset_by_name(name: &str) -> Option<ServicePreset> {
    Some(match name {
        "postgres" => ServicePreset::Postgres,
        "redis" => ServicePreset::Redis,
        "nats" => ServicePreset::Nats,
        "rabbitmq" => ServicePreset::Rabbitmq,
        "mysql" => ServicePreset::Mysql,
        "mongo" => ServicePreset::Mongo,
        "opensearch" => ServicePreset::Opensearch,
        "firestore" => ServicePreset::Firestore,
        "pubsub" => ServicePreset::Pubsub,
        "datastore" => ServicePreset::Datastore,
        "bigtable" => ServicePreset::Bigtable,
        "spanner" => ServicePreset::Spanner,
        "firebase" => ServicePreset::Firebase,
        "firebase-auth" => ServicePreset::FirebaseAuth,
        "cloud-tasks" => ServicePreset::CloudTasks,
        "cloud-scheduler" => ServicePreset::CloudScheduler,
        "cloud-workflows" => ServicePreset::CloudWorkflows,
        "cloud-storage" => ServicePreset::CloudStorage,
        "http-mock" => ServicePreset::HttpMock,
        "openapi" => ServicePreset::Openapi,
        _ => return None,
    })
}

fn cluster_backend_name(backend: ClusterBackend) -> &'static str {
    match backend {
        ClusterBackend::Auto => "auto",
        ClusterBackend::Kind => "kind",
        ClusterBackend::K3d => "k3d",
        ClusterBackend::Minikube => "minikube",
    }
}

fn egress_name(egress: EgressPolicy) -> &'static str {
    match egress {
        EgressPolicy::Open => "open",
        EgressPolicy::LocalhostOnly => "localhost-only",
        EgressPolicy::Deny => "deny",
    }
}

fn print_human(plan: &RunPlan) {
    println!(
        "vat plan: {} {}",
        plan.selection.kind, plan.selection.runner_id
    );
    println!("config {}", plan.config.path);
    println!("workspace {}", plan.workspace.base);
    if plan.services.is_empty() {
        println!("services none");
    } else {
        for service in &plan.services {
            println!(
                "service {} {} owned_by_vat={}",
                service.id, service.backing, service.owned_by_vat
            );
        }
    }
    if !plan.artifacts.is_empty() {
        println!("artifacts {}", plan.artifacts.join(", "));
    }
}
// CODEGEN-END
