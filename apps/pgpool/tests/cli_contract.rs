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
    assert!(stdout.contains("server-core"));
    assert!(stdout.contains("tcp-server"));
    assert!(stdout.contains("http-server"));
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
