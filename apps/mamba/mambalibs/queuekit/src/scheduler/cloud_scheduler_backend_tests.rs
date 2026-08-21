//! Tests for CloudSchedulerBackend
//!
//! Covers spec scenarios S1-S8 from cloud-scheduler-backend change spec.
//! Tests that require GCP network calls (create_job, delete_job, pause/resume API)
//! are tested indirectly through local state management and error mapping.

use super::*;
use chrono::TimeZone;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_config() -> CloudSchedulerConfig {
    CloudSchedulerConfig {
        project_id: "my-project".to_string(),
        location: "us-central1".to_string(),
        oidc_service_account_email: "sa@my-project.iam.gserviceaccount.com".to_string(),
        target_base_url: "https://app.example.com/tasks".to_string(),
        time_zone: "UTC".to_string(),
        credentials_path: None,
    }
}

fn test_backend() -> CloudSchedulerBackend {
    CloudSchedulerBackend::new(test_config()).expect("failed to create test backend")
}

fn sample_job() -> CloudSchedulerJob {
    CloudSchedulerJob {
        name: "projects/my-project/locations/us-central1/jobs/daily-cleanup".to_string(),
        schedule: "0 2 * * *".to_string(),
        time_zone: "UTC".to_string(),
        http_target: HttpTarget {
            uri: "https://app.example.com/tasks/cleanup".to_string(),
            http_method: "POST".to_string(),
            body: None,
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "application/json".to_string());
                h
            },
            oidc_token: Some(OidcTokenTarget {
                service_account_email: "sa@my-project.iam.gserviceaccount.com".to_string(),
                audience: Some("https://app.example.com".to_string()),
            }),
        },
        state: Some("ENABLED".to_string()),
        user_update_time: None,
        last_attempt_time: None,
        status: None,
    }
}

// ---------------------------------------------------------------------------
// S1: Leader election is no-op for cloud-managed backend (R1)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_acquire_leader_returns_true() {
    let backend = test_backend();
    let result = backend.acquire_leader(Duration::from_secs(15)).await;
    assert_eq!(result.unwrap(), true);
}

#[tokio::test]
async fn test_renew_leader_returns_true() {
    let backend = test_backend();
    let result = backend.renew_leader(Duration::from_secs(15)).await;
    assert_eq!(result.unwrap(), true);
}

