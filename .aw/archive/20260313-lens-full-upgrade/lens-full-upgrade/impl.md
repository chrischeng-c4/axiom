# Implementation Diff

## Summary

```
.../skills/cclab-release-patch/scripts/release.sh  |   3 +
 Cargo.lock                                         |  70 ++++++------
 Cargo.toml                                         |   2 +-
 crates/cclab-sdd-cli/src/commands.rs               | 122 +--------------------
 crates/cclab-sdd-cli/src/lib.rs                    |  26 ++---
 crates/cclab-sdd-cli/src/list.rs                   |   2 +-
 crates/cclab-sdd-cli/src/server.rs                 |   6 +-
 crates/cclab-sdd-cli/src/update.rs                 |  10 +-
 crates/cclab-sdd-cli/src/validate_proposal.rs      |   2 +-
 crates/cclab-sdd/src/cli/list.rs                   |   2 +-
 crates/cclab-sdd/src/cli/server.rs                 |   6 +-
 crates/cclab-sdd/src/cli/update.rs                 |  10 +-
 crates/cclab-sdd/src/cli/validate_proposal.rs      |   2 +-
 crates/cclab-sdd/src/models/change.rs              |  90 ++++++++-------
 14 files changed, 123 insertions(+), 230 deletions(-)
```

## Diff

