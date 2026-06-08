use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ── Identifiers ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RelationId(pub Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl RelationId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for RelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for RelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── Entity Types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Person,
    Organization,
    PoliticalParty,
    GovernmentAgency,
    Company,
    MediaOutlet,
    Event,
    Location,
    Document,
    Policy,
    FinancialFlow,
    Custom(String),
}

// ── Relation Types ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    MemberOf,
    LeaderOf,
    AffiliatedWith,
    MetWith,
    FundedBy,
    Funded,
    EmployedBy,
    Attended,
    ParticipatedIn,
    LocatedAt,
    OccurredAt,
    PrecededBy,
    SucceededBy,
    InfluencedBy,
    ControlledBy,
    ReportedBy,
    MentionedIn,
    CoOccurredWith,
    OpposedTo,
    AlliedWith,
    SanctionedBy,
    InvestigatedBy,
    RelatedTo,
    Custom(String),
}

// ── Property Value ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    DateTime(DateTime<Utc>),
    List(Vec<PropertyValue>),
    Map(HashMap<String, PropertyValue>),
}

// ── Temporal Range ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalRange {
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
}

impl TemporalRange {
    pub fn unbounded() -> Self {
        Self {
            valid_from: None,
            valid_to: None,
        }
    }

    pub fn from(start: DateTime<Utc>) -> Self {
        Self {
            valid_from: Some(start),
            valid_to: None,
        }
    }

    pub fn between(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            valid_from: Some(start),
            valid_to: Some(end),
        }
    }

    /// Returns true if the given point in time falls within this range.
    pub fn contains(&self, t: DateTime<Utc>) -> bool {
        let after_start = self.valid_from.map_or(true, |s| t >= s);
        let before_end = self.valid_to.map_or(true, |e| t <= e);
        after_start && before_end
    }

    /// Returns true if this range overlaps with another.
    pub fn overlaps(&self, other: &TemporalRange) -> bool {
        let start_ok = match (self.valid_from, other.valid_to) {
            (Some(s), Some(e)) => s <= e,
            _ => true,
        };
        let end_ok = match (self.valid_to, other.valid_from) {
            (Some(e), Some(s)) => e >= s,
            _ => true,
        };
        start_ok && end_ok
    }
}

// ── Source Reference ─────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceRef {
    pub ref_id: String,
    pub source_type: String,
    pub url: Option<String>,
    pub title: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

// ── Entity ───────────────────────────────────────────────────────────

/// A node in the knowledge graph.
///
/// Bitemporal model (see design-decisions.md#D1):
/// - `temporal` (`valid_from` / `valid_to`): when the fact is true in the real world.
/// - `tx_from` / `tx_to`: when the system knew the fact. `tx_to = None` marks the
///   current row; older rows are frozen with `tx_to = <freeze time>` and preserved
///   in the engine's history map keyed by `(id, version)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub entity_type: EntityType,
    pub name: String,
    pub aliases: Vec<String>,
    pub properties: HashMap<String, PropertyValue>,
    pub temporal: TemporalRange,
    pub source_refs: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u64,
    /// Transaction-time start — when the system learned of this row.
    /// Defaults to `Utc::now()` on `Entity::new` and at row-freeze replacement time.
    /// Older payloads without this field (`#[serde(default)]`) get `Utc::now()` —
    /// recovery fixes this up for snapshots by stamping `tx_from = snapshot.created_at`.
    #[serde(default = "Utc::now")]
    pub tx_from: DateTime<Utc>,
    /// Transaction-time end. `None` means this row is the current version.
    /// When an entity is updated or deleted, the previous row's `tx_to` is set to
    /// the freeze timestamp and the row is preserved in `entities_history`.
    #[serde(default)]
    pub tx_to: Option<DateTime<Utc>>,
}

impl Entity {
    pub fn new(entity_type: EntityType, name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            entity_type,
            name: name.into(),
            aliases: Vec::new(),
            properties: HashMap::new(),
            temporal: TemporalRange::unbounded(),
            source_refs: Vec::new(),
            created_at: now,
            updated_at: now,
            version: 0,
            tx_from: now,
            tx_to: None,
        }
    }

    pub fn with_temporal(mut self, temporal: TemporalRange) -> Self {
        self.temporal = temporal;
        self
    }

    pub fn with_property(mut self, key: impl Into<String>, value: PropertyValue) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    pub fn with_source_ref(mut self, ref_id: impl Into<String>) -> Self {
        self.source_refs.push(ref_id.into());
        self
    }
}

// ── Relation ─────────────────────────────────────────────────────────

/// An edge in the knowledge graph.
///
/// Bitemporal model — see the corresponding doc comment on [`Entity`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: RelationId,
    pub relation_type: RelationType,
    pub source: EntityId,
    pub target: EntityId,
    pub confidence: f64,
    pub properties: HashMap<String, PropertyValue>,
    pub temporal: TemporalRange,
    pub source_refs: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub version: u64,
    /// Transaction-time start. See [`Entity::tx_from`].
    #[serde(default = "Utc::now")]
    pub tx_from: DateTime<Utc>,
    /// Transaction-time end. See [`Entity::tx_to`].
    #[serde(default)]
    pub tx_to: Option<DateTime<Utc>>,
}

impl Relation {
    pub fn new(relation_type: RelationType, source: EntityId, target: EntityId) -> Self {
        let now = Utc::now();
        Self {
            id: RelationId::new(),
            relation_type,
            source,
            target,
            confidence: 1.0,
            properties: HashMap::new(),
            temporal: TemporalRange::unbounded(),
            source_refs: Vec::new(),
            created_at: now,
            version: 0,
            tx_from: now,
            tx_to: None,
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_temporal(mut self, temporal: TemporalRange) -> Self {
        self.temporal = temporal;
        self
    }

    pub fn with_property(mut self, key: impl Into<String>, value: PropertyValue) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    pub fn with_source_ref(mut self, ref_id: impl Into<String>) -> Self {
        self.source_refs.push(ref_id.into());
        self
    }
}

// ── Query helpers ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Outgoing,
    Incoming,
    Both,
}
