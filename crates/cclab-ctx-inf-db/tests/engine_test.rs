use cclab_ctx_inf_db::engine::*;
use cclab_ctx_inf_db::error::CtxInfError;
use cclab_ctx_inf_db::temporal::TimelineEntryType;
use cclab_ctx_inf_db::*;
use chrono::{TimeZone, Utc};

fn setup_political_graph() -> (CtxInfEngine, EntityId, EntityId, EntityId, EntityId) {
    let db = CtxInfEngine::new();

    let person_a = Entity::new(EntityType::Person, "Chen Wei-ting")
        .with_alias("Chen")
        .with_temporal(TemporalRange::from(
            Utc.with_ymd_and_hms(2010, 1, 1, 0, 0, 0).unwrap(),
        ));

    let person_b = Entity::new(EntityType::Person, "Lin Hsiu-mei").with_temporal(
        TemporalRange::from(Utc.with_ymd_and_hms(2012, 6, 1, 0, 0, 0).unwrap()),
    );

    let org = Entity::new(EntityType::Organization, "Cross-Strait Foundation").with_temporal(
        TemporalRange::between(
            Utc.with_ymd_and_hms(2008, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2025, 12, 31, 0, 0, 0).unwrap(),
        ),
    );

    let party = Entity::new(EntityType::PoliticalParty, "Unity Party").with_temporal(
        TemporalRange::from(Utc.with_ymd_and_hms(2005, 3, 15, 0, 0, 0).unwrap()),
    );

    let a = db.create_entity(person_a).unwrap();
    let b = db.create_entity(person_b).unwrap();
    let o = db.create_entity(org).unwrap();
    let p = db.create_entity(party).unwrap();

    (db, a.id, b.id, o.id, p.id)
}

// ── Entity CRUD ──────────────────────────────────────────────────────

#[test]
fn test_create_and_get_entity() {
    let db = CtxInfEngine::new();
    let entity = Entity::new(EntityType::Person, "Test Person");
    let id = entity.id;
    db.create_entity(entity).unwrap();

    let fetched = db.get_entity(id).unwrap();
    assert_eq!(fetched.name, "Test Person");
    assert_eq!(fetched.entity_type, EntityType::Person);
    assert_eq!(fetched.version, 0);
}

#[test]
fn test_entity_not_found() {
    let db = CtxInfEngine::new();
    let result = db.get_entity(EntityId::new());
    assert!(matches!(result, Err(CtxInfError::EntityNotFound(_))));
}

#[test]
fn test_update_entity_with_cas() {
    let db = CtxInfEngine::new();
    let entity = Entity::new(EntityType::Person, "Old Name");
    let id = entity.id;
    db.create_entity(entity).unwrap();

    // Correct version.
    let updated = db
        .update_entity(id, 0, |e| e.name = "New Name".into())
        .unwrap();
    assert_eq!(updated.name, "New Name");
    assert_eq!(updated.version, 1);

    // Stale version.
    let err = db.update_entity(id, 0, |e| e.name = "Stale".into());
    assert!(matches!(err, Err(CtxInfError::VersionConflict { .. })));
}

#[test]
fn test_delete_entity_cascade() {
    let (db, a, b, _, _) = setup_political_graph();

    let rel = Relation::new(RelationType::MetWith, a, b);
    db.create_relation(rel).unwrap();

    assert_eq!(db.stats().relation_count, 1);

    let result = db.delete_entity(a, true).unwrap();
    assert!(result.entity_deleted);
    assert_eq!(result.relations_deleted, 1);
    assert_eq!(db.stats().relation_count, 0);
}

// ── Relation CRUD ────────────────────────────────────────────────────

#[test]
fn test_create_relation() {
    let (db, a, b, _, _) = setup_political_graph();

    let rel = Relation::new(RelationType::MetWith, a, b)
        .with_confidence(0.9)
        .with_temporal(TemporalRange::between(
            Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2023, 1, 2, 0, 0, 0).unwrap(),
        ));

    let created = db.create_relation(rel).unwrap();
    assert_eq!(created.confidence, 0.9);

    let fetched = db.get_relation(created.id).unwrap();
    assert_eq!(fetched.source, a);
    assert_eq!(fetched.target, b);
}

#[test]
fn test_relation_dangling_reference() {
    let db = CtxInfEngine::new();
    let ghost = EntityId::new();
    let entity = Entity::new(EntityType::Person, "Real");
    let id = entity.id;
    db.create_entity(entity).unwrap();

    let rel = Relation::new(RelationType::MetWith, id, ghost);
    let err = db.create_relation(rel);
    assert!(matches!(err, Err(CtxInfError::DanglingReference(_))));
}

// ── Query ────────────────────────────────────────────────────────────

#[test]
fn test_find_entities_by_type() {
    let (db, _, _, _, _) = setup_political_graph();

    let people = db.find_entities(EntityFilter {
        entity_type: Some(EntityType::Person),
        ..Default::default()
    });
    assert_eq!(people.len(), 2);

    let parties = db.entities_by_type(&EntityType::PoliticalParty);
    assert_eq!(parties.len(), 1);
    assert_eq!(parties[0].name, "Unity Party");
}

#[test]
fn test_find_entities_name_contains() {
    let (db, _, _, _, _) = setup_political_graph();

    let results = db.find_entities(EntityFilter {
        name_contains: Some("chen".into()),
        ..Default::default()
    });
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Chen Wei-ting");
}