```diff
diff --git a/.claude/skills/cclab-release-patch/scripts/release.sh b/.claude/skills/cclab-release-patch/scripts/release.sh
index 0affd13..728a84e 100755
--- a/.claude/skills/cclab-release-patch/scripts/release.sh
+++ b/.claude/skills/cclab-release-patch/scripts/release.sh
@@ -19,6 +19,9 @@ echo "Bumping version: $CURRENT_VERSION → $NEW_VERSION"
 # Update Cargo.toml
 sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
 
+# Sync Cargo.lock so cargo detects the version change and recompiles
+cargo update -w 2>/dev/null || cargo generate-lockfile
+
 # Build and install
 cargo build -p cclab-cli && rm -f ~/.cargo/bin/cclab && cp target/debug/cclab ~/.cargo/bin/cclab && chmod +x ~/.cargo/bin/cclab
 
diff --git a/Cargo.lock b/Cargo.lock
index 62b7458..3702532 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1158,7 +1158,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-array"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "pyo3",
  "rayon",
@@ -1169,7 +1169,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cli"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "anyhow",
  "cclab-api",
@@ -1202,7 +1202,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cli-registry"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "anyhow",
  "clap",
@@ -1211,7 +1211,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cmd"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "anyhow",
  "pyo3",
@@ -1220,7 +1220,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-core"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "anyhow",
  "bson",
@@ -1238,7 +1238,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-crypto"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "aes-gcm",
  "argon2",
@@ -1265,7 +1265,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-fetch"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1292,7 +1292,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-frame"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1305,7 +1305,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-core"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "bitvec",
  "regex",
@@ -1332,7 +1332,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-formula"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "cclab-grid-core",
  "nom 7.1.3",
@@ -1342,7 +1342,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-history"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1350,7 +1350,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-server"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "anyhow",
  "axum 0.7.9",
@@ -1374,7 +1374,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-wasm"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1392,7 +1392,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1431,7 +1431,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-kv"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "async-trait",
  "bincode",
@@ -1460,7 +1460,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-learn"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1472,7 +1472,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mamba"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "anyhow",
  "base64 0.22.1",
@@ -1501,7 +1501,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-media"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "image",
  "pyo3",
@@ -1512,7 +1512,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mongo"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1534,7 +1534,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-nucleus"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "bson",
  "cclab-agent",
@@ -1565,7 +1565,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-pg"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1595,7 +1595,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-plot"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "pyo3",
  "serde",
@@ -1605,7 +1605,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-prism"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1634,7 +1634,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-qc"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1666,7 +1666,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-queue"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "async-nats",
  "async-trait",
@@ -1707,7 +1707,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-runtime"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1733,7 +1733,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-schema"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "bson",
  "dotenvy",
@@ -1748,7 +1748,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sci"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "cclab-array",
  "cclab-frame",
@@ -1761,7 +1761,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1809,7 +1809,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd-cli"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "anyhow",
  "cclab-cli-registry",
@@ -1835,7 +1835,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd-mcp"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1876,7 +1876,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-server"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "anyhow",
  "async-stream",
@@ -1902,7 +1902,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-text"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "pyo3",
  "rayon",
@@ -1914,14 +1914,14 @@ dependencies = [
 
 [[package]]
 name = "cclab-util"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "pyo3",
 ]
 
 [[package]]
 name = "cclab-vortex"
-version = "0.3.30"
+version = "0.3.33"
 dependencies = [
  "bytemuck",
  "env_logger",
diff --git a/Cargo.toml b/Cargo.toml
index 971c17e..4da702f 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -43,7 +43,7 @@ members = [
 resolver = "2"
 
 [workspace.package]
-version = "0.3.30"
+version = "0.3.33"
 authors = ["Chris Cheng <chris.cheng.c4@gmail.com>"]
 edition = "2021"
 license = "MIT"
diff --git a/crates/cclab-sdd-cli/src/commands.rs b/crates/cclab-sdd-cli/src/commands.rs
index 3d07ecb..a767f07 100644
--- a/crates/cclab-sdd-cli/src/commands.rs
+++ b/crates/cclab-sdd-cli/src/commands.rs
@@ -1,32 +1,15 @@
 use clap::Subcommand;
-use clap_complete::Shell;
 use cclab_sdd::Result;
 
-use crate::clarifications;
-use crate::file;
 use crate::fillback;
-use crate::implementation;
-use crate::knowledge;
 use crate::list;
-use crate::migrate_xml;
 use crate::platform;
-use crate::proposal;
-use crate::server;
-use crate::spec;
 use crate::status;
-use crate::tasks;
-use crate::update;
 use crate::view;
 
 /// SDD CLI commands
 #[derive(Subcommand)]
 pub enum Commands {
-    /// Migrate files to XML format
-    MigrateXml {
-        /// Change ID to migrate (optional, migrates all if not specified)
-        change_id: Option<String>,
-    },
-
     /// Show status of a change
     Status {
         /// Change ID to show status
@@ -37,29 +20,19 @@ pub enum Commands {
         json: bool,
     },
 
-    /// List all changes (for detailed archived view, use 'gen archived')
+    /// List all changes
     List {
         /// Show archived changes
         #[arg(short, long)]
         archived: bool,
     },
 
-    /// Show detailed list of archived changes
-    Archived,
-
-    /// Open Plan Viewer in browser
+    /// Open project viewer in browser
     View {
         /// Change ID (auto-selects if only one exists)
         change: Option<String>,
     },
 
-    /// Update genesis to the latest version
-    Update {
-        /// Check for updates without installing
-        #[arg(short, long)]
-        check: bool,
-    },
-
     /// Bootstrap SDD specs from existing codebase using AST analysis
     Fillback {
         /// Path to source directory to analyze (default: current directory)
@@ -75,44 +48,6 @@ pub enum Commands {
         force: bool,
     },
 
-    /// Generate shell completions
-    Completions {
-        /// Shell type (bash, zsh, fish, powershell)
-        #[arg(value_enum)]
-        shell: Shell,
-    },
-
-    /// Unified server management (dashboard + MCP + viewer)
-    #[command(subcommand)]
-    Server(server::ServerCommands),
-
-    /// Knowledge base operations
-    #[command(subcommand)]
-    Knowledge(knowledge::KnowledgeCommands),
-
-    /// Spec operations
-    #[command(subcommand)]
-    Spec(spec::SpecCommands),
-
-    /// File operations
-    #[command(subcommand)]
-    File(file::FileCommands),
-
-    /// Proposal operations
-    #[command(subcommand)]
-    Proposal(proposal::ProposalCommands),
-
-    /// Tasks operations
-    #[command(subcommand)]
-    Tasks(tasks::TasksCommands),
-
-    /// Implementation workflow commands
-    #[command(subcommand)]
-    Implementation(implementation::ImplementationCommands),
-
-    /// Create clarifications from Q&A
-    Clarifications(clarifications::ClarificationsArgs),
-
     /// Issue platform configuration (GitHub/GitLab/Jira)
     #[command(subcommand)]
     Platform(platform::PlatformCommands),
@@ -185,13 +120,9 @@ pub enum Commands {
     },
 }
 
-/// Run a genesis CLI command
+/// Run an SDD CLI command
 pub async fn run_command(cmd: Commands) -> Result<()> {
     match cmd {
-        Commands::MigrateXml { change_id } => {
-            migrate_xml::run(change_id.as_deref()).await?;
-        }
-
         Commands::Status { change_id, json } => {
             status::run(&change_id, json).await?;
         }
@@ -200,18 +131,10 @@ pub async fn run_command(cmd: Commands) -> Result<()> {
             list::run(archived)?;
         }
 
-        Commands::Archived => {
-            list::run_archived_detailed()?;
-        }
-
         Commands::View { change } => {
             view::run_view(change).await?;
         }
 
-        Commands::Update { check } => {
-            update::run(check).await?;
-        }
-
         Commands::Fillback {
             path,
             module,
@@ -220,44 +143,6 @@ pub async fn run_command(cmd: Commands) -> Result<()> {
             fillback::run(path.as_deref(), module.as_deref(), force).await?;
         }
 
-        Commands::Completions { shell } => {
-            // Note: This needs a command factory from the parent CLI
-            // For now, print a message
-            println!("Shell completions for {:?} - integrate with parent CLI", shell);
-        }
-
-        Commands::Server(cmd) => {
-            server::run(cmd).await?;
-        }
-
-        Commands::Knowledge(cmd) => {
-            knowledge::run(cmd)?;
-        }
-
-        Commands::Spec(cmd) => {
-            spec::run(cmd)?;
-        }
-
-        Commands::File(cmd) => {
-            file::run(cmd)?;
-        }
-
-        Commands::Proposal(cmd) => {
-            proposal::run(cmd)?;
-        }
-
-        Commands::Tasks(cmd) => {
-            tasks::run(cmd)?;
-        }
-
-        Commands::Implementation(cmd) => {
-            implementation::run(cmd)?;
-        }
-
-        Commands::Clarifications(args) => {
-            clarifications::run(args)?;
-        }
-
         Commands::Platform(cmd) => {
             platform::run(cmd)?;
         }
@@ -341,7 +226,6 @@ pub async fn run_command(cmd: Commands) -> Result<()> {
             let project_root = std::env::current_dir()?;
             let mut parsed_args: serde_json::Value =
                 serde_json::from_str(&args).unwrap_or_else(|_| serde_json::json!({}));
-            // Inject project_path if missing
             if parsed_args.get("project_path").is_none() {
                 parsed_args["project_path"] =
                     serde_json::json!(project_root.display().to_string());
diff --git a/crates/cclab-sdd-cli/src/lib.rs b/crates/cclab-sdd-cli/src/lib.rs
index 6a096ad..1b78fc2 100644
--- a/crates/cclab-sdd-cli/src/lib.rs
+++ b/crates/cclab-sdd-cli/src/lib.rs
@@ -3,31 +3,32 @@
 // Extracted from cclab-sdd::cli to allow independent compilation and
 // registration via the cclab-cli-registry distributed-slice mechanism.
 
-pub mod clarifications;
 pub mod commands;
-pub mod file;
 pub mod fillback;
-pub mod implementation;
 pub mod init;
-pub mod knowledge;
 pub mod list;
 pub mod mcp_server;
-pub mod migrate;
-pub mod migrate_xml;
 pub mod platform;
-pub mod proposal;
 pub mod server;
-pub mod spec;
 pub mod status;
-pub mod tasks;
 pub mod update;
-pub mod validate_proposal;
 pub mod view;
 
+// Legacy modules kept for init.rs / update.rs dependencies but not exposed as CLI commands
+pub(crate) mod clarifications;
+pub(crate) mod file;
+pub(crate) mod implementation;
+pub(crate) mod knowledge;
+pub(crate) mod migrate;
+pub(crate) mod migrate_xml;
+pub(crate) mod proposal;
+pub(crate) mod spec;
+pub(crate) mod tasks;
+pub(crate) mod validate_proposal;
+
 // Re-export the Commands enum and run_command from commands module
 pub use commands::{Commands, run_command};
 
-
 use cclab_cli_registry::{CliModule, CLI_MODULES};
 use clap::{ArgMatches, Command, FromArgMatches, Subcommand};
 use linkme::distributed_slice;
@@ -41,8 +42,7 @@ impl CliModule for SddCli {
 
     fn command(&self) -> Command {
         let cmd = Command::new("sdd")
-            .about("SDD - Spec-Driven Development orchestrator")
-            .alias("gen");
+            .about("SDD - Spec-Driven Development orchestrator");
         Commands::augment_subcommands(cmd)
     }
 
diff --git a/crates/cclab-sdd-cli/src/list.rs b/crates/cclab-sdd-cli/src/list.rs
index f5e95b3..1fc4ef3 100644
--- a/crates/cclab-sdd-cli/src/list.rs
+++ b/crates/cclab-sdd-cli/src/list.rs
@@ -12,7 +12,7 @@ pub fn run(archived: bool) -> Result<()> {
     let sdd_dir = project_root.join("sdd");
     let changes_dir = sdd_dir.join("changes");
     if !changes_dir.exists() {
-        println!("{}", "No changes found. Run 'cc gen init' first.".yellow());
+        println!("{}", "No changes found. Use 'cclab sdd run-change --change-id <id> --description \"...\"' to start.".yellow());
         return Ok(());
     }
 
diff --git a/crates/cclab-sdd-cli/src/server.rs b/crates/cclab-sdd-cli/src/server.rs
index b1b4b11..53b71b1 100644
--- a/crates/cclab-sdd-cli/src/server.rs
+++ b/crates/cclab-sdd-cli/src/server.rs
@@ -251,7 +251,7 @@ fn stop_project(project: Option<String>) -> Result<()> {
 /// List all registered projects
 fn list_projects() -> Result<()> {
     let registry = Registry::load().map_err(|_| {
-        anyhow::anyhow!("No server running. Use 'cc gen server start' to start the server.")
+        anyhow::anyhow!("No server running. Use 'cclab server start' to start the server.")
     })?;
 
     println!("\n{}", "Server Status".bold());
@@ -298,12 +298,12 @@ fn list_projects() -> Result<()> {
 /// Open Plan Viewer for a specific change in the default browser (R7)
 fn view_change(project: &str, change: &str) -> Result<()> {
     let registry = Registry::load().map_err(|_| {
-        anyhow::anyhow!("No server running. Use 'cc gen server start' to start the server first.")
+        anyhow::anyhow!("No server running. Use 'cclab server start' to start the server first.")
     })?;
 
     if !registry.is_server_running() {
         anyhow::bail!(
-            "Server is not running. Use 'cc gen server start' to start it first."
+            "Server is not running. Use 'cclab server start' to start it first."
         );
     }
 
diff --git a/crates/cclab-sdd-cli/src/update.rs b/crates/cclab-sdd-cli/src/update.rs
index c2f3190..d67b88f 100644
--- a/crates/cclab-sdd-cli/src/update.rs
+++ b/crates/cclab-sdd-cli/src/update.rs
@@ -34,7 +34,7 @@ pub async fn run(check_only: bool) -> Result<()> {
         println!();
 
         if check_only {
-            println!("{}", "💡 Run 'cc gen update' to install the update.".yellow());
+            println!("{}", "💡 Run 'cclab sdd update' to install the update.".yellow());
             return Ok(());
         }
 
@@ -147,13 +147,13 @@ fn update_binary(version: &str) -> Result<()> {
     println!();
     println!("{}", "✅ Update complete!".green().bold());
     println!();
-    println!("   Run 'cc gen --version' to verify.");
+    println!("   Run 'cclab --version' to verify.");
 
-    // If in an genesis project, suggest upgrading configs
-    if std::path::Path::new("sdd").exists() {
+    // If in an sdd project, suggest upgrading configs
+    if std::path::Path::new("cclab").exists() {
         println!();
         println!("{}", "💡 To upgrade project configs:".yellow());
-        println!("   cc gen init --force");
+        println!("   cclab sdd init --force");
     }
 
     Ok(())
diff --git a/crates/cclab-sdd-cli/src/validate_proposal.rs b/crates/cclab-sdd-cli/src/validate_proposal.rs
index bf6c814..ab13997 100644
--- a/crates/cclab-sdd-cli/src/validate_proposal.rs
+++ b/crates/cclab-sdd-cli/src/validate_proposal.rs
@@ -276,7 +276,7 @@ pub fn validate_proposal(
     let change_dir = project_root.join("cclab/changes").join(change_id);
     if !change_dir.exists() {
         anyhow::bail!(
-            "Change '{}' not found. Run 'cc gen proposal {}' first.",
+            "Change '{}' not found. Run 'cclab sdd run-change --change-id {}' first.",
             change_id,
             change_id
         );
diff --git a/crates/cclab-sdd/src/cli/list.rs b/crates/cclab-sdd/src/cli/list.rs
index d65e496..6960314 100644
--- a/crates/cclab-sdd/src/cli/list.rs
+++ b/crates/cclab-sdd/src/cli/list.rs
@@ -12,7 +12,7 @@ pub fn run(archived: bool) -> Result<()> {
     let sdd_dir = project_root.join("sdd");
     let changes_dir = sdd_dir.join("changes");
     if !changes_dir.exists() {
-        println!("{}", "No changes found. Run 'cc gen init' first.".yellow());
+        println!("{}", "No changes found. Use 'cclab sdd run-change --change-id <id> --description \"...\"' to start.".yellow());
         return Ok(());
     }
 
diff --git a/crates/cclab-sdd/src/cli/server.rs b/crates/cclab-sdd/src/cli/server.rs
index 1f69b71..bb598e8 100644
--- a/crates/cclab-sdd/src/cli/server.rs
+++ b/crates/cclab-sdd/src/cli/server.rs
@@ -251,7 +251,7 @@ fn stop_project(project: Option<String>) -> Result<()> {
 /// List all registered projects
 fn list_projects() -> Result<()> {
     let registry = Registry::load().map_err(|_| {
-        anyhow::anyhow!("No server running. Use 'cc gen server start' to start the server.")
+        anyhow::anyhow!("No server running. Use 'cclab server start' to start the server.")
     })?;
 
     println!("\n{}", "Server Status".bold());
@@ -298,12 +298,12 @@ fn list_projects() -> Result<()> {
 /// Open Plan Viewer for a specific change in the default browser (R7)
 fn view_change(project: &str, change: &str) -> Result<()> {
     let registry = Registry::load().map_err(|_| {
-        anyhow::anyhow!("No server running. Use 'cc gen server start' to start the server first.")
+        anyhow::anyhow!("No server running. Use 'cclab server start' to start the server first.")
     })?;
 
     if !registry.is_server_running() {
         anyhow::bail!(
-            "Server is not running. Use 'cc gen server start' to start it first."
+            "Server is not running. Use 'cclab server start' to start it first."
         );
     }
 
diff --git a/crates/cclab-sdd/src/cli/update.rs b/crates/cclab-sdd/src/cli/update.rs
index 81fc449..8e05a25 100644
--- a/crates/cclab-sdd/src/cli/update.rs
+++ b/crates/cclab-sdd/src/cli/update.rs
@@ -34,7 +34,7 @@ pub async fn run(check_only: bool) -> Result<()> {
         println!();
 
         if check_only {
-            println!("{}", "💡 Run 'cc gen update' to install the update.".yellow());
+            println!("{}", "💡 Run 'cclab sdd update' to install the update.".yellow());
             return Ok(());
         }
 
@@ -147,13 +147,13 @@ fn update_binary(version: &str) -> Result<()> {
     println!();
     println!("{}", "✅ Update complete!".green().bold());
     println!();
-    println!("   Run 'cc gen --version' to verify.");
+    println!("   Run 'cclab --version' to verify.");
 
-    // If in an genesis project, suggest upgrading configs
-    if std::path::Path::new("sdd").exists() {
+    // If in an sdd project, suggest upgrading configs
+    if std::path::Path::new("cclab").exists() {
         println!();
         println!("{}", "💡 To upgrade project configs:".yellow());
-        println!("   cc gen init --force");
+        println!("   cclab sdd init --force");
     }
 
     Ok(())
diff --git a/crates/cclab-sdd/src/cli/validate_proposal.rs b/crates/cclab-sdd/src/cli/validate_proposal.rs
index 076443a..550bf3f 100644
--- a/crates/cclab-sdd/src/cli/validate_proposal.rs
+++ b/crates/cclab-sdd/src/cli/validate_proposal.rs
@@ -276,7 +276,7 @@ pub fn validate_proposal(
     let change_dir = project_root.join("cclab/changes").join(change_id);
     if !change_dir.exists() {
         anyhow::bail!(
-            "Change '{}' not found. Run 'cc gen proposal {}' first.",
+            "Change '{}' not found. Run 'cclab sdd run-change --change-id {}' first.",
             change_id,
             change_id
         );
diff --git a/crates/cclab-sdd/src/models/change.rs b/crates/cclab-sdd/src/models/change.rs
index 4ea0b4a..de77cbe 100644
--- a/crates/cclab-sdd/src/models/change.rs
+++ b/crates/cclab-sdd/src/models/change.rs
@@ -281,13 +281,13 @@ pub struct GeminiModelConfig {
 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct GeminiConfig {
     /// CLI command (default: "gemini")
-    #[serde(default = "default_gemini_command")]
+    #[serde(skip, default = "default_gemini_command")]
     pub command: String,
-    /// Available models
-    #[serde(default = "default_gemini_models")]
+    /// Available models (fixed, not configurable)
+    #[serde(skip, default = "default_gemini_models")]
     pub models: Vec<GeminiModelConfig>,
-    /// Default model ID
-    #[serde(default = "default_gemini_default")]
+    /// Default model ID (fixed, not configurable)
+    #[serde(skip, default = "default_gemini_default")]
     pub default: String,
     /// Path to env file (relative to project root or absolute)
     #[serde(default)]
@@ -333,6 +333,11 @@ impl Default for GeminiConfig {
 }
 
 impl GeminiConfig {
+    /// Returns true if all serializable fields are default (envfile is None)
+    pub fn is_default(&self) -> bool {
+        self.envfile.is_none()
+    }
+
     /// Select model based on complexity
     pub fn select_model(&self, complexity: Complexity) -> &GeminiModelConfig {
         // Find the cheapest model that can handle this complexity
@@ -397,13 +402,13 @@ impl CodexModelConfig {
 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct CodexConfig {
     /// CLI command (default: "codex")
-    #[serde(default = "default_codex_command")]
+    #[serde(skip, default = "default_codex_command")]
     pub command: String,
-    /// Available models
-    #[serde(default = "default_codex_models")]
+    /// Available models (fixed, not configurable)
+    #[serde(skip, default = "default_codex_models")]
     pub models: Vec<CodexModelConfig>,
-    /// Default model ID
-    #[serde(default = "default_codex_default")]
+    /// Default model ID (fixed, not configurable)
+    #[serde(skip, default = "default_codex_default")]
     pub default: String,
     /// Path to env file (relative to project root or absolute)
     #[serde(default)]
@@ -475,6 +480,11 @@ impl Default for CodexConfig {
 }
 
 impl CodexConfig {
+    /// Returns true if all serializable fields are default (envfile is None)
+    pub fn is_default(&self) -> bool {
+        self.envfile.is_none()
+    }
+
     /// Select model based on complexity
     pub fn select_model(&self, complexity: Complexity) -> &CodexModelConfig {
         self.models
@@ -524,13 +534,13 @@ pub struct ClaudeModelConfig {
 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct ClaudeConfig {
     /// CLI command (default: "claude")
-    #[serde(default = "default_claude_command")]
+    #[serde(skip, default = "default_claude_command")]
     pub command: String,
-    /// Available models
-    #[serde(default = "default_claude_models")]
+    /// Available models (fixed, not configurable)
+    #[serde(skip, default = "default_claude_models")]
     pub models: Vec<ClaudeModelConfig>,
-    /// Default model ID
-    #[serde(default = "default_claude_default")]
+    /// Default model ID (fixed, not configurable)
+    #[serde(skip, default = "default_claude_default")]
     pub default: String,
     /// Path to env file (relative to project root or absolute)
     #[serde(default)]
@@ -583,6 +593,11 @@ impl Default for ClaudeConfig {
 }
 
 impl ClaudeConfig {
+    /// Returns true if all serializable fields are default (envfile is None)
+    pub fn is_default(&self) -> bool {
+        self.envfile.is_none()
+    }
+
     /// Select model based on complexity
     pub fn select_model(&self, complexity: Complexity) -> &ClaudeModelConfig {
         self.models
@@ -1230,32 +1245,32 @@ pub struct SddConfig {
     #[serde(default)]
     pub interface: SddInterface,
 
-    /// Project configuration (monorepo-aware modules)
-    #[serde(default)]
+    /// Project configuration (monorepo-aware modules, fixed)
+    #[serde(skip, default)]
     pub project: ProjectConfig,
 
     /// Workflow iteration settings
     #[serde(default)]
     pub workflow: WorkflowConfig,
 
-    /// Gemini configuration
-    #[serde(default)]
+    /// Gemini configuration (only envfile is configurable)
+    #[serde(default, skip_serializing_if = "GeminiConfig::is_default")]
     pub gemini: GeminiConfig,
 
-    /// Codex configuration
-    #[serde(default)]
+    /// Codex configuration (only envfile is configurable)
+    #[serde(default, skip_serializing_if = "CodexConfig::is_default")]
     pub codex: CodexConfig,
 
-    /// Codex Spark configuration
-    #[serde(default = "default_codex_spark_config", rename = "codex-spark")]
+    /// Codex Spark configuration (only envfile is configurable)
+    #[serde(default = "default_codex_spark_config", rename = "codex-spark", skip_serializing_if = "CodexConfig::is_default")]
     pub codex_spark: CodexConfig,
 
-    /// Claude configuration
-    #[serde(default)]
+    /// Claude configuration (only envfile is configurable)
+    #[serde(default, skip_serializing_if = "ClaudeConfig::is_default")]
     pub claude: ClaudeConfig,
 
-    /// Validation rules for spec files
-    #[serde(default)]
+    /// Validation rules for spec files (fixed, not configurable)
+    #[serde(skip, default)]
     pub validation: ValidationRules,
 
     // Legacy fields for backward compatibility (kept for TOML deserialization)
@@ -1592,23 +1607,14 @@ mod config_tests {
     }
 
     #[test]
-    fn test_project_config_toml_roundtrip() {
-        let config = SddConfig {
-            project: ProjectConfig {
-                modules: vec![ProjectModule {
-                    path: "crates/".to_string(),
-                    language: ConfigLanguage::Rust,
-                    framework: None,
-                }],
-            },
-            ..Default::default()
-        };
-
+    fn test_project_config_skipped_in_serialization() {
+        // project is #[serde(skip)] — serialize should not include it,
+        // deserialize always uses default (empty modules)
+        let config = SddConfig::default();
         let toml_str = toml::to_string_pretty(&config).unwrap();
+        assert!(!toml_str.contains("[project]"));
         let parsed: SddConfig = toml::from_str(&toml_str).unwrap();
-        assert_eq!(parsed.project.modules.len(), 1);
-        assert_eq!(parsed.project.modules[0].path, "crates/");
-        assert_eq!(parsed.project.modules[0].language, ConfigLanguage::Rust);
+        assert!(parsed.project.modules.is_empty());
     }
 
     #[test]
```
