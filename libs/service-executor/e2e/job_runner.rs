use std::sync::{Arc, Mutex};

use service_executor::{JobRunState, JobRunner, JobState};

#[derive(Default)]
struct RecordingState(Mutex<Vec<String>>);

impl JobState<u64> for RecordingState {
    type Error = &'static str;
    type Id = String;

    fn mark_running(&self, id: &Self::Id) -> Result<(), Self::Error> {
        self.0.lock().unwrap().push(format!("running:{id}"));
        Ok(())
    }

    fn succeed(&self, id: &Self::Id, output: u64) -> Result<(), Self::Error> {
        self.0
            .lock()
            .unwrap()
            .push(format!("succeeded:{id}:{output}"));
        Ok(())
    }

    fn fail(&self, id: &Self::Id, message: String) -> Result<(), Self::Error> {
        self.0
            .lock()
            .unwrap()
            .push(format!("failed:{id}:{message}"));
        Ok(())
    }
}

#[tokio::test]
async fn blocking_job_transitions_are_owned_by_the_runner() {
    let state = Arc::new(RecordingState::default());
    let runner = JobRunner::new(state.clone());
    let report = runner
        .spawn_blocking("job-1".to_string(), 21_u64, |input| {
            Ok::<_, &'static str>(input * 2)
        })
        .await
        .unwrap();
    assert_eq!(report.state, JobRunState::Succeeded);
    assert_eq!(
        *state.0.lock().unwrap(),
        vec!["running:job-1", "succeeded:job-1:42"]
    );
}

#[tokio::test]
async fn async_job_transitions_are_owned_by_the_runner() {
    let state = Arc::new(RecordingState::default());
    let runner = JobRunner::new(state.clone());
    let report = runner
        .spawn_async("job-async".to_string(), 21_u64, |input| async move {
            tokio::task::yield_now().await;
            Ok::<_, &'static str>(input * 2)
        })
        .await
        .unwrap();
    assert_eq!(report.state, JobRunState::Succeeded);
    assert_eq!(
        *state.0.lock().unwrap(),
        vec!["running:job-async", "succeeded:job-async:42"]
    );
}

#[tokio::test]
async fn errors_and_panics_become_explicit_persistent_failures() {
    let state = Arc::new(RecordingState::default());
    let runner = JobRunner::new(state.clone());
    let failed = runner
        .spawn_blocking("job-error".to_string(), (), |_| {
            Err::<u64, _>("query failed")
        })
        .await
        .unwrap();
    assert_eq!(failed.state, JobRunState::Failed);

    let panicked = runner
        .spawn_blocking(
            "job-panic".to_string(),
            (),
            |_| -> Result<u64, &'static str> { panic!("broken query engine") },
        )
        .await
        .unwrap();
    assert_eq!(panicked.state, JobRunState::Failed);
    let transitions = state.0.lock().unwrap().join("\n");
    assert!(transitions.contains("failed:job-error:query failed"));
    assert!(transitions.contains("failed:job-panic:job execution panicked"));
}
