# cclab init

初始化專案的 cclab 結構，根據專案類型自動偵測並建立適當的 specs 結構。

## 概述

```bash
cclab init [OPTIONS] [PATH]
```

| 參數 | 說明 | 預設值 |
|-----|------|-------|
| `PATH` | 專案根目錄 | `.` (當前目錄) |
| `--force` | 覆蓋現有結構 | `false` |
| `--dry-run` | 預覽變更，不實際建立 | `false` |

## 專案類型偵測

### 偵測順序

```
1. Cargo (Rust)     → Cargo.toml
2. uv (Python)      → pyproject.toml + [tool.uv]
3. Nx (Node)        → nx.json
4. pnpm (Node)      → pnpm-workspace.yaml
5. npm/yarn (Node)  → package.json + workspaces
6. Single project   → 無 workspace 設定
```

### 偵測邏輯

| 工具 | 檔案 | Monorepo 條件 |
|-----|------|--------------|
| **Cargo** | `Cargo.toml` | `[workspace]` 且 `members.len() > 1` |
| **uv** | `pyproject.toml` | `[tool.uv.workspace]` 存在 |
| **Nx** | `nx.json` | 檔案存在 |
| **pnpm** | `pnpm-workspace.yaml` | 檔案存在 |
| **npm/yarn** | `package.json` | `workspaces` 欄位存在 |

### Cargo 偵測

```toml
# Cargo.toml
[workspace]
members = [
    "crates/cclab-core",
    "crates/cclab-lens",
    "crates/cclab-cli",
]
```

偵測結果：
- `tool = "cargo"`
- `members = ["cclab-core", "cclab-lens", "cclab-cli"]`

### uv 偵測

```toml
# pyproject.toml
[tool.uv.workspace]
members = ["packages/*"]
```

或使用 `uv.lock` + 多個 `pyproject.toml` 子目錄。

### Nx 偵測

```json
// nx.json
{
  "workspaceLayout": {
    "appsDir": "apps",
    "libsDir": "libs"
  }
}
```

專案成員透過 `project.json` 檔案定義，或使用 `workspaceLayout` 設定的目錄結構。

### pnpm 偵測

```yaml
# pnpm-workspace.yaml
packages:
  - 'packages/*'
  - 'apps/*'
```

### npm/yarn 偵測

```json
{
  "workspaces": [
    "packages/*",
    "apps/*"
  ]
}
```

## 建立結構

### 共同結構（所有專案）

```
cclab/
├── specs/
│   └── README.md
├── knowledge/
│   └── index.md
└── changes/
    └── .gitkeep
```

### Monorepo 結構

偵測到 workspace members 後，為每個 member 建立子目錄：

```
.aw/tech-design/
├── README.md              # 總覽，連結所有 crates
├── crate-map.md           # Crate 依賴關係圖
├── cclab-core/
│   └── README.md
├── cclab-lens/
│   └── README.md
├── cclab-cli/
│   └── README.md
└── ...
```

### Single Project 結構

```
.aw/tech-design/
├── README.md              # 主要 spec 文件
├── architecture.md        # (可選) 架構說明
└── api.md                 # (可選) API 說明
```

## README.md 模板

### Monorepo 總覽 (specs/README.md)

```markdown
# {project_name} Specs

## Crates

| Crate | Description |
|-------|-------------|
{{#each members}}
| [{name}](./{name}/README.md) | TODO: Add description |
{{/each}}

## Quick Links

- [Crate Map](./crate-map.md)
```

### Crate README (specs/{crate}/README.md)

```markdown
# {crate_name}

> **Status**: Planning

## Overview

TODO: Add overview

## Key Areas

- TODO: Add key areas
```

### Single Project (specs/README.md)

```markdown
# {project_name} Spec

> **Status**: Planning

## Overview

TODO: Add overview

## Architecture

TODO: Add architecture description

## Key Components

- TODO: Add components
```

## 輸出範例

### Monorepo 初始化

```bash
$ cclab init

Detected: Cargo workspace with 3 members
Creating cclab structure...

  Created: .aw/tech-design/README.md
  Created: .aw/tech-design/crate-map.md
  Created: .aw/tech-design/cclab-core/README.md
  Created: .aw/tech-design/cclab-lens/README.md
  Created: .aw/tech-design/cclab-cli/README.md
  Created: cclab/knowledge/index.md
  Created: .aw/changes/.gitkeep

Done! Run 'cclab gen plan-change <id> "<desc>"' to start planning.
```

