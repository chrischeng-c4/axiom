// HANDWRITE-BEGIN gap="missing-generator:logic:9db6ba53" tracker="pending-tracker" reason="Own the Tauri builder, window-ready marker, test-only shutdown handshake, and clean lifecycle exit."
//! Native Workbench desktop host lifecycle.
//!
//! The host owns one native window and its process lifecycle. Agent processes,
//! PTY state, context renderers, and AW transitions belong to later slices.

use std::io::{BufRead, Write};

use tauri::Manager;

/// @spec apps/workbench/tech-design/logic/deliver-workbench-three-column-shell-and-registered-launch-folde.md#logic
pub mod folder_shell;

/// @spec apps/workbench/tech-design/interfaces/cli/launch-native-claude-code-codex-and-agy-clis-through-a-real-pty.md#logic
pub mod native_agent_pty;

/// Marker emitted only after the configured native window exists.
pub const HOST_READY_MARKER: &str = "WORKBENCH_HOST_READY";

const SMOKE_CONTROL_ENV: &str = "WORKBENCH_SMOKE_CONTROL";
const SMOKE_CONTROL_STDIO: &str = "stdio";

/// Launch the one-window Tauri desktop host.
///
/// @spec apps/workbench/tech-design/interfaces/rest/bootstrap-workbench-product-contract-and-runnable-desktop-applic.md#logic
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(folder_shell::FolderShellStore::default())
        .invoke_handler(tauri::generate_handler![
            folder_shell::load_shell_state,
            folder_shell::choose_launch_folder,
            folder_shell::select_launch_folder,
            folder_shell::selected_launch_path,
        ])
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
