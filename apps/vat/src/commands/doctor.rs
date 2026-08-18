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
use crate::lumen_release;

const APPLE_CONTAINER_ONLY_DOCKER_PROBE_SKIP: &str =
    "Docker daemon probe skipped for Apple-Container-only selected plan";
const DOCKER_FREE_DOCKER_PROBE_SKIP: &str =
    "Docker daemon probe skipped because selected plan has no Docker runtime";

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

/// A configuration-free host report. Unlike `vat doctor`, this never looks for
/// vat.toml, resolves a runner, opens a workspace, or starts a service.
#[derive(Debug, Serialize)]
struct HostOnlyDoctorReport {
    ok: bool,
    mode: &'static str,
    capabilities: CapabilitiesReport,
    gpu: crate::gpu::GpuInfo,
    checks: Vec<DoctorCheck>,
    next: &'static str,
}

/// One doctor invocation probes Apple's runtime once, then projects that
/// read-only result onto each selected MicroVM service. This keeps a hung
/// runtime bounded to one probe rather than one timeout per service.
#[derive(Debug, Clone)]
struct AppleContainerRuntimeProbe {
    ok: bool,
    message: String,
}

pub fn exec(target: PlanTarget, json: bool) -> Result<ExitCode> {
    let plan = plan::build(target)?;
    let cfg = config::load_file(Path::new(&plan.config.path))?;
    let capabilities = match selected_plan_docker_probe_skip_reason(&cfg, &plan) {
        Some(reason) => capabilities::report_without_docker_daemon_probe(reason),
        None => capabilities::report(),
    };
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

pub fn host_only_exec(json: bool) -> Result<ExitCode> {
    let capabilities = capabilities::report();
    let gpu = crate::gpu::detect();
    let mut checks = Vec::new();
    push_check(
        &mut checks,
        "workspace",
        "copy_on_write",
        capabilities.workspace.cow_clone,
        "cow_clone",
        format!(
            "copy-on-write primary method: {}",
            capabilities.workspace.primary_clone_method
        ),
    );
    for isolation in &capabilities.isolation {
        push_check(
            &mut checks,
            "isolation",
            &isolation.id,
            isolation.available,
            "isolation_availability",
            isolation.reason.clone(),
        );
    }
    push_check(
        &mut checks,
        "gpu",
        "host",
        gpu.accessible,
        "gpu_accessible",
        gpu.note.clone(),
    );
    push_check(
        &mut checks,
        "apple_container",
        "cli",
        capabilities.apple_container.cli,
        "apple_container_cli",
        "Apple Container CLI available on PATH".to_string(),
    );
    push_check(
        &mut checks,
        "docker",
        "daemon",
        capabilities.docker.daemon,
        "docker_daemon",
        capabilities
            .docker
            .error
            .clone()
            .unwrap_or_else(|| "Docker daemon reachable".to_string()),
    );
    push_check(
        &mut checks,
        "kubernetes",
        "kubectl",
        crate::commands::k8s::independent_kubectl_available(),
        "independent_kubectl",
        "independent kubectl available (OrbStack compatibility binary is rejected)".to_string(),
    );
    let report = HostOnlyDoctorReport {
        // This is an observation-only command: an unavailable optional host
        // substrate is reported in checks, not treated as a malformed config.
        ok: true,
        mode: "host_only",
        capabilities,
        gpu,
        checks,
        next: "vat capabilities --json",
    };
    if json {
        crate::commands::print_json(&report, false)?;
    } else {
        println!("vat doctor --host-only: complete");
        for check in &report.checks {
            println!(
                "{} {} {}: {}",
                if check.ok { "ok" } else { "unavailable" },
                check.component,
                check.id,
                check.message
            );
        }
        println!("next: {}", report.next);
    }
    Ok(ExitCode::SUCCESS)
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
    let needs_apple_container_runtime = plan.services.iter().any(|planned| {
        cfg.service(&planned.id)
            .is_ok_and(service_uses_apple_container_runtime)
    });
    let apple_container_runtime = needs_apple_container_runtime.then(apple_container_runtime_probe);
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
        check_service_host(
            &mut checks,
            cfg_service,
            capabilities,
            apple_container_runtime.as_ref(),
        );
    }
    checks
}

