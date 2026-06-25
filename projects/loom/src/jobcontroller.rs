//! `loom job-controller` — relay → k8s Job bridge (#164).
//!
//! A relay consumer for `runner = k8s-job` tasks: for each leased task it
//! creates a k8s Job that runs the task (a one-shot worker container reading the
//! task context from env, fetching its input from keep, and reporting back),
//! then acks. It is the *only* component that touches the k8s API, keeping
//! loom's core cluster-free and testable.
//!
//! The task→Job translation and the lease loop are pure + tested with a fake
//! [`KubeApi`]; the live [`KubectlApi`] shells out to `kubectl` and therefore
//! needs a configured cluster (`scripts/jobcontroller-smoke.sh`).

use async_trait::async_trait;

use crate::scheduler::TaskMessage;
use crate::worker::RelayConsumer;

/// A k8s Job to create for one task attempt. A minimal, deterministic spec —
/// enough to run a one-shot worker container; richer pod templates layer on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobSpec {
    /// Job name, unique per (run, node, attempt).
    pub name: String,
    /// Container image (the polyglot worker / task image).
    pub image: String,
    /// Container args (here: `loom run-task`).
    pub args: Vec<String>,
    /// Env passed to the container: task context + relay/keep endpoints.
    pub env: Vec<(String, String)>,
    /// `backoffLimit` — loom owns retries via relay lease expiry, so 0.
    pub backoff_limit: u32,
}

/// Build the k8s Job spec for one task. Pure + deterministic (testable).
pub fn task_to_job_spec(task: &TaskMessage, image: &str, relay: &str, keep: &str) -> JobSpec {
    let name = format!("loom-{}-{}-{}", task.run_id, task.node_id, task.attempt)
        .to_lowercase()
        .replace('_', "-");
    let input_refs = task.input_refs.iter().map(|r| r.0.clone()).collect::<Vec<_>>().join(",");
    JobSpec {
        name,
        image: image.to_string(),
        args: vec!["run-task".to_string()],
        env: vec![
            ("LOOM_TASK_RUN".to_string(), task.run_id.clone()),
            ("LOOM_TASK_NODE".to_string(), task.node_id.clone()),
            ("LOOM_TASK_ATTEMPT".to_string(), task.attempt.to_string()),
            ("LOOM_TASK_NAME".to_string(), task.task_name.clone()),
            ("LOOM_TASK_INPUT_REFS".to_string(), input_refs),
            ("LOOM_RELAY".to_string(), relay.to_string()),
            ("LOOM_KEEP".to_string(), keep.to_string()),
        ],
        backoff_limit: 0,
    }
}

impl JobSpec {
    /// Render the Job as a k8s manifest (YAML) for `kubectl create -f -`.
    pub fn to_manifest(&self) -> String {
        let env_yaml = self
            .env
            .iter()
            .map(|(k, v)| format!("        - name: {k}\n          value: \"{v}\""))
            .collect::<Vec<_>>()
            .join("\n");
        let args_yaml =
            self.args.iter().map(|a| format!("        - \"{a}\"")).collect::<Vec<_>>().join("\n");
        format!(
            "apiVersion: batch/v1\n\
             kind: Job\n\
             metadata:\n  name: {name}\n  labels:\n    app: loom\n\
             spec:\n  backoffLimit: {backoff}\n  template:\n    spec:\n      restartPolicy: Never\n      containers:\n      - name: task\n        image: {image}\n        args:\n{args}\n        env:\n{env}\n",
            name = self.name,
            backoff = self.backoff_limit,
            image = self.image,
            args = args_yaml,
            env = env_yaml,
        )
    }
}

/// The k8s control surface the job-controller needs. The only cluster-touching
/// seam — faked in tests, real over `kubectl`.
#[async_trait]
pub trait KubeApi: Send + Sync {
    async fn create_job(&self, spec: &JobSpec) -> anyhow::Result<()>;
}

