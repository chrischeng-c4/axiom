use crate::error::{CtxInfError, Result};
use crate::types::*;
use chrono::{DateTime, TimeZone, Utc};
use uuid::Uuid;

// ── Constants ────────────────────────────────────────────────────────

pub const PAGE_SIZE: usize = 16384;
pub const PAGE_HEADER_SIZE: usize = 64;
pub const PAGE_BODY_SIZE: usize = PAGE_SIZE - PAGE_HEADER_SIZE;
pub const PAGE_MAGIC: [u8; 4] = *b"CIDB";

pub const NODE_SLOT_SIZE: usize = 128;
pub const EDGE_SLOT_SIZE: usize = 100;
pub const NODES_PER_PAGE: usize = PAGE_BODY_SIZE / NODE_SLOT_SIZE; // 127
pub const EDGES_PER_PAGE: usize = PAGE_BODY_SIZE / EDGE_SLOT_SIZE; // 163

// ── Page Type ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PageType {
    Node = 1,
    Edge = 2,
    Property = 3,
    Index = 4,
    Overflow = 5,
}

impl PageType {
    pub fn from_u8(v: u8) -> Result<Self> {
        match v {
            1 => Ok(Self::Node),
            2 => Ok(Self::Edge),
            3 => Ok(Self::Property),
            4 => Ok(Self::Index),
            5 => Ok(Self::Overflow),
            _ => Err(CtxInfError::Storage(format!("invalid page type: {}", v))),
        }
    }
}

// ── Page Header (64 bytes) ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PageHeader {
    pub magic: [u8; 4],
    pub page_type: PageType,
    pub page_id: u64,
    pub slot_count: u16,
    pub free_offset: u16,
    pub next_page: u64,
    pub checksum: u32,
    pub lsn: u64,
    // 27 bytes reserved (padding to 64)
}

impl PageHeader {
    pub fn new(page_type: PageType, page_id: u64) -> Self {
        Self {
            magic: PAGE_MAGIC,
            page_type,
            page_id,
            slot_count: 0,
            free_offset: 0,
            next_page: 0,
            checksum: 0,
            lsn: 0,
        }
    }

    pub fn to_bytes(&self) -> [u8; PAGE_HEADER_SIZE] {
        let mut buf = [0u8; PAGE_HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4] = self.page_type as u8;
        buf[5..13].copy_from_slice(&self.page_id.to_le_bytes());
        buf[13..15].copy_from_slice(&self.slot_count.to_le_bytes());
        buf[15..17].copy_from_slice(&self.free_offset.to_le_bytes());
        buf[17..25].copy_from_slice(&self.next_page.to_le_bytes());
        buf[25..29].copy_from_slice(&self.checksum.to_le_bytes());
        buf[29..37].copy_from_slice(&self.lsn.to_le_bytes());
        // 37..64 reserved (zeroed)
        buf
    }

    pub fn from_bytes(buf: &[u8; PAGE_HEADER_SIZE]) -> Result<Self> {
        let magic: [u8; 4] = buf[0..4].try_into().unwrap();
        if magic != PAGE_MAGIC {
            return Err(CtxInfError::Storage(format!(
                "invalid page magic: {:?}",
                magic
            )));
        }

        Ok(Self {
            magic,
            page_type: PageType::from_u8(buf[4])?,
            page_id: u64::from_le_bytes(buf[5..13].try_into().unwrap()),
            slot_count: u16::from_le_bytes(buf[13..15].try_into().unwrap()),
            free_offset: u16::from_le_bytes(buf[15..17].try_into().unwrap()),
            next_page: u64::from_le_bytes(buf[17..25].try_into().unwrap()),
            checksum: u32::from_le_bytes(buf[25..29].try_into().unwrap()),
            lsn: u64::from_le_bytes(buf[29..37].try_into().unwrap()),
        })
    }
}

// ── Node Slot (128 bytes) ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NodeSlot {
    pub entity_id: [u8; 16],
    pub entity_type_id: u16,
    pub name_offset: u32,
    pub name_length: u16,
    pub valid_from: i64,
    pub valid_to: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub version: u64,
    pub property_page_id: u64,
    pub adjacency_page_id: u64,
    pub flags: u16,
    // 46 bytes reserved
}

