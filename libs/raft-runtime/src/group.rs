//! Which raft group a store, host and registry entry belong to -- and the one
//! id that is a sentinel rather than a name.
//!
//! [`GroupId`] is `#[serde(transparent)]`, so it is a bare string on the wire and
//! in the persisted state file's `group_id` field. Widening it into a struct with
//! fields would change that on-disk format for every existing node.
//!
//! [`LEGACY_GROUP_ID`] is not a group anyone named. It stands for the file layout
//! that existed before groups did, and `RaftStore` branches on it twice:
//!
//! - It picks the filename. The legacy id maps to `raft-<node>.state`; every
//!   other id maps to `raft-<node>-<hex>.state`, where `<hex>` is the id's bytes
//!   hex-encoded -- which is what lets an arbitrary group name be a legal
//!   filename on any filesystem.
//! - It closes both migration directions. `open_group` refuses a named group
//!   while a legacy state file for that node still exists, and
//!   `migrate_legacy_to_group` refuses the legacy id as a *target*. Moving
//!   between the two layouts is therefore always an explicit call, never a side
//!   effect of opening a store.
//!
//! So a caller must never use this string as an ordinary group name: doing so
//! silently aliases that group onto the legacy single-group file.

use serde::{Deserialize, Serialize};

pub const LEGACY_GROUP_ID: &str = "legacy_single_group";

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GroupId(pub String);