/// Live impl: `kubectl create -f -` with the rendered manifest. Needs a
/// configured cluster + `kubectl` on PATH.
pub struct KubectlApi {
    pub namespace: Option<String>,
}

#[async_trait]
impl KubeApi for KubectlApi {
    async fn create_job(&self, spec: &JobSpec) -> anyhow::Result<()> {
        let manifest = spec.to_manifest();
        let mut cmd = tokio::process::Command::new("kubectl");
        cmd.arg("create").arg("-f").arg("-");
        if let Some(ns) = &self.namespace {
            cmd.arg("-n").arg(ns);
        }
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        let mut child = cmd.spawn()?;
        {
            use tokio::io::AsyncWriteExt;
            let mut stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("no stdin"))?;
            stdin.write_all(manifest.as_bytes()).await?;
        }
        let out = child.wait_with_output().await?;
        anyhow::ensure!(
            out.status.success(),
            "kubectl create failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        Ok(())
    }
}

/// Lease one k8s-job task and create its Job. Returns true if a task was handled.
pub async fn run_once_job(
    consumer: &dyn RelayConsumer,
    kube: &dyn KubeApi,
    image: &str,
    relay: &str,
    keep: &str,
) -> anyhow::Result<bool> {
    let Some(task) = consumer.lease("loom-job-controller").await? else {
        return Ok(false);
    };
    let spec = task_to_job_spec(&task.message, image, relay, keep);
    kube.create_job(&spec).await?;
    consumer.ack(&task.lease_id, task.epoch).await?;
    Ok(true)
}

