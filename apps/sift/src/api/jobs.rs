use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::{bail, Context, Result};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{QueryRequestV1, QueryResponseV1};

static NEXT_JOB: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryJobStatusV1 {
    Queued,
    Running,
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QueryJobV1 {
    pub query_id: String,
    pub project: String,
    pub status: QueryJobStatusV1,
    pub created_at: String,
    pub updated_at: String,
    pub request: QueryRequestV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<QueryResponseV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub(crate) struct QueryJobStore {
    root: PathBuf,
}

impl QueryJobStore {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)
            .with_context(|| format!("create query job directory {}", root.display()))?;
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
            .with_context(|| format!("set query job directory mode {}", root.display()))?;
        let store = Self { root };
        for entry in fs::read_dir(&store.root)
            .with_context(|| format!("read query job directory {}", store.root.display()))?
        {
            let path = entry?.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let mut job = store.read_path(&path)?;
            if matches!(
                job.status,
                QueryJobStatusV1::Queued | QueryJobStatusV1::Running
            ) {
                job.status = QueryJobStatusV1::Failed;
                job.error = Some("query process stopped before the job completed".into());
                job.updated_at = now();
                store.write(&job)?;
            }
        }
        Ok(store)
    }

    pub fn create(&self, request: QueryRequestV1) -> Result<QueryJobV1> {
        let created_at = now();
        let nonce = NEXT_JOB.fetch_add(1, Ordering::Relaxed);
        let query_id = hex::encode(Sha256::digest(format!(
            "{}:{}:{}:{}",
            std::process::id(),
            created_at,
            nonce,
            request.project
        )))[..32]
            .to_string();
        let job = QueryJobV1 {
            query_id,
            project: request.project.clone(),
            status: QueryJobStatusV1::Queued,
            created_at: created_at.clone(),
            updated_at: created_at,
            request,
            result: None,
            error: None,
        };
        self.write(&job)?;
        Ok(job)
    }

    pub fn get(&self, query_id: &str) -> Result<Option<QueryJobV1>> {
        validate_id(query_id)?;
        let path = self.path(query_id);
        if !path.exists() {
            return Ok(None);
        }
        self.read_path(&path).map(Some)
    }

    pub fn mark_running(&self, query_id: &str) -> Result<()> {
        self.update(query_id, |job| {
            job.status = QueryJobStatusV1::Running;
            job.error = None;
        })
    }

    pub fn succeed(&self, query_id: &str, mut result: QueryResponseV1) -> Result<()> {
        result.query_id = Some(query_id.to_string());
        self.update(query_id, move |job| {
            job.status = QueryJobStatusV1::Succeeded;
            job.result = Some(result);
            job.error = None;
        })
    }

    pub fn fail(&self, query_id: &str, message: String) -> Result<()> {
        self.update(query_id, move |job| {
            job.status = QueryJobStatusV1::Failed;
            job.error = Some(message);
            job.result = None;
        })
    }

    fn update(&self, query_id: &str, update: impl FnOnce(&mut QueryJobV1)) -> Result<()> {
        let mut job = self
            .get(query_id)?
            .with_context(|| format!("query job `{query_id}` does not exist"))?;
        update(&mut job);
        job.updated_at = now();
        self.write(&job)
    }

    fn write(&self, job: &QueryJobV1) -> Result<()> {
        validate_id(&job.query_id)?;
        let path = self.path(&job.query_id);
        storage_durable::atomic_write(
            &path,
            &serde_json::to_vec(job).context("encode query job")?,
            storage_durable::FsyncPolicy::Always,
        )?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("set query job file mode {}", path.display()))?;
        Ok(())
    }

    fn read_path(&self, path: &Path) -> Result<QueryJobV1> {
        let bytes = fs::read(path).with_context(|| format!("read query job {}", path.display()))?;
        serde_json::from_slice(&bytes)
            .with_context(|| format!("decode query job {}", path.display()))
    }

    fn path(&self, query_id: &str) -> PathBuf {
        self.root.join(format!("{query_id}.json"))
    }
}

impl service_executor::JobState<QueryResponseV1> for QueryJobStore {
    type Error = anyhow::Error;
    type Id = String;

    fn mark_running(&self, id: &Self::Id) -> Result<()> {
        QueryJobStore::mark_running(self, id)
    }

    fn succeed(&self, id: &Self::Id, output: QueryResponseV1) -> Result<()> {
        QueryJobStore::succeed(self, id, output)
    }

    fn fail(&self, id: &Self::Id, message: String) -> Result<()> {
        QueryJobStore::fail(self, id, message)
    }
}

fn validate_id(query_id: &str) -> Result<()> {
    if query_id.len() != 32 || !query_id.bytes().all(|value| value.is_ascii_hexdigit()) {
        bail!("query id has an invalid format");
    }
    Ok(())
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}
