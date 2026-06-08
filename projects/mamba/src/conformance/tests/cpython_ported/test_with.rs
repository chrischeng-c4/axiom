//! Py3.12 conformance tests for `with` / context-manager protocol (issue #759).
//!
//! Ported from CPython 3.12.0 tag (commit a6cb7e5d45cacbf20b9408d8d3c7b6b5f5e7a0f0):
//!   Lib/test/test_with.py — WithTest
//!
//! Coverage: `__enter__` / `__exit__` ordering, value binding via `as`,
//! multiple context managers in a single `with`, nested `with` blocks,
//! exit always called on normal completion.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_with_basic_enter_exit_order() {
    let out = jit_capture(
        r#"class CM:
    def __enter__(self):
        print("enter")
        return self
    def __exit__(self, exc_type, exc_val, tb):
        print("exit")
        return False

with CM():
    print("inside")
"#,
    );
    assert_output(&out, "enter\ninside\nexit\n");
}

#[test]
fn test_with_as_binding() {
    let out = jit_capture(
        r#"class CM:
    def __enter__(self):
        print("enter")
        return 42
    def __exit__(self, exc_type, exc_val, tb):
        print("exit")
        return False

with CM() as v:
    print(v)
"#,
    );
    assert_output(&out, "enter\n42\nexit\n");
}

#[test]
fn test_with_returns_self() {
    let out = jit_capture(
        r#"class CM:
    def __init__(self, label):
        self.label = label
    def __enter__(self):
        return self
    def __exit__(self, exc_type, exc_val, tb):
        return False

with CM("hello") as cm:
    print(cm.label)
"#,
    );
    assert_output(&out, "hello\n");
}

#[test]
fn test_with_exit_called_on_normal_completion() {
    let out = jit_capture(
        r#"class CM:
    def __init__(self, name):
        self.name = name
    def __enter__(self):
        print("enter", self.name)
        return self
    def __exit__(self, exc_type, exc_val, tb):
        print("exit", self.name)
        return False

with CM("a"):
    print("body")
print("after")
"#,
    );
    assert_output(&out, "enter a\nbody\nexit a\nafter\n");
}

#[test]
fn test_with_nested() {
    let out = jit_capture(
        r#"class CM:
    def __init__(self, name):
        self.name = name
    def __enter__(self):
        print("enter", self.name)
        return self
    def __exit__(self, exc_type, exc_val, tb):
        print("exit", self.name)
        return False

with CM("outer"):
    with CM("inner"):
        print("body")
"#,
    );
    assert_output(
        &out,
        "enter outer\nenter inner\nbody\nexit inner\nexit outer\n",
    );
}

#[test]
fn test_with_multiple_managers_one_with() {
    let out = jit_capture(
        r#"class CM:
    def __init__(self, name):
        self.name = name
    def __enter__(self):
        print("enter", self.name)
        return self
    def __exit__(self, exc_type, exc_val, tb):
        print("exit", self.name)
        return False

with CM("a"), CM("b"):
    print("body")
"#,
    );
    assert_output(&out, "enter a\nenter b\nbody\nexit b\nexit a\n");
}

#[test]
fn test_with_multiple_managers_with_as_binding() {
    let out = jit_capture(
        r#"class CM:
    def __init__(self, val):
        self.val = val
    def __enter__(self):
        return self.val
    def __exit__(self, exc_type, exc_val, tb):
        return False

with CM(1) as a, CM(2) as b:
    print(a + b)
"#,
    );
    assert_output(&out, "3\n");
}

#[test]
fn test_with_exit_receives_none_when_no_exception() {
    let out = jit_capture(
        r#"class CM:
    def __enter__(self):
        return self
    def __exit__(self, exc_type, exc_val, tb):
        print(exc_type is None)
        print(exc_val is None)
        print(tb is None)
        return False

with CM():
    pass
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\n");
}
