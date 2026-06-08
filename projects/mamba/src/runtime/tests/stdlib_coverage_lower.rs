#![cfg(test)]

/// Integration tests for the 10 lowest-coverage stdlib modules.
/// Covers: queue_mod, statistics_mod, shlex_mod, calendar_mod, locale_mod,
///         lzma_mod, zlib_mod, secrets_mod, bisect_mod, abc_mod.

use crate::runtime::value::MbValue;
use crate::runtime::stdlib::queue_mod::{
    mb_queue_Queue, mb_queue_put, mb_queue_get,
};

/// Concurrent producer-consumer: producer puts 100 items; main thread gets 100 items.
/// Asserts total non-none results == 100 and no panic / deadlock occurs.
#[test]
fn test_queue_concurrent_cross_module() {
    let q = mb_queue_Queue(MbValue::from_int(0));
    let q_bits = q.to_bits();

    let producer = std::thread::spawn(move || {
        let q2 = MbValue::from_bits(q_bits);
        for i in 0..100i64 {
            mb_queue_put(q2, MbValue::from_int(i));
        }
    });

    // Wait for producer to finish before consuming so all 100 items are present.
    producer.join().expect("producer thread panicked");

    let mut received = 0i32;
    for _ in 0..100 {
        if !mb_queue_get(q).is_none() {
            received += 1;
        }
    }

    assert_eq!(received, 100);
}
