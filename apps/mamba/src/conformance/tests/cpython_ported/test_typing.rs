//! Py3.12 conformance tests for the `typing` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_typing.py):
//!   typing imports are accepted, annotated functions execute correctly,
//!   isinstance and type().__name__ still report concrete runtime types.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_typing_annotated_function_call() {
    let out = jit_capture(
        r#"from typing import List
def f(x: int) -> int:
    return x + 1
print(f(5))
"#,
    );
    assert_output(&out, "6\n");
}

#[test]
fn test_typing_isinstance_with_typing_import() {
    let out = jit_capture(
        r#"from typing import Optional
x = 42
print(isinstance(x, int))
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_typing_runtime_type_name() {
    let out = jit_capture(
        r#"from typing import Union
print(type(5).__name__)
print(type("a").__name__)
print(type([]).__name__)
"#,
    );
    assert_output(&out, "int\nstr\nlist\n");
}

#[test]
fn test_typing_list_annotation_on_var() {
    let out = jit_capture(
        r#"from typing import List
xs: List[int] = [1, 2, 3]
print(xs)
print(len(xs))
"#,
    );
    assert_output(&out, "[1, 2, 3]\n3\n");
}