impl NodeSlot {
    pub fn to_bytes(&self) -> [u8; NODE_SLOT_SIZE] {
        let mut buf = [0u8; NODE_SLOT_SIZE];
        buf[0..16].copy_from_slice(&self.entity_id);
        buf[16..18].copy_from_slice(&self.entity_type_id.to_le_bytes());
        buf[18..22].copy_from_slice(&self.name_offset.to_le_bytes());
        buf[22..24].copy_from_slice(&self.name_length.to_le_bytes());
        buf[24..32].copy_from_slice(&self.valid_from.to_le_bytes());
        buf[32..40].copy_from_slice(&self.valid_to.to_le_bytes());
        buf[40..48].copy_from_slice(&self.created_at.to_le_bytes());
        buf[48..56].copy_from_slice(&self.updated_at.to_le_bytes());
        buf[56..64].copy_from_slice(&self.version.to_le_bytes());
        buf[64..72].copy_from_slice(&self.property_page_id.to_le_bytes());
        buf[72..80].copy_from_slice(&self.adjacency_page_id.to_le_bytes());
        buf[80..82].copy_from_slice(&self.flags.to_le_bytes());
        // 82..128 reserved (zeroed)
        buf
    }

    pub fn from_bytes(buf: &[u8; NODE_SLOT_SIZE]) -> Self {
        Self {
            entity_id: buf[0..16].try_into().unwrap(),
            entity_type_id: u16::from_le_bytes(buf[16..18].try_into().unwrap()),
            name_offset: u32::from_le_bytes(buf[18..22].try_into().unwrap()),
            name_length: u16::from_le_bytes(buf[22..24].try_into().unwrap()),
            valid_from: i64::from_le_bytes(buf[24..32].try_into().unwrap()),
            valid_to: i64::from_le_bytes(buf[32..40].try_into().unwrap()),
            created_at: i64::from_le_bytes(buf[40..48].try_into().unwrap()),
            updated_at: i64::from_le_bytes(buf[48..56].try_into().unwrap()),
            version: u64::from_le_bytes(buf[56..64].try_into().unwrap()),
            property_page_id: u64::from_le_bytes(buf[64..72].try_into().unwrap()),
            adjacency_page_id: u64::from_le_bytes(buf[72..80].try_into().unwrap()),
            flags: u16::from_le_bytes(buf[80..82].try_into().unwrap()),
        }
    }

    /// Convert an Entity to a NodeSlot (properties/name go to property pages).
    pub fn from_entity(entity: &Entity) -> Self {
        Self {
            entity_id: *entity.id.0.as_bytes(),
            entity_type_id: entity_type_to_id(&entity.entity_type),
            name_offset: 0, // Filled in by page writer
            name_length: entity.name.len().min(u16::MAX as usize) as u16,
            valid_from: datetime_to_millis(entity.temporal.valid_from),
            valid_to: datetime_to_millis(entity.temporal.valid_to),
            created_at: entity.created_at.timestamp_millis(),
            updated_at: entity.updated_at.timestamp_millis(),
            version: entity.version,
            property_page_id: 0,
            adjacency_page_id: 0,
            flags: 0,
        }
    }

    /// Reconstruct partial Entity from slot (name/properties come from property page).
    pub fn to_entity_partial(&self) -> Entity {
        let uuid = Uuid::from_bytes(self.entity_id);
        let now = Utc::now();
        Entity {
            id: EntityId(uuid),
            entity_type: entity_type_from_id(self.entity_type_id),
            name: String::new(), // Filled from property page
            aliases: Vec::new(),
            properties: std::collections::HashMap::new(),
            temporal: TemporalRange {
                valid_from: millis_to_datetime(self.valid_from),
                valid_to: millis_to_datetime(self.valid_to),
            },
            source_refs: Vec::new(),
            created_at: Utc
                .timestamp_millis_opt(self.created_at)
                .single()
                .unwrap_or(now),
            updated_at: Utc
                .timestamp_millis_opt(self.updated_at)
                .single()
                .unwrap_or(now),
            version: self.version,
            // Bitemporal fields are not yet persisted in the page format (Phase 2.5 TODO).
            // Default to "current row at reconstruction time".
            tx_from: now,
            tx_to: None,
        }
    }
}

// ── Edge Slot (100 bytes) ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EdgeSlot {
    pub relation_id: [u8; 16],
    pub relation_type_id: u16,
    pub source_id: [u8; 16],
    pub target_id: [u8; 16],
    pub confidence: f64,
    pub valid_from: i64,
    pub valid_to: i64,
    pub created_at: i64,
    pub version: u64,
    pub property_page_id: u64,
    pub flags: u16,
}

impl EdgeSlot {
    pub fn to_bytes(&self) -> [u8; EDGE_SLOT_SIZE] {
        let mut buf = [0u8; EDGE_SLOT_SIZE];
        buf[0..16].copy_from_slice(&self.relation_id);
        buf[16..18].copy_from_slice(&self.relation_type_id.to_le_bytes());
        buf[18..34].copy_from_slice(&self.source_id);
        buf[34..50].copy_from_slice(&self.target_id);
        buf[50..58].copy_from_slice(&self.confidence.to_le_bytes());
        buf[58..66].copy_from_slice(&self.valid_from.to_le_bytes());
        buf[66..74].copy_from_slice(&self.valid_to.to_le_bytes());
        buf[74..82].copy_from_slice(&self.created_at.to_le_bytes());
        buf[82..90].copy_from_slice(&self.version.to_le_bytes());
        buf[90..98].copy_from_slice(&self.property_page_id.to_le_bytes());
        buf[98..100].copy_from_slice(&self.flags.to_le_bytes());
        buf
    }

