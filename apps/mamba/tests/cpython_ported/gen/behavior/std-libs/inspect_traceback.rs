use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/inspect_traceback/currentframe_yields_frame.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_traceback_currentframe_yields_frame() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "behavior"
# case = "currentframe_yields_frame"
# subject = "inspect.currentframe"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
"""inspect.currentframe: inspect.currentframe() returns the caller's frame object (not None) with a positive f_lineno"""
import inspect

frame = inspect.currentframe()
assert frame is not None
assert frame.f_lineno > 0, frame.f_lineno

print("currentframe_yields_frame OK")
"###);
    assert_output(&out, r###"currentframe_yields_frame OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect_traceback/extract_tb_yields_frames.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_traceback_extract_tb_yields_frames() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "behavior"
# case = "extract_tb_yields_frames"
# subject = "traceback.extract_tb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
"""traceback.extract_tb: inside an except block, traceback.extract_tb(e.__traceback__) walks the live traceback and yields at least one FrameSummary"""
import traceback

try:
    raise ValueError("boom")
except ValueError as exc:
    frames = traceback.extract_tb(exc.__traceback__)
    assert len(frames) >= 1, len(frames)

print("extract_tb_yields_frames OK")
"###);
    assert_output(&out, r###"extract_tb_yields_frames OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect_traceback/format_exception_names_exc.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_traceback_format_exception_names_exc() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "behavior"
# case = "format_exception_names_exc"
# subject = "traceback.format_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
"""traceback.format_exception: inside an except block, traceback.format_exception(*sys.exc_info()) returns a list of str lines and the active exception type name appears among them"""
import sys
import traceback

try:
    raise KeyError("missing")
except KeyError:
    lines = traceback.format_exception(*sys.exc_info())
    assert isinstance(lines, list)
    assert len(lines) > 0
    assert any("KeyError" in line for line in lines), lines

print("format_exception_names_exc OK")
"###);
    assert_output(&out, r###"format_exception_names_exc OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect_traceback/format_stack_nonempty_str_list.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_traceback_format_stack_nonempty_str_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "behavior"
# case = "format_stack_nonempty_str_list"
# subject = "traceback.format_stack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
"""traceback.format_stack: traceback.format_stack() at top level returns a non-empty list whose every entry is a str line of formatted stack text"""
import traceback

stack = traceback.format_stack()
assert isinstance(stack, list)
assert len(stack) > 0
assert all(isinstance(line, str) for line in stack)

print("format_stack_nonempty_str_list OK")
"###);
    assert_output(&out, r###"format_stack_nonempty_str_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect_traceback/print_exc_no_active_exception_no_raise.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_traceback_print_exc_no_active_exception_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "behavior"
# case = "print_exc_no_active_exception_no_raise"
# subject = "traceback.print_exc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
"""traceback.print_exc: traceback.print_exc() with no exception active does not raise; redirected to a StringIO it emits the 'NoneType: None' sentinel and returns None"""
import io
import traceback

buf = io.StringIO()
result = traceback.print_exc(file=buf)
assert result is None
assert "NoneType: None" in buf.getvalue(), repr(buf.getvalue())

print("print_exc_no_active_exception_no_raise OK")
"###);
    assert_output(&out, r###"print_exc_no_active_exception_no_raise OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect_traceback/stack_top_is_module.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_traceback_stack_top_is_module() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "behavior"
# case = "stack_top_is_module"
# subject = "inspect.stack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
"""inspect.stack: inspect.stack() returns a non-empty list of FrameInfo records; the top frame's .function is the running scope name ('<module>' at top level)"""
import inspect

st = inspect.stack()
assert isinstance(st, list)
assert len(st) > 0
top = st[0]
assert isinstance(top.function, str)
assert top.function == "<module>", top.function

print("stack_top_is_module OK")
"###);
    assert_output(&out, r###"stack_top_is_module OK
"###);
}
