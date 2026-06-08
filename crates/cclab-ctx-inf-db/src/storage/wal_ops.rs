use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{Entity, EntityId, Relation, RelationId};

/// WAL operation type for the graph database.
///
/// Each mutation to the engine is serialized as a GraphOp and appended to the WAL.
/// On recovery, these ops are replayed in order to reconstruct engine state.
///
/// Bitemporal (D1): `UpdateEntity` / `UpdateRelation` / `DeleteEntity` / `DeleteRelation`
/// carry the freeze-timestamp of the row(s) they displace so recovery can reconstruct the
/// exact `tx_to` stamps on history rows. New fields use `#[serde(default)]` for
/// backward-compatible decoding of pre-bitemporal WAL entries (freeze stamp defaults to the
/// unix epoch — such old entries predate history tracking and will reach the current-row
/// state through subsequent CreateEntity / UpdateEntity events in the log).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphOp {
    CreateEntity {
        entity: Entity,
    },
    UpdateEntity {
        id: EntityId,
        /// The new current row (post-update).
        entity: Entity,
        /// Freeze stamp applied to the previous row's `tx_to` at the time of update.
        /// Pre-bitemporal logs default to epoch and are replayed without history insertion.
        #[serde(default = "default_frozen_tx_to")]
        frozen_tx_to: DateTime<Utc>,
    },
    DeleteEntity {
        id: EntityId,
        cascade: bool,
        /// Freeze stamp applied to the deleted row's (and cascaded relations') `tx_to`.
        /// Pre-bitemporal logs default to epoch.
        #[serde(default = "default_frozen_tx_to")]
        tx_to: DateTime<Utc>,
    },
    CreateRelation {
        relation: Relation,
    },
    UpdateRelation {
        id: RelationId,
        relation: Relation,
        #[serde(default = "default_frozen_tx_to")]
        frozen_tx_to: DateTime<Utc>,
    },
    DeleteRelation {
        id: RelationId,
        #[serde(default = "default_frozen_tx_to")]
        tx_to: DateTime<Utc>,
    },
}

/// Default `tx_to` for WAL entries written before the bitemporal extension shipped.
/// We use the unix epoch (rather than `Utc::now()`) so the marker is deterministic and
/// recognizable; recovery treats epoch as "no history row to stamp."
fn default_frozen_tx_to() -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(0, 0).expect("unix epoch is a valid DateTime")
}
