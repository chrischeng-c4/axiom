use std::collections::BTreeMap;
use std::path::Path;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::models::{
    Feature, FeatureStatus, Prd, Project, Task, TaskStatus, TechDesign,
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

    pub fn get_project_summary(&self, project_id: &str) -> serde_json::Value {
        let project = self.get_project(project_id);
        let prds: Vec<&Prd> = self.list_prds(project_id);
        let tds: Vec<&TechDesign> = self.list_tech_designs(project_id);
        let features: Vec<&Feature> = self.list_features(project_id, None);
        let tasks: Vec<&Task> = self.list_tasks(project_id, None, None, None);

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

        serde_json::json!({
            "project": project,
            "counts": {
                "prds": prds.len(),
                "tech_designs": tds.len(),
                "features": features.len(),
                "tasks_total": tasks.len(),
                "tasks_by_status": task_counts,
                "features_by_status": feature_counts,
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