#[tokio::test]
async fn test_release_leader_returns_ok() {
    let backend = test_backend();
    let result = backend.release_leader().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_leader_election_full_cycle() {
    let backend = test_backend();
    // All three operations should succeed as no-ops
    assert!(backend
        .acquire_leader(Duration::from_secs(10))
        .await
        .unwrap());
    assert!(backend.renew_leader(Duration::from_secs(10)).await.unwrap());
    backend.release_leader().await.unwrap();
    // After release, acquire should still return true (cloud manages scheduling)
    assert!(backend
        .acquire_leader(Duration::from_secs(10))
        .await
        .unwrap());
}

// ---------------------------------------------------------------------------
// S2: Config helpers and job serialization (R3, R5)
// ---------------------------------------------------------------------------

#[test]
fn test_config_jobs_parent() {
    let config = test_config();
    assert_eq!(
        config.jobs_parent(),
        "projects/my-project/locations/us-central1"
    );
}

#[test]
fn test_config_job_name() {
    let config = test_config();
    assert_eq!(
        config.job_name("daily-cleanup"),
        "projects/my-project/locations/us-central1/jobs/daily-cleanup"
    );
}

#[test]
fn test_config_default() {
    let config = CloudSchedulerConfig::default();
    assert_eq!(config.location, "us-central1");
    assert_eq!(config.time_zone, "UTC");
    assert!(config.credentials_path.is_none());
    assert!(config.project_id.is_empty());
}

#[test]
fn test_job_serialization_camel_case() {
    let job = sample_job();
    let json = serde_json::to_value(&job).unwrap();

    // Verify camelCase field names
    assert!(json.get("name").is_some());
    assert!(json.get("schedule").is_some());
    assert!(json.get("timeZone").is_some());
    assert!(json.get("httpTarget").is_some());
    assert!(json.get("state").is_some());

    // Verify None fields are skipped
    assert!(json.get("userUpdateTime").is_none());
    assert!(json.get("lastAttemptTime").is_none());
    assert!(json.get("status").is_none());
}

#[test]
fn test_job_deserialization_from_gcp_response() {
    let json = serde_json::json!({
        "name": "projects/my-project/locations/us-central1/jobs/test-job",
        "schedule": "*/5 * * * *",
        "timeZone": "America/New_York",
        "httpTarget": {
            "uri": "https://app.example.com/tasks/test",
            "httpMethod": "POST",
            "headers": { "Content-Type": "application/json" },
            "oidcToken": {
                "serviceAccountEmail": "sa@my-project.iam.gserviceaccount.com",
                "audience": "https://app.example.com"
            }
        },
        "state": "ENABLED",
        "userUpdateTime": "2026-03-26T10:00:00Z",
        "lastAttemptTime": "2026-03-26T12:00:00Z",
        "status": { "code": 0 }
    });

    let job: CloudSchedulerJob = serde_json::from_value(json).unwrap();
    assert_eq!(
        job.name,
        "projects/my-project/locations/us-central1/jobs/test-job"
    );
    assert_eq!(job.schedule, "*/5 * * * *");
    assert_eq!(job.time_zone, "America/New_York");
    assert_eq!(job.http_target.uri, "https://app.example.com/tasks/test");
    assert_eq!(job.http_target.http_method, "POST");
    assert_eq!(job.state.as_deref(), Some("ENABLED"));
    assert!(job.user_update_time.is_some());
    assert!(job.last_attempt_time.is_some());
    assert!(job.status.is_some());
}

#[test]
fn test_job_deserialization_minimal() {
    // Minimal job with only required fields
    let json = serde_json::json!({
        "name": "projects/p/locations/l/jobs/j",
        "schedule": "0 * * * *",
        "httpTarget": {
            "uri": "https://example.com/task",
            "httpMethod": "POST",
            "oidcToken": {
                "serviceAccountEmail": "sa@p.iam.gserviceaccount.com"
            }
        }
    });

    let job: CloudSchedulerJob = serde_json::from_value(json).unwrap();
    assert_eq!(job.time_zone, "UTC"); // default
    assert!(job.state.is_none());
    assert!(job.http_target.body.is_none());
    assert!(job.http_target.headers.is_empty());
    assert!(job
        .http_target
        .oidc_token
        .as_ref()
        .unwrap()
        .audience
        .is_none());
}

#[test]
fn test_job_roundtrip_serialization() {
    let job = sample_job();
    let json = serde_json::to_string(&job).unwrap();
    let deserialized: CloudSchedulerJob = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, job.name);
    assert_eq!(deserialized.schedule, job.schedule);
    assert_eq!(deserialized.http_target.uri, job.http_target.uri);
}

#[test]
fn test_list_jobs_response_deserialization() {
    let json = serde_json::json!({
        "jobs": [
            {
                "name": "projects/p/locations/l/jobs/job1",
                "schedule": "0 * * * *",
                "httpTarget": {
                    "uri": "https://example.com/t1",
                    "httpMethod": "POST",
                    "oidcToken": { "serviceAccountEmail": "sa@p.iam.gserviceaccount.com" }
                }
            },
            {
                "name": "projects/p/locations/l/jobs/job2",
                "schedule": "*/10 * * * *",
                "httpTarget": {
                    "uri": "https://example.com/t2",
                    "httpMethod": "POST",
                    "oidcToken": { "serviceAccountEmail": "sa@p.iam.gserviceaccount.com" }
                }
            }
        ],
        "nextPageToken": "abc123"
    });

    let response: ListJobsResponse = serde_json::from_value(json).unwrap();
    assert_eq!(response.jobs.len(), 2);
    assert_eq!(response.next_page_token.as_deref(), Some("abc123"));
}

#[test]
fn test_list_jobs_response_empty() {
    let json = serde_json::json!({});
    let response: ListJobsResponse = serde_json::from_value(json).unwrap();
    assert!(response.jobs.is_empty());
    assert!(response.next_page_token.is_none());
}

// ---------------------------------------------------------------------------
// S4: Task state management (R2) — get, set, record_task_run
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_default_task_state_for_unknown_task() {
    let backend = test_backend();
    let state = backend.get_task_state("nonexistent-task").await.unwrap();
    assert!(state.enabled);
    assert!(state.last_run_at.is_none());
    assert_eq!(state.total_run_count, 0);
}

