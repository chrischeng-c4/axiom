use std::sync::{atomic::{AtomicU64, Ordering}, Arc};

use storage_durable::{CapacityGuard, CapacityLevel, CapacityThresholds, SpaceProbe};

struct FixedSpace(AtomicU64);

impl FixedSpace {
    fn new(bytes: u64) -> Self {
        Self(AtomicU64::new(bytes))
    }
}

impl SpaceProbe for FixedSpace {
    fn available_space(&self, _root: &std::path::Path) -> std::io::Result<u64> {
        Ok(self.0.load(Ordering::Acquire))
    }
}

#[test]
fn thresholds_and_raii_reservations_are_bounded() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("seed"), vec![0; 69]).unwrap();
    let guard = CapacityGuard::open_with_probe(
        temp.path(),
        100,
        0,
        CapacityThresholds::new(70, 80, 90).unwrap(),
        Arc::new(FixedSpace::new(1_000)),
    )
    .unwrap();
    assert_eq!(guard.level(), CapacityLevel::Normal);

    {
        let _reservation = guard.reserve(1).unwrap();
        assert_eq!(guard.level(), CapacityLevel::Warning);
    }
    assert_eq!(guard.used_bytes(), 69, "drop must release an uncommitted reservation");

    let reservation = guard.reserve(1).unwrap();
    reservation.commit();
    assert_eq!(guard.used_bytes(), 70, "commit keeps the durable byte estimate");
}

#[test]
fn backpressure_and_reconcile_follow_real_disk_usage() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("data"), vec![0; 79]).unwrap();
    let guard = CapacityGuard::open_with_probe(
        temp.path(),
        100,
        0,
        CapacityThresholds::default(),
        Arc::new(FixedSpace::new(1_000)),
    )
    .unwrap();
    assert!(guard.reserve(1).is_err(), "80 percent starts backpressure");

    std::fs::write(temp.path().join("data"), vec![0; 20]).unwrap();
    guard.reconcile().unwrap();
    assert_eq!(guard.used_bytes(), 20);
    assert!(guard.reserve(1).is_ok());
}
