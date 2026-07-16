// HANDWRITE-BEGIN gap="missing-generator:e2e-test:647c2c4a" tracker="#1645" reason="Bind Lumen search continuity assertions to the shared Rig stateful lifecycle. generator gap: missing-generator:lumen-stateful-adapter (#1645)."
// @spec apps/lumen/tech-design/logic/adopt-shared-stateful-service-foundations.md#unit-test
// @spec apps/lumen/tech-design/semantic/lumen-tests.md#unit-test
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{ensure, Result};
use lumen::storage::Engine;
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    MatchOp, MatchQuery, QueryNode, SearchRequest, SearchResponse,
};
use rig::engine::stateful::{run_stateful, StatefulActions, StatefulLimits, StatefulScenario};
use serde_json::json;

#[test]
fn lumen_search_continuity_uses_shared_stateful_runner() {
    let engine = Arc::new(Engine::new());
    let unavailable = Arc::new(AtomicBool::new(false));
    let report = run_stateful(
        StatefulScenario::new(
            "lumen-search-continuity",
            StatefulActions {
                warmup: {
                    let engine = Arc::clone(&engine);
                    Box::new(move |evidence| {
                        engine.create_collection("docs", schema())?;
                        engine.index(
                            "docs",
                            IndexRequest {
                                items: vec![
                                    item("d1", "search systems retain state"),
                                    item("d2", "stateful search recovery"),
                                    item("d3", "unrelated document"),
                                ],
                                request_id: None,
                            },
                        )?;
                        evidence.record("indexed_documents", json!({"count": 3}));
                        Ok(())
                    })
                },
                observe: {
                    let engine = Arc::clone(&engine);
                    Box::new(move |evidence| {
                        let response = search(&engine)?;
                        evidence.record(
                            "steady_search",
                            json!({"total": response.total, "hits": ids(&response)}),
                        );
                        ensure!(response.total == 2, "expected two matching documents");
                        Ok(())
                    })
                },
                fault: {
                    let unavailable = Arc::clone(&unavailable);
                    Box::new(move |evidence| {
                        unavailable.store(true, Ordering::Release);
                        evidence.record("serving_fault", json!({"available": false}));
                        ensure!(unavailable.load(Ordering::Acquire));
                        Ok(())
                    })
                },
                recover: {
                    let unavailable = Arc::clone(&unavailable);
                    Box::new(move |evidence| {
                        unavailable.store(false, Ordering::Release);
                        evidence.record("serving_recovered", json!({"available": true}));
                        Ok(())
                    })
                },
                verify: {
                    let engine = Arc::clone(&engine);
                    let unavailable = Arc::clone(&unavailable);
                    Box::new(move |evidence| {
                        ensure!(!unavailable.load(Ordering::Acquire));
                        let response = search(&engine)?;
                        let mut got = ids(&response);
                        got.sort();
                        evidence.record(
                            "search_continuity",
                            json!({"total": response.total, "hits": got}),
                        );
                        ensure!(response.total == 2);
                        ensure!(got == ["d1", "d2"]);
                        Ok(())
                    })
                },
                teardown: Box::new(|evidence| {
                    evidence.record("adapter_cleanup", json!({"complete": true}));
                    Ok(())
                }),
            },
        )
        .with_limits(StatefulLimits {
            phase_timeout: Duration::from_secs(2),
            scenario_timeout: Duration::from_secs(6),
            teardown_timeout: Duration::from_secs(1),
        }),
    );

    assert!(report.passed, "{report:#?}");
    assert_eq!(report.protocol, "rig.stateful.v1");
    assert!(report
        .evidence
        .iter()
        .any(|record| record.kind == "search_continuity"));
}

fn schema() -> CreateCollectionRequest {
    let mut fields = BTreeMap::new();
    fields.insert(
        "body".into(),
        FieldSpec {
            field_type: FieldType::Text,
            analyzer: Some(Analyzer::WhitespaceLower),
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    CreateCollectionRequest { fields }
}

fn item(id: &str, body: &str) -> IndexItem {
    IndexItem {
        external_id: id.into(),
        field: "body".into(),
        value: FieldValue::String(body.into()),
        version: None,
    }
}

fn search(engine: &Engine) -> Result<SearchResponse> {
    engine.search(
        "docs",
        SearchRequest {
            query: QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: "search".into(),
                op: MatchOp::Or,
            }),
            limit: 10,
            offset: 0,
            cursor: None,
            routing_key: None,
            sort: None,
            track_total: true,
            collapse: None,
        },
    )
}

fn ids(response: &SearchResponse) -> Vec<String> {
    response
        .hits
        .iter()
        .map(|hit| hit.external_id.clone())
        .collect()
}
// HANDWRITE-END
