// HANDWRITE-BEGIN gap="missing-generator:logic:a41ead03" tracker="pending-tracker" reason="Own canonical launch-folder identity, selected-id persistence, native folder registration, and future launch-path resolution."
//! Registered launch-folder state and native directory selection.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

const STATE_FILE: &str = "folder-shell.json";
const STATE_VERSION: u32 = 1;

/// A canonical local directory registered as a future agent launch root.
///
/// This is identity and selection state only. It is deliberately not terminal
/// cwd and does not represent a running agent session.
///
/// @spec apps/workbench/tech-design/logic/deliver-workbench-three-column-shell-and-registered-launch-folde.md#logic
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchFolder {
    pub id: String,
    pub name: String,
    pub path: String,
}

/// Durable state owned by the folder shell.
///
/// Only folder identity and the selected id are serialized. Layout state,
/// terminal cwd, process state, and renderer state do not belong here.
///
/// @spec apps/workbench/tech-design/logic/deliver-workbench-three-column-shell-and-registered-launch-folde.md#logic
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellState {
    #[serde(default)]
    pub folders: Vec<LaunchFolder>,
    #[serde(default)]
    pub selected_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedShellState {
    version: u32,
    folders: Vec<LaunchFolder>,
    selected_id: Option<String>,
}

/// In-process synchronization for Tauri commands.
#[derive(Debug, Default)]
pub struct FolderShellStore {
    state: Mutex<ShellState>,
}

impl ShellState {
    /// Register and select one existing local directory.
    pub fn register_path(&mut self, path: &Path) -> Result<LaunchFolder, String> {
        let canonical = path
            .canonicalize()
            .map_err(|error| format!("Cannot open {}: {error}", path.display()))?;
        if !canonical.is_dir() {
            return Err(format!("{} is not a directory", canonical.display()));
        }
        let canonical_text = canonical
            .to_str()
            .ok_or_else(|| "The selected directory path is not valid UTF-8".to_string())?
            .to_string();

        if let Some(existing) = self
            .folders
            .iter()
            .find(|folder| folder.path == canonical_text)
            .cloned()
        {
            self.selected_id = Some(existing.id.clone());
            return Ok(existing);
        }

        let name = canonical
            .file_name()
            .and_then(|value| value.to_str())
            .filter(|value| !value.is_empty())
            .unwrap_or(&canonical_text)
            .to_string();
        let folder = LaunchFolder {
            id: stable_folder_id(&canonical_text),
            name,
            path: canonical_text,
        };
        self.selected_id = Some(folder.id.clone());
        self.folders.push(folder.clone());
        Ok(folder)
    }

    /// Select an already registered folder.
    pub fn select(&mut self, folder_id: &str) -> Result<(), String> {
        if self.folders.iter().any(|folder| folder.id == folder_id) {
            self.selected_id = Some(folder_id.to_string());
            Ok(())
        } else {
            Err(format!("Registered folder {folder_id} was not found"))
        }
    }

    /// Resolve the path that a later agent-launch boundary will receive.
    pub fn selected_launch_path(&self) -> Option<&str> {
        let selected = self.selected_id.as_deref()?;
        self.folders
            .iter()
            .find(|folder| folder.id == selected)
            .map(|folder| folder.path.as_str())
    }

    /// Load state from an explicit path so the persistence contract is testable
    /// without launching a Tauri window.
    pub fn load_from(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let bytes = fs::read(path)
            .map_err(|error| format!("Cannot read folder registry {}: {error}", path.display()))?;
        let persisted: PersistedShellState = serde_json::from_slice(&bytes)
            .map_err(|error| format!("Folder registry {} is invalid: {error}", path.display()))?;
        if persisted.version != STATE_VERSION {
            return Err(format!(
                "Folder registry {} has unsupported version {}",
                path.display(),
                persisted.version
            ));
        }

        let mut state = Self {
            folders: persisted.folders,
            selected_id: persisted.selected_id,
        };
        let mut seen_ids = HashSet::new();
        let mut seen_paths = HashSet::new();
        state.folders.retain(|folder| {
            seen_ids.insert(folder.id.clone()) && seen_paths.insert(folder.path.clone())
        });
        if state
            .selected_id
            .as_ref()
            .is_some_and(|id| !state.folders.iter().any(|folder| &folder.id == id))
        {
            state.selected_id = None;
        }
        Ok(state)
    }

    /// Persist only the durable registry fields.
    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "Cannot create folder registry directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        let persisted = PersistedShellState {
            version: STATE_VERSION,
            folders: self.folders.clone(),
            selected_id: self.selected_id.clone(),
        };
        let bytes = serde_json::to_vec_pretty(&persisted)
            .map_err(|error| format!("Cannot encode folder registry: {error}"))?;
        fs::write(path, bytes)
            .map_err(|error| format!("Cannot persist folder registry {}: {error}", path.display()))
    }
}

fn stable_folder_id(path: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in path.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("folder-{hash:016x}")
}

fn state_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join(STATE_FILE))
        .map_err(|error| format!("Cannot resolve the Workbench configuration directory: {error}"))
}

/// Load the durable folder registry for the WebView.
#[tauri::command]
pub fn load_shell_state(
    app: AppHandle,
    store: State<'_, FolderShellStore>,
) -> Result<ShellState, String> {
    let loaded = ShellState::load_from(&state_path(&app)?)?;
    *store
        .state
        .lock()
        .map_err(|_| "Folder registry lock is poisoned".to_string())? = loaded.clone();
    Ok(loaded)
}

/// Open the native directory picker and persist the chosen folder.
///
/// Cancellation is represented by `Ok(None)` and never changes selection.
#[tauri::command]
pub async fn choose_launch_folder(
    app: AppHandle,
    store: State<'_, FolderShellStore>,
) -> Result<Option<ShellState>, String> {
    let Some(file_path) = app
        .dialog()
        .file()
        .set_title("Add a Workbench launch folder")
        .blocking_pick_folder()
    else {
        return Ok(None);
    };
    let chosen = file_path
        .into_path()
        .map_err(|error| format!("The selected folder cannot be used: {error}"))?;

    let current = store
        .state
        .lock()
        .map_err(|_| "Folder registry lock is poisoned".to_string())?
        .clone();
    let mut next = current;
    next.register_path(&chosen)?;
    next.save_to(&state_path(&app)?)?;
    *store
        .state
        .lock()
        .map_err(|_| "Folder registry lock is poisoned".to_string())? = next.clone();
    Ok(Some(next))
}

/// Persist a registered folder selection.
#[tauri::command]
pub fn select_launch_folder(
    folder_id: String,
    app: AppHandle,
    store: State<'_, FolderShellStore>,
) -> Result<ShellState, String> {
    let current = store
        .state
        .lock()
        .map_err(|_| "Folder registry lock is poisoned".to_string())?
        .clone();
    let mut next = current;
    next.select(&folder_id)?;
    next.save_to(&state_path(&app)?)?;
    *store
        .state
        .lock()
        .map_err(|_| "Folder registry lock is poisoned".to_string())? = next.clone();
    Ok(next)
}

/// Return the selected canonical path for the later native-agent slice.
#[tauri::command]
pub fn selected_launch_path(store: State<'_, FolderShellStore>) -> Result<Option<String>, String> {
    Ok(store
        .state
        .lock()
        .map_err(|_| "Folder registry lock is poisoned".to_string())?
        .selected_launch_path()
        .map(str::to_string))
}
// HANDWRITE-END