### Single Project 初始化

```bash
$ cclab init

Detected: Single Cargo project
Creating cclab structure...

  Created: .aw/tech-design/README.md
  Created: cclab/knowledge/index.md
  Created: .aw/changes/.gitkeep

Done! Run 'cclab gen plan-change <id> "<desc>"' to start planning.
```

### Dry Run

```bash
$ cclab init --dry-run

Detected: Cargo workspace with 3 members
Would create:

  .aw/tech-design/README.md
  .aw/tech-design/crate-map.md
  .aw/tech-design/cclab-core/README.md
  .aw/tech-design/cclab-lens/README.md
  .aw/tech-design/cclab-cli/README.md
  cclab/knowledge/index.md
  .aw/changes/.gitkeep

Run without --dry-run to create files.
```

## 錯誤處理

| 錯誤 | 訊息 | 解決方案 |
|-----|------|---------|
| 已存在 | `cclab/ already exists. Use --force to overwrite.` | 使用 `--force` 或手動刪除 |
| 無法偵測 | `Could not detect project type. No Cargo.toml, pyproject.toml, or package.json found.` | 確認在正確目錄 |
| 權限不足 | `Permission denied: cannot create cclab/` | 檢查目錄權限 |

## 實作細節

### 偵測函數

```rust
pub enum ProjectType {
    Monorepo {
        tool: WorkspaceTool,
        members: Vec<String>,
    },
    Single {
        tool: Option<ProjectTool>,
    },
}

pub enum WorkspaceTool {
    Cargo,
    Uv,
    Nx,
    Pnpm,
    Npm,
    Yarn,
}

pub fn detect_project_type(path: &Path) -> Result<ProjectType> {
    // 1. Check Cargo.toml
    if let Some(cargo) = parse_cargo_toml(path)? {
        if let Some(workspace) = cargo.workspace {
            if workspace.members.len() > 1 {
                return Ok(ProjectType::Monorepo {
                    tool: WorkspaceTool::Cargo,
                    members: resolve_glob_members(&workspace.members, path)?,
                });
            }
        }
        return Ok(ProjectType::Single { tool: Some(ProjectTool::Cargo) });
    }

    // 2. Check pyproject.toml (uv)
    if let Some(pyproject) = parse_pyproject_toml(path)? {
        if pyproject.tool.uv.workspace.is_some() {
            return Ok(ProjectType::Monorepo {
                tool: WorkspaceTool::Uv,
                members: resolve_uv_members(path)?,
            });
        }
        return Ok(ProjectType::Single { tool: Some(ProjectTool::Python) });
    }

    // 3. Check nx.json (Nx)
    if path.join("nx.json").exists() {
        return Ok(ProjectType::Monorepo {
            tool: WorkspaceTool::Nx,
            members: resolve_nx_projects(path)?,
        });
    }

    // 4. Check pnpm-workspace.yaml
    if path.join("pnpm-workspace.yaml").exists() {
        return Ok(ProjectType::Monorepo {
            tool: WorkspaceTool::Pnpm,
            members: parse_pnpm_workspace(path)?,
        });
    }

    // 5. Check package.json
    if let Some(pkg) = parse_package_json(path)? {
        if let Some(workspaces) = pkg.workspaces {
            return Ok(ProjectType::Monorepo {
                tool: WorkspaceTool::Npm, // or Yarn, check yarn.lock
                members: resolve_glob_members(&workspaces, path)?,
            });
        }
        return Ok(ProjectType::Single { tool: Some(ProjectTool::Node) });
    }

    Ok(ProjectType::Single { tool: None })
}
```

### Glob 解析

Workspace members 常使用 glob 模式：

```rust
fn resolve_glob_members(patterns: &[String], root: &Path) -> Result<Vec<String>> {
    let mut members = Vec::new();

    for pattern in patterns {
        for entry in glob(&root.join(pattern).to_string_lossy())? {
            let path = entry?;
            if path.is_dir() {
                // 取得相對於 root 的名稱
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| anyhow!("Invalid member path"))?;
                members.push(name.to_string());
            }
        }
    }

    members.sort();
    members.dedup();
    Ok(members)
}
```

## 相關命令

| 命令 | 說明 |
|-----|------|
| `cclab init` | 初始化專案結構 |
| `cclab gen plan-change` | 開始規劃工作流 |
| `cclab gen impl-change` | 開始實作工作流 |
| `cclab gen merge-change` | 歸檔完成的變更 |
