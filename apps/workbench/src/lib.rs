// HANDWRITE-BEGIN gap="missing-generator:logic:9db6ba53" tracker="pending-tracker" reason="Own the Tauri builder, window-ready marker, test-only shutdown handshake, and clean lifecycle exit."
//! Native Workbench desktop host lifecycle.
//!
//! The host owns one native window and its process lifecycle. Agent processes,
//! PTY state, context renderers, and AW transitions belong to later slices.

use std::io::{BufRead, Write};

use tauri::{Builder, Manager, Runtime};

/// @spec apps/workbench/tech-design/logic/deliver-workbench-three-column-shell-and-registered-launch-folde.md#logic
pub mod folder_shell;

/// @spec apps/workbench/tech-design/logic/synchronize-authoritative-pty-cwd-into-workbench-active-context.md#logic
pub mod cwd_context;

/// @spec apps/workbench/tech-design/interfaces/cli/launch-native-claude-code-codex-and-agy-clis-through-a-real-pty.md#logic
pub mod native_agent_pty;

/// Platform-neutral multi-tab PTY ownership used by native clients.
///
/// @spec apps/workbench/tech-design/interfaces/cli/replace-workbench-tauri-host-with-a-macos-native-swiftui-client.md#logic
pub mod terminal_core;

/// Versioned local sidecar protocol used by the macOS-native client.
///
/// @spec apps/workbench/tech-design/interfaces/cli/replace-workbench-tauri-host-with-a-macos-native-swiftui-client.md#logic
pub mod core_protocol;

/// Provider-neutral, read-only context renderer registry.
///
/// @spec apps/workbench/tech-design/logic/add-workbench-context-renderer-registry-with-markdown-and-git-re.md#logic
pub mod context;

/// Complete desktop session boundary assembled from folder, PTY, cwd, and context slices.
///
/// @spec apps/workbench/tech-design/logic/prove-the-workbench-folder-to-agent-to-artifact-production-journ.md#logic
pub mod production_journey;

/// Read-only local snapshot and diagnostic CLI contract for the native client.
///
/// @spec apps/workbench/tech-design/interfaces/cli/expose-local-snapshot-and-diagnostics-cli.md#contract
pub mod observability_cli;

/// Marker emitted only after the configured native window exists.
pub const HOST_READY_MARKER: &str = "WORKBENCH_HOST_READY";

const SMOKE_CONTROL_ENV: &str = "WORKBENCH_SMOKE_CONTROL";
const SMOKE_CONTROL_STDIO: &str = "stdio";

/// Register the exact production stores and browser-to-Rust IPC commands.
///
/// `run` and the external-contract production-boundary test both use this
/// function, so missing handlers or argument drift fail before release.
pub fn configure_builder<R: Runtime>(
    builder: Builder<R>,
    folder_store: folder_shell::FolderShellStore,
    journey_store: production_journey::ProductionJourneyStore,
) -> Builder<R> {
    builder
        .manage(folder_store)
        .manage(journey_store)
        .invoke_handler(tauri::generate_handler![
            folder_shell::load_shell_state,
            folder_shell::choose_launch_folder,
            folder_shell::select_launch_folder,
            folder_shell::selected_launch_path,
            production_journey::launch_journey_agent,
            production_journey::poll_journey_agent,
            production_journey::send_journey_input,
            production_journey::resize_journey_agent,
            production_journey::interrupt_journey_agent,
            production_journey::terminate_journey_agent,
            production_journey::render_journey_context,
        ])
}

/// Launch the one-window Tauri desktop host.
///
/// @spec apps/workbench/tech-design/interfaces/rest/bootstrap-workbench-product-contract-and-runnable-desktop-applic.md#logic
pub fn run() -> tauri::Result<()> {
    configure_builder(
        tauri::Builder::default().plugin(tauri_plugin_dialog::init()),
        folder_shell::FolderShellStore::default(),
        production_journey::ProductionJourneyStore::default(),
    )
    .setup(|app| {
        if app.get_webview_window("main").is_none() {
            return Err("configured main Workbench window was not created".into());
        }

        if std::env::var(SMOKE_CONTROL_ENV).as_deref() == Ok(SMOKE_CONTROL_STDIO) {
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut command = String::new();
                let read = std::io::stdin().lock().read_line(&mut command);
                if read.is_ok() && command.trim() == "shutdown" {
                    handle.exit(0);
                } else {
                    handle.exit(2);
                }
            });
        }

        println!("{HOST_READY_MARKER}");
        std::io::stdout().flush()?;
        Ok(())
    })
    .run(tauri::generate_context!())
}
// HANDWRITE-END
