use crate::{
    domain::todo::{normalize_due_date, normalize_title, CreateTodo, OptionalField, Todo, TodoError, UpdateTodo},
    infrastructure::sqlite::TodoRepository,
};

#[derive(Clone)]
pub struct TodoService {
    repository: TodoRepository,
}

impl TodoService {
    pub fn new(repository: TodoRepository) -> Self {
        Self { repository }
    }

    pub fn list(&self) -> Result<Vec<Todo>, TodoError> {
        self.repository.list()
    }

    pub fn create(&self, input: CreateTodo) -> Result<Todo, TodoError> {
        self.repository.create(normalize_title(input.title)?, input.priority, normalize_due_date(input.due_date)?)
    }

    pub fn update(&self, id: i64, input: UpdateTodo) -> Result<Todo, TodoError> {
        let mut todo = self.repository.get(id)?;
        if input.title.is_some() {
            todo.title = normalize_title(input.title)?;
        }
        if let Some(priority) = input.priority {
            todo.priority = priority;
        }
        if let OptionalField::Value(due_date) = input.due_date {
            todo.due_date = normalize_due_date(due_date)?;
        }
        if let Some(completed) = input.completed {
            todo.completed = completed;
        }
        self.repository.update(&todo)
    }

    pub fn delete(&self, id: i64) -> Result<(), TodoError> {
        self.repository.delete(id)
    }
}
