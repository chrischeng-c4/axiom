use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

pub const MAX_TITLE_LENGTH: usize = 160;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Medium
    }
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub fn parse(value: &str) -> Result<Self, TodoError> {
        match value {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            _ => Err(TodoError::validation(
                "priority must be low, medium, or high",
            )),
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
    pub priority: Priority,
    pub due_date: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: Option<String>,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub due_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodo {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub due_date: OptionalField<String>,
    #[serde(default)]
    pub completed: Option<bool>,
}

#[derive(Debug, Default)]
pub enum OptionalField<T> {
    #[default]
    Missing,
    Value(Option<T>),
}

impl<'de, T> Deserialize<'de> for OptionalField<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::Value(Option::<T>::deserialize(deserializer)?))
    }
}

#[derive(Debug)]
pub enum TodoError {
    Validation(String),
    NotFound(i64),
    Storage(String),
}

impl TodoError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }
}

pub fn normalize_title(value: Option<String>) -> Result<String, TodoError> {
    let Some(value) = value else {
        return Err(TodoError::validation("title is required"));
    };
    let title = value.trim();
    if title.is_empty() {
        return Err(TodoError::validation("title cannot be empty"));
    }
    if title.chars().count() > MAX_TITLE_LENGTH {
        return Err(TodoError::validation("title must be at most 160 characters"));
    }
    Ok(title.to_owned())
}

pub fn normalize_due_date(value: Option<String>) -> Result<Option<String>, TodoError> {
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    NaiveDate::parse_from_str(&value, "%Y-%m-%d")
        .map(|date| Some(date.format("%Y-%m-%d").to_string()))
        .map_err(|_| TodoError::validation("due_date must use YYYY-MM-DD"))
}

pub fn now() -> String {
    DateTime::<Utc>::from(std::time::SystemTime::now()).to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::{normalize_due_date, normalize_title, TodoError, MAX_TITLE_LENGTH};

    #[test]
    fn title_is_trimmed_and_bounded() {
        assert_eq!(normalize_title(Some("  Ship it  ".into())).unwrap(), "Ship it");
        assert!(matches!(normalize_title(Some(" ".into())), Err(TodoError::Validation(_))));
        assert!(matches!(
            normalize_title(Some("x".repeat(MAX_TITLE_LENGTH + 1))),
            Err(TodoError::Validation(_))
        ));
    }

    #[test]
    fn due_date_requires_iso_calendar_date() {
        assert_eq!(normalize_due_date(Some("2026-07-23".into())).unwrap(), Some("2026-07-23".into()));
        assert!(matches!(normalize_due_date(Some("23-07-2026".into())), Err(TodoError::Validation(_))));
    }
}