#[tokio::test]
async fn test_set_and_get_task_state() {
    let backend = test_backend();
    let now = Utc::now();
    let state = TaskScheduleState {
        enabled: false,
        last_run_at: Some(now),
        total_run_count: 42,
    };

    backend.set_task_state("my-task", &state).await.unwrap();
    let retrieved = backend.get_task_state("my-task").await.unwrap();

    assert!(!retrieved.enabled);
    assert_eq!(retrieved.total_run_count, 42);
    assert_eq!(retrieved.last_run_at, Some(now));
}

#[tokio::test]
async fn test_set_task_state_overwrites() {
    let backend = test_backend();

    let state1 = TaskScheduleState {
        enabled: true,
        last_run_at: None,
        total_run_count: 1,
    };
    backend.set_task_state("task-a", &state1).await.unwrap();

    let state2 = TaskScheduleState {
        enabled: false,
        last_run_at: Some(Utc::now()),
        total_run_count: 99,
    };
    backend.set_task_state("task-a", &state2).await.unwrap();

    let retrieved = backend.get_task_state("task-a").await.unwrap();
    assert!(!retrieved.enabled);
    assert_eq!(retrieved.total_run_count, 99);
}

#[tokio::test]
async fn test_record_task_run_increments_count() {
    let backend = test_backend();

    // First run on fresh task
    backend.record_task_run("hourly-sync").await.unwrap();
    let state = backend.get_task_state("hourly-sync").await.unwrap();
    assert_eq!(state.total_run_count, 1);
    assert!(state.last_run_at.is_some());

    // Second run
    backend.record_task_run("hourly-sync").await.unwrap();
    let state = backend.get_task_state("hourly-sync").await.unwrap();
    assert_eq!(state.total_run_count, 2);
}

#[tokio::test]
async fn test_record_task_run_updates_last_run_at() {
    let backend = test_backend();

    // Set initial state with old timestamp
    let old_time = Utc.with_ymd_and_hms(2026, 3, 26, 10, 0, 0).unwrap();
    let initial = TaskScheduleState {
        enabled: true,
        last_run_at: Some(old_time),
        total_run_count: 5,
    };
    backend
        .set_task_state("hourly-sync", &initial)
        .await
        .unwrap();

    // Record a new run
    let before = Utc::now();
    backend.record_task_run("hourly-sync").await.unwrap();
    let after = Utc::now();

    let state = backend.get_task_state("hourly-sync").await.unwrap();
    assert_eq!(state.total_run_count, 6);
    let last_run = state.last_run_at.unwrap();
    assert!(last_run >= before && last_run <= after);
}

#[tokio::test]
async fn test_record_task_run_preserves_enabled() {
    let backend = test_backend();

    // Task starts enabled (default)
    backend.record_task_run("task-x").await.unwrap();
    let state = backend.get_task_state("task-x").await.unwrap();
    assert!(state.enabled);
}

#[tokio::test]
async fn test_multiple_tasks_isolated() {
    let backend = test_backend();

    backend.record_task_run("task-a").await.unwrap();
    backend.record_task_run("task-a").await.unwrap();
    backend.record_task_run("task-b").await.unwrap();

    let state_a = backend.get_task_state("task-a").await.unwrap();
    let state_b = backend.get_task_state("task-b").await.unwrap();
    let state_c = backend.get_task_state("task-c").await.unwrap();

    assert_eq!(state_a.total_run_count, 2);
    assert_eq!(state_b.total_run_count, 1);
    assert_eq!(state_c.total_run_count, 0); // never recorded
}

// ---------------------------------------------------------------------------
// S3 / S7: Pause and resume — local state effects (R7)
// (pause_task/resume_task make HTTP calls, so we test via set_task_state)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_pause_resume_via_local_state() {
    let backend = test_backend();

    // Simulate what pause_task does locally
    let mut state = backend.get_task_state("daily-cleanup").await.unwrap();
    assert!(state.enabled);

    state.enabled = false;
    backend
        .set_task_state("daily-cleanup", &state)
        .await
        .unwrap();

    assert!(!backend.is_task_enabled("daily-cleanup").await.unwrap());

    // Simulate what resume_task does locally
    state.enabled = true;
    backend
        .set_task_state("daily-cleanup", &state)
        .await
        .unwrap();

    assert!(backend.is_task_enabled("daily-cleanup").await.unwrap());
}

#[tokio::test]
async fn test_is_task_enabled_default_true() {
    let backend = test_backend();
    // Unknown task defaults to enabled
    assert!(backend.is_task_enabled("new-task").await.unwrap());
}

// ---------------------------------------------------------------------------
// S5: OidcTokenCache validity logic (R4)
// ---------------------------------------------------------------------------

