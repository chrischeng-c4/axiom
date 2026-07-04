// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-tests-kind-lifecycle-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use preview::{render_files, BaseWorkloadContract, RenderInput};

const SMOKE_IMAGE: &str = "preview-kind-smoke:local";

fn preview_bin() -> &'static str {
    env!("CARGO_BIN_EXE_preview")
}

fn input() -> RenderInput {
    RenderInput {
        mr: 123,
        sha: "abc123".to_string(),
        image: std::env::var("PREVIEW_KIND_IMAGE").unwrap_or_else(|_| SMOKE_IMAGE.to_string()),
        app: "checkout".to_string(),
        host: "uat.example.com".to_string(),
        base_namespace: "uat-base".to_string(),
        owner: "payments-sre".to_string(),
        ttl_hours: 2,
        control_namespace: "preview-system".to_string(),
        workload_identity: "preview-runner".to_string(),
        base_contract: None,
    }
}

#[test]
fn kind_applies_rolls_out_routes_and_cleans_rendered_lifecycle_objects() {
    if std::env::var("PREVIEW_KIND_E2E").as_deref() != Ok("1") {
        eprintln!("skipping kind lifecycle EC; set PREVIEW_KIND_E2E=1 to run");
        return;
    }

    let Some(context) =
        optional_output(Command::new("kubectl").args(["config", "current-context"]))
    else {
        eprintln!("skipping kind lifecycle EC; kubectl current-context is not configured");
        return;
    };
    assert!(
        context.starts_with("kind-")
            || std::env::var("PREVIEW_ALLOW_NON_KIND").as_deref() == Ok("1"),
        "refusing to run kind EC outside a kind context; context={context:?}"
    );
    if let Some(cluster_name) = context.strip_prefix("kind-") {
        build_and_load_smoke_image(cluster_name);
    }

    let dir = tempfile::tempdir().expect("tempdir");
    let mut cleanup = NamespaceCleanup::new();
    cleanup.add("uat-mr-123");
    if kubectl_create_namespace_if_missing("preview-system") {
        cleanup.add("preview-system");
    }
    if kubectl_create_namespace_if_missing("uat-base") {
        cleanup.add("uat-base");
    }
    write_base_workload_fixture(dir.path(), &input().image);
    kubectl_apply(&dir.path().join("base/deployment.yaml"));
    kubectl_apply(&dir.path().join("base/service.yaml"));

    let contract_path = dir.path().join("base-contract.json");
    command_ok(
        Command::new(preview_bin())
            .args([
                "discover-base",
                "--namespace",
                "uat-base",
                "--app",
                "checkout",
                "--out",
            ])
            .arg(&contract_path),
        "preview discover-base",
    );
    let discovered: BaseWorkloadContract = serde_json::from_str(
        &fs::read_to_string(&contract_path).expect("read discovered base contract"),
    )
    .expect("parse discovered base contract");
    assert_eq!(discovered.namespace, "uat-base");
    assert_eq!(discovered.container.ports[0].container_port, 8080);
    assert_eq!(discovered.service_ports[0].target_port, "8080");

    let mut render_input = input();
    render_input.base_contract = Some(discovered);
    for file in render_files(&render_input).expect("render") {
        if !file.path.starts_with("k8s/") && file.path != "router/route-binding.yaml" {
            continue;
        }
        let path = dir.path().join(&file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, file.contents).expect("write manifest");
    }
    preview_apply_rendered_lifecycle(dir.path(), &context, false);
    preview_apply_rendered_lifecycle(dir.path(), &context, true);
    preview_apply_rendered_lifecycle(dir.path(), &context, false);
    kubectl_rollout_status("uat-mr-123", "checkout");
    assert_service_has_endpoint("uat-mr-123", "checkout");
    assert_port_forward_reaches_readyz("uat-mr-123", "checkout");
    assert_workload_rbac_is_least_privilege();
    assert_quota_rejects_oversized_pod(dir.path(), &input().image);
    assert_namespace_exists("uat-base");
    kubectl_server_side_dry_run(&dir.path().join("router/route-binding.yaml"));
    drop(cleanup);
}

