// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-commands-doctor-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `vat doctor` — cheap host preflight for the selected vat.toml run.

use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

use anyhow::Result;
use serde::Serialize;

use crate::cluster;
use crate::commands::capabilities::{self, CapabilitiesReport};
use crate::commands::plan::{self, PlanTarget, RunPlan};
use crate::config::{self, ServiceConfig, ServicePreset, ServiceRuntime};

#[derive(Debug, Serialize)]
struct DoctorReport {
    ok: bool,
    plan: RunPlan,
    capabilities: CapabilitiesReport,
    checks: Vec<DoctorCheck>,
}

#[derive(Debug, Serialize)]
struct DoctorCheck {
    component: String,
    id: String,
    ok: bool,
    code: String,
    message: String,
}

pub fn exec(target: PlanTarget, json: bool) -> Result<ExitCode> {
    let plan = plan::build(target)?;
    let capabilities = capabilities::report();
    let cfg = config::load_file(Path::new(&plan.config.path))?;
    let checks = checks_for(&cfg, &plan, &capabilities);
    let ok = checks.iter().all(|check| check.ok);
    let report = DoctorReport {
        ok,
        plan,
        capabilities,
        checks,
    };
    if json {
        crate::commands::print_json(&report, false)?;
    } else {
        print_human(&report);
    }
    Ok(if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn checks_for(
    cfg: &config::VatConfig,
    plan: &RunPlan,
    capabilities: &CapabilitiesReport,
) -> Vec<DoctorCheck> {
    let mut checks = Vec::new();
    push_check(
        &mut checks,
        "workspace",
        "base",
        Path::new(&plan.workspace.base).is_dir(),
        "workspace_base",
        format!("workspace base exists: {}", plan.workspace.base),
    );
    check_network_isolation(&mut checks, plan, capabilities);
    for service in &plan.services {
        let Ok(cfg_service) = cfg.service(&service.id) else {
            push_check(
                &mut checks,
                "service",
                &service.id,
                false,
                "service_missing",
                format!("service `{}` missing from config", service.id),
            );
            continue;
        };
        check_service_files(&mut checks, cfg, cfg_service);
        check_service_host(&mut checks, cfg_service, capabilities);
    }
    checks
}

fn check_network_isolation(
    checks: &mut Vec<DoctorCheck>,
    plan: &RunPlan,
    capabilities: &CapabilitiesReport,
) {
    if plan.network.egress == "open" {
        return;
    }
    let enforceable = capabilities::isolation_available(capabilities, "macos-seatbelt")
        || capabilities::isolation_available(capabilities, "linux-netns");
    push_check(
        checks,
        "isolation",
        "egress",
        enforceable,
        "egress_enforcement",
        format!(
            "network egress policy `{}` is enforceable",
            plan.network.egress
        ),
    );
}

fn check_service_files(
    checks: &mut Vec<DoctorCheck>,
    cfg: &config::VatConfig,
    service: &ServiceConfig,
) {
    for seed in &service.seed {
        let path = config::resolve_relative(&cfg.root, seed);
        push_check(
            checks,
            "service",
            &service.id,
            path.is_file(),
            "seed_file",
            format!("seed file exists: {}", path.display()),
        );
    }
    if let Some(spec) = &service.spec {
        let path = config::resolve_relative(&cfg.root, Path::new(spec));
        push_check(
            checks,
            "service",
            &service.id,
            path.is_file(),
            "openapi_spec",
            format!("OpenAPI spec exists: {}", path.display()),
        );
    }
}

fn check_service_host(
    checks: &mut Vec<DoctorCheck>,
    service: &ServiceConfig,
    capabilities: &CapabilitiesReport,
) {
    if let Some(external) = &service.external {
        push_check(
            checks,
            "external",
            &service.id,
            tcp_reachable(&external.host, external.port),
            "external_tcp",
            format!(
                "external endpoint reachable: {}:{}",
                external.host, external.port
            ),
        );
    } else if service.image.is_some() || matches!(service.runtime, ServiceRuntime::Docker) {
        push_check(
            checks,
            "docker",
            &service.id,
            capabilities.docker.daemon,
            "docker_daemon",
            "Docker CLI and daemon reachable".to_string(),
        );
    }

    if let Some(cluster_backend) = service.cluster {
        let resolved = cluster::resolve_backend(cluster_backend);
        let ok = resolved.is_ok();
        let message = match resolved {
            Ok(backend) => format!("cluster backend available: {}", backend.name()),
            Err(unavailable) => unavailable.message(),
        };
        push_check(
            checks,
            "cluster",
            &service.id,
            ok,
            "cluster_backend",
            message,
        );
    }

    if !service.cmd.is_empty() {
        check_binary(checks, "cmd", &service.id, &service.cmd[0]);
    }
    if let Some(preset) = service.preset {
        check_preset(checks, service, preset, capabilities);
    }
}

fn check_preset(
    checks: &mut Vec<DoctorCheck>,
    service: &ServiceConfig,
    preset: ServicePreset,
    capabilities: &CapabilitiesReport,
) {
    if preset.is_builtin() && service.runtime == ServiceRuntime::Auto {
        push_check(
            checks,
            "preset",
            &service.id,
            true,
            "builtin_emulator",
            format!(
                "preset `{}` uses vat's built-in emulator",
                service_preset_name(preset)
            ),
        );
        return;
    }

    match service.runtime {
        ServiceRuntime::Docker => {}
        ServiceRuntime::Native => {
            for binary in required_binaries(preset) {
                check_binary(checks, "preset", &service.id, binary);
            }
        }
        ServiceRuntime::Auto => {
            let missing = missing_required_binaries(preset);
            let native_ok = missing.is_empty();
            let docker_ok = !firebase_family(preset) && capabilities.docker.daemon;
            let message = if native_ok {
                format!(
                    "preset `{}` can run with native binaries",
                    service_preset_name(preset)
                )
            } else if docker_ok {
                format!(
                    "preset `{}` can run with Docker fallback",
                    service_preset_name(preset)
                )
            } else if firebase_family(preset) {
                format!(
                    "preset `{}` requires native binaries: {}",
                    service_preset_name(preset),
                    missing.join(", ")
                )
            } else {
                format!(
                    "preset `{}` requires native binaries ({}) or Docker",
                    service_preset_name(preset),
                    missing.join(", ")
                )
            };
            push_check(
                checks,
                "preset",
                &service.id,
                native_ok || docker_ok,
                "preset_runtime",
                message,
            );
        }
    }
}

fn check_binary(checks: &mut Vec<DoctorCheck>, component: &str, id: &str, binary: &str) {
    push_check(
        checks,
        component,
        id,
        which(binary),
        "binary_on_path",
        format!("binary `{binary}` is on PATH"),
    );
}

fn push_check(
    checks: &mut Vec<DoctorCheck>,
    component: &str,
    id: &str,
    ok: bool,
    code: &str,
    message: String,
) {
    checks.push(DoctorCheck {
        component: component.to_string(),
        id: id.to_string(),
        ok,
        code: code.to_string(),
        message,
    });
}

fn tcp_reachable(host: &str, port: u16) -> bool {
    let Ok(mut addrs) = (host, port).to_socket_addrs() else {
        return false;
    };
    let Some(addr) = addrs.next() else {
        return false;
    };
    TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
}

fn which(binary: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(binary).is_file())
}

fn required_binaries(preset: ServicePreset) -> &'static [&'static str] {
    match preset {
        ServicePreset::Postgres => &["postgres", "initdb", "pg_isready", "pg_ctl", "psql"],
        ServicePreset::Redis => &["redis-server"],
        ServicePreset::Nats => &["nats-server"],
        ServicePreset::Rabbitmq => &["rabbitmq-server"],
        ServicePreset::Mysql => &["mysqld", "mysqladmin"],
        ServicePreset::Mongo => &["mongod"],
        ServicePreset::Opensearch => &["opensearch"],
        ServicePreset::Firestore
        | ServicePreset::Pubsub
        | ServicePreset::Datastore
        | ServicePreset::Bigtable
        | ServicePreset::Spanner => &["gcloud", "java"],
        ServicePreset::Firebase
        | ServicePreset::FirebaseAuth
        | ServicePreset::CloudTasks
        | ServicePreset::CloudScheduler
        | ServicePreset::CloudWorkflows
        | ServicePreset::CloudStorage
        | ServicePreset::HttpMock
        | ServicePreset::Openapi => &["firebase", "java"],
    }
}

fn missing_required_binaries(preset: ServicePreset) -> Vec<&'static str> {
    required_binaries(preset)
        .iter()
        .copied()
        .filter(|binary| !which(binary))
        .collect()
}

fn firebase_family(preset: ServicePreset) -> bool {
    matches!(
        preset,
        ServicePreset::Firebase
            | ServicePreset::FirebaseAuth
            | ServicePreset::CloudTasks
            | ServicePreset::CloudScheduler
            | ServicePreset::CloudWorkflows
            | ServicePreset::CloudStorage
            | ServicePreset::HttpMock
            | ServicePreset::Openapi
    )
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

fn print_human(report: &DoctorReport) {
    println!("vat doctor: {}", if report.ok { "ok" } else { "failed" });
    for check in &report.checks {
        println!(
            "{} {} {}: {}",
            if check.ok { "ok" } else { "fail" },
            check.component,
            check.id,
            check.message
        );
    }
}
// CODEGEN-END
