//! Colocated unit tests for the runtime, plus the process-wide-registry and
//! refcount-instrument tests added for issue #4243.
//!
//! Four runtime registries (generators/iterators, open files, hashlib state,
//! and `random.Random` state) used to live in `thread_local!` blocks, while a
//! handle handed to Python is a plain integer index into the table. That works
//! only while every handle is created and consumed on one OS thread: a handle
//! made in the parent thread indexes an *empty* table in a child, so the child
//! sees a bare integer rather than the object. Closures already avoided this by
//! living in the process-wide `program_state().closures`
//! (`crate::runtime::closure`), which is the pattern the four registries move
//! to.
//!
//! The tests below do not run Python — the end-to-end observation is owned by
//! `apps/mamba/e2e/tier1_cross_thread_identity.rs`. They pin the property that
//! makes those fixtures work: a handle minted on one OS thread resolves on
//! another.

mod adversarial_refcount_challenge;
mod async_gen_event_loop_interleaving_gate;
mod base64_memory_gate;
mod container_lock_perf;
mod generator_runtime_type_gate;
mod jit_refcount_audit;
mod list_literal_perf;
mod list_sort_builtin_perf_gate;
mod pymalloc_freelist;
mod runtime_core;
mod runtime_integration;
mod stdlib_coverage_lower;
mod stdlib_coverage_remaining;
mod string_concat_perf_gate;
mod thread_safety;

use crate::runtime::file_io;
use crate::runtime::iter;
use crate::runtime::rc::{self, MbObject};
use crate::runtime::stdlib::hashlib_mod;
use crate::runtime::stdlib::random_mod;
use crate::runtime::value::MbValue;

/// Run `f` on a freshly spawned OS thread and return its value, failing the
/// test if that thread panicked.
fn on_another_thread<T, F>(f: F) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    std::thread::spawn(f)
        .join()
        .expect("the child OS thread panicked")
}

/// An iterator handle minted on the parent thread must still resolve on a
/// child thread. Backs
/// `e2e/tier1/concurrency/identity/generator/next_in_child_yields_all_values.py`,
/// where a child that cannot resolve the handle sees a bare int and reports
/// `TypeError: 'int' object is not iterable`.
#[test]
fn iterator_handle_created_in_parent_resolves_in_child() {
    let seq = MbValue::from_ptr(MbObject::new_str("abc".to_string()));
    let handle = iter::mb_iter(seq);
    assert!(
        iter::mb_is_iterator_handle(handle),
        "the creating thread must see its own iterator handle"
    );

    let seen_in_child = on_another_thread(move || iter::mb_is_iterator_handle(handle));

    assert!(
        seen_in_child,
        "an iterator handle created on the parent thread did not resolve on a \
         child thread; the iterator registry is still thread-local, so the \
         child indexes an empty table and the handle degrades to a bare int"
    );
}

/// An open-file handle minted on the parent thread must still resolve on a
/// child thread. Backs
/// `e2e/tier1/concurrency/identity/io/parent_file_handle_writes_in_child.py`,
/// where the child otherwise reports
/// `AttributeError: 'int' object has no attribute 'write'`.
#[test]
fn file_handle_created_in_parent_resolves_in_child() {
    let dir = std::env::temp_dir().join(format!(
        "mamba_cross_thread_file_{}_{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).expect("could not create the scratch directory");
    let path = dir.join("parent_opened.txt");

    let path_val = MbValue::from_ptr(MbObject::new_str(path.to_string_lossy().into_owned()));
    let mode_val = MbValue::from_ptr(MbObject::new_str("w".to_string()));
    let handle = file_io::mb_open(path_val, mode_val);
    let id = handle
        .as_int()
        .expect("open() must return an integer file handle") as u64;
    assert!(
        file_io::is_file_handle(id),
        "the creating thread must see its own file handle"
    );

    let seen_in_child = on_another_thread(move || file_io::is_file_handle(id));

    file_io::mb_file_close(handle);
    let _ = std::fs::remove_dir_all(&dir);

    assert!(
        seen_in_child,
        "an open-file handle created on the parent thread did not resolve on a \
         child thread; the open-file registry is still thread-local"
    );
}

/// A hashlib handle minted on the parent thread must still resolve on a child
/// thread. Backs
/// `e2e/tier1/concurrency/identity/hashlib/parent_hash_object_hexdigests_in_child.py`,
/// where the child otherwise reports
/// `TypeError: 'NoneType' object is not callable`.
#[test]
fn hash_handle_created_in_parent_resolves_in_child() {
    let handle = hashlib_mod::mb_hashlib_new_handle("sha256", MbValue::none());
    let id = handle
        .as_int()
        .expect("hashlib.sha256() must return an integer handle") as u64;
    assert!(
        hashlib_mod::is_hashlib_handle(id),
        "the creating thread must see its own hashlib handle"
    );

    let seen_in_child = on_another_thread(move || hashlib_mod::is_hashlib_handle(id));

    assert!(
        seen_in_child,
        "a hashlib handle created on the parent thread did not resolve on a \
         child thread; the hashlib registry is still thread-local"
    );
}

/// A `random.Random` handle minted on the parent thread must still resolve on
/// a child thread. Backs
/// `e2e/tier1/concurrency/identity/random/parent_random_instance_draws_in_child.py`,
/// where the child otherwise reports
/// `TypeError: 'NoneType' object is not callable`.
#[test]
fn random_handle_created_in_parent_resolves_in_child() {
    let handle = random_mod::test_only_new_random_handle(1234);
    let id = handle
        .as_int()
        .expect("Random(seed) must return an integer handle") as u64;
    assert!(
        random_mod::is_random_handle(id),
        "the creating thread must see its own Random handle"
    );

    let seen_in_child = on_another_thread(move || random_mod::is_random_handle(id));

    assert!(
        seen_in_child,
        "a random.Random handle created on the parent thread did not resolve on \
         a child thread; the Random registry is still thread-local"
    );
}

/// The `debug_assertions`-only refcount-delta instrument must not fire because
/// *another* thread retained the same object.
///
/// `store_owned` reads the refcount, calls `retain_if_ptr`, then reads it
/// again and asserts the delta is exactly +1. Those two reads are not one
/// atomic step, so a second OS thread retaining the same object in between
/// makes a perfectly correct atomic refcount look wrong. In the real runtime
/// that surfaced as
/// `store_owned: expected refcount delta +1 for 0x…, before=2601, after=2603`
/// killing roughly 5 % of the `e2e/tier1/concurrency/atomicity/**` runs — a
/// broken instrument, not a broken product, and one that blocks every gate in
/// this Milestone at random.
///
/// Two threads hammering one shared object is exactly that race, concentrated.
#[test]
fn store_owned_instrument_tolerates_a_second_thread_retaining_the_same_object() {
    let shared = MbValue::from_ptr(MbObject::new_str("contended".to_string()));

    let workers: Vec<_> = (0..2)
        .map(|_| {
            std::thread::spawn(move || {
                for _ in 0..200_000 {
                    let owned = rc::store_owned(shared);
                    rc::release_owned(owned);
                }
            })
        })
        .collect();

    for (i, worker) in workers.into_iter().enumerate() {
        assert!(
            worker.join().is_ok(),
            "worker {i} panicked inside store_owned: the debug refcount-delta \
             instrument read a count that the other thread changed between its \
             two reads"
        );
    }
}
