//! Py3.12 conformance tests for nested `for` loops and accumulators
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_grammar.py — loop
//! sections):
//!   nested `for` accumulators, matrix-style printing with `end=" "`,
//!   and a unique-pair builder over a small list.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_nested_accumulator() {
    let out = jit_capture(
        r#"total = 0
for i in range(3):
    for j in range(3):
        total = total + 1
print(total)
"#,
    );
    assert_output(&out, "9\n");
}

#[test]
fn test_matrix_print_with_end_space() {
    let out = jit_capture(
        r#"for i in range(3):
    for j in range(3):
        print(i * 3 + j, end=" ")
    print()
"#,
    );
    assert_output(&out, "0 1 2 \n3 4 5 \n6 7 8 \n");
}

#[test]
fn test_pair_combinations() {
    let out = jit_capture(
        r#"items = [1, 2, 3, 4]
pairs = []
for i in range(len(items)):
    for j in range(i + 1, len(items)):
        pairs.append((items[i], items[j]))
print(pairs)
"#,
    );
    assert_output(&out, "[(1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4)]\n");
}
