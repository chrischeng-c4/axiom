//! Task scheduling
//!
//! Delayed and periodic task scheduling with distributed coordination.

pub mod backend;
pub mod delay;
pub mod memory_backend;
pub mod periodic;

#[cfg(feature = "scheduler")]
pub mod ion_backend;

#[cfg(feature = "cloud-scheduler")]
pub mod cloud_scheduler_backend;

#[cfg(feature = "k8s-scheduler")]
pub mod k8s_cronjob_backend;

#[cfg(feature = "push-receiver")]
pub mod push_auth;
#[cfg(feature = "push-receiver")]
pub mod push_receiver;

#[cfg(feature = "scheduler")]
pub mod schedule_monitor;

// Re-exports
pub use backend::{SchedulerBackend, SchedulingMode, TaskScheduleState};
#[cfg(feature = "cloud-scheduler")]
pub use cloud_scheduler_backend::{CloudSchedulerBackend, CloudSchedulerConfig};
#[cfg(feature = "nats")]
pub use delay::{DelayedTaskConfig, DelayedTaskScheduler};
#[cfg(feature = "scheduler")]
pub use ion_backend::IonSchedulerBackend;
#[cfg(feature = "k8s-scheduler")]
pub use k8s_cronjob_backend::{K8sCronJobBackend, K8sCronJobConfig};
pub use memory_backend::MemorySchedulerBackend;
pub use periodic::{PeriodicSchedule, PeriodicScheduler, PeriodicSchedulerConfig, PeriodicTask};
#[cfg(feature = "push-receiver")]
pub use push_receiver::{PushReceiver, PushReceiverConfig};
#[cfg(feature = "scheduler")]
pub use schedule_monitor::{FireStatus, ScheduleMonitor, ScheduleMonitorConfig};

// ---------------------------------------------------------------------------
// Tests for scheduler-backends-gcp spec (module registration & feature isolation)
// ---------------------------------------------------------------------------

/// Tests that require the cloud-scheduler feature — covers S1, S3, S4 from spec.
/// S2 (feature exclusion) is implicitly verified: this entire module is behind
/// `#[cfg(feature = "cloud-scheduler")]`, so it only compiles when the feature is active.
/// Without the feature, `CloudSchedulerBackend` and `CloudSchedulerConfig` are absent
/// from the `scheduler` module — verified by the conditional `pub mod` and `pub use`.
#[cfg(all(test, feature = "cloud-scheduler"))]
mod backends_gcp_tests {
    use super::*;
    use std::time::Duration;

    // -----------------------------------------------------------------------
    // S1: Cloud Scheduler backend is available when feature is enabled (R4, R6)
    // -----------------------------------------------------------------------

    #[test]
    fn s1_cloud_scheduler_backend_importable() {
        // Verifies R4: CloudSchedulerBackend is importable from scheduler module
        let config = CloudSchedulerConfig {
            project_id: "test-project".to_string(),
            location: "us-central1".to_string(),
            oidc_service_account_email: "sa@test.iam.gserviceaccount.com".to_string(),
            target_base_url: "https://example.com/tasks".to_string(),
            time_zone: "UTC".to_string(),
            credentials_path: None,
        };
        let backend = CloudSchedulerBackend::new(config);
        assert!(
            backend.is_ok(),
            "CloudSchedulerBackend should be constructible when cloud-scheduler feature is enabled"
        );
    }

    #[test]
    fn s1_cloud_scheduler_config_importable() {
        // Verifies R4: CloudSchedulerConfig is importable from scheduler module
        let config = CloudSchedulerConfig::default();
        assert_eq!(config.location, "us-central1");
        assert_eq!(config.time_zone, "UTC");
    }

    #[test]
    fn s1_cloud_scheduler_module_reexports() {
        // Verifies R4: re-exports work — types are accessible via scheduler::*
        // If this compiles, the re-exports in mod.rs are correct.
        fn _assert_type_exists(_b: CloudSchedulerBackend) {}
        fn _assert_config_exists(_c: CloudSchedulerConfig) {}
    }

