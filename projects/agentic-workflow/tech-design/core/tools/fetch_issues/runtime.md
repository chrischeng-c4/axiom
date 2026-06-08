---
id: projects-sdd-src-tools-fetch-issues-rs-runtime
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue and platform-sync tool TDs expose AW Core workflow state through configured external clients."
---

# sdd fetch issues runtime helpers

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/fetch_issues.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/fetch_issues.rs | function | pub | 29 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/fetch_issues.rs | function | pub | 61 | execute(args: &Value, project_root: &Path) -> Result<String> |
| `list_issues_by_labels` | projects/agentic-workflow/src/tools/fetch_issues.rs | function | pub | 657 | list_issues_by_labels(     labels: &[String],     repo: Option<&str>,     project_root: Option<&Path>, ) -> Result<Vec<u64>> |
## Source
<!-- type: source lang: rust -->

````rust
// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FetchedIssue {
    number: u64,
    title: String,
    body: String,
    labels: Vec<String>,
    state: String,
    dependencies: Vec<u64>,
}

// ---------------------------------------------------------------------------
// GitHub CLI integration
// ---------------------------------------------------------------------------

/// Detect platform type from .aw/config.toml `[platform] type`
fn detect_platform(project_root: &Path) -> PlatformType {
    let config_path = project_root.join(".aw/config.toml");
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        // Simple TOML parsing — look for type = "gitlab" under [platform]
        let mut in_platform = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "[platform]" {
                in_platform = true;
                continue;
            }
            if trimmed.starts_with('[') {
                in_platform = false;
                continue;
            }
            if in_platform && trimmed.starts_with("type") {
                if trimmed.contains("\"gitlab\"") {
                    return PlatformType::GitLab;
                }
            }
        }
    }
    PlatformType::GitHub // Default
}

/// Detect repo from .aw/config.toml `[platform] repo`
fn detect_repo_from_config(project_root: &Path) -> Option<String> {
    let config_path = project_root.join(".aw/config.toml");
    let content = std::fs::read_to_string(&config_path).ok()?;
    let mut in_platform = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[platform]" {
            in_platform = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_platform = false;
            continue;
        }
        if in_platform && trimmed.starts_with("repo") {
            // repo = "owner/repo"
            if let Some(val) = trimmed.split('=').nth(1) {
                let repo = val.trim().trim_matches('"').trim_matches('\'');
                if !repo.is_empty() && repo != "owner/repo" {
                    return Some(repo.to_string());
                }
            }
        }
    }
    None
}

/// Detect repo from issue URL refs (GitHub or GitLab)
fn detect_repo(refs: &[String]) -> Option<String> {
    // GitHub URLs
    let gh_re = Regex::new(r"github\.com/([^/]+/[^/]+)/issues/\d+").ok()?;
    for r in refs {
        if let Some(caps) = gh_re.captures(r) {
            return Some(caps[1].to_string());
        }
    }
    // GitLab URLs: gitlab.com/owner/repo/-/issues/NNN
    let gl_re = Regex::new(r"gitlab\.com/([^/]+/[^/]+)/-/issues/\d+").ok();
    if let Some(re) = gl_re {
        for r in refs {
            if let Some(caps) = re.captures(r) {
                return Some(caps[1].to_string());
            }
        }
    }
    None
}

/// Parse issue number from URL or #NNN.
/// Supports GitHub (`/issues/123`) and GitLab (`/-/issues/123`) URL formats.
fn parse_issue_number(reference: &str) -> Option<u64> {
    let trimmed = reference.trim();

    // URL format: .../issues/123 or .../-/issues/123
    if let Some(num_str) = trimmed.rsplit('/').next() {
        if trimmed.contains("/issues/") {
            return num_str.parse().ok();
        }
    }

    // #NNN format
    if let Some(num_str) = trimmed.strip_prefix('#') {
        return num_str.parse().ok();
    }

    // Plain number
    trimmed.parse().ok()
}

