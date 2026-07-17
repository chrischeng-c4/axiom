use std::process::Command;

fn pgpool() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pgpool"))
}

#[test]
fn help_ships_standard_commands_and_runtime_plan() {
    let output = pgpool().arg("--help").output().expect("run pgpool help");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(stdout.contains("runtime-plan"));
    assert!(stdout.contains("spec"));
    assert!(stdout.contains("llm"));
    assert!(stdout.contains("upgrade"));
    assert!(stdout.contains("issue"));
    assert!(stdout.contains("k8s"));
}

#[test]
fn k8s_instance_render_is_operator_consumed_custom_resource() {
    let output = pgpool()
        .args(["k8s", "instance", "render", "--profile", "prod"])
        .output()
        .expect("render pgpool instance");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(stdout.contains("kind: Pgpool"));
    assert!(stdout.contains("apiVersion: pgpool.axiom.dev/v1alpha1"));
    assert!(stdout.contains("provider: plain_postgres"));
    assert!(stdout.contains("perPodQuota:"));
    assert!(!stdout.contains("StatefulSet"));
    assert!(!stdout.contains("volumeClaimTemplates"));
    assert!(!stdout.contains("sessionAffinity: ClientIP"));
}

#[test]
fn k8s_crd_and_operator_layers_render_parseable_assets() {
    let crd = pgpool()
        .args(["k8s", "crd", "render"])
        .output()
        .expect("render pgpool CRD");
    assert!(crd.status.success());
    let crd_stdout = String::from_utf8(crd.stdout).expect("utf8");
    assert!(crd_stdout.contains("kind: CustomResourceDefinition"));
    assert!(crd_stdout.contains("name: pgpools.pgpool.axiom.dev"));

    let operator = pgpool()
        .args([
            "k8s",
            "operator",
            "render",
            "--namespace",
            "database-system",
        ])
        .output()
        .expect("render pgpool operator");
    assert!(operator.status.success());
    let operator_stdout = String::from_utf8(operator.stdout).expect("utf8");
    assert!(operator_stdout.contains("kind: ClusterRole"));
    assert!(operator_stdout.contains("kind: Deployment"));
    assert!(operator_stdout.contains("namespace: database-system"));
    for command_part in ["- pgpool", "- k8s", "- operator", "- run"] {
        assert!(operator_stdout.contains(command_part));
    }
}

#[test]
fn serve_accepts_control_plane_backend_quota() {
    let output = pgpool()
        .args(["serve", "--help"])
        .output()
        .expect("run serve help");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(stdout.contains("--max-backend-connections"));
    assert!(stdout.contains("PGPOOL_MAX_BACKEND_CONNECTIONS"));
}

#[test]
fn runtime_plan_is_chainable_and_names_shared_libs() {
    let output = pgpool()
        .arg("runtime-plan")
        .output()
        .expect("run runtime-plan");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(stdout.contains("\"app_id\": \"pgpool\""));
    assert!(stdout.contains("server-lifecycle"));
    assert!(stdout.contains("server-tcp"));
    assert!(stdout.contains("server-http"));
    assert!(stdout.contains("next: pgpool spec --format routes"));
}

#[test]
fn routes_include_admin_and_postgres_frontend() {
    let output = pgpool()
        .args(["spec", "--format", "routes"])
        .output()
        .expect("run routes");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(stdout.contains("/readyz"));
    assert!(stdout.contains("/pools/{pool}/stats"));
    assert!(stdout.contains("postgresql-wire"));
}

/// verify: cli_contract::help_and_llm_workflow_topic_mention_serve (AC5)
#[test]
fn help_and_llm_workflow_topic_mention_serve() {
    let help = pgpool().arg("--help").output().expect("run pgpool help");
    assert!(help.status.success());
    let help_stdout = String::from_utf8(help.stdout).expect("utf8");
    assert!(help_stdout.contains("serve"));

    let llm = pgpool()
        .args(["llm", "--topic", "workflow"])
        .output()
        .expect("run llm workflow topic");
    assert!(llm.status.success());
    let llm_stdout = String::from_utf8(llm.stdout).expect("utf8");
    assert!(llm_stdout.contains("pgpool serve"));
}
