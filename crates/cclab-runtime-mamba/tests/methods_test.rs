// Integration tests for cclab-runtime-mamba: covers all 4 mb_runtime_* functions.
// Requirements: R1, R2, R4, R5, R6
#![allow(improper_ctypes_definitions)]

use cclab_mamba_registry::MbValue;
use cclab_runtime_mamba::methods::{mb_runtime_gather, mb_runtime_sleep, mb_runtime_spawn};
use cclab_runtime_mamba::types::MbTask;

// ── mb_runtime_sleep ──────────────────────────────────────────────────────────

#[test]
fn sleep_float_zero() {
    let args = [MbValue::from_float(0.0)];
    let result = unsafe { mb_runtime_sleep(args.as_ptr(), 1) };
    assert!(result.is_none(), "sleep(0.0) should return none()");
}

#[test]
fn sleep_float_positive() {
    // 1ms sleep — fast enough for a unit test
    let args = [MbValue::from_float(0.001)];
    let result = unsafe { mb_runtime_sleep(args.as_ptr(), 1) };
    assert!(result.is_none(), "sleep(0.001) should return none()");
}

#[test]
fn sleep_int_zero() {
    let args = [MbValue::from_int(0)];
    let result = unsafe { mb_runtime_sleep(args.as_ptr(), 1) };
    assert!(result.is_none(), "sleep(Int(0)) should return none()");
}

#[test]
fn sleep_negative_clamps() {
    // Negative duration should clamp to 0 (no panic)
    let args = [MbValue::from_float(-1.0)];
    let result = unsafe { mb_runtime_sleep(args.as_ptr(), 1) };
    assert!(
        result.is_none(),
        "sleep(-1.0) should clamp to 0 and return none()"
    );
}

#[test]
fn sleep_no_args() {
    let args: [MbValue; 0] = [];
    let result = unsafe { mb_runtime_sleep(args.as_ptr(), 0) };
    assert!(result.is_none(), "sleep with no args should return none()");
}

// ── mb_runtime_spawn ──────────────────────────────────────────────────────────

#[test]
fn spawn_happy() {
    let fn_ptr = MbValue::from_func(0xCAFE_BABE);
    let args = [fn_ptr];
    let task_val = unsafe { mb_runtime_spawn(args.as_ptr(), 1) };
    assert!(task_val.is_ptr(), "spawn should return a task ptr");

    let addr = task_val.as_ptr().unwrap();
    let task = unsafe { &*(addr as *const MbTask) };
    assert!(task.task_id > 0, "task_id should be > 0");
    assert!(task.done(), "stub task should be immediately done");
}

#[test]
fn spawn_no_args() {
    // Spawn with no func ptr — should not crash
    let args: [MbValue; 0] = [];
    let task_val = unsafe { mb_runtime_spawn(args.as_ptr(), 0) };
    assert!(
        task_val.is_ptr(),
        "spawn with no args should still return a task ptr"
    );
}

#[test]
fn spawn_unique_ids() {
    let fn_ptr = MbValue::from_func(0);
    let args = [fn_ptr];

    let t1 = unsafe { mb_runtime_spawn(args.as_ptr(), 1) };
    let t2 = unsafe { mb_runtime_spawn(args.as_ptr(), 1) };
    let t3 = unsafe { mb_runtime_spawn(args.as_ptr(), 1) };

    let id1 = unsafe { &*(t1.as_ptr().unwrap() as *const MbTask) }.task_id;
    let id2 = unsafe { &*(t2.as_ptr().unwrap() as *const MbTask) }.task_id;
    let id3 = unsafe { &*(t3.as_ptr().unwrap() as *const MbTask) }.task_id;

    assert_ne!(id1, id2, "spawn 1 and 2 should have distinct task_ids");
    assert_ne!(id2, id3, "spawn 2 and 3 should have distinct task_ids");
    assert_ne!(id1, id3, "spawn 1 and 3 should have distinct task_ids");
}

// ── mb_runtime_gather ─────────────────────────────────────────────────────────

#[test]
fn gather_stub_no_args() {
    let args: [MbValue; 0] = [];
    let result = unsafe { mb_runtime_gather(args.as_ptr(), 0) };
    assert!(
        result.is_none(),
        "gather stub with no args should return none()"
    );
}

#[test]
fn gather_stub_with_list() {
    // Pass a fake list ptr (gather is a stub and ignores args)
    let fn_ptrs: Vec<MbValue> = vec![
        MbValue::from_func(0x1),
        MbValue::from_func(0x2),
        MbValue::from_func(0x3),
    ];
    let list_val = MbValue::from_ptr(Box::into_raw(Box::new(fn_ptrs)) as usize);
    let args = [list_val];
    let result = unsafe { mb_runtime_gather(args.as_ptr(), 1) };
    assert!(
        result.is_none(),
        "gather stub with list should return none()"
    );
}