#[test]
fn test_active_at() {
    let (db, _, _, _, _) = setup_political_graph();

    // In 2007 only Unity Party doesn't exist yet, Cross-Strait Foundation doesn't exist yet.
    let active_2007 = db.active_at(Utc.with_ymd_and_hms(2007, 1, 1, 0, 0, 0).unwrap());
    let names: Vec<_> = active_2007.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"Unity Party"));
    assert!(!names.contains(&"Cross-Strait Foundation"));
}

// ── Neighbors ────────────────────────────────────────────────────────

#[test]
fn test_neighbors() {
    let (db, a, b, org, _) = setup_political_graph();

    db.create_relation(Relation::new(RelationType::MetWith, a, b))
        .unwrap();
    db.create_relation(Relation::new(RelationType::MemberOf, a, org))
        .unwrap();

    let out = db.neighbors(a, Direction::Outgoing, None).unwrap();
    assert_eq!(out.len(), 2);

    let filtered = db
        .neighbors(
            a,
            Direction::Outgoing,
            Some(&NeighborFilter {
                relation_type: Some(RelationType::MetWith),
                ..Default::default()
            }),
        )
        .unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].1.name, "Lin Hsiu-mei");
}

// ── Graph algorithms ─────────────────────────────────────────────────

#[test]
fn test_shortest_path() {
    let (db, a, b, org, party) = setup_political_graph();

    // a → org, org → party, so a→party = 2 hops.
    db.create_relation(Relation::new(RelationType::MemberOf, a, org))
        .unwrap();
    db.create_relation(Relation::new(RelationType::AffiliatedWith, org, party))
        .unwrap();

    let path = db.shortest_path(a, party, 5).unwrap().unwrap();
    assert_eq!(path.hop_count, 2);
    assert_eq!(path.nodes.len(), 3);
    assert_eq!(path.nodes[0], a);
    assert_eq!(path.nodes[2], party);

    // No path from b to party.
    let no_path = db.shortest_path(b, party, 5).unwrap();
    assert!(no_path.is_none());
}

#[test]
fn test_reachable() {
    let (db, a, b, org, party) = setup_political_graph();

    db.create_relation(Relation::new(RelationType::MetWith, a, b))
        .unwrap();
    db.create_relation(Relation::new(RelationType::MemberOf, a, org))
        .unwrap();
    db.create_relation(Relation::new(RelationType::AffiliatedWith, org, party))
        .unwrap();

    let reach = db.reachable(a, 2, Direction::Outgoing).unwrap();
    let ids: Vec<_> = reach.iter().map(|(id, _)| *id).collect();
    assert!(ids.contains(&b));
    assert!(ids.contains(&org));
    assert!(ids.contains(&party));
}

#[test]
fn test_connected_components() {
    let db = CtxInfEngine::new();

    let a = db
        .create_entity(Entity::new(EntityType::Person, "A"))
        .unwrap();
    let b = db
        .create_entity(Entity::new(EntityType::Person, "B"))
        .unwrap();
    let _c = db
        .create_entity(Entity::new(EntityType::Person, "C"))
        .unwrap();

    // A-B connected, C isolated.
    db.create_relation(Relation::new(RelationType::MetWith, a.id, b.id))
        .unwrap();

    let components = db.connected_components();
    assert_eq!(components.len(), 2);
}

#[test]
fn test_degree_centrality() {
    let (db, a, b, org, _) = setup_political_graph();

    db.create_relation(Relation::new(RelationType::MetWith, a, b))
        .unwrap();
    db.create_relation(Relation::new(RelationType::MemberOf, a, org))
        .unwrap();

    let centrality = db.degree_centrality();
    // a has 2 outgoing edges, most connected.
    assert!(centrality[&a] > centrality[&b]);
    assert!(centrality[&a] > centrality[&org]);
}

// ── Timeline ─────────────────────────────────────────────────────────

#[test]
fn test_timeline() {
    let (db, a, b, _org, _) = setup_political_graph();

    let meeting_time = Utc.with_ymd_and_hms(2023, 6, 15, 0, 0, 0).unwrap();
    db.create_relation(Relation::new(RelationType::MetWith, a, b).with_temporal(
        TemporalRange::between(
            meeting_time,
            Utc.with_ymd_and_hms(2023, 6, 16, 0, 0, 0).unwrap(),
        ),
    ))
    .unwrap();

    let timeline = db.timeline(
        &[a, b],
        Some(Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap()),
        Some(Utc.with_ymd_and_hms(2023, 12, 31, 0, 0, 0).unwrap()),
    );

    assert!(!timeline.is_empty());
    assert!(timeline
        .iter()
        .any(|e| e.entry_type == TimelineEntryType::RelationStart));
}

// ── Stats ────────────────────────────────────────────────────────────

#[test]
fn test_stats() {
    let (db, a, b, _, _) = setup_political_graph();

    db.create_relation(Relation::new(RelationType::MetWith, a, b))
        .unwrap();

    let stats = db.stats();
    assert_eq!(stats.entity_count, 4);
    assert_eq!(stats.relation_count, 1);
    assert_eq!(stats.type_counts[&EntityType::Person], 2);
    assert_eq!(stats.type_counts[&EntityType::PoliticalParty], 1);
}
