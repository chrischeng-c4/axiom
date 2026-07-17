// HANDWRITE-BEGIN gap="sift-embedded-lumen-adapter" tracker="1660" reason="Wrap lumen Engine and RdbSnapshot for fixed-field indexing/search without a second service or durable log."
use std::{any::Any, collections::BTreeMap, sync::RwLock};

use anyhow::{bail, Context, Result};
use lumen::{
    storage::{Engine, SnapshotV1},
    types::{
        Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem,
        IndexRequest, MatchOp, MatchQuery, QueryNode, RangeBound, RangeQuery, SearchRequest,
        TermQuery,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::StoredEvent;

use super::{
    model::ProjectionDescriptor,
    runtime::{Projection, PROJECTION_EVENT_INDEX},
};

const COLLECTION: &str = "sift_operational_events_v1";
const KEYWORD_FIELDS: &[&str] = &[
    "project",
    "environment",
    "signal",
    "severity",
    "trace_id",
    "session_id",
    "occurred_at",
];

pub struct EmbeddedLumenProjection {
    engine: Engine,
    documents: RwLock<BTreeMap<String, CanonicalDocument>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CanonicalDocument {
    version: u64,
    fields: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct EmbeddedSnapshot {
    lumen: SnapshotV1,
    documents: BTreeMap<String, CanonicalDocument>,
}

impl EmbeddedLumenProjection {
    pub fn new() -> Result<Self> {
        let engine = Engine::new();
        engine
            .create_collection(
                COLLECTION,
                CreateCollectionRequest {
                    fields: fixed_schema(),
                },
            )
            .context("create embedded Sift Lumen collection")?;
        Ok(Self {
            engine,
            documents: RwLock::new(BTreeMap::new()),
        })
    }

    pub fn search_text(&self, text: &str, limit: u32) -> Result<Vec<String>> {
        self.search(
            QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: text.into(),
                op: MatchOp::And,
            }),
            limit,
        )
    }

    pub fn search_keyword(&self, field: &str, value: &str, limit: u32) -> Result<Vec<String>> {
        if !KEYWORD_FIELDS.contains(&field) {
            bail!("field {field} is not in the embedded projection keyword allowlist");
        }
        self.search(
            QueryNode::Term(TermQuery {
                field: field.into(),
                value: FieldValue::String(value.into()),
            }),
            limit,
        )
    }

    pub fn search_number_range(
        &self,
        field: &str,
        minimum: Option<f64>,
        maximum: Option<f64>,
        limit: u32,
    ) -> Result<Vec<String>> {
        if field != "cursor" {
            bail!("field {field} is not in the embedded projection number allowlist");
        }
        self.search(
            QueryNode::Range(RangeQuery {
                field: field.into(),
                gt: None,
                gte: minimum.map(RangeBound::Number),
                lt: None,
                lte: maximum.map(RangeBound::Number),
            }),
            limit,
        )
    }

    fn search(&self, query: QueryNode, limit: u32) -> Result<Vec<String>> {
        let response = self.engine.search(
            COLLECTION,
            SearchRequest {
                query,
                limit,
                offset: 0,
                cursor: None,
                routing_key: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )?;
        Ok(response
            .hits
            .into_iter()
            .map(|hit| hit.external_id)
            .collect())
    }
}

impl Projection for EmbeddedLumenProjection {
    fn descriptor(&self) -> ProjectionDescriptor {
        ProjectionDescriptor {
            name: PROJECTION_EVENT_INDEX.into(),
            schema_version: 1,
            retention: "raw-journal-retention".into(),
        }
    }

    fn apply_idempotent(&self, stored: &StoredEvent) -> Result<()> {
        let event = &stored.event;
        let mut items = Vec::with_capacity(9);
        let mut push_string = |field: &str, value: &str| {
            if !value.is_empty() {
                items.push(IndexItem {
                    external_id: event.event_id.clone(),
                    field: field.into(),
                    value: FieldValue::String(value.into()),
                    version: Some(stored.cursor),
                });
            }
        };
        push_string("body", &event_body(stored));
        push_string("project", &event.project);
        push_string("environment", &event.environment);
        push_string("signal", &event.signal.to_string());
        push_string("severity", event.severity.as_deref().unwrap_or_default());
        push_string("trace_id", event.trace_id.as_deref().unwrap_or_default());
        push_string(
            "session_id",
            event.session_id.as_deref().unwrap_or_default(),
        );
        push_string("occurred_at", &event.occurred_at);
        items.push(IndexItem {
            external_id: event.event_id.clone(),
            field: "cursor".into(),
            value: FieldValue::Number(stored.cursor as f64),
            version: Some(stored.cursor),
        });
        let fields = items
            .iter()
            .map(|item| Ok((item.field.clone(), serde_json::to_value(&item.value)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        self.engine.index(
            COLLECTION,
            IndexRequest {
                items,
                request_id: Some(format!("sift:{}:{}", event.event_id, stored.cursor)),
            },
        )?;
        let mut documents = self
            .documents
            .write()
            .expect("embedded Lumen document manifest lock poisoned");
        if documents
            .get(&event.event_id)
            .is_none_or(|current| current.version <= stored.cursor)
        {
            documents.insert(
                event.event_id.clone(),
                CanonicalDocument {
                    version: stored.cursor,
                    fields,
                },
            );
        }
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        canonical_snapshot(&EmbeddedSnapshot {
            lumen: self.engine.snapshot()?,
            documents: self
                .documents
                .read()
                .expect("embedded Lumen document manifest lock poisoned")
                .clone(),
        })
    }

    fn restore(&self, state: &[u8]) -> Result<()> {
        let snapshot: EmbeddedSnapshot =
            serde_json::from_slice(state).context("decode embedded Lumen snapshot")?;
        self.engine.restore(snapshot.lumen)?;
        *self
            .documents
            .write()
            .expect("embedded Lumen document manifest lock poisoned") = snapshot.documents;
        Ok(())
    }

    fn semantic_digest(&self) -> Result<String> {
        let documents = self
            .documents
            .read()
            .expect("embedded Lumen document manifest lock poisoned");
        Ok(hex::encode(Sha256::digest(serde_json::to_vec(
            &*documents,
        )?)))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn fixed_schema() -> BTreeMap<String, FieldSpec> {
    let mut fields = BTreeMap::new();
    fields.insert(
        "body".into(),
        field_spec(FieldType::Text, Some(Analyzer::WhitespaceLower)),
    );
    for name in KEYWORD_FIELDS {
        fields.insert((*name).into(), field_spec(FieldType::Keyword, None));
    }
    fields.insert("cursor".into(), field_spec(FieldType::Number, None));
    fields
}

fn field_spec(field_type: FieldType, analyzer: Option<Analyzer>) -> FieldSpec {
    FieldSpec {
        field_type,
        analyzer,
        multi: None,
        dim: None,
        metric: None,
        backend: None,
        quantize: None,
    }
}

fn event_body(stored: &StoredEvent) -> String {
    stored
        .event
        .payload
        .get("body")
        .or_else(|| stored.event.payload.get("message"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| stored.event.payload.to_string())
}

fn canonical_snapshot(snapshot: &EmbeddedSnapshot) -> Result<Vec<u8>> {
    // Snapshot internals contain hash maps. Round-tripping through Value gives
    // object keys serde_json's deterministic map ordering before hashing.
    let value: serde_json::Value = serde_json::from_slice(&serde_json::to_vec(snapshot)?)?;
    serde_json::to_vec(&value).map_err(Into::into)
}

// HANDWRITE-END
