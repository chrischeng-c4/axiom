#![cfg(test)]

/// Integration tests for thread-safe Mamba runtime.
///
/// Tests safepoint-based STW GC, multi-thread async scheduling,
/// and concurrent access to global async state.

use crate::runtime::value::MbValue;
use crate::runtime::rc::MbObject;
use crate::runtime::gc;
use crate::runtime::async_rt;
use crate::runtime::tokio_exec;

#[test]
fn test_safepoint_register_unregister() {
    // Register and unregister a thread — no panic, no deadlock
    gc::gc_register_thread();
    gc::gc_safepoint();
    gc::gc_unregister_thread();
}

#[test]
fn test_safepoint_gc_collect_from_registered_thread() {
    // A registered mutator that triggers collect() must not self-deadlock
    gc::gc_register_thread();

    let obj = MbObject::new_list(vec![]);
    gc::gc_add_root(MbValue::from_ptr(obj));
    let _freed = gc::collect();
    // collect returned successfully = no deadlock
    gc::gc_remove_root(MbValue::from_ptr(obj));
    gc::gc_untrack(obj);
    unsafe { drop(Box::from_raw(obj)); }

    gc::gc_unregister_thread();
}

#[test]
fn test_multi_thread_coroutine_access() {
    // Multiple threads reading/writing coroutine state concurrently
    let name = MbValue::from_ptr(MbObject::new_str("shared".to_string()));
    let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
    let coro = async_rt::mb_coroutine_new(name, locals);

    let coro_bits = coro.to_bits();
    let mut handles = Vec::new();
    for i in 0..4 {
        handles.push(std::thread::spawn(move || {
            let handle = MbValue::from_bits(coro_bits);
            // Read state from different threads
            let state = async_rt::mb_coroutine_get_state(handle);
            assert!(state < u32::MAX, "should read valid state from thread {i}");
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    async_rt::mb_coroutine_release(coro);
}

#[test]
fn test_concurrent_coroutine_creation() {
    // Multiple threads creating coroutines concurrently — IDs must be unique
    let mut handles = Vec::new();
    for _ in 0..4 {
        handles.push(std::thread::spawn(|| {
            let mut ids = Vec::new();
            for _ in 0..100 {
                let name = MbValue::from_ptr(MbObject::new_str("cc".to_string()));
                let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
                let coro = async_rt::mb_coroutine_new(name, locals);
                ids.push(coro.as_int().unwrap());
            }
            ids
        }));
    }

    let mut all_ids: Vec<i64> = Vec::new();
    for h in handles {
        all_ids.extend(h.join().unwrap());
    }
    let total = all_ids.len();
    all_ids.sort();
    all_ids.dedup();
    assert_eq!(all_ids.len(), total, "all coroutine IDs must be unique across threads");
}

#[test]
fn test_tokio_spawn_from_multiple_threads() {
    // Spawn Tokio tasks from multiple threads
    let mut handles = Vec::new();
    for i in 0..4 {
        handles.push(std::thread::spawn(move || {
            let name = MbValue::from_ptr(
                MbObject::new_str(format!("mt_coro_{i}"))
            );
            let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
            let coro = async_rt::mb_coroutine_new(name, locals);
            async_rt::mb_coroutine_complete(coro, MbValue::from_int(i as i64));
            let task = tokio_exec::mb_tokio_spawn(coro);
            task.as_int().unwrap()
        }));
    }

    let task_ids: Vec<i64> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();

    // Give Tokio time to complete
    std::thread::sleep(std::time::Duration::from_millis(100));

    for tid in &task_ids {
        let done = async_rt::TASKS.read().unwrap()
            .get(&(*tid as u64)).map(|t| t.done).unwrap_or(false);
        assert!(done, "Tokio task {tid} should be done");
    }
}

#[test]
fn test_tokio_gather_parallel_execution() {
    let mut coros = Vec::new();
    for i in 0..4 {
        let name = MbValue::from_ptr(
            MbObject::new_str(format!("gather_coro_{i}"))
        );
        let locals = MbValue::from_ptr(MbObject::new_list(vec![]));
        let coro = async_rt::mb_coroutine_new(name, locals);
        async_rt::mb_coroutine_complete(coro, MbValue::from_int((i + 1) * 10));
        coros.push(coro);
    }

    let coro_list = MbValue::from_ptr(MbObject::new_list(coros));
    let results = tokio_exec::mb_tokio_gather(coro_list);

    assert!(results.as_ptr().is_some(), "gather should return a list");
    unsafe {
        if let crate::runtime::rc::ObjData::List(ref lock) = (*results.as_ptr().unwrap()).data {
            let items = lock.read().unwrap();
            assert_eq!(items.len(), 4, "should have 4 results");
        }
    }
}
