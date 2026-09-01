use super::super::{
    StandaloneGkeArgs, StandaloneGkeCmd, StandaloneGkeInitArgs, StandaloneGkeRenderArgs,
};
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use service_k8s::render::common::{network_policy, NetworkPolicy, ServicePodTemplate};
use service_k8s::render::stateful_instance::{
    stateful_instance, ExistingClaim, StatefulInstancePlan, StatefulStorageAttachment,
};
use service_k8s::render::{
    client_service, requested_resources, restricted_container_security_context,
    restricted_pod_security_context, RenderCtx,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use storage_durable::{atomic_write, sync_parent_dir, FsyncPolicy};

const MARKER: &str = "lumen-standalone-managed/v1\n";
const IMAGE: &str = "ghcr.io/chrischeng-c4/lumen:0.4.31";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Config {
    #[serde(default = "default_name")]
    name: String,
    #[serde(default = "default_namespace")]
    namespace: String,
    node_pool: String,
    cpu: String,
    memory: String,
    #[serde(default = "default_storage_size")]
    storage_size: String,
    #[serde(default = "default_storage_class")]
    storage_class: String,
    allowed_service_accounts: Vec<String>,
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
pub(crate) struct BackupTarget {
    pub(crate) name: String,
    pub(crate) namespace: String,
}

fn load_config(path: &Path) -> Result<Config> {
    let config: Config =
        serde_yaml::from_slice(&fs::read(path).context("read config")?).context("parse config")?;
    validate(&config)?;
    Ok(config)
}

#[cfg(all(feature = "backup", feature = "delegated-auth"))]
pub(crate) fn load_target(path: &Path) -> Result<BackupTarget> {
    let config = load_config(path)?;
    Ok(BackupTarget {
        name: config.name,
        namespace: config.namespace,
    })
}
fn default_name() -> String {
    "lumen".into()
}
fn default_namespace() -> String {
    "lumen".into()
}
fn default_storage_size() -> String {
    "20Gi".into()
}
fn default_storage_class() -> String {
    "premium-rwo".into()
}

pub(crate) fn run(args: StandaloneGkeArgs) -> Result<()> {
    match args.cmd {
        StandaloneGkeCmd::Init(a) => init(a),
        StandaloneGkeCmd::Render(a) => render(a),
    }
}
fn init(a: StandaloneGkeInitArgs) -> Result<()> {
    if a.out.exists() {
        bail!("refusing to overwrite existing file")
    }
    let parent = parent_dir(&a.out);
    fs::create_dir_all(parent)?;
    let text = "name: lumen\nnamespace: lumen\nnodePool: REQUIRED\ncpu: REQUIRED\nmemory: REQUIRED\nstorageSize: 20Gi\nstorageClass: premium-rwo\nallowedServiceAccounts:\n  - namespace/name\n";
    write_file(&a.out, text.as_bytes())
}
fn render(a: StandaloneGkeRenderArgs) -> Result<()> {
    let cfg = load_config(&a.file)?;
    let docs = build(&cfg)?;
    validate_output(&a.out)?;
    let parent = parent_dir(&a.out);
    fs::create_dir_all(parent)?;
    let stage = unique_sibling(parent, ".lumen-stage")?;
    fs::create_dir(&stage)?;
    if let Err(error) = write_stage(&stage, docs) {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }

    if !a.out.exists() {
        fs::rename(&stage, &a.out).context("commit rendered output")?;
        sync_parent_dir(&a.out)?;
        return Ok(());
    }

    let old = unique_sibling(parent, ".lumen-old")?;
    fs::rename(&a.out, &old).context("stage previous managed output")?;
    sync_parent_dir(&a.out)?;
    if let Err(commit_error) = fs::rename(&stage, &a.out) {
        let restore = fs::rename(&old, &a.out);
        let _ = sync_parent_dir(&a.out);
        let _ = fs::remove_dir_all(&stage);
        if let Err(restore_error) = restore {
            bail!(
                "could not commit rendered output ({commit_error}) and could not restore the previous managed output ({restore_error})"
            );
        }
        return Err(commit_error).context("commit rendered output");
    }
    sync_parent_dir(&a.out)?;
    fs::remove_dir_all(&old).context("remove previous managed output")?;
    sync_parent_dir(&old)?;
    Ok(())
}

fn write_stage(stage: &Path, docs: (Vec<(String, Value)>, Vec<(String, Value)>)) -> Result<()> {
    let storage = stage.join("storage");
    let runtime = stage.join("runtime");
    fs::create_dir(&storage)?;
    fs::create_dir(&runtime)?;
    write_file(&stage.join(".lumen-standalone-managed"), MARKER.as_bytes())?;
    for (root, files) in [(storage, docs.0), (runtime, docs.1)] {
        for (name, value) in files {
            write_file(&root.join(name), serde_yaml::to_string(&value)?.as_bytes())?;
        }
    }
    Ok(())
}

fn validate_output(out: &Path) -> Result<()> {
    if out.file_name().is_none() {
        bail!("output root must name a directory")
    }
    if fs::symlink_metadata(out)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        bail!("output root must not be a symlink")
    }
    if out.exists() {
        if !out.is_dir() {
            bail!("output root must be a directory")
        }
        let marker = fs::read(out.join(".lumen-standalone-managed")).unwrap_or_default();
        let entries: Vec<_> = fs::read_dir(out)?.collect::<std::io::Result<_>>()?;
        if !entries.is_empty() && marker != MARKER.as_bytes() {
            bail!("refusing unmanaged output")
        }
        for sub in ["storage", "runtime"] {
            let path = out.join(sub);
            if fs::symlink_metadata(&path)
                .map(|metadata| metadata.file_type().is_symlink())
                .unwrap_or(false)
            {
                bail!("generated root must not be a symlink")
            }
        }
    }
    Ok(())
}

fn parent_dir(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn unique_sibling(parent: &Path, prefix: &str) -> Result<PathBuf> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for attempt in 0..1000 {
        let path = parent.join(format!("{prefix}-{}-{stamp}-{attempt}", std::process::id()));
        if !path.exists() {
            return Ok(path);
        }
    }
    bail!("could not allocate a staging directory")
}
fn valid_dns(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 63
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !s.starts_with('-')
        && !s.ends_with('-')
}
fn valid_node_pool(s: &str) -> bool {
    s.len() <= 40 && valid_dns(s) && s.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
}
fn quantity(s: &str) -> bool {
    if s.is_empty()
        || s.len() > 32
        || s.trim() != s
        || matches!(s.as_bytes().first(), Some(b'+' | b'-'))
    {
        return false;
    }
    let suffix = [
        "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "n", "u", "m", "k", "K", "M", "G", "T", "P", "E",
    ]
    .into_iter()
    .find(|suffix| s.ends_with(suffix))
    .unwrap_or("");
    let number = s.strip_suffix(suffix).unwrap_or(s);
    if number.is_empty()
        || number.chars().filter(|ch| *ch == '.').count() > 1
        || !number.chars().all(|ch| ch.is_ascii_digit() || ch == '.')
        || !number.chars().any(|ch| ch.is_ascii_digit())
    {
        return false;
    }
    number
        .parse::<f64>()
        .map(|value| value.is_finite() && value > 0.0)
        .unwrap_or(false)
}
fn valid_dns_subdomain(s: &str) -> bool {
    !s.is_empty() && s.len() <= 253 && s.split('.').all(valid_dns)
}
fn placeholder(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    s.trim().is_empty()
        || lower == "required"
        || lower == "namespace/name"
        || lower.contains("replace_me")
        || lower.contains("placeholder")
        || s.contains('<')
        || s.contains('>')
        || s.contains('$')
}
fn validate(c: &Config) -> Result<()> {
    if placeholder(&c.name)
        || placeholder(&c.namespace)
        || !valid_dns(&c.name)
        || !valid_dns(&c.namespace)
    {
        bail!("invalid DNS name")
    };
    let mut derived = vec![
        format!("{}-data", c.name),
        format!("{}-admin", c.name),
        format!("{}-client", c.name),
    ];
    let mut accounts = c.allowed_service_accounts.clone();
    accounts.sort();
    derived.extend(
        accounts
            .iter()
            .enumerate()
            .map(|(index, _)| format!("{}-client-{index:03}", c.name)),
    );
    for name in derived {
        if !valid_dns(&name) {
            bail!("invalid derived Kubernetes name")
        }
    }
    let binding = format!("lumen.{}.{}.auth-delegator", c.namespace, c.name);
    if !valid_dns_subdomain(&binding) {
        bail!("invalid derived ClusterRoleBinding name")
    }
    if placeholder(&c.node_pool) || !valid_node_pool(&c.node_pool) {
        bail!("invalid node pool")
    };
    for (n, v) in [("cpu", &c.cpu), ("memory", &c.memory)] {
        if placeholder(v) {
            bail!("{n} is required")
        };
    }
    if !quantity(&c.cpu) || !quantity(&c.memory) || !quantity(&c.storage_size) {
        bail!("invalid resource quantity")
    };
    if placeholder(&c.storage_class) || !valid_dns_subdomain(&c.storage_class) {
        bail!("invalid storage class")
    }
    if c.allowed_service_accounts.is_empty() {
        bail!("allowedServiceAccounts must not be empty")
    };
    let mut seen = BTreeSet::new();
    for x in &accounts {
        let p: Vec<_> = x.split('/').collect();
        if placeholder(x) || p.len() != 2 || !valid_dns(p[0]) || !valid_dns(p[1]) || !seen.insert(x)
        {
            bail!("invalid or duplicate service account")
        };
    }
    Ok(())
}
fn meta(c: &Config, name: &str, component: &str) -> Value {
    json!({"name":name,"namespace":c.namespace,"labels":{"app.kubernetes.io/name":"lumen","app.kubernetes.io/instance":c.name,"app.kubernetes.io/component":component,"app.kubernetes.io/managed-by":"lumen-standalone","lumen.axiom.dev/instance":c.name,"lumen.axiom.dev/profile":"gke","lumen.axiom.dev/storage":format!("{}-data",c.name)}})
}
fn attach_identity(value: &mut Value, c: &Config) {
    value["metadata"]["labels"]["lumen.axiom.dev/instance"] = json!(c.name);
    value["metadata"]["labels"]["lumen.axiom.dev/profile"] = json!("gke");
    value["metadata"]["labels"]["lumen.axiom.dev/storage"] = json!(format!("{}-data", c.name));
    value["metadata"]["annotations"]["lumen.axiom.dev/instance-identity"] =
        json!(format!("{}/{}", c.namespace, c.name));
}
fn build(c: &Config) -> Result<(Vec<(String, Value)>, Vec<(String, Value)>)> {
    let cx = RenderCtx {
        app: "lumen",
        manager: "lumen-standalone",
        api_version: "v1",
        kind: "Standalone",
        name: &c.name,
        ns: &c.namespace,
        owner: None,
    };
    let labels = cx
        .selector("serving")
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().unwrap().into()))
        .collect();
    let pvc_template = json!({"spec":{"accessModes":["ReadWriteOnce"],"resources":{"requests":{"storage":c.storage_size}},"storageClassName":c.storage_class}});
    let claim = ExistingClaim::new(
        "data",
        format!("{}-data", c.name),
        pvc_template,
        "/var/lib/lumen/data",
    );
    let pod = ServicePodTemplate {
        cx: &cx,
        component: "serving",
        image: IMAGE,
        image_pull_policy: "IfNotPresent",
        command: vec!["lumen".into(), "serve".into()],
        args: vec![],
        ports: vec![json!({"name":"http","containerPort":7373,"protocol":"TCP"})],
        env: vec![
            json!({"name":"LUMEN_AUTH","value":"in-cluster"}),
            json!({"name":"LUMEN_AUTH_NAMESPACE","value":c.namespace}),
        ],
        env_from: vec![],
        resources: requested_resources(&c.cpu, &c.memory),
        readiness_probe: Some(json!({
            "httpGet":{"path":"/readyz","port":"http","scheme":"HTTP"},
            "initialDelaySeconds":5,"periodSeconds":10,"timeoutSeconds":3,"failureThreshold":60,
        })),
        liveness_probe: Some(json!({
            "httpGet":{"path":"/healthz","port":"http","scheme":"HTTP"},
            "initialDelaySeconds":15,"periodSeconds":30,"timeoutSeconds":5,"failureThreshold":3,
        })),
        startup_probe: Some(json!({
            "httpGet":{"path":"/healthz","port":"http","scheme":"HTTP"},
            "periodSeconds":5,"timeoutSeconds":3,"failureThreshold":120,
        })),
        lifecycle: None,
        container_security_context: Some(restricted_container_security_context()),
        pod_security_context: Some(restricted_pod_security_context()),
        service_account_name: Some(&c.name),
        termination_grace_period_seconds: Some(30),
        volumes: vec![json!({"name":"tmp","emptyDir":{}})],
        volume_mounts: vec![json!({"name":"tmp","mountPath":"/tmp"})],
        pod_annotations: Some(
            json!({"prometheus.io/scrape":"true","prometheus.io/port":"7373","prometheus.io/path":"/metrics"}),
        ),
        topology_spread_constraints: vec![],
    };
    let mut plan = StatefulInstancePlan::new(
        &cx,
        c.name.clone(),
        1,
        pod,
        StatefulStorageAttachment::ExistingClaim(claim),
    );
    plan.labels = labels;
    plan.labels
        .insert("lumen.axiom.dev/instance".into(), c.name.clone());
    plan.labels
        .insert("lumen.axiom.dev/profile".into(), "gke".into());
    plan.labels
        .insert("lumen.axiom.dev/storage".into(), format!("{}-data", c.name));
    plan.node_selector = Some(json!({"cloud.google.com/gke-nodepool":c.node_pool}));
    // A Service named `lumen` would otherwise inject `LUMEN_PORT=tcp://...`,
    // which collides with Lumen's numeric `LUMEN_PORT` serving option.
    plan.enable_service_links = Some(false);
    let mut rendered = stateful_instance(plan).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    attach_identity(&mut rendered.workload, c);
    let mut pvc = rendered
        .storage
        .take()
        .expect("existing claim renders a PVC");
    attach_identity(&mut pvc, c);
    let mut service = client_service(&cx, &c.name, "serving", 7373);
    attach_identity(&mut service, c);
    let storage = vec![
        (
            "namespace.yaml".into(),
            json!({"apiVersion":"v1","kind":"Namespace","metadata":{"name":c.namespace,"labels":{"pod-security.kubernetes.io/enforce":"restricted","pod-security.kubernetes.io/audit":"restricted","pod-security.kubernetes.io/warn":"restricted"}}}),
        ),
        ("pvc.yaml".into(), pvc),
        (
            "kustomization.yaml".into(),
            json!({"apiVersion":"kustomize.config.k8s.io/v1beta1","kind":"Kustomization","resources":["namespace.yaml","pvc.yaml"]}),
        ),
    ];
    let mut runtime = vec![
        ("statefulset.yaml".into(), rendered.workload),
        ("service.yaml".into(), service),
        (
            "serviceaccount.yaml".into(),
            json!({"apiVersion":"v1","kind":"ServiceAccount","automountServiceAccountToken":true,"metadata":meta(c,&c.name,"serving")}),
        ),
        (
            "admin-serviceaccount.yaml".into(),
            json!({"apiVersion":"v1","kind":"ServiceAccount","automountServiceAccountToken":true,"metadata":meta(c,&format!("{}-admin",c.name),"admin")}),
        ),
        (
            "clusterrolebinding.yaml".into(),
            json!({"apiVersion":"rbac.authorization.k8s.io/v1","kind":"ClusterRoleBinding","metadata":{"name":format!("lumen.{}.{}.auth-delegator",c.namespace,c.name),"labels":{"app.kubernetes.io/name":"lumen","app.kubernetes.io/instance":c.name,"app.kubernetes.io/component":"auth-delegation","app.kubernetes.io/managed-by":"lumen-standalone","lumen.axiom.dev/owner-namespace":c.namespace,"lumen.axiom.dev/profile":"gke"}},"roleRef":{"apiGroup":"rbac.authorization.k8s.io","kind":"ClusterRole","name":"system:auth-delegator"},"subjects":[{"kind":"ServiceAccount","name":c.name,"namespace":c.namespace}]}),
        ),
    ];
    let resources = json!(["lumencollections"]);
    runtime.push(("client-role.yaml".into(),json!({"apiVersion":"rbac.authorization.k8s.io/v1","kind":"Role","metadata":meta(c,&format!("{}-client",c.name),"rbac"),"rules":[{"apiGroups":["lumen.axiom.dev"],"resources":resources,"verbs":["get","update","delete"]}]})));
    runtime.push(("admin-role.yaml".into(),json!({"apiVersion":"rbac.authorization.k8s.io/v1","kind":"Role","metadata":meta(c,&format!("{}-admin",c.name),"rbac"),"rules":[{"apiGroups":["lumen.axiom.dev"],"resources":["lumencollections","lumenadmin"],"verbs":["get","update","delete"]}]})));
    let mut accounts = c.allowed_service_accounts.clone();
    accounts.sort();
    for (index, x) in accounts.iter().enumerate() {
        let p: Vec<_> = x.split('/').collect();
        runtime.push((format!("client-rolebinding-{index:03}.yaml"),json!({"apiVersion":"rbac.authorization.k8s.io/v1","kind":"RoleBinding","metadata":meta(c,&format!("{}-client-{index:03}",c.name),"rbac"),"roleRef":{"apiGroup":"rbac.authorization.k8s.io","kind":"Role","name":format!("{}-client",c.name)},"subjects":[{"kind":"ServiceAccount","name":p[1],"namespace":p[0]}]})));
    }
    runtime.push(("admin-rolebinding.yaml".into(),json!({"apiVersion":"rbac.authorization.k8s.io/v1","kind":"RoleBinding","metadata":meta(c,&format!("{}-admin",c.name),"rbac"),"roleRef":{"apiGroup":"rbac.authorization.k8s.io","kind":"Role","name":format!("{}-admin",c.name)},"subjects":[{"kind":"ServiceAccount","name":format!("{}-admin",c.name),"namespace":c.namespace}]})));
    runtime.push((
        "networkpolicy.yaml".into(),
        network_policy(NetworkPolicy {
            cx: &cx,
            name: &c.name,
            component: "serving",
            client_ports: vec![7373],
            peer_ports: vec![],
            extra_egress: vec![],
        }),
    ));
    let mut names: Vec<_> = runtime.iter().map(|x| x.0.clone()).collect();
    names.sort();
    runtime.push(("kustomization.yaml".into(),json!({"apiVersion":"kustomize.config.k8s.io/v1beta1","kind":"Kustomization","resources":names})));
    Ok((storage, runtime))
}
fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    atomic_write(path, bytes, FsyncPolicy::Always)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(name: &str, accounts: Vec<String>) -> Config {
        Config {
            name: name.into(),
            namespace: "lumen".into(),
            node_pool: "pool".into(),
            cpu: "1".into(),
            memory: "1Gi".into(),
            storage_size: "20Gi".into(),
            storage_class: "premium-rwo".into(),
            allowed_service_accounts: accounts,
        }
    }

    #[test]
    fn fifty_two_char_name_with_one_account_is_valid() {
        let name = "a".repeat(52);
        assert!(validate(&config(&name, vec!["ns/sa".into()])).is_ok());
    }

    #[test]
    fn fifty_three_char_name_fails_indexed_binding() {
        let name = "a".repeat(53);
        assert!(validate(&config(&name, vec!["ns/sa".into()])).is_err());
    }

    #[test]
    fn fifty_eight_char_name_fails_admin_name() {
        let name = "a".repeat(58);
        assert!(validate(&config(&name, vec!["ns/sa".into()])).is_err());
    }

    #[test]
    fn too_many_accounts_fail_index_one_thousand() {
        let accounts = (0..1001).map(|index| format!("ns-{index}/sa")).collect();
        assert!(validate(&config(&"a".repeat(52), accounts)).is_err());
    }

    #[test]
    fn every_rendered_metadata_name_is_valid() {
        let cfg = config("a".repeat(52).as_str(), vec!["ns/sa".into()]);
        validate(&cfg).unwrap();
        let (storage, runtime) = build(&cfg).unwrap();
        for (filename, document) in storage.into_iter().chain(runtime) {
            let Some(metadata_name) = document
                .get("metadata")
                .and_then(|metadata| metadata.get("name"))
                .and_then(Value::as_str)
            else {
                continue;
            };
            if filename == "clusterrolebinding.yaml" {
                assert!(valid_dns_subdomain(metadata_name), "{metadata_name}");
            } else {
                assert!(valid_dns(metadata_name), "{metadata_name}");
            }
        }
    }
}