#[test]
fn test_oidc_cache_new_is_invalid() {
    let cache = OidcTokenCache::new();
    assert!(!cache.is_valid());
    assert!(cache.access_token.is_none());
    assert!(cache.expires_at.is_none());
}

#[test]
fn test_oidc_cache_valid_token() {
    let cache = OidcTokenCache {
        access_token: Some("token-abc".to_string()),
        expires_at: Some(Utc::now() + chrono::Duration::seconds(3600)),
    };
    assert!(cache.is_valid());
}

#[test]
fn test_oidc_cache_expired_token() {
    let cache = OidcTokenCache {
        access_token: Some("token-old".to_string()),
        expires_at: Some(Utc::now() - chrono::Duration::seconds(60)),
    };
    assert!(!cache.is_valid());
}

#[test]
fn test_oidc_cache_within_refresh_buffer() {
    // Token expires in 4 minutes -- within the 5-minute buffer -> invalid
    let cache = OidcTokenCache {
        access_token: Some("token-soon".to_string()),
        expires_at: Some(Utc::now() + chrono::Duration::seconds(240)),
    };
    assert!(!cache.is_valid());
}

#[test]
fn test_oidc_cache_outside_refresh_buffer() {
    // Token expires in 6 minutes -- outside the 5-minute buffer -> valid
    let cache = OidcTokenCache {
        access_token: Some("token-ok".to_string()),
        expires_at: Some(Utc::now() + chrono::Duration::seconds(360)),
    };
    assert!(cache.is_valid());
}

#[test]
fn test_oidc_cache_token_none_with_expiry() {
    let cache = OidcTokenCache {
        access_token: None,
        expires_at: Some(Utc::now() + chrono::Duration::seconds(3600)),
    };
    assert!(!cache.is_valid());
}

#[test]
fn test_oidc_cache_token_present_no_expiry() {
    let cache = OidcTokenCache {
        access_token: Some("token".to_string()),
        expires_at: None,
    };
    assert!(!cache.is_valid());
}

// ---------------------------------------------------------------------------
// S6: GCP API error mapping (R8)
// ---------------------------------------------------------------------------

#[test]
fn test_map_gcp_error_404_not_found() {
    let err = CloudSchedulerBackend::map_gcp_error(reqwest::StatusCode::NOT_FOUND, "Job not found");
    match err {
        TaskError::TaskNotFound(msg) => assert_eq!(msg, "Job not found"),
        other => panic!("Expected TaskNotFound, got: {:?}", other),
    }
}

#[test]
fn test_map_gcp_error_401_authentication() {
    let err = CloudSchedulerBackend::map_gcp_error(
        reqwest::StatusCode::UNAUTHORIZED,
        "Invalid credentials",
    );
    match err {
        TaskError::Authentication(msg) => {
            assert!(msg.contains("401"));
            assert!(msg.contains("Invalid credentials"));
        }
        other => panic!("Expected Authentication, got: {:?}", other),
    }
}

#[test]
fn test_map_gcp_error_403_authentication() {
    let err = CloudSchedulerBackend::map_gcp_error(reqwest::StatusCode::FORBIDDEN, "Access denied");
    match err {
        TaskError::Authentication(msg) => {
            assert!(msg.contains("403"));
            assert!(msg.contains("Access denied"));
        }
        other => panic!("Expected Authentication, got: {:?}", other),
    }
}

#[test]
fn test_map_gcp_error_429_rate_limited() {
    let err = CloudSchedulerBackend::map_gcp_error(
        reqwest::StatusCode::TOO_MANY_REQUESTS,
        "Rate limit exceeded",
    );
    match err {
        TaskError::RateLimited(duration) => {
            assert_eq!(duration, Duration::from_secs(60));
        }
        other => panic!("Expected RateLimited, got: {:?}", other),
    }
}

#[test]
fn test_map_gcp_error_500_backend() {
    let err = CloudSchedulerBackend::map_gcp_error(
        reqwest::StatusCode::INTERNAL_SERVER_ERROR,
        "Internal error",
    );
    match err {
        TaskError::Backend(msg) => {
            assert!(msg.contains("500"));
            assert!(msg.contains("Internal error"));
        }
        other => panic!("Expected Backend, got: {:?}", other),
    }
}