/// Docker is probed only when the selected plan can use it. Looking at
/// `plan.services` rather than every service in vat.toml keeps an unrelated
/// Docker service from breaking a deliberate Apple-Container-only invocation.
/// A cluster is intentionally conservative: `cluster::resolve_backend` itself
/// executes `docker info`, so selected clusters must take the normal path.
fn selected_plan_docker_probe_skip_reason(
    cfg: &config::VatConfig,
    plan: &RunPlan,
) -> Option<&'static str> {
    if selected_plan_requires_docker_probe(cfg, plan) {
        return None;
    }
    if !plan.services.is_empty()
        && plan.services.iter().all(|planned| {
            cfg.service(&planned.id).is_ok_and(|service| {
                matches!(service.runtime, ServiceRuntime::MicroVm)
                    && (service.image.is_some() || service.preset.is_some())
            })
        })
    {
        Some(APPLE_CONTAINER_ONLY_DOCKER_PROBE_SKIP)
    } else {
        Some(DOCKER_FREE_DOCKER_PROBE_SKIP)
    }
}

fn selected_plan_requires_docker_probe(cfg: &config::VatConfig, plan: &RunPlan) -> bool {
    plan.services.iter().any(|planned| {
        cfg.service(&planned.id)
            .map(service_requires_docker_probe)
            // A malformed plan/config relationship must never cause a false
            // no-Docker claim.
            .unwrap_or(true)
    })
}

