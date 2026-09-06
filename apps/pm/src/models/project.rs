use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub root_path: String,
    pub created_at: String,
    pub updated_at: String,
}