fn write_base_workload_fixture(root: &Path, image: &str) {
    let dir = root.join("base");
    fs::create_dir_all(&dir).expect("create base fixture dir");
    fs::write(
        dir.join("deployment.yaml"),
        format!(
            r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: checkout
  namespace: uat-base
spec:
  replicas: 1
  selector:
    matchLabels:
      app.kubernetes.io/name: checkout
      tier: web
  template:
    metadata:
      labels:
        app.kubernetes.io/name: checkout
        tier: web
    spec:
      containers:
      - name: checkout
        image: {image}
        ports:
        - name: http
          containerPort: 8080
        env:
        - name: APP_MODE
          value: uat
        resources:
          requests:
            cpu: 200m
            memory: 256Mi
          limits:
            cpu: 500m
            memory: 512Mi
        readinessProbe:
          httpGet:
            path: /readyz
            port: 8080
        livenessProbe:
          httpGet:
            path: /healthz
            port: 8080
"#
        ),
    )
    .expect("write base deployment");
    fs::write(
        dir.join("service.yaml"),
        r#"apiVersion: v1
kind: Service
metadata:
  name: checkout
  namespace: uat-base
spec:
  type: ClusterIP
  selector:
    app.kubernetes.io/name: checkout
    tier: web
  ports:
  - name: http
    port: 80
    targetPort: 8080
"#,
    )
    .expect("write base service");
}

struct NamespaceCleanup {
    names: Vec<String>,
}

impl NamespaceCleanup {
    fn new() -> Self {
        Self { names: Vec::new() }
    }

    fn add(&mut self, name: impl Into<String>) {
        self.names.push(name.into());
    }
}

impl Drop for NamespaceCleanup {
    fn drop(&mut self) {
        for name in &self.names {
            let _ = Command::new("kubectl")
                .args(["delete", "namespace", name, "--ignore-not-found=true"])
                .status();
        }
    }
}

fn kubectl_apply(path: &Path) {
    let status = Command::new("kubectl")
        .args(["apply", "-f"])
        .arg(path)
        .status()
        .unwrap_or_else(|err| panic!("kubectl apply failed to start: {err}"));
    assert!(
        status.success(),
        "kubectl apply failed for {}",
        path.display()
    );
}

fn kubectl_create_namespace_if_missing(name: &str) -> bool {
    if Command::new("kubectl")
        .args(["get", "namespace", name])
        .output()
        .is_ok_and(|output| output.status.success())
    {
        return false;
    }
    let status = Command::new("kubectl")
        .args(["create", "namespace", name])
        .status()
        .unwrap_or_else(|err| panic!("kubectl create namespace failed to start: {err}"));
    assert!(status.success(), "kubectl create namespace {name} failed");
    true
}

fn kubectl_server_side_dry_run(path: &Path) {
    let status = Command::new("kubectl")
        .args(["apply", "--dry-run=server", "-f"])
        .arg(path)
        .status()
        .unwrap_or_else(|err| panic!("kubectl apply dry-run failed to start: {err}"));
    assert!(
        status.success(),
        "kubectl dry-run failed for {}",
        path.display()
    );
}

fn preview_apply_rendered_lifecycle(root: &Path, context: &str, dry_run: bool) {
    let mut command = Command::new(preview_bin());
    command
        .args(["apply", "--dir"])
        .arg(root)
        .args(["--context", context]);
    if dry_run {
        command.arg("--dry-run");
    }
    command_ok(&mut command, "preview apply rendered lifecycle");
}

fn build_and_load_smoke_image(cluster_name: &str) {
    let dir = tempfile::tempdir().expect("smoke image tempdir");
    let dockerfile = dir.path().join("Dockerfile");
    fs::write(
        &dockerfile,
        r#"FROM nginx:1.27-alpine
RUN printf 'ok\n' > /usr/share/nginx/html/readyz \
 && printf 'ok\n' > /usr/share/nginx/html/healthz \
 && printf 'preview-kind-smoke\n' > /usr/share/nginx/html/index.html \
 && sed -i 's/listen       80;/listen       8080;/' /etc/nginx/conf.d/default.conf
"#,
    )
    .expect("write smoke Dockerfile");
    command_ok(
        Command::new("docker")
            .args(["build", "-t", SMOKE_IMAGE])
            .arg(dir.path()),
        "docker build smoke image",
    );
    command_ok(
        Command::new("kind").args(["load", "docker-image", SMOKE_IMAGE, "--name", cluster_name]),
        "kind load smoke image",
    );
}

fn kubectl_rollout_status(namespace: &str, deployment: &str) {
    command_ok(
        Command::new("kubectl").args([
            "rollout",
            "status",
            &format!("deployment/{deployment}"),
            "-n",
            namespace,
            "--timeout=120s",
        ]),
        "kubectl rollout status",
    );
}

