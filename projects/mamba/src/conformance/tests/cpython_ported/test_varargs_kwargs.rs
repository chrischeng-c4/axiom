//! Py3.12 conformance tests for `*args` and `**kwargs` (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_funcattrs.py /
//! test_grammar.py — variadic parameter sections): variadic
//! positional `*args` (zero, one, many), variadic keyword `**kwargs`,
//! and combining a fixed positional with `*args`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_star_args_arity() {
    let out = jit_capture(
        r#"def variadic(*args):
    return (len(args), sum(args))

print(variadic())
print(variadic(10))
print(variadic(1, 2, 3))
print(variadic(1, 2, 3, 4, 5))
"#,
    );
    assert_output(&out, "(0, 0)\n(1, 10)\n(3, 6)\n(5, 15)\n");
}

#[test]
fn test_kwargs_only() {
    let out = jit_capture(
        r#"def kw_only(**kwargs):
    return sorted(kwargs.items())

print(kw_only())
print(kw_only(a=1, b=2))
print(kw_only(x=10, y=20, z=30))
"#,
    );
    assert_output(
        &out,
        "[]\n[('a', 1), ('b', 2)]\n[('x', 10), ('y', 20), ('z', 30)]\n",
    );
}

#[test]
fn test_fixed_then_star_args() {
    let out = jit_capture(
        r#"def head_rest(first, *rest):
    return (first, list(rest))

print(head_rest(10))
print(head_rest(10, 20))
print(head_rest("a", "b", "c", "d"))
"#,
    );
    assert_output(&out, "(10, [])\n(10, [20])\n('a', ['b', 'c', 'd'])\n");
}
