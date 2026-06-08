// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/tasks.md#source
// CODEGEN-BEGIN
//! Tasks CLI commands

use crate::services::task_generator;
use crate::services::tasks_service::{self, CreateTasksInput, FileActionData, TaskData};
use crate::Result;
use clap::Subcommand;
use std::env;
use std::path::PathBuf;

// Available subcommands for `score tasks`.
// @spec projects/agentic-workflow/tech-design/surface/tasks.md#schema
#[derive(Subcommand)]
pub enum TasksCommands {
    /// Auto-generate tasks from specs (recommended).
    Generate { change_id: String },
    /// Create tasks file from JSON file (legacy, for manual override).
    Create {
        change_id: String,
        #[arg(long)]
        json_file: PathBuf,
    },
}
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/tasks.md#source
pub fn run(cmd: TasksCommands) -> Result<()> {
    let project_root = crate::find_project_root()?;

    match cmd {
        TasksCommands::Generate { change_id } => {
            // Auto-generate tasks from specs
            let input = task_generator::generate_tasks_from_specs(&change_id, &project_root)?;
            let task_count = input.tasks.len();

            // Create the tasks.md file
            let result = tasks_service::create_tasks(input, &project_root)?;
            println!("{}", result);
            println!("Auto-generated {} tasks from specs", task_count);
        }

        TasksCommands::Create {
            change_id,
            json_file,
        } => {
            // Read and parse JSON file
            let json_content = std::fs::read_to_string(&json_file)?;
            let json: serde_json::Value = serde_json::from_str(&json_content)?;

            // Extract tasks array from JSON
            let tasks: Vec<TaskData> = json
                .get("tasks")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Missing 'tasks' field"))?
                .iter()
                .filter_map(|t| {
                    let file = t.get("file")?;
                    Some(TaskData {
                        layer: t.get("layer")?.as_str()?.to_string(),
                        number: t.get("number")?.as_u64()? as u32,
                        title: t.get("title")?.as_str()?.to_string(),
                        file: FileActionData {
                            path: file.get("path")?.as_str()?.to_string(),
                            action: file.get("action")?.as_str()?.to_string(),
                        },
                        spec_ref: t.get("spec_ref")?.as_str()?.to_string(),
                        description: t.get("description")?.as_str()?.to_string(),
                        depends: t
                            .get("depends")
                            .and_then(|d| d.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default(),
                    })
                })
                .collect();

            // Create input struct
            let input = CreateTasksInput {
                change_id,
                tasks,
                agent: None,         // CLI doesn't pass agent info
                duration_secs: None, // CLI doesn't track duration
            };

            // Create tasks
            let result = tasks_service::create_tasks(input, &project_root)?;
            println!("{}", result);
        }
    }

    Ok(())
}

// CODEGEN-END
