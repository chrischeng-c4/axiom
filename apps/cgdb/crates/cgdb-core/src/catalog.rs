// HANDWRITE-BEGIN gap="missing-generator:hand-written:b6564d3c" tracker="2087" reason="Catalog TOML schema + load/save with atomic rename."
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Catalog {
    pub version: u32,
    #[serde(default)]
    pub projects: Vec<CatalogProject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogProject {
    pub name: String,
    pub td_path: String,
    pub source_path: String,
    pub registered_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sync: Option<String>,
}

impl Catalog {
    pub fn new() -> Self {
        Self { version: 1, projects: Vec::new() }
    }

    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let s = fs::read_to_string(path)
            .with_context(|| format!("read catalog {}", path.display()))?;
        let c: Catalog = toml::from_str(&s)
            .with_context(|| format!("parse catalog {}", path.display()))?;
        Ok(c)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let body = toml::to_string_pretty(self)?;
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let tmp: PathBuf = path.with_extension("toml.tmp");
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(body.as_bytes())?;
            f.sync_all()?;
        }
        fs::rename(&tmp, path)?;
        Ok(())
    }

    pub fn find(&self, name: &str) -> Option<&CatalogProject> {
        self.projects.iter().find(|p| p.name == name)
    }

    pub fn upsert(&mut self, project: CatalogProject) {
        if let Some(existing) = self.projects.iter_mut().find(|p| p.name == project.name) {
            *existing = project;
        } else {
            self.projects.push(project);
        }
    }

    pub fn remove(&mut self, name: &str) -> Option<CatalogProject> {
        let idx = self.projects.iter().position(|p| p.name == name)?;
        Some(self.projects.remove(idx))
    }
}
// HANDWRITE-END
