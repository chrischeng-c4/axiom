use serde::{Deserialize, Serialize};

pub const LEGACY_GROUP_ID: &str = "legacy_single_group";

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GroupId(pub String);
