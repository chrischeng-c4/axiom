//! Ported from Lib/test/test_re_ported.py
//! Integration tests: stdlib/re.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_fnmatch_star_extension() {
    let out = jit_capture(
        r#"import fnmatch
print(fnmatch.fnmatch("foo.py", "*.py"))
print(fnmatch.fnmatch("foo.txt", "*.py"))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_fnmatch_question_mark() {
    let out = jit_capture(
        r#"import fnmatch
print(fnmatch.fnmatch("ab", "?b"))
print(fnmatch.fnmatch("abc", "?b"))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_fnmatch_filter_only_py() {
    let out = jit_capture(
        r#"import fnmatch
print(fnmatch.filter(["a.py", "b.txt", "c.py", "d.md"], "*.py"))
"#,
    );
    assert_output(&out, "['a.py', 'c.py']\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_fnmatch_char_class() {
    let out = jit_capture(
        r#"import fnmatch
print(fnmatch.fnmatch("a1", "a[0-9]"))
print(fnmatch.fnmatch("aa", "a[0-9]"))
"#,
    );
    assert_output(&out, "True\nFalse\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_reversed_list_str_range() {
    let out = jit_capture(
        r#"print(list(reversed([1, 2, 3])))
print(list(reversed("hello")))
print(list(reversed(range(5))))
"#,
    );
    assert_output(
        &out,
        "[3, 2, 1]\n['o', 'l', 'l', 'e', 'h']\n[4, 3, 2, 1, 0]\n",
    );
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_sorted_reverse_true() {
    let out = jit_capture(
        r#"print(sorted([3, 1, 4, 1, 5, 9, 2, 6], reverse=True))
"#,
    );
    assert_output(&out, "[9, 6, 5, 4, 3, 2, 1, 1]\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_sorted_with_key_lambda() {
    let out = jit_capture(
        r#"print(sorted([(1, "b"), (3, "a"), (2, "c")], key=lambda p: p[1]))
"#,
    );
    assert_output(&out, "[(3, 'a'), (1, 'b'), (2, 'c')]\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_textwrap_dedent_uniform_indent() {
    let out = jit_capture(
        r#"import textwrap
print(textwrap.dedent("    line1\n    line2"))
"#,
    );
    assert_output(&out, "line1\nline2\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_textwrap_dedent_no_indent_passthrough() {
    let out = jit_capture(
        r#"import textwrap
print(textwrap.dedent("line1\nline2"))
"#,
    );
    assert_output(&out, "line1\nline2\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_textwrap_dedent_empty_string() {
    let out = jit_capture(
        r#"import textwrap
print(repr(textwrap.dedent("")))
"#,
    );
    assert_output(&out, "''\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_repr_quotes_string_and_canonicalizes_scalars() {
    let out = jit_capture(
        r#"print(repr("hello"))
print(repr(42))
print(repr(None))
print(repr(True))
"#,
    );
    assert_output(&out, "'hello'\n42\nNone\nTrue\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_repr_containers_match_literal_forms() {
    let out = jit_capture(
        r#"print(repr([1, 2, 3]))
print(repr((1, 2)))
print(repr({1: 2}))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n(1, 2)\n{1: 2}\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_str_of_scalars_and_containers() {
    let out = jit_capture(
        r#"print(str(42))
print(str([1, 2, 3]))
print(str(None))
"#,
    );
    assert_output(&out, "42\n[1, 2, 3]\nNone\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_re_match_group_one() {
    let out = jit_capture(
        r#"import re
m = re.match(r"hello (\w+)", "hello world")
print(m.group(1))
"#,
    );
    assert_output(&out, "world\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_re_search_first_hit() {
    let out = jit_capture(
        r#"import re
m = re.search(r"\d+", "abc 42 def")
print(m.group(0))
"#,
    );
    assert_output(&out, "42\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_re_sub_replace_all() {
    let out = jit_capture(
        r#"import re
print(re.sub(r"\d+", "X", "abc123def456"))
"#,
    );
    assert_output(&out, "abcXdefX\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_re_findall_numbers() {
    let out = jit_capture(
        r#"import re
print(re.findall(r"\d+", "a1 b22 c333"))
"#,
    );
    assert_output(&out, "['1', '22', '333']\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_re_split_on_commas() {
    let out = jit_capture(
        r#"import re
print(re.split(r",\s*", "a, b,c,  d"))
"#,
    );
    assert_output(&out, "['a', 'b', 'c', 'd']\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
#[test]
fn test_re_fullmatch_anchored() {
    let out = jit_capture(
        r#"import re
print(re.fullmatch(r"\d+", "1234") is not None)
print(re.fullmatch(r"\d+", "12a4") is None)
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

/// Ported from `Lib/test/test_re_ported.py`.
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

/// Ported from `Lib/test/test_re_ported.py`.
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

/// Ported from `Lib/test/test_re_ported.py`.
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

