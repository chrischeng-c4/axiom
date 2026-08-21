//! Py3.12 conformance tests for basic generator functions
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_generators.py —
//! basic yield sections): iterating a generator, materializing
//! one to a `list`, and summing the values it produces.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_generator_for_loop() {
    let out = jit_capture(
        r#"def count_up_to(n):
    i = 0
    while i < n:
        yield i
        i = i + 1

for v in count_up_to(5):
    print(v)
"#,
    );
    assert_output(&out, "0\n1\n2\n3\n4\n");
}

#[test]
fn test_generator_materialized_to_list() {
    let out = jit_capture(
        r#"def count_up_to(n):
    i = 0
    while i < n:
        yield i
        i = i + 1

print(list(count_up_to(4)))
"#,
    );
    assert_output(&out, "[0, 1, 2, 3]\n");
}

#[test]
fn test_generator_sum_and_list() {
    let out = jit_capture(
        r#"def squares(n):
    for i in range(n):
        yield i * i

print(sum(squares(5)))
print(list(squares(4)))
"#,
    );
    assert_output(&out, "30\n[0, 1, 4, 9]\n");
}
