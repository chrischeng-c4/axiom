use std::{path::{Path, PathBuf}, sync::Arc};

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::todo::{now, Priority, Todo, TodoError};

#[derive(Clone)]
pub struct TodoRepository {
    database_path: Arc<PathBuf>,
}

impl TodoRepository {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, TodoError> {
        let database_path = path.into();
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent).map_err(storage_error)?;
        }
        let repository = Self { database_path: Arc::new(database_path) };
        repository.with_connection(|connection| {
            connection.execute_batch(
                "CREATE TABLE IF NOT EXISTS todos (\
                    id INTEGER PRIMARY KEY AUTOINCREMENT,\
                    title TEXT NOT NULL,\
                    completed INTEGER NOT NULL DEFAULT 0,\
                    priority TEXT NOT NULL DEFAULT 'medium',\
                    due_date TEXT,\
                    created_at TEXT NOT NULL\
                )",
            ).map_err(storage_error)
        })?;
        Ok(repository)
    }

    pub fn list(&self) -> Result<Vec<Todo>, TodoError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT id, title, completed, priority, due_date, created_at FROM todos \
                 ORDER BY completed ASC, CASE priority WHEN 'high' THEN 0 WHEN 'medium' THEN 1 ELSE 2 END, created_at DESC",
            ).map_err(storage_error)?;
            let todos = statement
                .query_map([], map_todo)
                .map_err(storage_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(storage_error)?;
            Ok(todos)
        })
    }

    pub fn get(&self, id: i64) -> Result<Todo, TodoError> {
        self.with_connection(|connection| {
            connection.query_row(
                "SELECT id, title, completed, priority, due_date, created_at FROM todos WHERE id = ?1",
                params![id], map_todo,
            ).optional().map_err(storage_error)?.ok_or(TodoError::NotFound(id))
        })
    }

    pub fn create(&self, title: String, priority: Priority, due_date: Option<String>) -> Result<Todo, TodoError> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO todos (title, priority, due_date, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![title, priority.as_str(), due_date, now()],
            ).map_err(storage_error)?;
            let id = connection.last_insert_rowid();
            connection.query_row(
                "SELECT id, title, completed, priority, due_date, created_at FROM todos WHERE id = ?1",
                params![id], map_todo,
            ).map_err(storage_error)
        })
    }

    pub fn update(&self, todo: &Todo) -> Result<Todo, TodoError> {
        self.with_connection(|connection| {
            let changed = connection.execute(
                "UPDATE todos SET title = ?1, completed = ?2, priority = ?3, due_date = ?4 WHERE id = ?5",
                params![todo.title, todo.completed, todo.priority.as_str(), todo.due_date, todo.id],
            ).map_err(storage_error)?;
            if changed == 0 { return Err(TodoError::NotFound(todo.id)); }
            Ok(todo.clone())
        })
    }

    pub fn delete(&self, id: i64) -> Result<(), TodoError> {
        self.with_connection(|connection| {
            if connection.execute("DELETE FROM todos WHERE id = ?1", params![id]).map_err(storage_error)? == 0 {
                return Err(TodoError::NotFound(id));
            }
            Ok(())
        })
    }

    fn with_connection<T>(&self, operation: impl FnOnce(&Connection) -> Result<T, TodoError>) -> Result<T, TodoError> {
        let connection = Connection::open(Path::new(self.database_path.as_ref())).map_err(storage_error)?;
        operation(&connection)
    }
}

fn map_todo(row: &rusqlite::Row<'_>) -> rusqlite::Result<Todo> {
    let priority: String = row.get(3)?;
    Ok(Todo {
        id: row.get(0)?, title: row.get(1)?, completed: row.get::<_, i64>(2)? != 0,
        priority: Priority::parse(&priority).map_err(|error| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(std::io::Error::other(format!("invalid priority: {error:?}")))) )?,
        due_date: row.get(4)?, created_at: row.get(5)?,
    })
}

fn storage_error(error: impl std::fmt::Display) -> TodoError { TodoError::Storage(error.to_string()) }

#[cfg(test)]
mod tests {
    use super::TodoRepository;
    use crate::domain::todo::Priority;

    #[test]
    fn sqlite_round_trip_persists_the_todo_contract() {
        let path = std::env::temp_dir().join(format!("focusflow-test-{}.sqlite3", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let repository = TodoRepository::open(&path).unwrap();
        let created = repository.create("Rust API".into(), Priority::High, Some("2026-07-23".into())).unwrap();
        assert_eq!(repository.list().unwrap(), vec![created]);
        std::fs::remove_file(path).unwrap();
    }
}
