//! Py3.12 conformance tests for the `heapq` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_heapq.py):
//!   heapify, heappush, heappop, heappushpop, heapreplace, nsmallest, nlargest.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_heapq_heapify_min_heap_property() {
    let out = jit_capture(
        r#"import heapq
h = [9, 5, 7, 3, 1]
heapq.heapify(h)
print(h[0])
"#,
    );
    assert_output(&out, "1\n");
}

#[test]
fn test_heapq_heappop_ascending_order() {
    let out = jit_capture(
        r#"import heapq
h = [3, 1, 4, 1, 5, 9, 2, 6]
heapq.heapify(h)
out = []
while h:
    out.append(heapq.heappop(h))
print(out)
"#,
    );
    assert_output(&out, "[1, 1, 2, 3, 4, 5, 6, 9]\n");
}

#[test]
fn test_heapq_heappush_maintains_heap() {
    let out = jit_capture(
        r#"import heapq
h = []
for x in [5, 2, 8, 1, 9, 3]:
    heapq.heappush(h, x)
print(heapq.heappop(h))
print(heapq.heappop(h))
print(heapq.heappop(h))
"#,
    );
    assert_output(&out, "1\n2\n3\n");
}

#[test]
fn test_heapq_heappushpop_smaller_than_min() {
    let out = jit_capture(
        r#"import heapq
print(heapq.heappushpop([1, 2, 3], 0))
"#,
    );
    assert_output(&out, "0\n");
}

#[test]
fn test_heapq_heapreplace_returns_old_min() {
    let out = jit_capture(
        r#"import heapq
h = [1, 2, 3]
print(heapq.heapreplace(h, 5))
print(h[0])
"#,
    );
    assert_output(&out, "1\n2\n");
}

#[test]
fn test_heapq_nsmallest_three() {
    let out = jit_capture(
        r#"import heapq
print(heapq.nsmallest(3, [5, 1, 9, 2, 8, 4]))
"#,
    );
    assert_output(&out, "[1, 2, 4]\n");
}

#[test]
fn test_heapq_nlargest_three() {
    let out = jit_capture(
        r#"import heapq
print(heapq.nlargest(3, [5, 1, 9, 2, 8, 4]))
"#,
    );
    assert_output(&out, "[9, 8, 5]\n");
}