/// Fetch a single issue via gh CLI
fn fetch_issue(number: u64, repo: Option<&str>) -> Result<FetchedIssue> {
    let mut args = vec![
        "issue".to_string(),
        "view".to_string(),
        number.to_string(),
        "--json".to_string(),
        "number,title,body,labels,state".to_string(),
    ];
    if let Some(r) = repo {
        args.push("--repo".to_string());
        args.push(r.to_string());
    }

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = run_gh(&args_ref)?;

    let json: Value = serde_json::from_str(&output)
        .map_err(|e| anyhow::anyhow!("Failed to parse gh output for #{}: {}", number, e))?;

    let title = json["title"].as_str().unwrap_or("(untitled)").to_string();
    let body = json["body"].as_str().unwrap_or("").to_string();
    let state = json["state"].as_str().unwrap_or("OPEN").to_string();
    let labels: Vec<String> = json["labels"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|l| l["name"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let dependencies = extract_dependencies(&body);

    Ok(FetchedIssue {
        number,
        title,
        body,
        labels,
        state,
        dependencies,
    })
}

/// Run a gh CLI command and return stdout
fn run_gh(args: &[&str]) -> Result<String> {
    let output = Command::new("gh").args(args).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh command failed: {}", sanitize_error(&stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Sanitize error output to avoid leaking tokens
fn sanitize_error(output: &str) -> String {
    let token_re = Regex::new(r"gh[opsu]_[A-Za-z0-9_]+").unwrap();
    let sanitized = token_re.replace_all(output, "[REDACTED]");
    let gl_token_re = Regex::new(r"glpat-[A-Za-z0-9_\-]+").unwrap();
    let sanitized = gl_token_re.replace_all(&sanitized, "[REDACTED]");
    let bearer_re = Regex::new(r"Bearer\s+[A-Za-z0-9_\-\.]+").unwrap();
    bearer_re
        .replace_all(&sanitized, "Bearer [REDACTED]")
        .to_string()
}

// ---------------------------------------------------------------------------
// GitLab CLI integration
// ---------------------------------------------------------------------------

/// Fetch a single issue via glab CLI
fn fetch_issue_glab(number: u64, repo: Option<&str>) -> Result<FetchedIssue> {
    let mut args = vec![
        "issue".to_string(),
        "view".to_string(),
        number.to_string(),
        "--output".to_string(),
        "json".to_string(),
    ];
    if let Some(r) = repo {
        args.push("--repo".to_string());
        args.push(r.to_string());
    }

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = run_glab(&args_ref)?;

    let json: Value = serde_json::from_str(&output)
        .map_err(|e| anyhow::anyhow!("Failed to parse glab output for #{}: {}", number, e))?;

    let title = json["title"].as_str().unwrap_or("(untitled)").to_string();
    let body = json["description"].as_str().unwrap_or("").to_string();
    let state = json["state"].as_str().unwrap_or("opened").to_string();
    let labels: Vec<String> = json["labels"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|l| l.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Extract dependencies from body text
    let mut dependencies = extract_dependencies(&body);

    // Also try to fetch issue links via glab API for DAG edges
    if let Ok(link_deps) = fetch_glab_issue_links(number, repo) {
        for dep in link_deps {
            if !dependencies.contains(&dep) {
                dependencies.push(dep);
            }
        }
    }

    Ok(FetchedIssue {
        number,
        title,
        body,
        labels,
        state,
        dependencies,
    })
}

/// Fetch GitLab issue links (blocks/is_blocked_by) via glab api
fn fetch_glab_issue_links(issue_iid: u64, repo: Option<&str>) -> Result<Vec<u64>> {
    // Use glab api to get issue links
    let endpoint = format!("projects/:id/issues/{}/links", issue_iid);
    let mut args = vec!["api", &endpoint];
    let repo_str;
    if let Some(r) = repo {
        repo_str = r.to_string();
        args.push("--repo");
        args.push(&repo_str);
    }

    let output = match run_glab(&args) {
        Ok(o) => o,
        Err(_) => return Ok(vec![]), // Links API is best-effort
    };

    let links: Vec<Value> = serde_json::from_str(&output).unwrap_or_default();
    let mut deps = Vec::new();

    for link in &links {
        let link_type = link["link_type"].as_str().unwrap_or("");
        // "blocks" and "is_blocked_by" indicate dependency edges
        if link_type == "is_blocked_by" {
            if let Some(iid) = link["iid"].as_u64() {
                deps.push(iid);
            }
        }
    }

    Ok(deps)
}

/// Run a glab CLI command and return stdout
fn run_glab(args: &[&str]) -> Result<String> {
    let output = Command::new("glab").args(args).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("glab command failed: {}", sanitize_error(&stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// ---------------------------------------------------------------------------
// Dependency extraction
// ---------------------------------------------------------------------------

/// Extract issue dependencies from body text
fn extract_dependencies(body: &str) -> Vec<u64> {
    let mut deps = HashSet::new();

    // Pattern: "blocked by #NNN" or "blockedBy #NNN" or "depends on #NNN"
    let blocked_re = Regex::new(r"(?i)(?:blocked\s*by|blockedby|depends\s*on)\s+#(\d+)").unwrap();
    for cap in blocked_re.captures_iter(body) {
        if let Ok(num) = cap[1].parse::<u64>() {
            deps.insert(num);
        }
    }

    // Pattern: "- [ ] #NNN" in task lists (common dependency pattern)
    let tasklist_re = Regex::new(r"(?m)^[\s]*-\s*\[[ x]\]\s*#(\d+)").unwrap();
    for cap in tasklist_re.captures_iter(body) {
        if let Ok(num) = cap[1].parse::<u64>() {
            deps.insert(num);
        }
    }

    deps.into_iter().collect()
}

// ---------------------------------------------------------------------------
// DAG operations
// ---------------------------------------------------------------------------

/// Topological sort of fetched issues (Kahn's algorithm)
fn topological_sort(issues: &HashMap<u64, FetchedIssue>) -> Vec<u64> {
    let issue_nums: HashSet<u64> = issues.keys().copied().collect();
    let mut in_degree: HashMap<u64, usize> = HashMap::new();
    let mut dependents: HashMap<u64, Vec<u64>> = HashMap::new();

    for (num, issue) in issues {
        in_degree.entry(*num).or_insert(0);
        for dep in &issue.dependencies {
            if issue_nums.contains(dep) {
                *in_degree.entry(*num).or_insert(0) += 1;
                dependents.entry(*dep).or_default().push(*num);
            }
        }
    }

    // Kahn's with sorted tie-breaking for determinism
    let mut ready: Vec<u64> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();
    ready.sort();

    let mut order = Vec::new();
    while let Some(id) = ready.first().copied() {
        ready.remove(0);
        order.push(id);

        if let Some(deps) = dependents.get(&id) {
            for &dep in deps {
                if let Some(deg) = in_degree.get_mut(&dep) {
                    *deg -= 1;
                    if *deg == 0 {
                        // Insert sorted
                        let pos = ready.binary_search(&dep).unwrap_or_else(|p| p);
                        ready.insert(pos, dep);
                    }
                }
            }
        }
    }

    order
}

/// Update STATE.yaml with dag section
fn update_state_dag(
    change_dir: &Path,
    topo_order: &[u64],
    issues: &HashMap<u64, FetchedIssue>,
) -> Result<()> {
    let mut sm = StateManager::load(change_dir)?;

    // Build a map of dependents (reverse edges) for the DAG
    let mut dependents_map: HashMap<u64, Vec<u64>> = HashMap::new();
    for (num, issue) in issues {
        for dep in &issue.dependencies {
            dependents_map.entry(*dep).or_default().push(*num);
        }
    }

    let dag_issues: Vec<DagIssue> = topo_order
        .iter()
        .map(|num| {
            let deps = issues
                .get(num)
                .map(|i| i.dependencies.clone())
                .unwrap_or_default();
            DagIssue {
                number: *num,
                title: issues.get(num).map(|i| i.title.clone()).unwrap_or_default(),
                depends: deps.clone(),
                dependents: dependents_map.get(num).cloned().unwrap_or_default(),
                processed: false,
                blocked_by: deps,
            }
        })
        .collect();

    sm.state_mut().dag = Some(DagState {
        issues: dag_issues,
        current_index: 0,
        complete: false,
    });

    sm.save()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Markdown formatting
// ---------------------------------------------------------------------------

/// Slugify a title for use in filenames.
/// Converts to lowercase, replaces non-alphanumeric chars with hyphens,
/// collapses multiple hyphens, and truncates to 50 chars.
fn slugify(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    slug.split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .take(50)
        .collect()
}

/// Format a fetched issue as markdown
fn format_issue_md(issue: &FetchedIssue) -> String {
    let state = issue.state.to_lowercase();

    let labels_str = if issue.labels.is_empty() {
        String::new()
    } else {
        format!("\nlabels: [{}]", issue.labels.join(", "))
    };

    let deps_str = if issue.dependencies.is_empty() {
        String::new()
    } else {
        format!(
            "\ndependencies: [{}]",
            issue
                .dependencies
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    format!(
        "---\nnumber: {}\ntitle: \"{}\"\nstate: {}{}{}\n---\n\n# #{} — {}\n\n{}\n",
        issue.number,
        issue.title.replace('"', "\\\""),
        state,
        labels_str,
        deps_str,
        issue.number,
        issue.title,
        issue.body,
    )
}

// ---------------------------------------------------------------------------
// Label-based listing
// ---------------------------------------------------------------------------

/// List issue numbers matching the given labels.
/// Auto-detects platform (gh/glab) from config. Returns issue numbers sorted ascending.
pub fn list_issues_by_labels(
    labels: &[String],
    repo: Option<&str>,
    project_root: Option<&Path>,
) -> Result<Vec<u64>> {
    let platform = project_root
        .map(|p| detect_platform(p))
        .unwrap_or(PlatformType::GitHub);

    match platform {
        PlatformType::GitHub => list_issues_by_labels_gh(labels, repo),
        PlatformType::GitLab => list_issues_by_labels_glab(labels, repo),
    }
}

fn list_issues_by_labels_gh(labels: &[String], repo: Option<&str>) -> Result<Vec<u64>> {
    let base = vec!["issue", "list", "--json", "number", "--limit", "50"];
    let label_flags: Vec<String> = labels
        .iter()
        .flat_map(|l| vec!["--label".to_string(), l.clone()])
        .collect();
    let label_refs: Vec<&str> = label_flags.iter().map(|s| s.as_str()).collect();
    let mut full_args: Vec<&str> = base;
    full_args.extend(&label_refs);

    let repo_flag;
    if let Some(r) = repo {
        repo_flag = r.to_string();
        full_args.push("--repo");
        full_args.push(&repo_flag);
    }

    let output = run_gh(&full_args)?;
    let parsed: Vec<Value> = serde_json::from_str(&output)
        .map_err(|e| anyhow::anyhow!("Failed to parse gh issue list output: {}", e))?;

    let mut nums: Vec<u64> = parsed.iter().filter_map(|v| v["number"].as_u64()).collect();
    nums.sort();
    Ok(nums)
}

fn list_issues_by_labels_glab(labels: &[String], repo: Option<&str>) -> Result<Vec<u64>> {
    let mut args = vec![
        "issue".to_string(),
        "list".to_string(),
        "--output".to_string(),
        "json".to_string(),
    ];
    for label in labels {
        args.push("--label".to_string());
        args.push(label.clone());
    }
    if let Some(r) = repo {
        args.push("--repo".to_string());
        args.push(r.to_string());
    }

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = run_glab(&args_ref)?;
    let parsed: Vec<Value> = serde_json::from_str(&output)
        .map_err(|e| anyhow::anyhow!("Failed to parse glab issue list output: {}", e))?;

    let mut nums: Vec<u64> = parsed.iter().filter_map(|v| v["iid"].as_u64()).collect();
    nums.sort();
    Ok(nums)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/fetch_issues.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:projects-sdd-src-tools-fetch-issues-rs-runtime>"
    description: "Fetch issues runtime helpers and CLI adapters."
```