fn assert_service_has_endpoint(namespace: &str, service: &str) {
    let output = command_output(
        Command::new("kubectl").args([
            "get",
            "endpoints",
            service,
            "-n",
            namespace,
            "-o",
            "jsonpath={.subsets[0].addresses[0].ip}",
        ]),
        "kubectl get endpoints",
    );
    assert!(!output.trim().is_empty(), "service endpoint was empty");
}

fn assert_namespace_exists(namespace: &str) {
    command_ok(
        Command::new("kubectl").args(["get", "namespace", namespace]),
        "kubectl get namespace",
    );
}

fn assert_port_forward_reaches_readyz(namespace: &str, service: &str) {
    let port = free_local_port();
    let _forward = ChildGuard::spawn(
        Command::new("kubectl")
            .args([
                "port-forward",
                "-n",
                namespace,
                &format!("svc/{service}"),
                &format!("{port}:80"),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null()),
        "kubectl port-forward",
    );

    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Ok(response) = http_get(port, "/readyz") {
            assert!(
                response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200"),
                "unexpected readyz response: {response:?}"
            );
            assert!(response.contains("ok"), "readyz body did not contain ok");
            return;
        }
        assert!(Instant::now() < deadline, "port-forward never became ready");
        std::thread::sleep(Duration::from_millis(250));
    }
}

fn assert_workload_rbac_is_least_privilege() {
    let subject = "system:serviceaccount:uat-mr-123:preview-runner";
    let own_namespace = command_output(
        Command::new("kubectl").args([
            "auth",
            "can-i",
            "get",
            "pods",
            "-n",
            "uat-mr-123",
            "--as",
            subject,
        ]),
        "kubectl auth can-i get pods",
    );
    assert_eq!(own_namespace.trim(), "yes");

    let cluster_delete = command_output_allow_failure(Command::new("kubectl").args([
        "auth",
        "can-i",
        "delete",
        "namespaces",
        "--as",
        subject,
    ]));
    assert_eq!(cluster_delete.trim(), "no");
}

fn assert_quota_rejects_oversized_pod(root: &Path, image: &str) {
    let pod_path = root.join("k8s/oversized-pod.yaml");
    fs::write(&pod_path, oversized_pod_yaml(image)).expect("write oversized pod");
    let output = Command::new("kubectl")
        .args(["apply", "--dry-run=server", "-f"])
        .arg(&pod_path)
        .output()
        .unwrap_or_else(|err| panic!("kubectl oversized pod dry-run failed to start: {err}"));
    assert!(
        !output.status.success(),
        "oversized pod unexpectedly passed quota dry-run"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("exceeded quota") || stderr.contains("maximum"),
        "oversized pod failed for unexpected reason: {stderr}"
    );
}

fn oversized_pod_yaml(image: &str) -> String {
    format!(
        r#"apiVersion: v1
kind: Pod
metadata:
  name: oversized
  namespace: uat-mr-123
spec:
  restartPolicy: Never
  serviceAccountName: preview-runner
  containers:
  - name: oversized
    image: {image}
    resources:
      requests:
        cpu: "2"
        memory: 2Gi
      limits:
        cpu: "2"
        memory: 2Gi
"#
    )
}

struct ChildGuard {
    child: Child,
}

impl ChildGuard {
    fn spawn(command: &mut Command, label: &str) -> Self {
        let child = command
            .spawn()
            .unwrap_or_else(|err| panic!("{label} failed to start: {err}"));
        Self { child }
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn free_local_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .expect("bind local port")
        .local_addr()
        .expect("local addr")
        .port()
}

fn http_get(port: u16, path: &str) -> std::io::Result<String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.write_all(
        format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n").as_bytes(),
    )?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

fn command_ok(command: &mut Command, label: &str) {
    let status = command
        .status()
        .unwrap_or_else(|err| panic!("{label} failed to start: {err}"));
    assert!(status.success(), "{label} failed");
}

fn command_output(command: &mut Command, label: &str) -> String {
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("{label} failed to start: {err}"));
    assert!(
        output.status.success(),
        "{label} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn command_output_allow_failure(command: &mut Command) -> String {
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("command failed to start: {err}"));
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn optional_output(command: &mut Command) -> Option<String> {
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// </HANDWRITE>
