//! Ported from Lib/test/test_typing_ported.py
//! Integration tests: stdlib/typing.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_typing_ported.py`.
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

/// Ported from `Lib/test/test_typing_ported.py`.
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

/// Ported from `Lib/test/test_typing_ported.py`.
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

/// Ported from `Lib/test/test_typing_ported.py`.
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

