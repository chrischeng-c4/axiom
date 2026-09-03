use std::sync::Arc;

use super::*;
use crate::wal::{CommitCoordinator, WalStore};

fn wal_backed_state(dir: &std::path::Path) -> AppState {
    let (store, journal) = WalStore::open(dir).expect("open WAL store");
    let state = AppState::new(journal, None, 8 * 1024 * 1024);
    let coordinator = Arc::new(CommitCoordinator::spawn(store, state.journal_handle()));
    state.with_wal(coordinator)
}

#[tokio::test]
async fn missing_startup_subscription_is_recovered_from_wal() {
    let dir = tempfile::tempdir().unwrap();
    let state = wal_backed_state(dir.path());

    let outcome = state
        .provision_startup_subscription("orders".into(), "billing".into())
        .await
        .expect("current startup seam mutates the in-memory journal");
    assert!(matches!(
        outcome,
        TapeOutcome::SubscriptionCreated(Ok(subscription)) if subscription.topic == "orders" && subscription.name == "billing"
    ));
    drop(state);

    let (store, recovered) = WalStore::open(dir.path()).expect("reopen WAL store");
    drop(store);
    assert!(
        recovered.subscription("orders", "billing").is_some(),
        "a startup-created subscription must survive WAL recovery"
    );
}

#[tokio::test]
async fn recovered_startup_subscription_does_not_append_a_second_wal_frame() {
    let dir = tempfile::tempdir().unwrap();
    let state = wal_backed_state(dir.path());
    state
        .apply_mutation(TapeCommand::SubscriptionCreate {
            topic: "orders".into(),
            name: "billing".into(),
        })
        .await
        .expect("seed subscription through WAL");
    drop(state);

    let state = wal_backed_state(dir.path());
    assert!(
        state
            .journal_handle()
            .lock()
            .unwrap()
            .subscription("orders", "billing")
            .is_some(),
        "test precondition: the seed must come from WAL recovery"
    );
    let wal_path = dir.path().join("journal.wal");
    let before = std::fs::metadata(&wal_path).unwrap().len();

    let outcome = state
        .provision_startup_subscription("orders".into(), "billing".into())
        .await
        .expect("current startup seam returns the domain outcome");
    assert!(matches!(
        outcome,
        TapeOutcome::SubscriptionCreated(Err(SubscriptionError::AlreadyExists { .. }))
    ));
    assert_eq!(
        std::fs::metadata(&wal_path).unwrap().len(),
        before,
        "an already recovered subscription must not append a second WAL frame"
    );
}
