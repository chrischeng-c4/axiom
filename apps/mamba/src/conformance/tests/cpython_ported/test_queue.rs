//! Py3.12 conformance tests for the `queue` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_queue.py):
//!   FIFO Queue: put, get, qsize, empty.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_queue_fifo_order() {
    let out = jit_capture(
        r#"import queue
q = queue.Queue()
q.put(1)
q.put(2)
q.put(3)
print(q.get())
print(q.get())
print(q.get())
"#,
    );
    assert_output(&out, "1\n2\n3\n");
}

#[test]
fn test_queue_qsize_tracks_inserts() {
    let out = jit_capture(
        r#"import queue
q = queue.Queue()
q.put(1)
q.put(2)
q.put(3)
print(q.qsize())
q.get()
print(q.qsize())
"#,
    );
    assert_output(&out, "3\n2\n");
}

#[test]
fn test_queue_empty_state_transitions() {
    let out = jit_capture(
        r#"import queue
q = queue.Queue()
print(q.empty())
q.put("a")
print(q.empty())
q.get()
print(q.empty())
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\n");
}
