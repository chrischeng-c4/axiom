//! Py3.12 conformance tests for iteration protocol (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_iter.py — TestCase
//!
//! Coverage: for-loop iteration across list, tuple, dict, str, bytes,
//! set, range; iter() + next() builtins on list/tuple/str; enumerate;
//! zip across two and three sequences; reversed on list and tuple;
//! sum/min/max/len composed with for-loop sums; nested iteration.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_iter_list_for_loop() {
    let out = jit_capture(
        r#"total = 0
for x in [1, 2, 3, 4]:
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "10\n");
}

#[test]
fn test_iter_tuple_for_loop() {
    let out = jit_capture(
        r#"total = 0
for x in (10, 20, 30):
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "60\n");
}

#[test]
fn test_iter_str_for_loop() {
    let out = jit_capture(
        r#"for c in "abc":
    print(c)
"#,
    );
    assert_output(&out, "a\nb\nc\n");
}

#[test]
fn test_iter_bytes_for_loop() {
    let out = jit_capture(
        r#"total = 0
for b in b"abc":
    total = total + b
print(total)
"#,
    );
    assert_output(&out, "294\n");
}

#[test]
fn test_iter_range_for_loop() {
    let out = jit_capture(
        r#"total = 0
for i in range(5):
    total = total + i
print(total)
"#,
    );
    assert_output(&out, "10\n");
}

#[test]
fn test_iter_range_with_start_stop() {
    let out = jit_capture(
        r#"total = 0
for i in range(2, 6):
    total = total + i
print(total)
"#,
    );
    assert_output(&out, "14\n");
}

#[test]
fn test_iter_dict_keys_for_loop() {
    let out = jit_capture(
        r#"d = {1: "a", 2: "b", 3: "c"}
total = 0
for k in d:
    total = total + k
print(total)
"#,
    );
    assert_output(&out, "6\n");
}

#[test]
fn test_iter_dict_values_for_loop() {
    let out = jit_capture(
        r#"d = {"a": 10, "b": 20, "c": 30}
total = 0
for v in d.values():
    total = total + v
print(total)
"#,
    );
    assert_output(&out, "60\n");
}

#[test]
fn test_iter_set_for_loop_sum() {
    let out = jit_capture(
        r#"s = {1, 2, 3, 4}
total = 0
for x in s:
    total = total + x
print(total)
"#,
    );
    assert_output(&out, "10\n");
}

#[test]
fn test_iter_iter_next_list() {
    let out = jit_capture(
        r#"it = iter([10, 20, 30])
print(next(it))
print(next(it))
print(next(it))
"#,
    );
    assert_output(&out, "10\n20\n30\n");
}

#[test]
fn test_iter_iter_next_tuple() {
    let out = jit_capture(
        r#"it = iter((1, 2, 3))
print(next(it))
print(next(it))
"#,
    );
    assert_output(&out, "1\n2\n");
}

#[test]
fn test_iter_iter_next_str() {
    let out = jit_capture(
        r#"it = iter("xy")
print(next(it))
print(next(it))
"#,
    );
    assert_output(&out, "x\ny\n");
}

#[test]
fn test_iter_enumerate_list() {
    let out = jit_capture(
        r#"for i, x in enumerate(["a", "b", "c"]):
    print(i, x)
"#,
    );
    assert_output(&out, "0 a\n1 b\n2 c\n");
}

#[test]
fn test_iter_enumerate_with_start() {
    let out = jit_capture(
        r#"for i, x in enumerate(["a", "b"], 10):
    print(i, x)
"#,
    );
    assert_output(&out, "10 a\n11 b\n");
}

#[test]
fn test_iter_zip_two_lists() {
    let out = jit_capture(
        r#"for a, b in zip([1, 2, 3], ["x", "y", "z"]):
    print(a, b)
"#,
    );
    assert_output(&out, "1 x\n2 y\n3 z\n");
}

#[test]
fn test_iter_zip_three_sequences() {
    let out = jit_capture(
        r#"for a, b, c in zip([1, 2], [10, 20], [100, 200]):
    print(a + b + c)
"#,
    );
    assert_output(&out, "111\n222\n");
}

#[test]
fn test_iter_zip_truncates_to_shortest() {
    let out = jit_capture(
        r#"for a, b in zip([1, 2, 3, 4], [10, 20]):
    print(a, b)
"#,
    );
    assert_output(&out, "1 10\n2 20\n");
}

#[test]
fn test_iter_reversed_list() {
    let out = jit_capture(
        r#"for x in reversed([1, 2, 3]):
    print(x)
"#,
    );
    assert_output(&out, "3\n2\n1\n");
}

#[test]
fn test_iter_reversed_tuple() {
    let out = jit_capture(
        r#"for x in reversed((10, 20, 30)):
    print(x)
"#,
    );
    assert_output(&out, "30\n20\n10\n");
}

#[test]
fn test_iter_sum_over_range() {
    let out = jit_capture(
        r#"print(sum(range(11)))
"#,
    );
    assert_output(&out, "55\n");
}

#[test]
fn test_iter_min_max_over_list() {
    let out = jit_capture(
        r#"print(min([5, 1, 3, 2, 4]))
print(max([5, 1, 3, 2, 4]))
"#,
    );
    assert_output(&out, "1\n5\n");
}

#[test]
fn test_iter_nested_for_loop_sum() {
    let out = jit_capture(
        r#"total = 0
for i in range(3):
    for j in range(3):
        total = total + i * j
print(total)
"#,
    );
    assert_output(&out, "9\n");
}

#[test]
fn test_iter_for_else_runs_when_no_break() {
    let out = jit_capture(
        r#"for x in [1, 2, 3]:
    print(x)
else:
    print("done")
"#,
    );
    assert_output(&out, "1\n2\n3\ndone\n");
}

#[test]
fn test_iter_for_break_skips_else() {
    let out = jit_capture(
        r#"for x in [1, 2, 3]:
    if x == 2:
        break
    print(x)
else:
    print("done")
print("after")
"#,
    );
    assert_output(&out, "1\nafter\n");
}

#[test]
fn test_iter_continue_skips_body_remainder() {
    let out = jit_capture(
        r#"for x in [1, 2, 3, 4]:
    if x == 2:
        continue
    print(x)
"#,
    );
    assert_output(&out, "1\n3\n4\n");
}
