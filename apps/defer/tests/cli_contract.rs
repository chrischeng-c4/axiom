// HANDWRITE-BEGIN gap="missing-generator:unit-test:defer-cli-contract" tracker="#766" reason="CLI convention, agent onboarding, offline OpenAPI, and shared client-codegen regression proof."
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

fn defer() -> Command {
    Command::new(env!("CARGO_BIN_EXE_defer"))
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="#2213" reason="Own the fail-closed behavior oracle for exact command grammar, offline llm, exact TypeScript client generation, and deployment-render exit status.">
#[test]
fn help_exposes_standard_and_domain_surfaces() {
    let output = defer().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for command in [
        "serve",
        "spec",
        "llm",
        "upgrade",
        "issue",
        "queue",
        "task",
        "dispatch",
        "backup",
        "k8s",
        "dockerfile",
    ] {
        assert!(stdout.contains(command), "missing {command} in --help");
    }

    for (group, expected) in [
        ("task", &["create", "status", "cancel"][..]),
        ("queue", &["get", "put", "control"][..]),
        ("issue", &["search", "view", "create"][..]),
    ] {
        let output = defer().args([group, "--help"]).output().unwrap();
        assert!(
            output.status.success(),
            "{group} --help failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        for command in expected {
            assert!(
                stdout.contains(command),
                "missing {command} in {group} --help"
            );
        }
    }
}
// </HANDWRITE>

#[test]
fn serve_exposes_shared_structured_log_configuration() {
    let output = defer().args(["serve", "--help"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--log-format"));
    assert!(stdout.contains("pretty"));
    assert!(stdout.contains("json"));
}

#[test]
fn deploy_artifacts_render_by_lifecycle_layer() {
    let crd = defer().args(["k8s", "crd", "render"]).output().unwrap();
    assert!(crd.status.success());
    assert!(String::from_utf8(crd.stdout)
        .unwrap()
        .contains("kind: CustomResourceDefinition"));

    let operator = defer()
        .args(["k8s", "operator", "render", "--namespace", "control"])
        .output()
        .unwrap();
    assert!(
        operator.status.success(),
        "operator render failed: {}",
        String::from_utf8_lossy(&operator.stderr)
    );
    let operator = String::from_utf8(operator.stdout).unwrap();
    assert!(operator.contains("kind: Deployment"));
    assert!(operator.contains("namespace: control"));
    assert!(operator.contains("name: POD_NAME"));
    assert!(operator.contains("name: POD_NAMESPACE"));
    assert!(operator.contains("fieldPath: metadata.namespace"));

    let instance = defer()
        .args(["k8s", "instance", "render", "--profile", "prod"])
        .output()
        .unwrap();
    assert!(
        instance.status.success(),
        "instance render failed: {}",
        String::from_utf8_lossy(&instance.stderr)
    );
    let instance = String::from_utf8(instance.stdout).unwrap();
    assert!(instance.contains("kind: Defer"));
    assert!(instance.contains("replicasPerShard: 3"));
    assert!(instance.contains("backup:"));

    for variant in ["source", "release"] {
        let dockerfile = defer()
            .args(["dockerfile", "render", "--variant", variant])
            .output()
            .unwrap();
        assert!(dockerfile.status.success());
        assert!(String::from_utf8(dockerfile.stdout)
            .unwrap()
            .contains("ENTRYPOINT"));
    }

    let release = defer()
        .args(["dockerfile", "render", "--variant", "release"])
        .output()
        .expect("render release Dockerfile");
    assert!(release.status.success());
    assert!(String::from_utf8(release.stdout)
        .expect("release Dockerfile stdout")
        .contains(&format!(
            "ARG DEFER_VERSION=defer@{}",
            env!("CARGO_PKG_VERSION")
        )));
}

#[test]
fn llm_outline_advertises_cross_scope_topics_and_terminates() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let proxy = format!("http://{}", listener.local_addr().unwrap());
    let connected = Arc::new(AtomicBool::new(false));
    let stop = Arc::new(AtomicBool::new(false));
    let watcher_connected = Arc::clone(&connected);
    let watcher_stop = Arc::clone(&stop);
    let watcher = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !watcher_stop.load(Ordering::Relaxed) && Instant::now() < deadline {
            match listener.accept() {
                Ok((_stream, _)) => {
                    watcher_connected.store(true, Ordering::Relaxed);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("proxy trap failed: {error}"),
            }
        }
    });

    let mut command = defer();
    command.args(["llm", "--topic", "outline"]);
    for name in [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
    ] {
        command.env(name, &proxy);
    }
    command
        .env_remove("NO_PROXY")
        .env_remove("no_proxy")
        .env_remove("GITHUB_TOKEN");
    let output = command.output().unwrap();
    stop.store(true, Ordering::Relaxed);
    watcher.join().unwrap();
    assert!(output.status.success());
    assert!(
        !connected.load(Ordering::Relaxed),
        "offline llm attempted a network connection"
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    for topic in ["workflow", "api", "delivery", "ha", "auth"] {
        assert!(stdout.contains(&format!("`{topic}`")), "missing {topic}");
    }
    assert!(stdout.contains("next: done"));
}

#[test]
fn offline_spec_and_typed_client_generation_use_one_contract() {
    let output = defer()
        .args(["spec", "--format", "openapi"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("/v1/queues/{queue}/tasks"));
    assert!(stdout.contains("next: done"));

    let out = tempfile::tempdir().unwrap();
    let generated = defer()
        .args([
            "spec",
            "gen",
            "--lang",
            "ts",
            "--out",
            out.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let mut actual = out
        .path()
        .read_dir()
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    actual.sort();
    assert_eq!(
        actual,
        [
            "client.ts",
            "hooks.ts",
            "index.ts",
            "runtime.ts",
            "types.ts"
        ]
    );
    let client = std::fs::read_to_string(out.path().join("client.ts")).unwrap();
    for symbol in [
        "createDeferClient",
        "taskCreate(data",
        "taskStatus(data",
        "taskCancel(data",
        "/v1/queues/${data.path.queue}/tasks",
    ] {
        assert!(client.contains(symbol), "generated client missing {symbol}");
    }
    assert!(String::from_utf8(generated.stdout)
        .unwrap()
        .contains("next: done"));
}

// HANDWRITE-END
