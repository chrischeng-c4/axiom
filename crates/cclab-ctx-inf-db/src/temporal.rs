use chrono::{DateTime, Utc};

use crate::engine::CtxInfEngine;
use crate::types::*;

/// Temporal query operations on the engine.
impl CtxInfEngine {
    /// Get all entities active during the given time range.
    pub fn active_during(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Vec<Entity> {
        let query_range = TemporalRange::between(from, to);
        self.entities
            .iter()
            .filter(|e| e.value().temporal.overlaps(&query_range))
            .map(|e| e.value().clone())
            .collect()
    }

    /// Get all relations active at a given point in time.
    pub fn relations_at(&self, t: DateTime<Utc>) -> Vec<Relation> {
        self.relations
            .iter()
            .filter(|r| r.value().temporal.contains(t))
            .map(|r| r.value().clone())
            .collect()
    }

    /// Build a timeline for the given entities within a date range.
    /// Returns entries sorted by timestamp.
    pub fn timeline(
        &self,
        entity_ids: &[EntityId],
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Vec<TimelineEntry> {
        let mut entries = Vec::new();
        let query_range = TemporalRange {
            valid_from: from,
            valid_to: to,
        };

        // Collect entity temporal events.
        for &eid in entity_ids {
            if let Some(entity) = self.entities.get(&eid) {
                let e = entity.value();
                if !e.temporal.overlaps(&query_range) {
                    continue;
                }

                if let Some(start) = e.temporal.valid_from {
                    if query_range.contains(start) {
                        entries.push(TimelineEntry {
                            timestamp: start,
                            entry_type: TimelineEntryType::EntityStart,
                            entity_id: Some(eid),
                            entity_name: Some(e.name.clone()),
                            entity_type: Some(e.entity_type.clone()),
                            relation_id: None,
                            relation_type: None,
                            counterpart_id: None,
                            counterpart_name: None,
                        });
                    }
                }

                if let Some(end) = e.temporal.valid_to {
                    if query_range.contains(end) {
                        entries.push(TimelineEntry {
                            timestamp: end,
                            entry_type: TimelineEntryType::EntityEnd,
                            entity_id: Some(eid),
                            entity_name: Some(e.name.clone()),
                            entity_type: Some(e.entity_type.clone()),
                            relation_id: None,
                            relation_type: None,
                            counterpart_id: None,
                            counterpart_name: None,
                        });
                    }
                }

                // For Event entities, emit EventOccurred at valid_from.
                if e.entity_type == EntityType::Event {
                    if let Some(t) = e.temporal.valid_from {
                        if query_range.contains(t) {
                            entries.push(TimelineEntry {
                                timestamp: t,
                                entry_type: TimelineEntryType::EventOccurred,
                                entity_id: Some(eid),
                                entity_name: Some(e.name.clone()),
                                entity_type: Some(e.entity_type.clone()),
                                relation_id: None,
                                relation_type: None,
                                counterpart_id: None,
                                counterpart_name: None,
                            });
                        }
                    }
                }
            }
        }

        // Collect relation temporal events between these entities.
        let id_set: std::collections::HashSet<EntityId> = entity_ids.iter().copied().collect();
        for entry in self.relations.iter() {
            let rel = entry.value();
            if !id_set.contains(&rel.source) && !id_set.contains(&rel.target) {
                continue;
            }
            if !rel.temporal.overlaps(&query_range) {
                continue;
            }

            let counterpart_id = if id_set.contains(&rel.source) {
                rel.target
            } else {
                rel.source
            };
            let counterpart_name = self.entities.get(&counterpart_id).map(|e| e.name.clone());

            if let Some(start) = rel.temporal.valid_from {
                if query_range.contains(start) {
                    entries.push(TimelineEntry {
                        timestamp: start,
                        entry_type: TimelineEntryType::RelationStart,
                        entity_id: Some(rel.source),
                        entity_name: self.entities.get(&rel.source).map(|e| e.name.clone()),
                        entity_type: None,
                        relation_id: Some(rel.id),
                        relation_type: Some(rel.relation_type.clone()),
                        counterpart_id: Some(counterpart_id),
                        counterpart_name,
                    });
                }
            }

            if let Some(end) = rel.temporal.valid_to {
                if query_range.contains(end) {
                    let cp_name = self.entities.get(&counterpart_id).map(|e| e.name.clone());
                    entries.push(TimelineEntry {
                        timestamp: end,
                        entry_type: TimelineEntryType::RelationEnd,
                        entity_id: Some(rel.source),
                        entity_name: self.entities.get(&rel.source).map(|e| e.name.clone()),
                        entity_type: None,
                        relation_id: Some(rel.id),
                        relation_type: Some(rel.relation_type.clone()),
                        counterpart_id: Some(counterpart_id),
                        counterpart_name: cp_name,
                    });
                }
            }
        }

        entries.sort_by_key(|e| e.timestamp);
        entries
    }
}

// ── Timeline types ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TimelineEntry {
    pub timestamp: DateTime<Utc>,
    pub entry_type: TimelineEntryType,
    pub entity_id: Option<EntityId>,
    pub entity_name: Option<String>,
    pub entity_type: Option<EntityType>,
    pub relation_id: Option<RelationId>,
    pub relation_type: Option<RelationType>,
    pub counterpart_id: Option<EntityId>,
    pub counterpart_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineEntryType {
    EntityStart,
    EntityEnd,
    RelationStart,
    RelationEnd,
    EventOccurred,
}
