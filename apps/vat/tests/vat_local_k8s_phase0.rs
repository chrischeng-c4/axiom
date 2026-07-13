// HANDWRITE-BEGIN gap="missing-generator:e2e-test:afed504d" tracker="#1539" reason="R1-R9/AC1-AC4 Phase 0 starts with an opt-in Apple-container machine-exec control. It owns one unique temporary machine, records a structured evidence report, and removes only that machine through explicit and Drop cleanup. The systemd/k3s journey is intentionally blocked until this control passes, so a repaired host cannot be mistaken for a completed Kubernetes proof. The real test is #[ignore] and requires VAT_LOCAL_K8S_E2E=1."

//! Real-host gates for the Apple-container Local Kubernetes feasibility spike.
//!
//! This first gate proves that the host can create a persistent machine and
//! execute one command through the machine API before Phase 0 attempts k3s.
//! A failed control is a meaningful NO-GO for the next Phase 0 step: it writes
//! evidence and fails the opt-in test rather than letting a later k3s error
//! obscure the substrate failure. A passed control is only a prerequisite, not
//! a completed Kubernetes proof.

use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const E2E_OPT_IN: &str = "VAT_LOCAL_K8S_E2E";

#[derive(Debug, Clone, Serialize)]
struct CommandEvidence {
    label: String,
    argv: Vec<String>,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl CommandEvidence {
    fn passed(&self) -> bool {
        self.exit_code == Some(0)
    }
}

#[derive(Debug, Serialize)]
struct ControlReport {
    schema: &'static str,
    phase: &'static str,
    machine: String,
    container_version: CommandEvidence,
    machine_create: CommandEvidence,
    machine_exec: CommandEvidence,
    machine_inspect: CommandEvidence,
    machine_logs: CommandEvidence,
    machine_delete: CommandEvidence,
    verdict: &'static str,
    blocker: Option<&'static str>,
}

/// Runs one `container` invocation without a shell so the evidence records the
/// exact argv. The test intentionally does not use an ambient default machine.
fn container_command(label: &str, args: &[&str]) -> CommandEvidence {
    match Command::new("container").args(args).output() {
        Ok(output) => evidence_from_output(label, args, output),
        Err(error) => CommandEvidence {
            label: label.to_string(),
            argv: std::iter::once("container".to_string())
                .chain(args.iter().map(|arg| (*arg).to_string()))
                .collect(),
            exit_code: None,
            stdout: String::new(),
            stderr: format!("spawn container {}: {error}", args.join(" ")),
        },
    }
}

fn evidence_from_output(label: &str, args: &[&str], output: Output) -> CommandEvidence {
    CommandEvidence {
        label: label.to_string(),
        argv: std::iter::once("container".to_string())
            .chain(args.iter().map(|arg| (*arg).to_string()))
            .collect(),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn unique_machine_name() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_millis();
    format!("vat-k8s-phase0-control-{}-{millis}", std::process::id())
}

fn evidence_path(machine: &str) -> PathBuf {
    let root = std::env::var_os("VAT_LOCAL_K8S_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/private/tmp"));
    root.join(format!("{machine}.json"))
}

/// Deletes only the exact, test-created machine. This is called explicitly so
/// cleanup is recorded, and from Drop as a last resort if an assertion panics.
struct MachineCleanup {
    name: String,
    deleted: bool,
}

impl MachineCleanup {
    fn new(name: String) -> Self {
        Self {
            name,
            deleted: false,
        }
    }

    fn delete(&mut self) -> CommandEvidence {
        let evidence = container_command("machine_delete", &["machine", "delete", &self.name]);
        self.deleted = evidence.passed();
        evidence
    }
}

impl Drop for MachineCleanup {
    fn drop(&mut self) {
        if !self.deleted {
            let _ = Command::new("container")
                .args(["machine", "delete", &self.name])
                .output();
        }
    }
}

#[test]
#[ignore = "real Apple-container machine probe; run only with VAT_LOCAL_K8S_E2E=1"]
fn apple_machine_exec_control_is_usable_before_k3s() {
    if std::env::var(E2E_OPT_IN).as_deref() != Ok("1") {
        eprintln!("{E2E_OPT_IN}=1 is required; skipping destructive real-host probe");
        return;
    }

    let container_version = container_command("container_version", &["--version"]);
    if !container_version.passed() {
        eprintln!("container CLI unavailable; skipping Apple-machine feasibility probe");
        return;
    }

    let name = unique_machine_name();
    let report_path = evidence_path(&name);
    let mut cleanup = MachineCleanup::new(name.clone());

    // `--no-boot` makes the first boot attributable to the exact exec command
    // below. Alpine is deliberate: this is a control for the machine API, not a
    // k3s/systemd test; a failure here cannot be blamed on the heavier probe
    // image or on k3s prerequisites.
    let machine_create = container_command(
        "machine_create",
        &[
            "machine",
            "create",
            "--no-boot",
            "--name",
            &name,
            "--home-mount",
            "none",
            "--cpus",
            "1",
            "--memory",
            "1G",
            "alpine:latest",
        ],
    );
    let machine_exec = container_command(
        "machine_exec",
        &[
            "machine",
            "run",
            "--name",
            &name,
            "--",
            "echo",
            "vat-k8s-phase0-control-ok",
        ],
    );
    let machine_inspect = container_command("machine_inspect", &["machine", "inspect", &name]);
    let machine_logs = container_command("machine_logs", &["machine", "logs", &name]);
    let machine_delete = cleanup.delete();

    let control_passed = machine_create.passed()
        && machine_exec.passed()
        && machine_exec.stdout.contains("vat-k8s-phase0-control-ok")
        && machine_delete.passed();
    let report = ControlReport {
        schema: "vat.local-k8s.phase0.control.v1",
        phase: "machine-exec-control",
        machine: name,
        container_version,
        machine_create,
        machine_exec,
        machine_inspect,
        machine_logs,
        machine_delete,
        verdict: if control_passed { "go" } else { "no-go" },
        blocker: (!control_passed).then_some(
            "Apple container machine create/run must reliably execute a control command before k3s, host kubeconfig, or workload probes are meaningful.",
        ),
    };

    fs::write(
        &report_path,
        serde_json::to_vec_pretty(&report).expect("serialize Phase 0 control evidence"),
    )
    .unwrap_or_else(|error| panic!("write {}: {error}", report_path.display()));
    println!(
        "vat local-k8s Phase 0 control evidence: {}\n{}",
        report_path.display(),
        serde_json::to_string_pretty(&report).expect("render Phase 0 control evidence")
    );

    assert!(
        report.machine_delete.passed(),
        "Phase 0 cleanup failed; inspect {} before retrying",
        report_path.display()
    );
    assert!(
        control_passed,
        "Apple container machine control failed; Phase 0 is NO-GO until the host machine API is repaired. Evidence: {}",
        report_path.display()
    );
}

// HANDWRITE-END
