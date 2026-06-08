---
id: implementation
type: change_implementation
change_id: mamba-config-unify
---

# Implementation

## Summary

Fix cclab-cli build: update entry_point field access to use the new entry_point() method after MambaConfig unification.

## Diff

```diff
diff --git a/crates/cclab-cli/src/mamba.rs b/crates/cclab-cli/src/mamba.rs
index 9c095908..625e465d 100644
--- a/crates/cclab-cli/src/mamba.rs
+++ b/crates/cclab-cli/src/mamba.rs
@@ -136,7 +136,7 @@ impl CliModule for MambaCli {
 
                 // Determine source file: explicit arg > mamba.toml entry_point.
                 let file_arg = sub.get_one::<String>("file");
-                let entry_from_config = project_config.as_ref().map(|c| c.entry_point.clone());
+                let entry_from_config = project_config.as_ref().and_then(|c| c.entry_point().map(|s| s.to_string()));
                 let file: String = match (file_arg, entry_from_config) {
                     (Some(f), _) => f.clone(),
                     (None, Some(ep)) => ep,
@@ -225,7 +225,7 @@ impl CliModule for MambaCli {
 
                 // Determine source file: explicit arg > mamba.toml entry_point.
                 let file_arg = sub.get_one::<String>("file");
-                let entry_from_config = project_config.as_ref().map(|c| c.entry_point.clone());
+                let entry_from_config = project_config.as_ref().and_then(|c| c.entry_point().map(|s| s.to_string()));
                 let file: String = match (file_arg, entry_from_config) {
                     (Some(f), _) => f.clone(),
                     (None, Some(ep)) => ep,

```

## Review: mamba-config-unify-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-config-unify

**Summary**: Implementation correctly unifies the dual MambaConfig structs. All config and driver tests pass. No regressions.

