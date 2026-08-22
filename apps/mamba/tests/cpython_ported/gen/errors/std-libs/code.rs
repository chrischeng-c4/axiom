use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/code/code_replace_nlocals_high_raises.py`.
#[test]
fn test_gen_errors_std_libs_code_code_replace_nlocals_high_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "errors"
# case = "code_replace_nlocals_high_raises"
# subject = "types.CodeType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: code_replace_nlocals_high_raises (errors)."""
import types

_raised = False
try:
    (lambda a, b: a + b).__code__.replace(co_nlocals=3)
except ValueError:
    _raised = True
assert _raised, "code_replace_nlocals_high_raises: expected ValueError"
print("code_replace_nlocals_high_raises OK")
"###);
    assert_output(&out, r###"code_replace_nlocals_high_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/code/code_replace_nlocals_low_raises.py`.
#[test]
fn test_gen_errors_std_libs_code_code_replace_nlocals_low_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "errors"
# case = "code_replace_nlocals_low_raises"
# subject = "types.CodeType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: code_replace_nlocals_low_raises (errors)."""
import types

_raised = False
try:
    (lambda a, b: a + b).__code__.replace(co_nlocals=1)
except ValueError:
    _raised = True
assert _raised, "code_replace_nlocals_low_raises: expected ValueError"
print("code_replace_nlocals_low_raises OK")
"###);
    assert_output(&out, r###"code_replace_nlocals_low_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/code/compile_command_bad_syntax_raises.py`.
#[test]
fn test_gen_errors_std_libs_code_compile_command_bad_syntax_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "errors"
# case = "compile_command_bad_syntax_raises"
# subject = "code.compile_command"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""code.compile_command: compile_command_bad_syntax_raises (errors)."""
import code

_raised = False
try:
    code.compile_command("def (:")
except SyntaxError:
    _raised = True
assert _raised, "compile_command_bad_syntax_raises: expected SyntaxError"
print("compile_command_bad_syntax_raises OK")
"###);
    assert_output(&out, r###"compile_command_bad_syntax_raises OK
"###);
}
