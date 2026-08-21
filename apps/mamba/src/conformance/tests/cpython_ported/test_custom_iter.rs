//! Py3.12 conformance tests for user-defined iterators (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_iter.py — custom
//! iterator section):
//!   class implements `__iter__` + `__next__`, raises `StopIteration`
//!   to terminate, drives a `for` loop end-to-end.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_custom_iter_in_for_loop_countdown() {
    let out = jit_capture(
        r#"class CountDown:
    def __init__(self, n):
        self.n = n
    def __iter__(self):
        return self
    def __next__(self):
        if self.n <= 0:
            raise StopIteration
        self.n -= 1
        return self.n + 1

for v in CountDown(3):
    print(v)
"#,
    );
    assert_output(&out, "3\n2\n1\n");
}

#[test]
fn test_custom_iter_consumed_by_list() {
    let out = jit_capture(
        r#"class Range3:
    def __init__(self):
        self.i = 0
    def __iter__(self):
        return self
    def __next__(self):
        if self.i >= 3:
            raise StopIteration
        v = self.i
        self.i += 1
        return v

print(list(Range3()))
"#,
    );
    assert_output(&out, "[0, 1, 2]\n");
}
