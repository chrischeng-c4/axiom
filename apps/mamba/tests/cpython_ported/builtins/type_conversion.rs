//! Ported from Lib/test/test_type_conv_ported.py
//! Integration tests: builtins/type_conversion.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_type_conv_ported.py`.
#[test]
fn test_bool_constructor_truthiness() {
    let out = jit_capture(
        r#"print(bool(0))
print(bool(1))
print(bool(""))
print(bool("x"))
print(bool([]))
print(bool([0]))
"#,
    );
    assert_output(&out, "False\nTrue\nFalse\nTrue\nFalse\nTrue\n");
}

