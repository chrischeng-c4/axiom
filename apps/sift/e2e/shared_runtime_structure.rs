//! Structural contracts that keep shared runtime mechanics out of Sift.

#[test]
fn group_commit_is_owned_by_service_executor() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("read Sift manifest");
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"))
        .expect("read Sift library source");

    assert!(
        manifest.contains("service-executor ="),
        "Sift must compose the shared group-commit runtime"
    );
    assert!(
        source.contains("impl service_executor::GroupCommitRequest for IngestBatchRequest"),
        "Sift must provide only its batch data adapter"
    );
    assert!(
        !source.contains("async fn run_ingest_batcher"),
        "Sift must not own the batch timer and fan-out loop"
    );
}

#[test]
fn ingest_admission_mechanics_are_owned_by_service_http() {
    let source =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/ingest/limits.rs"))
            .expect("read Sift ingest limits");

    assert!(
        source.contains("service_http::WeightedAdmission"),
        "Sift must compose shared weighted admission"
    );
    assert!(
        source.contains("service_http::decode_request_body"),
        "Sift must compose shared bounded gzip decoding"
    );
    for local_mechanism in [
        "struct ProjectAdmission",
        "impl Drop for AdmissionPermit",
        "GzDecoder",
    ] {
        assert!(
            !source.contains(local_mechanism),
            "Sift must not retain {local_mechanism}"
        );
    }
}

#[test]
fn reverse_proxy_runtime_is_owned_by_service_http() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/proxy.rs"))
        .expect("read Sift proxy adapter");

    assert!(
        source.contains("impl service_http::ReverseProxyPolicy for SiftRolePolicy"),
        "Sift must provide only its upstream selection policy"
    );
    for local_mechanism in ["reqwest::Client", "async fn forward", "fn is_hop_header"] {
        assert!(
            !source.contains(local_mechanism),
            "Sift must not retain {local_mechanism}"
        );
    }
}

#[test]
fn persistent_query_job_transitions_use_service_executor() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"))
        .expect("read Sift library source");

    assert!(
        source.contains("service_executor::JobRunner::new"),
        "Sift must run persistent work through the shared job runner"
    );
    assert!(
        !source.contains("let job_store = worker_state.query_jobs.clone();"),
        "Sift must not own query-job transition control flow"
    );
}

#[test]
fn shutdown_order_and_task_failures_use_server_lifecycle() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bin/sift.rs"))
        .expect("read Sift binary source");

    assert!(
        source.contains("server_lifecycle::TaskSupervisor::new"),
        "the binary must compose the shared task supervisor"
    );
    assert!(
        source.contains("server_lifecycle::HookStage::FinalFlush"),
        "Sift must map its projection flush into the shared shutdown order"
    );
    assert!(
        !source.contains("if let Some((shutdown, task)) = grpc"),
        "the binary must not hand-roll task shutdown and joins"
    );
}

#[test]
fn scoped_bearer_middleware_is_owned_by_service_auth() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/auth.rs"))
        .expect("read Sift auth adapter");

    assert!(
        source.contains("impl service_auth::ScopedAuthorization for SiftVerifier"),
        "Sift must provide route, project, and role policy through the shared trait"
    );
    assert!(
        source.contains("pub use service_auth::scoped_authorization_middleware as auth_middleware"),
        "Sift must reuse the shared bearer middleware"
    );
    assert!(
        !source.contains("pub async fn auth_middleware"),
        "Sift must not retain the middleware control flow"
    );
}

#[test]
fn replicated_host_startup_is_owned_by_raft_runtime() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"))
        .expect("read Sift library source");

    assert!(
        source.contains("impl raft_runtime::MembershipPolicy for SiftMembershipPolicy"),
        "Sift must provide only its three-voter membership policy"
    );
    assert!(
        source.contains("raft_runtime::ReplicaHostBuilder::new"),
        "Sift must compose the shared replicated-host startup"
    );
    for local_mechanism in [
        "ClusterTopology::from_env_with_scheme",
        "PeerTransport::from_config",
        "RaftStore::open",
        "RaftHost::spawn_with_peer_transport",
    ] {
        assert!(
            !source.contains(local_mechanism),
            "Sift must not retain {local_mechanism}"
        );
    }
}

#[test]
fn kubernetes_workloads_are_owned_by_service_k8s() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/operator.rs"))
        .expect("read Sift operator adapter");

    assert!(
        source.contains("render::WorkloadPlan::new"),
        "Sift must compose one shared typed workload plan"
    );
    assert!(
        source.contains("render::NetworkPolicyPlan::new"),
        "Sift role reachability must use the typed network policy plan"
    );
    for local_renderer in [
        "fn stateful_role(",
        "fn deployment_role(",
        "fn agent_daemonset(",
        "fn disruption_budget(",
        "fn network_policy(",
    ] {
        assert!(
            !source.contains(local_renderer),
            "Sift must not retain {local_renderer}"
        );
    }
}

#[test]
fn live_backup_http_transport_is_owned_by_service_backup() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/backup.rs"))
        .expect("read Sift backup adapter");

    assert!(
        source.contains("service_backup::AdminSnapshotTransport"),
        "Sift must compose the shared admin snapshot transport"
    );
    assert!(
        source.contains("service_backup::AdminSnapshotRequest"),
        "Sift must provide only its project and credential policy"
    );
    for local_mechanism in [
        "reqwest::Client::builder",
        "ProjectedTokenFile::new",
        "while diagnostic.len()",
    ] {
        assert!(
            !source.contains(local_mechanism),
            "Sift must not retain {local_mechanism}"
        );
    }
}

#[test]
fn logging_text_index_is_owned_by_index_text() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/projection/logging.rs"
    ))
    .expect("read Sift logging projection");

    assert!(source.contains("index_text::"));
    assert!(source.contains("MemoryTextIndex"));
    assert!(!source.contains("lumen::"));
    assert!(!source.contains("CreateCollectionRequest"));
}

#[test]
fn manifest_last_archive_flow_is_owned_by_storage_segment() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/storage/archive.rs"
    ))
    .expect("read Sift archive adapter");

    assert!(source.contains("storage_segment::ArchiveCoordinator"));
    assert!(source.contains("impl storage_segment::RecordCodec<StoredEvent>"));
    assert!(!source.contains("sink.put_object"));
}

#[test]
fn typed_projection_flow_is_owned_by_service_projection() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/projection/runtime.rs"
    ))
    .expect("read Sift projection adapter");

    assert!(source.contains("service_projection::ProjectionRegistry"));
    assert!(source.contains("ProjectionHandle<StoredEvent, LoggingProjection>"));
    for local_mechanism in ["downcast_ref", "struct ProjectionSlot", "fn persist("] {
        assert!(
            !source.contains(local_mechanism),
            "Sift must not retain {local_mechanism}"
        );
    }
}

#[test]
fn otlp_wire_and_direct_grpc_runtime_are_owned_by_transport_otlp() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("read Sift manifest");
    let normalizer = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ingest/otlp/mod.rs"
    ))
    .expect("read Sift OTLP adapter");
    let grpc = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/grpc.rs"))
        .expect("read Sift gRPC adapter");

    assert!(manifest.contains("transport-otlp ="));
    assert!(normalizer.contains("pub use transport_otlp::proto as wire"));
    assert!(grpc.contains("transport_otlp::serve_grpc"));
    assert!(grpc.contains("impl transport_otlp::OtlpConsumer for SiftGrpcConsumer"));
    assert!(!std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ingest/otlp/wire.rs"
    ))
    .exists());
}