    // -----------------------------------------------------------------------
    // S3: Cloud Scheduler backend implements SchedulerBackend trait (R5)
    // -----------------------------------------------------------------------

    fn make_test_backend() -> CloudSchedulerBackend {
        CloudSchedulerBackend::new(CloudSchedulerConfig {
            project_id: "trait-test".to_string(),
            location: "us-central1".to_string(),
            oidc_service_account_email: "sa@test.iam.gserviceaccount.com".to_string(),
            target_base_url: "https://example.com".to_string(),
            time_zone: "UTC".to_string(),
            credentials_path: None,
        })
        .unwrap()
    }

    #[test]
    fn s3_cloud_scheduler_satisfies_scheduler_backend_trait() {
        // Verifies R5: CloudSchedulerBackend implements SchedulerBackend
        // If this compiles, the trait is implemented.
        let backend = make_test_backend();
        let _boxed: Box<dyn SchedulerBackend> = Box::new(backend);
    }

    #[tokio::test]
    async fn s3_trait_object_acquire_leader() {
        // Verifies R5: acquire_leader callable through trait object
        let backend = make_test_backend();
        let dyn_backend: Box<dyn SchedulerBackend> = Box::new(backend);
        let result = dyn_backend.acquire_leader(Duration::from_secs(15)).await;
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn s3_trait_object_renew_leader() {
        // Verifies R5: renew_leader callable through trait object
        let backend = make_test_backend();
        let dyn_backend: Box<dyn SchedulerBackend> = Box::new(backend);
        let result = dyn_backend.renew_leader(Duration::from_secs(15)).await;
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn s3_trait_object_release_leader() {
        // Verifies R5: release_leader callable through trait object
        let backend = make_test_backend();
        let dyn_backend: Box<dyn SchedulerBackend> = Box::new(backend);
        assert!(dyn_backend.release_leader().await.is_ok());
    }

    #[tokio::test]
    async fn s3_trait_object_get_task_state() {
        // Verifies R5: get_task_state callable through trait object
        let backend = make_test_backend();
        let dyn_backend: Box<dyn SchedulerBackend> = Box::new(backend);
        let state = dyn_backend.get_task_state("test-task").await.unwrap();
        assert!(state.enabled);
        assert_eq!(state.total_run_count, 0);
        assert!(state.last_run_at.is_none());
    }

    #[tokio::test]
    async fn s3_trait_object_set_task_state() {
        // Verifies R5: set_task_state callable through trait object
        let backend = make_test_backend();
        let dyn_backend: Box<dyn SchedulerBackend> = Box::new(backend);

        let state = TaskScheduleState {
            enabled: false,
            last_run_at: Some(chrono::Utc::now()),
            total_run_count: 10,
        };
        dyn_backend.set_task_state("my-task", &state).await.unwrap();

        let retrieved = dyn_backend.get_task_state("my-task").await.unwrap();
        assert!(!retrieved.enabled);
        assert_eq!(retrieved.total_run_count, 10);
    }

    #[tokio::test]
    async fn s3_trait_object_all_methods_in_sequence() {
        // Verifies R5: all trait methods work through a single Box<dyn SchedulerBackend>
        // This simulates the PeriodicScheduler usage pattern
        let backend = make_test_backend();
        let dyn_backend: Box<dyn SchedulerBackend> = Box::new(backend);

        // Leader election cycle
        assert!(dyn_backend
            .acquire_leader(Duration::from_secs(30))
            .await
            .unwrap());
        assert!(dyn_backend
            .renew_leader(Duration::from_secs(30))
            .await
            .unwrap());

        // Task state operations
        let state = dyn_backend.get_task_state("periodic-task").await.unwrap();
        assert!(state.enabled);

        let updated = TaskScheduleState {
            enabled: true,
            last_run_at: Some(chrono::Utc::now()),
            total_run_count: 1,
        };
        dyn_backend
            .set_task_state("periodic-task", &updated)
            .await
            .unwrap();

        let retrieved = dyn_backend.get_task_state("periodic-task").await.unwrap();
        assert_eq!(retrieved.total_run_count, 1);

        // Graceful shutdown
        dyn_backend.release_leader().await.unwrap();
    }

    // -----------------------------------------------------------------------
    // S4: All three backends coexist (R6)
    // -----------------------------------------------------------------------

    #[test]
    fn s4_memory_and_cloud_backends_coexist() {
        // Verifies R6: MemorySchedulerBackend and CloudSchedulerBackend available simultaneously
        let _memory = MemorySchedulerBackend::new();
        let _cloud = make_test_backend();
    }

    #[tokio::test]
    async fn s4_backends_interchangeable_at_runtime() {
        // Verifies R6: application can select backend at runtime based on configuration
        let backends: Vec<Box<dyn SchedulerBackend>> = vec![
            Box::new(MemorySchedulerBackend::new()),
            Box::new(make_test_backend()),
        ];

        // Both backends work through the same trait interface
        for (i, backend) in backends.iter().enumerate() {
            let acquired = backend
                .acquire_leader(Duration::from_secs(10))
                .await
                .unwrap();
            assert!(acquired, "Backend {} should acquire leadership", i);

            let state = backend.get_task_state("shared-task").await.unwrap();
            assert!(
                state.enabled,
                "Backend {} default state should be enabled",
                i
            );

            let new_state = TaskScheduleState {
                enabled: true,
                last_run_at: Some(chrono::Utc::now()),
                total_run_count: 1,
            };
            backend
                .set_task_state("shared-task", &new_state)
                .await
                .unwrap();

            let retrieved = backend.get_task_state("shared-task").await.unwrap();
            assert_eq!(
                retrieved.total_run_count, 1,
                "Backend {} state should persist",
                i
            );

            backend.release_leader().await.unwrap();
        }
    }

    #[tokio::test]
    async fn s4_backends_independent_state() {
        // Verifies R6: each backend instance maintains independent state
        let memory = MemorySchedulerBackend::new();
        let cloud = make_test_backend();

        // Set state on memory backend
        let state = TaskScheduleState {
            enabled: false,
            last_run_at: None,
            total_run_count: 42,
        };
        memory.set_task_state("task-a", &state).await.unwrap();

        // Cloud backend should have independent default state
        let cloud_state = cloud.get_task_state("task-a").await.unwrap();
        assert!(cloud_state.enabled);
        assert_eq!(cloud_state.total_run_count, 0);

        // Memory backend state unchanged
        let memory_state = memory.get_task_state("task-a").await.unwrap();
        assert!(!memory_state.enabled);
        assert_eq!(memory_state.total_run_count, 42);
    }

    // -----------------------------------------------------------------------
    // R5: No modifications to TaskScheduleState struct
    // -----------------------------------------------------------------------

    #[test]
    fn r5_task_schedule_state_unchanged() {
        // Verify TaskScheduleState still has the expected fields
        let state = TaskScheduleState::default();
        assert!(state.enabled);
        assert!(state.last_run_at.is_none());
        assert_eq!(state.total_run_count, 0);

        // Verify it can be constructed with all fields
        let _state = TaskScheduleState {
            enabled: false,
            last_run_at: Some(chrono::Utc::now()),
            total_run_count: 99,
        };
    }

    // -----------------------------------------------------------------------
    // R6: Feature isolation — cloud-scheduler does not affect other backends
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn r6_memory_backend_unaffected_by_cloud_feature() {
        // Verifies R6: enabling cloud-scheduler does not change MemorySchedulerBackend behavior
        let backend = MemorySchedulerBackend::new();

        // Leadership works as expected (acquire -> true, renew while leader -> true)
        assert!(backend
            .acquire_leader(Duration::from_secs(10))
            .await
            .unwrap());
        assert!(backend.renew_leader(Duration::from_secs(10)).await.unwrap());

        // Task state management works
        backend.record_task_run("test-task").await.unwrap();
        let state = backend.get_task_state("test-task").await.unwrap();
        assert_eq!(state.total_run_count, 1);

        // Release works
        backend.release_leader().await.unwrap();
        // After release, renew should report not leader (Memory backend tracks this)
        assert!(!backend.renew_leader(Duration::from_secs(10)).await.unwrap());
    }

    // -----------------------------------------------------------------------
    // Scheduler Mode Selection: Cross-backend mode tests
    // -----------------------------------------------------------------------

    #[test]
    fn scheduling_mode_reexported_from_scheduler_module() {
        // SchedulingMode is re-exported via mod.rs pub use backend::SchedulingMode
        let _mode: SchedulingMode = SchedulingMode::SelfHosted;
        let _mode2: SchedulingMode = SchedulingMode::ExternalPush;
    }

    #[test]
    fn memory_backend_scheduling_mode_is_self_hosted() {
        // S1/S8: MemorySchedulerBackend uses default → SelfHosted
        let backend = MemorySchedulerBackend::new();
        assert_eq!(backend.scheduling_mode(), SchedulingMode::SelfHosted);
    }

    #[test]
    fn cloud_scheduler_backend_scheduling_mode_is_external_push() {
        // S2/S12: CloudSchedulerBackend overrides → ExternalPush
        let backend = make_test_backend();
        assert_eq!(backend.scheduling_mode(), SchedulingMode::ExternalPush);
    }

    #[test]
    fn mode_decision_table_memory_self_hosted() {
        // Verify Mode Decision Table from spec: Memory → SelfHosted
        let backend = MemorySchedulerBackend::new();
        assert_eq!(backend.scheduling_mode(), SchedulingMode::SelfHosted);
    }

    #[test]
    fn mode_decision_table_cloud_external_push() {
        // Verify Mode Decision Table from spec: Cloud → ExternalPush
        let backend = make_test_backend();
        assert_eq!(backend.scheduling_mode(), SchedulingMode::ExternalPush);
    }
}

/// Tests for scheduler mode selection spec — verifies cross-backend scheduling mode
/// and SchedulingMode re-exports without requiring cloud-scheduler feature.
#[cfg(test)]
mod mode_selection_tests {
    use super::*;

    // -----------------------------------------------------------------------
    // SchedulingMode enum re-export and variant access
    // -----------------------------------------------------------------------

    #[test]
    fn scheduling_mode_reexported() {
        // SchedulingMode should be accessible via scheduler module re-export
        let sh = SchedulingMode::SelfHosted;
        let ep = SchedulingMode::ExternalPush;
        assert_ne!(sh, ep);
    }

    // -----------------------------------------------------------------------
    // S1/S8: MemorySchedulerBackend uses default scheduling_mode → SelfHosted
    // -----------------------------------------------------------------------

    #[test]
    fn s1_memory_backend_default_scheduling_mode() {
        let backend = MemorySchedulerBackend::new();
        assert_eq!(backend.scheduling_mode(), SchedulingMode::SelfHosted);
    }

    // -----------------------------------------------------------------------
    // S8: Default register_external_schedule is no-op for Memory backend
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn s8_memory_backend_register_external_schedule_noop() {
        let backend = MemorySchedulerBackend::new();
        let task = PeriodicTask {
            name: "test-task".to_string(),
            task_name: "handler".to_string(),
            schedule: PeriodicSchedule::Interval(60),
            args: serde_json::json!({}),
            queue: "default".to_string(),
            enabled: true,
        };
        // Default implementation returns Ok(()) — no-op
        let result = backend.register_external_schedule(&task).await;
        assert!(result.is_ok());
    }

    // -----------------------------------------------------------------------
    // PeriodicSchedulerConfig defaults are correct
    // -----------------------------------------------------------------------

    #[test]
    fn config_defaults_are_correct() {
        let config = PeriodicSchedulerConfig::default();
        assert_eq!(config.leader_ttl, std::time::Duration::from_secs(15));
        assert_eq!(config.follower_sleep, std::time::Duration::from_secs(5));
        assert_eq!(
            config.leader_renew_interval,
            std::time::Duration::from_secs(5)
        );
    }
}