#[test]
fn test_map_gcp_error_503_backend() {
    let err = CloudSchedulerBackend::map_gcp_error(
        reqwest::StatusCode::SERVICE_UNAVAILABLE,
        "Service unavailable",
    );
    match err {
        TaskError::Backend(msg) => {
            assert!(msg.contains("503"));
        }
        other => panic!("Expected Backend, got: {:?}", other),
    }
}

#[test]
fn test_map_gcp_error_502_backend() {
    let err = CloudSchedulerBackend::map_gcp_error(reqwest::StatusCode::BAD_GATEWAY, "Bad gateway");
    match err {
        TaskError::Backend(msg) => {
            assert!(msg.contains("502"));
        }
        other => panic!("Expected Backend, got: {:?}", other),
    }
}

#[test]
fn test_map_gcp_error_unknown_status() {
    let err =
        CloudSchedulerBackend::map_gcp_error(reqwest::StatusCode::BAD_REQUEST, "Bad request body");
    match err {
        TaskError::Backend(msg) => {
            assert!(msg.contains("400"));
            assert!(msg.contains("Bad request body"));
        }
        other => panic!("Expected Backend for unknown status, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// S8: Delete job -- local state cleanup (R3)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_delete_removes_local_state_directly() {
    let backend = test_backend();

    // Set up task state
    backend.record_task_run("daily-cleanup").await.unwrap();
    let state = backend.get_task_state("daily-cleanup").await.unwrap();
    assert_eq!(state.total_run_count, 1);

    // Directly remove from local state (simulates what delete_job does locally)
    backend.task_states.write().await.remove("daily-cleanup");

    // State should now be default
    let state = backend.get_task_state("daily-cleanup").await.unwrap();
    assert_eq!(state.total_run_count, 0);
    assert!(state.last_run_at.is_none());
}

// ---------------------------------------------------------------------------
// Construction and trait bounds
// ---------------------------------------------------------------------------

#[test]
fn test_backend_new_success() {
    let result = CloudSchedulerBackend::new(test_config());
    assert!(result.is_ok());
}

#[test]
fn test_backend_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CloudSchedulerBackend>();
}

// ---------------------------------------------------------------------------
// HttpTarget serialization (R5)
// ---------------------------------------------------------------------------

#[test]
fn test_http_target_serialization() {
    let target = HttpTarget {
        uri: "https://example.com/task".to_string(),
        http_method: "POST".to_string(),
        body: Some("eyJrZXkiOiJ2YWx1ZSJ9".to_string()), // base64
        headers: {
            let mut h = HashMap::new();
            h.insert("Content-Type".to_string(), "application/json".to_string());
            h
        },
        oidc_token: Some(OidcTokenTarget {
            service_account_email: "sa@p.iam.gserviceaccount.com".to_string(),
            audience: Some("https://example.com".to_string()),
        }),
    };

    let json = serde_json::to_value(&target).unwrap();
    assert_eq!(json["uri"], "https://example.com/task");
    assert_eq!(json["httpMethod"], "POST");
    assert_eq!(json["body"], "eyJrZXkiOiJ2YWx1ZSJ9");
    assert_eq!(json["headers"]["Content-Type"], "application/json");
    assert_eq!(
        json["oidcToken"]["serviceAccountEmail"],
        "sa@p.iam.gserviceaccount.com"
    );
    assert_eq!(json["oidcToken"]["audience"], "https://example.com");
}

#[test]
fn test_http_target_empty_headers_omitted() {
    let target = HttpTarget {
        uri: "https://example.com/task".to_string(),
        http_method: "POST".to_string(),
        body: None,
        headers: HashMap::new(),
        oidc_token: None,
    };

    let json = serde_json::to_value(&target).unwrap();
    // Empty headers should be skipped
    assert!(json.get("headers").is_none());
    // None body should be skipped
    assert!(json.get("body").is_none());
    // None oidcToken should be skipped
    assert!(json.get("oidcToken").is_none());
}

// ---------------------------------------------------------------------------
// MetadataTokenResponse deserialization
// ---------------------------------------------------------------------------

#[test]
fn test_metadata_token_response_deserialization() {
    let json = serde_json::json!({
        "access_token": "ya29.AHES6ZRN3-HlhAPy",
        "expires_in": 3599,
        "token_type": "Bearer"
    });

    let response: MetadataTokenResponse = serde_json::from_value(json).unwrap();
    assert_eq!(response.access_token, "ya29.AHES6ZRN3-HlhAPy");
    assert_eq!(response.expires_in, 3599);
    assert_eq!(response.token_type, "Bearer");
}
