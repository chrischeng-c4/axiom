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

const EXPECTED_DOMAIN_OPERATIONS: &[&str] = &[
    "DELETE /v1/queues/{queue}/tasks/{task_id}",
    "GET /admin/backup",
    "GET /v1/queues/{queue}",
    "GET /v1/queues/{queue}/tasks/{task_id}",
    "POST /v1/queues/{queue}/control",
    "POST /v1/queues/{queue}/dispatch",
    "POST /v1/queues/{queue}/tasks",
    "POST /v1/queues/{queue}/tasks:batch",
    "PUT /v1/queues/{queue}",
];

fn chainable_json(output: std::process::Output, surface: &str) -> serde_json::Value {
    assert!(
        output.status.success(),
        "{surface} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json = stdout
        .strip_suffix("next: done\n")
        .expect("chainable JSON ends with the terminal marker")
        .trim_end();
    serde_json::from_str(json).unwrap_or_else(|error| panic!("{surface} JSON: {error}"))
}

fn assert_exact_domain_operations(spec: &serde_json::Value) {
    let paths = spec["paths"].as_object().expect("OpenAPI paths object");
    let mut actual = Vec::new();
    for (path, item) in paths {
        let item = item.as_object().expect("OpenAPI path item");
        for method in [
            "delete", "get", "head", "options", "patch", "post", "put", "trace",
        ] {
            if item.contains_key(method) {
                actual.push(format!("{} {path}", method.to_ascii_uppercase()));
            }
        }
    }
    actual.sort();
    assert_eq!(
        actual,
        EXPECTED_DOMAIN_OPERATIONS
            .iter()
            .map(|operation| (*operation).to_string())
            .collect::<Vec<_>>()
    );
}

fn assert_generated_client(
    lang: &str,
    expected_files: &[&str],
    client_file: &str,
    symbols: &[&str],
) {
    let out = tempfile::tempdir().unwrap();
    let generated = defer()
        .args([
            "spec",
            "gen",
            "--lang",
            lang,
            "--out",
            out.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{lang} generation failed: {}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let mut actual = out
        .path()
        .read_dir()
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    actual.sort();
    let mut expected = expected_files
        .iter()
        .map(|file| (*file).to_string())
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(actual, expected, "{lang} generated file inventory");
    let client = std::fs::read_to_string(out.path().join(client_file)).unwrap();
    for symbol in symbols {
        assert!(
            client.contains(symbol),
            "{lang} generated client missing {symbol}"
        );
    }
    assert!(
        String::from_utf8(generated.stdout)
            .unwrap()
            .ends_with("next: done\n"),
        "{lang} generation must emit a terminal marker"
    );
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

// <HANDWRITE gap="missing-generator:unit-test" tracker="#2219" reason="Own semantic equality of offline and canonical OpenAPI, the exact routes twin, and exact TypeScript, Python, and Rust client file and symbol inventories.">
#[test]
fn offline_spec_and_typed_client_generation_use_one_contract() {
    let offline = chainable_json(
        defer()
            .args(["spec", "--format", "openapi"])
            .output()
            .unwrap(),
        "defer spec --format openapi",
    );
    let canonical = serde_json::to_value(defer::openapi::openapi()).unwrap();
    assert_eq!(
        offline, canonical,
        "offline CLI spec must equal canonical IR"
    );
    assert_exact_domain_operations(&offline);

    let routes = chainable_json(
        defer()
            .args(["spec", "--format", "routes"])
            .output()
            .unwrap(),
        "defer spec --format routes",
    );
    assert_eq!(
        routes["routes"],
        serde_json::json!([
            "PUT /v1/queues/{queue}",
            "GET /v1/queues/{queue}",
            "POST /v1/queues/{queue}/control",
            "POST /v1/queues/{queue}/tasks",
            "POST /v1/queues/{queue}/tasks:batch",
            "GET /v1/queues/{queue}/tasks/{task_id}",
            "DELETE /v1/queues/{queue}/tasks/{task_id}",
            "POST /v1/queues/{queue}/dispatch",
            "GET /admin/backup"
        ])
    );

    assert_generated_client(
        "ts",
        &[
            "client.ts",
            "hooks.ts",
            "index.ts",
            "runtime.ts",
            "types.ts",
        ],
        "client.ts",
        &[
            "createDeferClient",
            "adminBackup()",
            "queueGet(data",
            "queuePut(data",
            "queueControl(data",
            "dispatchOne(data",
            "taskCreate(data",
            "taskStatus(data",
            "taskCancel(data",
            "taskCreateBatch(data",
            "/v1/queues/${data.path.queue}/tasks:batch",
        ],
    );
    assert_generated_client(
        "py",
        &["__init__.py", "client.py", "h2c_runtime.py", "models.py"],
        "client.py",
        &[
            "class Client:",
            "class AsyncClient:",
            "def admin_backup(",
            "def queue_get(",
            "def queue_put(",
            "def queue_control(",
            "def dispatch_one(",
            "def task_create(",
            "def task_status(",
            "def task_cancel(",
            "def task_create_batch(",
            "/v1/queues/{queue}/tasks:batch",
        ],
    );
    assert_generated_client(
        "rust",
        &["client.rs", "mod.rs", "models.rs"],
        "client.rs",
        &[
            "pub fn admin_backup(",
            "pub fn queue_get(",
            "pub fn queue_put(",
            "pub fn queue_control(",
            "pub fn dispatch_one(",
            "pub fn task_create(",
            "pub fn task_status(",
            "pub fn task_cancel(",
            "pub fn task_create_batch(",
            "/v1/queues/{}/tasks:batch",
        ],
    );
}
// </HANDWRITE>

// HANDWRITE-END
