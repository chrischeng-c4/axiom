//! Py3.12 conformance tests for recursive function patterns
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_funcattrs.py /
//! test_grammar.py — recursion sections): naive recursive Fibonacci,
//! recursive countdown using `print`, and a recursive list-sum.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_recursive_fibonacci_table() {
    let out = jit_capture(
        r#"def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

results = []
for i in range(10):
    results.append(fib(i))
print(results)
print(fib(15))
"#,
    );
    assert_output(&out, "[0, 1, 1, 2, 3, 5, 8, 13, 21, 34]\n610\n");
}

#[test]
fn test_recursive_countdown_print() {
    let out = jit_capture(
        r#"def countdown(n):
    if n == 0:
        print("go")
        return
    print(n, end=" ")
    countdown(n - 1)

countdown(5)
"#,
    );
    assert_output(&out, "5 4 3 2 1 go\n");
}

#[test]
fn test_recursive_list_sum() {
    let out = jit_capture(
        r#"def rsum(xs):
    if len(xs) == 0:
        return 0
    return xs[0] + rsum(xs[1:])

print(rsum([]))
print(rsum([7]))
print(rsum([1, 2, 3, 4, 5]))
print(rsum([10, -5, 3, -2]))
"#,
    );
    assert_output(&out, "0\n7\n15\n6\n");
}