    pub fn from_bytes(buf: &[u8; EDGE_SLOT_SIZE]) -> Self {
        Self {
            relation_id: buf[0..16].try_into().unwrap(),
            relation_type_id: u16::from_le_bytes(buf[16..18].try_into().unwrap()),
            source_id: buf[18..34].try_into().unwrap(),
            target_id: buf[34..50].try_into().unwrap(),
            confidence: f64::from_le_bytes(buf[50..58].try_into().unwrap()),
            valid_from: i64::from_le_bytes(buf[58..66].try_into().unwrap()),
            valid_to: i64::from_le_bytes(buf[66..74].try_into().unwrap()),
            created_at: i64::from_le_bytes(buf[74..82].try_into().unwrap()),
            version: u64::from_le_bytes(buf[82..90].try_into().unwrap()),
            property_page_id: u64::from_le_bytes(buf[90..98].try_into().unwrap()),
            flags: u16::from_le_bytes(buf[98..100].try_into().unwrap()),
        }
    }

    pub fn from_relation(rel: &Relation) -> Self {
        Self {
            relation_id: *rel.id.0.as_bytes(),
            relation_type_id: relation_type_to_id(&rel.relation_type),
            source_id: *rel.source.0.as_bytes(),
            target_id: *rel.target.0.as_bytes(),
            confidence: rel.confidence,
            valid_from: datetime_to_millis(rel.temporal.valid_from),
            valid_to: datetime_to_millis(rel.temporal.valid_to),
            created_at: rel.created_at.timestamp_millis(),
            version: rel.version,
            property_page_id: 0,
            flags: 0,
        }
    }

    pub fn to_relation_partial(&self) -> Relation {
        let now = Utc::now();
        Relation {
            id: RelationId(Uuid::from_bytes(self.relation_id)),
            relation_type: relation_type_from_id(self.relation_type_id),
            source: EntityId(Uuid::from_bytes(self.source_id)),
            target: EntityId(Uuid::from_bytes(self.target_id)),
            confidence: self.confidence,
            properties: std::collections::HashMap::new(),
            temporal: TemporalRange {
                valid_from: millis_to_datetime(self.valid_from),
                valid_to: millis_to_datetime(self.valid_to),
            },
            source_refs: Vec::new(),
            created_at: Utc
                .timestamp_millis_opt(self.created_at)
                .single()
                .unwrap_or(now),
            version: self.version,
            // Bitemporal fields are not yet persisted in the page format (Phase 2.5 TODO).
            tx_from: now,
            tx_to: None,
        }
    }
}

// ── Type Registry ────────────────────────────────────────────────────

fn entity_type_to_id(t: &EntityType) -> u16 {
    match t {
        EntityType::Person => 1,
        EntityType::Organization => 2,
        EntityType::PoliticalParty => 3,
        EntityType::GovernmentAgency => 4,
        EntityType::Company => 5,
        EntityType::MediaOutlet => 6,
        EntityType::Event => 7,
        EntityType::Location => 8,
        EntityType::Document => 9,
        EntityType::Policy => 10,
        EntityType::FinancialFlow => 11,
        EntityType::Custom(_) => 0,
    }
}

fn entity_type_from_id(id: u16) -> EntityType {
    match id {
        1 => EntityType::Person,
        2 => EntityType::Organization,
        3 => EntityType::PoliticalParty,
        4 => EntityType::GovernmentAgency,
        5 => EntityType::Company,
        6 => EntityType::MediaOutlet,
        7 => EntityType::Event,
        8 => EntityType::Location,
        9 => EntityType::Document,
        10 => EntityType::Policy,
        11 => EntityType::FinancialFlow,
        _ => EntityType::Custom("unknown".into()),
    }
}

fn relation_type_to_id(t: &RelationType) -> u16 {
    match t {
        RelationType::MemberOf => 1,
        RelationType::LeaderOf => 2,
        RelationType::AffiliatedWith => 3,
        RelationType::MetWith => 4,
        RelationType::FundedBy => 5,
        RelationType::Funded => 6,
        RelationType::EmployedBy => 7,
        RelationType::Attended => 8,
        RelationType::ParticipatedIn => 9,
        RelationType::LocatedAt => 10,
        RelationType::OccurredAt => 11,
        RelationType::PrecededBy => 12,
        RelationType::SucceededBy => 13,
        RelationType::InfluencedBy => 14,
        RelationType::ControlledBy => 15,
        RelationType::ReportedBy => 16,
        RelationType::MentionedIn => 17,
        RelationType::CoOccurredWith => 18,
        RelationType::OpposedTo => 19,
        RelationType::AlliedWith => 20,
        RelationType::SanctionedBy => 21,
        RelationType::InvestigatedBy => 22,
        RelationType::RelatedTo => 23,
        RelationType::Custom(_) => 0,
    }
}

