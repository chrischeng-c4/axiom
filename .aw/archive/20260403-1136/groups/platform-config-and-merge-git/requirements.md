---
change: 1136
group: platform-config-and-merge-git
date: 2026-04-03
---

# Requirements

1. Restructure [sdd] config in cclab/config.toml into distinct platform sections: issue_platform (existing), repo_platform (new — git/PR operations), spec_platform (new — local spec storage), docs_platform (future, commented out). 2. Add config parsing for the new platform sections in cclab-sdd config module. 3. After ChangeArchived phase in merge workflow, auto git-commit affected paths (specs/, changes/, archive/) when repo_platform.auto_commit=true. 4. Optionally open PR to repo_platform.default_branch when repo_platform.auto_pr=true. 5. Commit message uses conventional format: chore(sdd): merge {change_id} — {description}.
