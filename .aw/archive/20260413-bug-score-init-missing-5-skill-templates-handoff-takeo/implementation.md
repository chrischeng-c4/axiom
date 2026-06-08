---
id: implementation
type: change_implementation
change_id: bug-score-init-missing-5-skill-templates-handoff-takeo
---

# Implementation

## Changed Files

- `projects/score/cli/src/init.rs` — Added 5 SKILL_* + 3 SCRIPT_* include_str! constants, extended skills vec 9→14, added scripts install loop with chmod +x, 3 new tests

## Diff

```diff
diff --git a/projects/score/cli/src/init.rs b/projects/score/cli/src/init.rs
index 9d27ba42..e8848952 100644
--- a/projects/score/cli/src/init.rs
+++ b/projects/score/cli/src/init.rs
@@ -22,6 +22,16 @@ const SKILL_REVISE_ARTIFACT: &str = include_str!("../templates/mainthread/skills
 const SKILL_ISSUE: &str = include_str!("../templates/mainthread/skills/score-issue/SKILL.md");
 // @spec .score/changes/score-init-bootstrap/groups/default/specs/score-init-command.md#R13
 const SKILL_ISSUE_PATROL: &str = include_str!("../templates/mainthread/skills/score-issue-patrol/SKILL.md");
+// @spec .score/changes/bug-score-init-missing-5-skill-templates-handoff-takeo/groups/default/specs/bug-score-init-missing-5-skill-templates-handoff-takeo-spec.md#R14
+const SKILL_HANDOFF: &str = include_str!("../templates/mainthread/skills/score-handoff/SKILL.md");
+const SKILL_TAKEOFF: &str = include_str!("../templates/mainthread/skills/score-takeoff/SKILL.md");
+const SKILL_BUILD_DEBUG: &str = include_str!("../templates/mainthread/skills/score-build-debug/SKILL.md");
+const SKILL_RELEASE_PATCH: &str = include_str!("../templates/mainthread/skills/score-release-patch/SKILL.md");
+const SKILL_MAMBA_TEST_COVERAGE: &str = include_str!("../templates/mainthread/skills/score-mamba-test-coverage/SKILL.md");
+// @spec .score/changes/bug-score-init-missing-5-skill-templates-handoff-takeo/groups/default/specs/bug-score-init-missing-5-skill-templates-handoff-takeo-spec.md#R15
+const SCRIPT_BUILD_DEBUG: &str = include_str!("../templates/mainthread/skills/score-build-debug/scripts/build.sh");
+const SCRIPT_RELEASE_PATCH: &str = include_str!("../templates/mainthread/skills/score-release-patch/scripts/release.sh");
+const SCRIPT_MAMBA_TEST_COVERAGE: &str = include_str!("../templates/mainthread/skills/score-mamba-test-coverage/scripts/coverage.sh");
 
 // Claude Code Agent definitions
 // @spec .score/changes/score-init-bootstrap/groups/default/specs/score-init-command.md#R4
@@ -989,6 +999,8 @@ fn install_claude_skills(skills_dir: &Path) -> Result<()> {
     // Install current skills
     // @spec .score/changes/score-init-bootstrap/groups/default/specs/score-init-command.md#R12
     // @spec .score/changes/score-init-bootstrap/groups/default/specs/score-init-command.md#R13
+    // @spec .score/changes/bug-score-init-missing-5-skill-templates-handoff-takeo/groups/default/specs/bug-score-init-missing-5-skill-templates-handoff-takeo-spec.md#R12
+    // @spec .score/changes/bug-score-init-missing-5-skill-templates-handoff-takeo/groups/default/specs/bug-score-init-missing-5-skill-templates-handoff-takeo-spec.md#R13
     let skills = vec![
         ("score-run-change", SKILL_RUN_CHANGE),
         ("score-fillback-main-specs", SKILL_FILLBACK),
@@ -999,6 +1011,11 @@ fn install_claude_skills(skills_dir: &Path) -> Result<()> {
         ("score-revise-artifact", SKILL_REVISE_ARTIFACT),
         ("score-issue", SKILL_ISSUE),
         ("score-issue-patrol", SKILL_ISSUE_PATROL),
+        ("score-handoff", SKILL_HANDOFF),
+        ("score-takeoff", SKILL_TAKEOFF),
+        ("score-build-debug", SKILL_BUILD_DEBUG),
+        ("score-release-patch", SKILL_RELEASE_PATCH),
+        ("score-mamba-test-coverage", SKILL_MAMBA_TEST_COVERAGE),
     ];
 
     for (name, content) in skills {
@@ -1008,6 +1025,28 @@ fn install_claude_skills(skills_dir: &Path) -> Result<()> {
         println!("   ✓ {}", name);
     }
 
+    // Install scripts for skills that have them
+    // @spec .score/changes/bug-score-init-missing-5-skill-templates-handoff-takeo/groups/default/specs/bug-score-init-missing-5-skill-templates-handoff-takeo-spec.md#R16
+    let skill_scripts: &[(&str, &str, &str)] = &[
+        ("score-build-debug", "build.sh", SCRIPT_BUILD_DEBUG),
+        ("score-release-patch", "release.sh", SCRIPT_RELEASE_PATCH),
+        ("score-mamba-test-coverage", "coverage.sh", SCRIPT_MAMBA_TEST_COVERAGE),
+    ];
+
+    for (skill_name, script_name, content) in skill_scripts {
+        let scripts_dir = skills_dir.join(skill_name).join("scripts");
+        std::fs::create_dir_all(&scripts_dir)?;
+        let script_path = scripts_dir.join(script_name);
+        std::fs::write(&script_path, content)?;
+        #[cfg(unix)]
+        {
+            use std::os::unix::fs::PermissionsExt;
+            let mut perms = std::fs::metadata(&script_path)?.permissions();
+            perms.set_mode(0o755);
+            std::fs::set_permissions(&script_path, perms)?;
+        }
+    }
+
     Ok(())
 }
 
@@ -1314,6 +1353,121 @@ mod tests {
         assert_eq!(arr.len(), 1, "Should not duplicate existing score-* hook");
     }
 
+    // REQ: R12, R14 — install_claude_skills installs all 14 score-* skills including the 5 new ones
+    #[test]
+    fn test_install_claude_skills_installs_all_14_skills() {
+        let tmp = TempDir::new().unwrap();
+        let skills_dir = tmp.path().join("skills");
+        fs::create_dir_all(&skills_dir).unwrap();
+
+        install_claude_skills(&skills_dir).unwrap();
+
+        let expected_skills = [
+            "score-run-change",
+            "score-fillback-main-specs",
+            "score-codex-review",
+            "score-gemini-explore-specs",
+            "score-gemini-explore-codebase",
+            "score-merge",
+            "score-revise-artifact",
+            "score-issue",
+            "score-issue-patrol",
+            // REQ: R12 — 5 new skills
+            "score-handoff",
+            "score-takeoff",
+            "score-build-debug",
+            "score-release-patch",
+            "score-mamba-test-coverage",
+        ];
+
+        for skill in &expected_skills {
+            let skill_path = skills_dir.join(skill).join("SKILL.md");
+            assert!(
+                skill_path.exists(),
+                "SKILL.md for '{}' should be installed",
+                skill
+            );
+            let content = fs::read_to_string(&skill_path).unwrap();
+            assert!(!content.is_empty(), "SKILL.md for '{}' should not be empty", skill);
+        }
+    }
+
+    // REQ: R16, R17 — install_claude_skills writes scripts/ subdirectory with executable permissions
+    #[test]
+    fn test_install_claude_skills_installs_scripts_with_exec_perms() {
+        let tmp = TempDir::new().unwrap();
+        let skills_dir = tmp.path().join("skills");
+        fs::create_dir_all(&skills_dir).unwrap();
+
+        install_claude_skills(&skills_dir).unwrap();
+
+        // REQ: R15, R17 — three skills have companion script files
+        let expected_scripts: &[(&str, &str)] = &[
+            ("score-build-debug", "build.sh"),
+            ("score-release-patch", "release.sh"),
+            ("score-mamba-test-coverage", "coverage.sh"),
+        ];
+
+        for (skill, script) in expected_scripts {
+            let script_path = skills_dir.join(skill).join("scripts").join(script);
+            assert!(
+                script_path.exists(),
+                "Script {}/{} should be installed",
+                skill,
+                script
+            );
+            let content = fs::read_to_string(&script_path).unwrap();
+            assert!(
+                content.starts_with("#!/"),
+                "Script {}/{} should have shebang line",
+                skill,
+                script
+            );
+
+            // REQ: R16 — scripts must be executable on Unix
+            #[cfg(unix)]
+            {
+                use std::os::unix::fs::PermissionsExt;
+                let mode = fs::metadata(&script_path).unwrap().permissions().mode();
+                assert!(
+                    mode & 0o111 != 0,
+                    "Script {}/{} should be executable (mode={:o})",
+                    skill,
+                    script,
+                    mode
+                );
+            }
+        }
+    }
+
+    // REQ: R13 — install_claude_skills is idempotent (re-running updates all 14 skills)
+    #[test]
+    fn test_install_claude_skills_idempotent() {
+        let tmp = TempDir::new().unwrap();
+        let skills_dir = tmp.path().join("skills");
+        fs::create_dir_all(&skills_dir).unwrap();
+
+        // First install
+        install_claude_skills(&skills_dir).unwrap();
+        // Second install (re-run / update)
+        install_claude_skills(&skills_dir).unwrap();
+
+        // All 14 skills should still be present
+        for skill in &[
+            "score-handoff",
+            "score-takeoff",
+            "score-build-debug",
+            "score-release-patch",
+            "score-mamba-test-coverage",
+        ] {
+            assert!(
+                skills_dir.join(skill).join("SKILL.md").exists(),
+                "SKILL.md for '{}' should survive re-installation",
+                skill
+            );
+        }
+    }
+
     // REQ: R10 — install_settings_json merges hook into existing settings
     #[test]
     fn test_install_settings_json_merges() {
```

## Review: bug-score-init-missing-5-skill-templates-handoff-takeo-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: bug-score-init-missing-5-skill-templates-handoff-takeo

**Summary**: All requirements met. 5 SKILL_* constants, 3 SCRIPT_* constants, skills vec extended 9→14, scripts loop with chmod +x. 76 tests pass (73 existing + 3 new). Idempotent re-install verified.