fn relation_type_from_id(id: u16) -> RelationType {
    match id {
        1 => RelationType::MemberOf,
        2 => RelationType::LeaderOf,
        3 => RelationType::AffiliatedWith,
        4 => RelationType::MetWith,
        5 => RelationType::FundedBy,
        6 => RelationType::Funded,
        7 => RelationType::EmployedBy,
        8 => RelationType::Attended,
        9 => RelationType::ParticipatedIn,
        10 => RelationType::LocatedAt,
        11 => RelationType::OccurredAt,
        12 => RelationType::PrecededBy,
        13 => RelationType::SucceededBy,
        14 => RelationType::InfluencedBy,
        15 => RelationType::ControlledBy,
        16 => RelationType::ReportedBy,
        17 => RelationType::MentionedIn,
        18 => RelationType::CoOccurredWith,
        19 => RelationType::OpposedTo,
        20 => RelationType::AlliedWith,
        21 => RelationType::SanctionedBy,
        22 => RelationType::InvestigatedBy,
        23 => RelationType::RelatedTo,
        _ => RelationType::Custom("unknown".into()),
    }
}

// ── Temporal helpers ─────────────────────────────────────────────────

fn datetime_to_millis(dt: Option<DateTime<Utc>>) -> i64 {
    dt.map_or(0, |d| d.timestamp_millis())
}

fn millis_to_datetime(millis: i64) -> Option<DateTime<Utc>> {
    if millis == 0 {
        None
    } else {
        Utc.timestamp_millis_opt(millis).single()
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_header_roundtrip() {
        let mut header = PageHeader::new(PageType::Node, 42);
        header.slot_count = 10;
        header.free_offset = 1280;
        header.next_page = 99;
        header.checksum = 0xDEADBEEF;
        header.lsn = 12345;

        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), PAGE_HEADER_SIZE);

        let decoded = PageHeader::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.page_type, PageType::Node);
        assert_eq!(decoded.page_id, 42);
        assert_eq!(decoded.slot_count, 10);
        assert_eq!(decoded.free_offset, 1280);
        assert_eq!(decoded.next_page, 99);
        assert_eq!(decoded.checksum, 0xDEADBEEF);
        assert_eq!(decoded.lsn, 12345);
    }

    #[test]
    fn test_node_slot_roundtrip() {
        let entity = Entity::new(EntityType::Person, "Test Person").with_temporal(
            TemporalRange::from(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
        );

        let slot = NodeSlot::from_entity(&entity);
        let bytes = slot.to_bytes();
        assert_eq!(bytes.len(), NODE_SLOT_SIZE);

        let decoded = NodeSlot::from_bytes(&bytes);
        assert_eq!(decoded.entity_id, slot.entity_id);
        assert_eq!(decoded.entity_type_id, 1); // Person
        assert_eq!(decoded.version, 0);
        assert_eq!(decoded.valid_from, slot.valid_from);

        let partial = decoded.to_entity_partial();
        assert_eq!(partial.id, entity.id);
        assert_eq!(partial.entity_type, EntityType::Person);
        assert_eq!(partial.version, 0);
    }

    #[test]
    fn test_edge_slot_roundtrip() {
        let src = EntityId::new();
        let dst = EntityId::new();
        let rel = Relation::new(RelationType::MetWith, src, dst).with_confidence(0.85);

        let slot = EdgeSlot::from_relation(&rel);
        let bytes = slot.to_bytes();
        assert_eq!(bytes.len(), EDGE_SLOT_SIZE);

        let decoded = EdgeSlot::from_bytes(&bytes);
        assert_eq!(decoded.relation_type_id, 4); // MetWith
        assert!((decoded.confidence - 0.85).abs() < f64::EPSILON);

        let partial = decoded.to_relation_partial();
        assert_eq!(partial.id, rel.id);
        assert_eq!(partial.source, src);
        assert_eq!(partial.target, dst);
        assert_eq!(partial.relation_type, RelationType::MetWith);
    }

    #[test]
    fn test_page_constants() {
        assert_eq!(PAGE_SIZE, 16384);
        assert_eq!(PAGE_BODY_SIZE, 16320);
        assert_eq!(NODES_PER_PAGE, 127);
        assert_eq!(EDGES_PER_PAGE, 163);
    }
}