/// Entry point for `loom job-controller`. Leases `k8s-job`-routed tasks from
/// relay and creates a k8s Job per task. Needs LOOM_RELAY + a configured cluster.
pub fn run() -> anyhow::Result<()> {
    let relay = std::env::var("LOOM_RELAY")
        .map_err(|_| anyhow::anyhow!("LOOM_RELAY required for job-controller"))?;
    let keep = std::env::var("LOOM_KEEP").unwrap_or_default();
    let image = std::env::var("LOOM_JOB_IMAGE").unwrap_or_else(|_| "loom:latest".to_string());
    let namespace = std::env::var("LOOM_JOB_NAMESPACE").ok();
    // Match the dispatcher's route: it publishes to the bare runner route
    // (`k8s-job`), same convention the resident worker leases (`resident`).
    let subject = crate::runner::RunnerClass::K8sJob.relay_route().to_string();

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let consumer = crate::relay_client::RelayWorkConsumer::new(&relay, &subject)?;
        let kube = KubectlApi { namespace };
        eprintln!("loom job-controller: leasing `{subject}` from {relay}, image {image}");
        let idle = std::time::Duration::from_millis(300);
        loop {
            match run_once_job(&consumer, &kube, &image, &relay, &keep).await {
                Ok(true) => {}
                Ok(false) => tokio::time::sleep(idle).await,
                Err(e) => {
                    eprintln!("loom job-controller: {e}");
                    tokio::time::sleep(idle).await;
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::KeepRef;
    use crate::runner::RunnerClass;
    use crate::worker::LeasedTask;
    use std::sync::Mutex;

    fn task() -> TaskMessage {
        TaskMessage {
            run_id: "run1".into(),
            node_id: "nodeA".into(),
            attempt: 2,
            task_name: "crunch".into(),
            args: serde_json::Value::Null,
            input_refs: vec![KeepRef("in:1".into()), KeepRef("in:2".into())],
            input_inline: None,
            runner: RunnerClass::K8sJob,
        }
    }

    #[test]
    fn job_spec_is_deterministic_and_carries_task_context() {
        let s = task_to_job_spec(&task(), "myimg:1", "http://relay", "http://keep");
        assert_eq!(s.name, "loom-run1-nodea-2");
        assert_eq!(s.image, "myimg:1");
        assert_eq!(s.backoff_limit, 0); // loom owns retries via lease expiry
        assert!(s.env.contains(&("LOOM_TASK_NAME".into(), "crunch".into())));
        assert!(s.env.contains(&("LOOM_TASK_INPUT_REFS".into(), "in:1,in:2".into())));
        assert!(s.env.contains(&("LOOM_RELAY".into(), "http://relay".into())));
        // manifest is valid-looking k8s Job YAML
        let m = s.to_manifest();
        assert!(m.contains("kind: Job"));
        assert!(m.contains("name: loom-run1-nodea-2"));
        assert!(m.contains("image: myimg:1"));
        assert!(m.contains("restartPolicy: Never"));
    }

    struct FakeConsumer {
        leased: Mutex<bool>,
        acked: Mutex<Vec<String>>,
    }
    #[async_trait]
    impl RelayConsumer for FakeConsumer {
        async fn lease(&self, _c: &str) -> anyhow::Result<Option<LeasedTask>> {
            let mut l = self.leased.lock().unwrap();
            if *l {
                return Ok(None);
            }
            *l = true;
            Ok(Some(LeasedTask { lease_id: "L1".into(), epoch: 7, message: task() }))
        }
        async fn ack(&self, lease_id: &str, _epoch: u64) -> anyhow::Result<()> {
            self.acked.lock().unwrap().push(lease_id.to_string());
            Ok(())
        }
    }

    struct FakeKube {
        created: Mutex<Vec<JobSpec>>,
    }
    #[async_trait]
    impl KubeApi for FakeKube {
        async fn create_job(&self, spec: &JobSpec) -> anyhow::Result<()> {
            self.created.lock().unwrap().push(spec.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn leases_a_task_creates_a_job_and_acks() {
        let consumer = FakeConsumer { leased: Mutex::new(false), acked: Mutex::new(vec![]) };
        let kube = FakeKube { created: Mutex::new(vec![]) };

        let handled = run_once_job(&consumer, &kube, "img", "http://relay", "http://keep")
            .await
            .unwrap();
        assert!(handled);
        assert_eq!(kube.created.lock().unwrap().len(), 1);
        assert_eq!(kube.created.lock().unwrap()[0].name, "loom-run1-nodea-2");
        assert_eq!(consumer.acked.lock().unwrap().as_slice(), &["L1".to_string()]);

        // second lease is empty → no-op
        let again = run_once_job(&consumer, &kube, "img", "http://relay", "http://keep")
            .await
            .unwrap();
        assert!(!again);
        assert_eq!(kube.created.lock().unwrap().len(), 1);
    }

    /// Exercises the real `KubectlApi.create_job` path against a live cluster:
    /// renders our manifest, `kubectl create`s it, and waits for the Job to run
    /// to completion. Needs a kube context (e.g. `kind create cluster`). Run:
    ///   cargo test -p loom --lib -- --ignored kubectl_create_job_against_real_cluster
    #[tokio::test]
    #[ignore = "needs a real k8s cluster (kind); run with --ignored"]
    async fn kubectl_create_job_against_real_cluster() {
        use tokio::process::Command;
        let spec = JobSpec {
            name: "loom-live-verify".into(),
            image: "busybox:stable".into(),
            args: vec!["sh".into(), "-c".into(), "echo loom-ok".into()],
            env: vec![("LOOM_TASK_NAME".into(), "verify".into())],
            backoff_limit: 0,
        };
        let del = || async {
            let _ = Command::new("kubectl")
                .args(["delete", "job", "loom-live-verify", "--ignore-not-found"])
                .output()
                .await;
        };
        del().await;
        // THE gated code path: our manifest → kubectl create → real k8s API.
        kube_create_or_skip(&spec).await;
        let out = Command::new("kubectl")
            .args(["wait", "--for=condition=complete", "job/loom-live-verify", "--timeout=90s"])
            .output()
            .await
            .expect("kubectl wait");
        del().await;
        assert!(
            out.status.success(),
            "Job did not complete: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    async fn kube_create_or_skip(spec: &JobSpec) {
        let kube = KubectlApi { namespace: None };
        if let Err(e) = kube.create_job(spec).await {
            panic!("create_job against real cluster failed: {e}");
        }
    }
}