fn service_requires_docker_probe(service: &ServiceConfig) -> bool {
    if service.cluster.is_some() {
        return true;
    }
    if service.image.is_some() {
        return service.runtime != ServiceRuntime::MicroVm;
    }
    let Some(preset) = service.preset else {
        return matches!(service.runtime, ServiceRuntime::Docker);
    };
    match service.runtime {
        ServiceRuntime::Docker => true,
        // Auto may select Docker as its fallback except for VAT's built-in
        // emulators and Firebase-family presets, which deliberately do not
        // offer a Docker fallback.
        ServiceRuntime::Auto => !preset.is_builtin() && !firebase_family(preset),
        ServiceRuntime::Native | ServiceRuntime::MicroVm => false,
    }
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
        || capabilities::isolation_available(capabilities, "linux-netns")
        || capabilities::isolation_available(capabilities, "vm");
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
    apple_container_runtime: Option<&AppleContainerRuntimeProbe>,
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
    } else if service_uses_apple_container_runtime(service) {
        let runtime = apple_container_runtime.expect(
            "selected MicroVM service must have one precomputed Apple Container runtime probe",
        );
        check_apple_container_runtime(checks, service, runtime);
        check_apple_container_builder_advisory(
            checks,
            service,
            &capabilities.apple_container.builder,
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

// <HANDWRITE gap="vat-versioned-native-lumen-preset-doctor" tracker="#1813" reason="Surface cache/download readiness and remediation.">
fn check_preset(
    checks: &mut Vec<DoctorCheck>,
    service: &ServiceConfig,
    preset: ServicePreset,
    capabilities: &CapabilitiesReport,
) {
    if preset == ServicePreset::Lumen {
        let selector = lumen_release::normalize_selector(service.version.as_deref());
        let tools_ok = ["curl", "tar", "shasum"].iter().all(|tool| which(tool));
        let detail = match selector {
            Ok(Some(ref tag)) => format!("native Lumen release `{tag}` will use VAT-owned cache"),
            Ok(None) => "native latest Lumen release will use VAT-owned cache".to_string(),
            Err(ref error) => error.to_string(),
        };
        push_check(
            checks,
            "preset",
            &service.id,
            selector.is_ok() && tools_ok,
            "lumen_native_release",
            if tools_ok {
                detail
            } else {
                format!("{detail}; install curl, tar, and shasum for native release resolution")
            },
        );
        return;
    }
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
        ServiceRuntime::MicroVm => {
            if !crate::commands::run::preset_has_microvm_image_route(preset) {
                push_check(
                    checks,
                    "preset",
                    &service.id,
                    false,
                    "microvm_preset_unsupported",
                    format!(
                        "preset `{}` has no declared Apple Container OCI image route for runtime `micro_vm`; VAT will not fall back to Docker",
                        service_preset_name(preset)
                    ),
                );
            } else if !service.volumes.is_empty() {
                push_check(
                    checks,
                    "preset",
                    &service.id,
                    false,
                    "microvm_preset_volumes_unsupported",
                    format!(
                        "preset `{}` with runtime `micro_vm` and named volumes is unsupported until VAT proves its Apple Container volume ownership/cleanup contract; VAT will not fall back to Docker",
                        service_preset_name(preset)
                    ),
                );
            }
        }
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
// </HANDWRITE>

fn service_uses_apple_container_runtime(service: &ServiceConfig) -> bool {
    service.external.is_none()
        && matches!(service.runtime, ServiceRuntime::MicroVm)
        && (service.image.is_some() || service.preset.is_some())
}

/// Read-only preflight for an explicit Apple Container (MicroVM) service.
/// Never starts the system or falls back to Docker: `vat doctor` only reports
/// whether `container system status` is already usable once per invocation.
fn apple_container_runtime_probe() -> AppleContainerRuntimeProbe {
    let (ok, message) = if !crate::sandbox::microvm::available() {
        (
            false,
            "Apple Container CLI `container` is not on PATH; install it, then run `container system status`."
                .to_string(),
        )
    } else if !crate::sandbox::microvm::system_up() {
        (
            false,
            "Apple Container `container system status` did not succeed; start or repair Apple Container, then retry `container system status`."
                .to_string(),
        )
    } else {
        (
            true,
            "Apple Container `container system status` succeeded.".to_string(),
        )
    };
    AppleContainerRuntimeProbe { ok, message }
}

fn check_apple_container_runtime(
    checks: &mut Vec<DoctorCheck>,
    service: &ServiceConfig,
    runtime: &AppleContainerRuntimeProbe,
) {
    push_check(
        checks,
        "apple_container",
        &service.id,
        runtime.ok,
        "apple_container_system",
        runtime.message.clone(),
    );
}

/// A shared builder is useful host evidence but never a readiness gate. Apple
/// Container does not expose VAT/project ownership, and the bounded capability
/// probes may time out independently of a healthy MicroVM runtime.
fn check_apple_container_builder_advisory(
    checks: &mut Vec<DoctorCheck>,
    service: &ServiceConfig,
    advisory: &crate::sandbox::microvm::AppleContainerBuilderAdvisory,
) {
    let mut message = format!(
        "Apple Container shared builder advisory: state={} ownership={} automatic_cleanup={}",
        advisory.state, advisory.ownership, advisory.automatic_cleanup
    );
    if let Some(configuration) = &advisory.configuration {
        message.push_str(&format!(" configuration.id={}", configuration.id));
    }
    if let Some(stats) = &advisory.observed_stats {
        if let Some(memory_usage_bytes) = stats.memory_usage_bytes {
            message.push_str(&format!(" observed_memory_bytes={memory_usage_bytes}"));
        }
    }
    if !advisory.probe_errors.is_empty() {
        let probes = advisory
            .probe_errors
            .iter()
            .map(|error| error.probe.as_str())
            .collect::<Vec<_>>()
            .join(",");
        message.push_str(&format!(" unavailable_probes={probes}"));
    }
    message.push_str("; VAT does not start, stop, delete, or prune shared builder resources.");
    push_check(
        checks,
        "apple_container",
        &service.id,
        true,
        "apple_container_builder_shared",
        message,
    );
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
        ServicePreset::Lumen => &[],
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
        ServicePreset::Firestore => "gcloud-firestore",
        ServicePreset::Pubsub => "gcloud-pubsub",
        ServicePreset::Datastore => "gcloud-datastore",
        ServicePreset::Bigtable => "gcloud-bigtable",
        ServicePreset::Spanner => "gcloud-spanner",
        ServicePreset::Firebase => "firebase",
        ServicePreset::FirebaseAuth => "firebase-auth",
        ServicePreset::CloudTasks => "gcloud-cloud-tasks",
        ServicePreset::CloudScheduler => "cloud-scheduler",
        ServicePreset::CloudWorkflows => "cloud-workflows",
        ServicePreset::CloudStorage => "cloud-storage",
        ServicePreset::HttpMock => "http-mock",
        ServicePreset::Openapi => "openapi",
        ServicePreset::Lumen => "lumen",
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
