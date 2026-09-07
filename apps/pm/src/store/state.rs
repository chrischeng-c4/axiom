use std::collections::BTreeMap;
use std::path::Path;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::models::{
    Defect, DefectSeverity, DefectStatus, Feature, FeatureStatus, GateRun, Prd, Project,
    ReviewComment, ReviewSeverity, Task, TaskStatus, TechDesign,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PmState {
    #[serde(default)]
    pub projects: BTreeMap<String, Project>,
    #[serde(default)]
    pub prds: BTreeMap<String, Prd>,
    #[serde(default)]
    pub tech_designs: BTreeMap<String, TechDesign>,
    #[serde(default)]
    pub features: BTreeMap<String, Feature>,
    #[serde(default)]
    pub tasks: BTreeMap<String, Task>,
    #[serde(default)]
    pub defects: BTreeMap<String, Defect>,
    #[serde(default)]
    pub gate_runs: BTreeMap<String, Vec<GateRun>>,
    #[serde(default)]
    pub review_comments: BTreeMap<String, ReviewComment>,
}

impl PmState {
    pub fn load_or_default(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read state file at {}", path.display()))?;
        if data.trim().is_empty() {
            return Ok(Self::default());
        }
        serde_json::from_str(&data)
            .with_context(|| format!("Failed to parse JSON state file at {}", path.display()))
    }

    pub fn save_atomic(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let tmp_path = path.with_extension(format!("tmp.{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        let json_bytes = serde_json::to_vec_pretty(self)?;
        std::fs::write(&tmp_path, json_bytes)?;
        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }

    // Projects
    pub fn upsert_project(&mut self, project: Project) {
        self.projects.insert(project.id.clone(), project);
    }

    pub fn get_project(&self, id: &str) -> Option<&Project> {
        self.projects.get(id)
    }

    pub fn list_projects(&self) -> Vec<&Project> {
        self.projects.values().collect()
    }

    // PRDs
    pub fn upsert_prd(&mut self, prd: Prd) {
        self.prds.insert(prd.id.clone(), prd);
    }

    pub fn get_prd(&self, id: &str) -> Option<&Prd> {
        self.prds.get(id)
    }

    pub fn list_prds(&self, project_id: &str) -> Vec<&Prd> {
        self.prds
            .values()
            .filter(|p| p.project_id == project_id)
            .collect()
    }

    // Tech Designs
    pub fn upsert_tech_design(&mut self, td: TechDesign) {
        self.tech_designs.insert(td.id.clone(), td);
    }

    pub fn get_tech_design(&self, id: &str) -> Option<&TechDesign> {
        self.tech_designs.get(id)
    }

    pub fn list_tech_designs(&self, project_id: &str) -> Vec<&TechDesign> {
        self.tech_designs
            .values()
            .filter(|td| td.project_id == project_id)
            .collect()
    }

    // Features
    pub fn upsert_feature(&mut self, feature: Feature) {
        self.features.insert(feature.id.clone(), feature);
    }

    pub fn get_feature(&self, id: &str) -> Option<&Feature> {
        self.features.get(id)
    }

    pub fn list_features(&self, project_id: &str, status: Option<FeatureStatus>) -> Vec<&Feature> {
        self.features
            .values()
            .filter(|f| f.project_id == project_id && status.map_or(true, |s| f.status == s))
            .collect()
    }

    // Tasks
    pub fn upsert_task(&mut self, task: Task) {
        self.tasks.insert(task.id.clone(), task);
    }

    pub fn get_task(&self, id: &str) -> Option<&Task> {
        self.tasks.get(id)
    }

    pub fn list_tasks(
        &self,
        project_id: &str,
        feature_id: Option<&str>,
        status: Option<TaskStatus>,
        assignee: Option<&str>,
    ) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|t| {
                t.project_id == project_id
                    && feature_id.map_or(true, |fid| t.feature_id.as_deref() == Some(fid))
                    && status.map_or(true, |s| t.status == s)
                    && assignee.map_or(true, |a| t.assignee.as_deref() == Some(a))
            })
            .collect()
    }

    /// Automatically transitions any Blocked tasks whose prerequisites in `blocked_by`
    /// are all now in `Done` status into `Todo` status.
    pub fn unblock_dependent_tasks(&mut self, completed_task_id: &str) -> Vec<String> {
        let mut newly_unblocked = Vec::new();
        let candidate_ids: Vec<String> = self
            .tasks
            .values()
            .filter(|t| t.status == TaskStatus::Blocked && t.blocked_by.iter().any(|d| d == completed_task_id))
            .map(|t| t.id.clone())
            .collect();

        for tid in candidate_ids {
            let all_deps_done = if let Some(task) = self.tasks.get(&tid) {
                task.blocked_by.iter().all(|dep_id| {
                    self.tasks.get(dep_id).map_or(false, |dt| dt.status == TaskStatus::Done)
                })
            } else {
                false
            };

            if all_deps_done {
                if let Some(task) = self.tasks.get_mut(&tid) {
                    task.status = TaskStatus::Todo;
                    task.updated_at = Utc::now().to_rfc3339();
                    newly_unblocked.push(tid);
                }
            }
        }

        newly_unblocked
    }

    // Defects (Bug Tracking)
    pub fn upsert_defect(&mut self, defect: Defect) {
        self.defects.insert(defect.id.clone(), defect);
    }

    pub fn get_defect(&self, id: &str) -> Option<&Defect> {
        self.defects.get(id)
    }

    pub fn list_defects(
        &self,
        project_id: &str,
        status: Option<DefectStatus>,
        severity: Option<DefectSeverity>,
    ) -> Vec<&Defect> {
        self.defects
            .values()
            .filter(|d| {
                d.project_id == project_id
                    && status.map_or(true, |st| d.status == st)
                    && severity.map_or(true, |sev| d.severity == sev)
            })
            .collect()
    }

    // Gate Runs (Pre-merge test verification receipts)
    pub fn record_gate_run(&mut self, run: GateRun) {
        self.gate_runs
            .entry(run.task_id.clone())
            .or_default()
            .push(run);
    }

    pub fn get_gate_runs(&self, task_id: &str) -> Vec<&GateRun> {
        self.gate_runs
            .get(task_id)
            .map(|list| list.iter().collect())
            .unwrap_or_default()
    }

    pub fn get_latest_gate_run(&self, task_id: &str) -> Option<&GateRun> {
        self.gate_runs
            .get(task_id)
            .and_then(|list| list.last())
    }

    // Review Comments (Pre-merge review comments)
    pub fn upsert_review_comment(&mut self, comment: ReviewComment) {
        self.review_comments.insert(comment.id.clone(), comment);
    }

    pub fn get_review_comment(&self, id: &str) -> Option<&ReviewComment> {
        self.review_comments.get(id)
    }

    pub fn list_review_comments(&self, task_id: &str, unresolved_only: bool) -> Vec<&ReviewComment> {
        self.review_comments
            .values()
            .filter(|c| c.task_id == task_id && (!unresolved_only || !c.resolved))
            .collect()
    }

    pub fn has_unresolved_blocking_reviews(&self, task_id: &str) -> bool {
        self.review_comments
            .values()
            .any(|c| c.task_id == task_id && c.severity == ReviewSeverity::Blocking && !c.resolved)
    }

    pub fn get_project_summary(&self, project_id: &str) -> serde_json::Value {
        let project = self.get_project(project_id);
        let prds: Vec<&Prd> = self.list_prds(project_id);
        let tds: Vec<&TechDesign> = self.list_tech_designs(project_id);
        let features: Vec<&Feature> = self.list_features(project_id, None);
        let tasks: Vec<&Task> = self.list_tasks(project_id, None, None, None);
        let defects: Vec<&Defect> = self.list_defects(project_id, None, None);

        let mut task_counts = BTreeMap::new();
        task_counts.insert("todo", 0);
        task_counts.insert("in_progress", 0);
        task_counts.insert("in_review", 0);
        task_counts.insert("done", 0);
        task_counts.insert("blocked", 0);

        for task in &tasks {
            let key = match task.status {
                TaskStatus::Todo => "todo",
                TaskStatus::InProgress => "in_progress",
                TaskStatus::InReview => "in_review",
                TaskStatus::Done => "done",
                TaskStatus::Blocked => "blocked",
            };
            *task_counts.entry(key).or_insert(0) += 1;
        }

        let mut feature_counts = BTreeMap::new();
        for feature in &features {
            let key = match feature.status {
                FeatureStatus::Backlog => "backlog",
                FeatureStatus::Planned => "planned",
                FeatureStatus::InProgress => "in_progress",
                FeatureStatus::Completed => "completed",
                FeatureStatus::Cancelled => "cancelled",
            };
            *feature_counts.entry(key).or_insert(0) += 1;
        }

        let mut defect_counts = BTreeMap::new();
        for defect in &defects {
            let key = match defect.status {
                DefectStatus::Open => "open",
                DefectStatus::InProgress => "in_progress",
                DefectStatus::Resolved => "resolved",
                DefectStatus::WontFix => "wont_fix",
            };
            *defect_counts.entry(key).or_insert(0) += 1;
        }

        serde_json::json!({
            "project": project,
            "counts": {
                "prds": prds.len(),
                "tech_designs": tds.len(),
                "features": features.len(),
                "tasks_total": tasks.len(),
                "tasks_by_status": task_counts,
                "features_by_status": feature_counts,
                "defects_total": defects.len(),
                "defects_by_status": defect_counts,
            },
            "recent_prds": prds.iter().map(|p| serde_json::json!({
                "id": p.id,
                "title": p.title,
                "version": p.version,
                "status": p.status.to_string(),
            })).collect::<Vec<_>>(),
            "recent_features": features.iter().map(|f| serde_json::json!({
                "id": f.id,
                "title": f.title,
                "status": f.status.to_string(),
                "priority": f.priority.to_string(),
            })).collect::<Vec<_>>(),
        })
    }
}
