use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/traceback/clear_frames_empties_frame_locals.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_clear_frames_empties_frame_locals() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "clear_frames_empties_frame_locals"
# subject = "traceback.clear_frames"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.clear_frames: clear_frames(tb) empties each traceback frame's f_locals: an innermost frame with one local has 0 locals after the call"""
import traceback


def _outer():
    _middle()


def _middle():
    _inner()


def _inner():
    _i = 1
    1 / 0


try:
    _outer()
except ZeroDivisionError as e:
    _tb = e.__traceback__
_innermost = _tb.tb_next.tb_next.tb_next.tb_frame
assert len(_innermost.f_locals) == 1, f"locals before clear = {len(_innermost.f_locals)!r}"
traceback.clear_frames(_tb)
assert len(_innermost.f_locals) == 0, f"locals after clear = {len(_innermost.f_locals)!r}"

print("clear_frames_empties_frame_locals OK")
"###);
    assert_output(&out, r###"clear_frames_empties_frame_locals OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/extract_tb_entries_have_frame_fields.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_extract_tb_entries_have_frame_fields() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "extract_tb_entries_have_frame_fields"
# subject = "traceback.extract_tb"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.extract_tb: extract_tb of a live traceback yields StackSummary entries whose filename/lineno/name have str/int/str types"""
import sys
import traceback

try:
    raise StopIteration("stop")
except StopIteration:
    _ss = traceback.extract_tb(sys.exc_info()[2])
assert len(_ss) >= 1, "stack has frames"
_frame = _ss[-1]
assert isinstance(_frame.filename, str), "filename type"
assert isinstance(_frame.lineno, int), "lineno type"
assert isinstance(_frame.name, str), "name type"

print("extract_tb_entries_have_frame_fields OK")
"###);
    assert_output(&out, r###"extract_tb_entries_have_frame_fields OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/extract_tb_none_returns_empty_summary.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_extract_tb_none_returns_empty_summary() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "extract_tb_none_returns_empty_summary"
# subject = "traceback.extract_tb"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.extract_tb: extract_tb(None) returns a StackSummary of length 0 (no frames)"""
import traceback

assert len(traceback.extract_tb(None)) == 0

print("extract_tb_none_returns_empty_summary OK")
"###);
    assert_output(&out, r###"extract_tb_none_returns_empty_summary OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/format_exc_captures_active_exception.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_format_exc_captures_active_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exc_captures_active_exception"
# subject = "traceback.format_exc"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exc: inside an except block, format_exc() includes the 'Traceback (most recent call last):' header and the 'RuntimeError: runtime msg' type+message"""
import traceback

try:
    raise RuntimeError("runtime msg")
except RuntimeError:
    _fe = traceback.format_exc()
assert "RuntimeError: runtime msg" in _fe, f"format_exc has type+msg: {_fe!r}"
assert "Traceback (most recent call last):" in _fe, "has Traceback header"

print("format_exc_captures_active_exception OK")
"###);
    assert_output(&out, r###"format_exc_captures_active_exception OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/format_exc_outside_except_is_none_sentinel.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_format_exc_outside_except_is_none_sentinel() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exc_outside_except_is_none_sentinel"
# subject = "traceback.format_exc"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exc: format_exc() with no active exception returns the sentinel 'NoneType: None\\n'"""
import traceback

_outside = traceback.format_exc()
assert _outside == "NoneType: None\n", f"outside = {_outside!r}"

print("format_exc_outside_except_is_none_sentinel OK")
"###);
    assert_output(&out, r###"format_exc_outside_except_is_none_sentinel OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/format_exception_one_arg_instance_form.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_format_exception_one_arg_instance_form() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exception_one_arg_instance_form"
# subject = "traceback.format_exception"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception: the 3.12 single-arg form accepts a bare exception instance: format_exception(Exception('projector')) and format_exception_only(...) both yield ['Exception: projector\\n']"""
import traceback

_ex = Exception("projector")
assert traceback.format_exception(_ex) == ["Exception: projector\n"], "1-arg format_exception"
assert traceback.format_exception_only(_ex) == ["Exception: projector\n"], "1-arg format_exception_only"

print("format_exception_one_arg_instance_form OK")
"###);
    assert_output(&out, r###"format_exception_one_arg_instance_form OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/format_exception_only_bare_baseexception_type_only.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_format_exception_only_bare_baseexception_type_only() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exception_only_bare_baseexception_type_only"
# subject = "traceback.format_exception_only"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception_only: a BaseException with no message (KeyboardInterrupt()) renders type-only ['KeyboardInterrupt\\n'] with no trailing ': message'"""
import traceback

_kbi = KeyboardInterrupt()
assert traceback.format_exception_only(_kbi.__class__, _kbi) == ["KeyboardInterrupt\n"], \
    "bare BaseException renders type-only"

print("format_exception_only_bare_baseexception_type_only OK")
"###);
    assert_output(&out, r###"format_exception_only_bare_baseexception_type_only OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/format_exception_only_renders_type_and_message.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_format_exception_only_renders_type_and_message() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exception_only_renders_type_and_message"
# subject = "traceback.format_exception_only"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception_only: format_exception_only(ValueError, ValueError('bad value')) returns a single line 'ValueError: bad value\\n'"""
import traceback

_lines = traceback.format_exception_only(ValueError, ValueError("bad value"))
assert len(_lines) == 1, f"exception_only lines = {len(_lines)!r}"
assert _lines[0] == "ValueError: bad value\n", f"format = {_lines[0]!r}"

print("format_exception_only_renders_type_and_message OK")
"###);
    assert_output(&out, r###"format_exception_only_renders_type_and_message OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/format_exception_only_swallows_broken_str.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_format_exception_only_swallows_broken_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exception_only_swallows_broken_str"
# subject = "traceback.format_exception_only"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception_only: when an exception's __str__ raises, format_exception_only swallows it and renders '<exception str() failed>' on a single line"""
import traceback


class _BadStr(Exception):
    def __str__(self):
        1 / 0


_err = traceback.format_exception_only(_BadStr, _BadStr())
assert len(_err) == 1, f"bad-str lines = {len(_err)!r}"
assert "<exception str() failed>" in _err[0], f"bad-str render: {_err[0]!r}"

print("format_exception_only_swallows_broken_str OK")
"###);
    assert_output(&out, r###"format_exception_only_swallows_broken_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/format_exception_only_syntaxerror_three_lines.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_format_exception_only_syntaxerror_three_lines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exception_only_syntaxerror_three_lines"
# subject = "traceback.format_exception_only"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception_only: a SyntaxError with (file, line, col, text) renders three lines: File location, the source line ('bad syntax'), then 'SyntaxError: error\\n'"""
import traceback

_se = SyntaxError("error", ("x.py", 23, None, "bad syntax"))
_se_lines = traceback.format_exception_only(SyntaxError, _se)
assert len(_se_lines) == 3, f"syntaxerror lines = {len(_se_lines)!r}"
assert _se_lines[1].strip() == "bad syntax", f"source line: {_se_lines[1]!r}"
assert _se_lines[-1] == "SyntaxError: error\n", f"final line: {_se_lines[-1]!r}"

print("format_exception_only_syntaxerror_three_lines OK")
"###);
    assert_output(&out, r###"format_exception_only_syntaxerror_three_lines OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/format_exception_three_arg_returns_str_list.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_format_exception_three_arg_returns_str_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exception_three_arg_returns_str_list"
# subject = "traceback.format_exception"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception: the 3-arg form format_exception(type, value, tb) of a live IndexError returns a list of str whose join contains 'IndexError: idx'"""
import sys
import traceback

try:
    raise IndexError("idx")
except IndexError:
    _exc_type, _exc_val, _exc_tb = sys.exc_info()
    _parts = traceback.format_exception(_exc_type, _exc_val, _exc_tb)
assert isinstance(_parts, list), f"format_exception type = {type(_parts)!r}"
_combined = "".join(_parts)
assert "IndexError: idx" in _combined, f"IndexError in format_exception: {_combined!r}"

print("format_exception_three_arg_returns_str_list OK")
"###);
    assert_output(&out, r###"format_exception_three_arg_returns_str_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/format_list_empty_returns_empty_list.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_format_list_empty_returns_empty_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_list_empty_returns_empty_list"
# subject = "traceback.format_list"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_list: format_list([]) over an empty stack returns an empty list (no lines)"""
import traceback

_res = traceback.format_list([])
assert _res == [], f"empty format_list = {_res!r}"

print("format_list_empty_returns_empty_list OK")
"###);
    assert_output(&out, r###"format_list_empty_returns_empty_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/format_tb_includes_source_and_raise.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_format_tb_includes_source_and_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_tb_includes_source_and_raise"
# subject = "traceback.format_tb"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_tb: format_tb of a live traceback joins to a string containing a '.py' filename and the offending 'raise TypeError' source statement"""
import sys
import traceback

try:
    raise TypeError("tb check")
except TypeError:
    _tb = sys.exc_info()[2]
    _tb_str = "".join(traceback.format_tb(_tb))
assert ".py" in _tb_str, f"format_tb has filename: {_tb_str!r}"
assert "raise TypeError" in _tb_str, f"format_tb has raise statement: {_tb_str!r}"

print("format_tb_includes_source_and_raise OK")
"###);
    assert_output(&out, r###"format_tb_includes_source_and_raise OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/framesummary_missing_line_is_none.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_framesummary_missing_line_is_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "framesummary_missing_line_is_none"
# subject = "traceback.FrameSummary"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.FrameSummary: FrameSummary('f', None, 'dummy') with no lineno and no explicit line has .line is None (no source lookup)"""
import traceback

g = traceback.FrameSummary("f", None, "dummy")
assert g.line is None, f"missing line = {g.line!r}"

print("framesummary_missing_line_is_none OK")
"###);
    assert_output(&out, r###"framesummary_missing_line_is_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/framesummary_stores_explicit_fields.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_framesummary_stores_explicit_fields() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "framesummary_stores_explicit_fields"
# subject = "traceback.FrameSummary"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.FrameSummary: FrameSummary('f', 1, 'dummy', line='line') stores filename/lineno/name/line verbatim and reports len() == 4 logical fields"""
import traceback

f = traceback.FrameSummary("f", 1, "dummy", line="line")
assert f.line == "line", f"explicit line = {f.line!r}"
assert f.filename == "f", f"filename = {f.filename!r}"
assert f.lineno == 1, f"lineno = {f.lineno!r}"
assert f.name == "dummy", f"name = {f.name!r}"
assert len(f) == 4, f"len(FrameSummary) = {len(f)!r}"

print("framesummary_stores_explicit_fields OK")
"###);
    assert_output(&out, r###"framesummary_stores_explicit_fields OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/limit_tests__test_extract_stack.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_limit_tests__test_extract_stack() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "limit_tests__test_extract_stack"
# subject = "cpython.test_traceback.LimitTests.test_extract_stack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
import unittest, io
from test import test_traceback
_suite = unittest.defaultTestLoader.loadTestsFromName("LimitTests.test_extract_stack", test_traceback)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LimitTests.test_extract_stack did not pass"
print("LimitTests::test_extract_stack: ok")
"###);
    assert_output(&out, r###"LimitTests::test_extract_stack: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/misc_test__test_levenshtein_distance.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_misc_test__test_levenshtein_distance() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "misc_test__test_levenshtein_distance"
# subject = "cpython.test_traceback.MiscTest.test_levenshtein_distance"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
from collections import namedtuple
from io import StringIO
import linecache
import sys
import types
import inspect
import builtins
import re
import tempfile
import random
import string
import shutil
import json
import textwrap
import traceback
from functools import partial
from pathlib import Path

def CHECK(a, b, expected):
    actual = traceback._levenshtein_distance(a, b, 4044)
    assert actual == expected
CHECK('', '', 0)
CHECK('', 'a', 2)
CHECK('a', 'A', 1)
CHECK('Apple', 'Aple', 2)
CHECK('Banana', 'B@n@n@', 6)
CHECK('Cherry', 'Cherry!', 2)
CHECK('---0---', '------', 2)
CHECK('abc', 'y', 6)
CHECK('aa', 'bb', 4)
CHECK('aaaaa', 'AAAAA', 5)
CHECK('wxyz', 'wXyZ', 2)
CHECK('wxyz', 'wXyZ123', 8)
CHECK('Python', 'Java', 12)
CHECK('Java', 'C#', 8)
CHECK('AbstractFoobarManager', 'abstract_foobar_manager', 3 + 2 * 2)
CHECK('CPython', 'PyPy', 10)
CHECK('CPython', 'pypy', 11)
CHECK('AttributeError', 'AttributeErrop', 2)
CHECK('AttributeError', 'AttributeErrorTests', 10)
CHECK('ABA', 'AAB', 4)

print("MiscTest::test_levenshtein_distance: ok")
"###);
    assert_output(&out, r###"MiscTest::test_levenshtein_distance: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/misc_traceback_cases__test_extract_stack.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_misc_traceback_cases__test_extract_stack() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "misc_traceback_cases__test_extract_stack"
# subject = "cpython.test_traceback.MiscTracebackCases.test_extract_stack"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
import unittest, io
from test import test_traceback
_suite = unittest.defaultTestLoader.loadTestsFromName("MiscTracebackCases.test_extract_stack", test_traceback)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MiscTracebackCases.test_extract_stack did not pass"
print("MiscTracebackCases::test_extract_stack: ok")
"###);
    assert_output(&out, r###"MiscTracebackCases::test_extract_stack: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/none_exception_renders_sentinel_across_apis.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_none_exception_renders_sentinel_across_apis() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "none_exception_renders_sentinel_across_apis"
# subject = "traceback.format_exception"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception: a None exception renders 'NoneType: None\\n' across format_exc(None), format_exception(None)/(None,None,None), and format_exception_only(None)/(None,None)"""
import traceback

_NONE = "NoneType: None\n"
assert traceback.format_exc(None) == _NONE, "format_exc(None)"
assert traceback.format_exception(None) == [_NONE], "format_exception(None)"
assert traceback.format_exception(None, None, None) == [_NONE], "format_exception(None,None,None)"
assert traceback.format_exception_only(None) == [_NONE], "format_exception_only(None)"
assert traceback.format_exception_only(None, None) == [_NONE], "format_exception_only(None,None)"

print("none_exception_renders_sentinel_across_apis OK")
"###);
    assert_output(&out, r###"none_exception_renders_sentinel_across_apis OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/print_exc_no_active_exception_returns_none.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_print_exc_no_active_exception_returns_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "print_exc_no_active_exception_returns_none"
# subject = "traceback.print_exc"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.print_exc: print_exc(file=StringIO()) with no active exception returns None (the return-value contract is the parity oracle)"""
import io
import traceback

assert traceback.print_exc(file=io.StringIO()) is None

print("print_exc_no_active_exception_returns_none OK")
"###);
    assert_output(&out, r###"print_exc_no_active_exception_returns_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/print_exc_writes_to_stream.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_print_exc_writes_to_stream() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "print_exc_writes_to_stream"
# subject = "traceback.print_exc"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.print_exc: print_exc(file=StringIO()) inside an except block writes the exception type 'KeyError' and message 'key_msg' to the given stream"""
import io
import traceback

_buf = io.StringIO()
try:
    raise KeyError("key_msg")
except KeyError:
    traceback.print_exc(file=_buf)
_out = _buf.getvalue()
assert "KeyError" in _out, f"print_exc KeyError: {_out!r}"
assert "key_msg" in _out, f"print_exc message: {_out!r}"

print("print_exc_writes_to_stream OK")
"###);
    assert_output(&out, r###"print_exc_writes_to_stream OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/print_exception_three_arg_and_one_arg_forms.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_print_exception_three_arg_and_one_arg_forms() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "print_exception_three_arg_and_one_arg_forms"
# subject = "traceback.print_exception"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.print_exception: print_exception accepts both the 3-arg (type, value, None) and 1-arg (instance) forms; both emit 'Exception: projector\\n' to the stream"""
import io
import traceback

_o3 = io.StringIO()
traceback.print_exception(Exception, Exception("projector"), None, file=_o3)
assert _o3.getvalue() == "Exception: projector\n", f"3-arg print: {_o3.getvalue()!r}"
_o1 = io.StringIO()
traceback.print_exception(Exception("projector"), file=_o1)
assert _o1.getvalue() == "Exception: projector\n", f"1-arg print: {_o1.getvalue()!r}"

print("print_exception_three_arg_and_one_arg_forms OK")
"###);
    assert_output(&out, r###"print_exception_three_arg_and_one_arg_forms OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/stacksummary_entries_are_mutable_and_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_stacksummary_entries_are_mutable_and_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "stacksummary_entries_are_mutable_and_roundtrip"
# subject = "traceback.StackSummary"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.StackSummary: StackSummary entries are mutable; editing s[0]'s lineno and feeding it back through from_list re-formats with the new line number"""
import traceback

s = traceback.StackSummary.from_list([("foo.py", 1, "fred", "line")])
s[0] = ("foo.py", 2, "fred", "line")
s2 = traceback.StackSummary.from_list(s)
assert s2.format() == ['  File "foo.py", line 2, in fred\n    line\n'], \
    f"edited format = {s2.format()!r}"

print("stacksummary_entries_are_mutable_and_roundtrip OK")
"###);
    assert_output(&out, r###"stacksummary_entries_are_mutable_and_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/stacksummary_extract_capture_locals.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_stacksummary_extract_capture_locals() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "stacksummary_extract_capture_locals"
# subject = "traceback.StackSummary"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.StackSummary: StackSummary.extract(..., capture_locals=True) stores frame locals as repr-strings ({'something': '1'}); without the flag .locals is None"""
import sys
import traceback


def make_frame():
    something = 1
    return sys._getframe()


fr = make_frame()
with_locals = traceback.StackSummary.extract(iter([(fr, fr.f_lineno)]), capture_locals=True)
assert with_locals[0].locals == {"something": "1"}, f"locals = {with_locals[0].locals!r}"
without = traceback.StackSummary.extract(iter([(fr, fr.f_lineno)]))
assert without[0].locals is None, f"no-capture locals = {without[0].locals!r}"

print("stacksummary_extract_capture_locals OK")
"###);
    assert_output(&out, r###"stacksummary_extract_capture_locals OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/stacksummary_from_list_formats_one_block_per_frame.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_stacksummary_from_list_formats_one_block_per_frame() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "stacksummary_from_list_formats_one_block_per_frame"
# subject = "traceback.StackSummary"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.StackSummary: StackSummary.from_list([('foo.py', 1, 'fred', 'line')]).format() yields one '  File "foo.py", line 1, in fred\\n    line\\n' block"""
import traceback

s = traceback.StackSummary.from_list([("foo.py", 1, "fred", "line")])
assert s.format() == ['  File "foo.py", line 1, in fred\n    line\n'], \
    f"from_list format = {s.format()!r}"

print("stacksummary_from_list_formats_one_block_per_frame OK")
"###);
    assert_output(&out, r###"stacksummary_from_list_formats_one_block_per_frame OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/test_stack__test_extract_stack_limit.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_test_stack__test_extract_stack_limit() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "test_stack__test_extract_stack_limit"
# subject = "cpython.test_traceback.TestStack.test_extract_stack_limit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
import unittest, io
from test import test_traceback
_suite = unittest.defaultTestLoader.loadTestsFromName("TestStack.test_extract_stack_limit", test_traceback)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestStack.test_extract_stack_limit did not pass"
print("TestStack::test_extract_stack_limit: ok")
"###);
    assert_output(&out, r###"TestStack::test_extract_stack_limit: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/tracebackexception_equality_between_equivalent_captures.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_tracebackexception_equality_between_equivalent_captures() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "tracebackexception_equality_between_equivalent_captures"
# subject = "traceback.TracebackException"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.TracebackException: two from_exception captures of the same exception are distinct objects (is not) yet compare equal (==), and differ from an unrelated object"""
import traceback

try:
    1 / 0
except ZeroDivisionError as e:
    te = traceback.TracebackException.from_exception(e)
    te_again = traceback.TracebackException.from_exception(e)
assert te is not te_again, "captures are distinct objects"
assert te == te_again, "equivalent captures compare equal"
assert te != object(), "unrelated object compares unequal"

print("tracebackexception_equality_between_equivalent_captures OK")
"###);
    assert_output(&out, r###"tracebackexception_equality_between_equivalent_captures OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/tracebackexception_from_exception_attrs.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_tracebackexception_from_exception_attrs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "tracebackexception_from_exception_attrs"
# subject = "traceback.TracebackException"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.TracebackException: from_exception(e) of a ZeroDivisionError exposes exc_type, a matching str(), and a clean chain (__cause__/__context__ None, __suppress_context__ False)"""
import traceback

try:
    1 / 0
except ZeroDivisionError as e:
    exc_obj = e
    te = traceback.TracebackException.from_exception(e)
assert te.exc_type is ZeroDivisionError, f"exc_type = {te.exc_type!r}"
assert str(te) == str(exc_obj), f"str = {str(te)!r}"
assert te.__cause__ is None, f"cause = {te.__cause__!r}"
assert te.__context__ is None, f"context = {te.__context__!r}"
assert te.__suppress_context__ is False, f"suppress = {te.__suppress_context__!r}"

print("tracebackexception_from_exception_attrs OK")
"###);
    assert_output(&out, r###"tracebackexception_from_exception_attrs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/tracebackexception_header_only_one_line.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_tracebackexception_header_only_one_line() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "tracebackexception_header_only_one_line"
# subject = "traceback.TracebackException"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.TracebackException: a header-only TracebackException(Exception, Exception('haven'), None) with no traceback formats to the single line 'Exception: haven\\n'"""
import traceback

header = traceback.TracebackException(Exception, Exception("haven"), None)
assert list(header.format()) == ["Exception: haven\n"], \
    f"header format = {list(header.format())!r}"

print("tracebackexception_header_only_one_line OK")
"###);
    assert_output(&out, r###"tracebackexception_header_only_one_line OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/tracebackexception_raise_from_none_suppresses_context.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_tracebackexception_raise_from_none_suppresses_context() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "tracebackexception_raise_from_none_suppresses_context"
# subject = "traceback.TracebackException"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.TracebackException: 'raise RuntimeError(...) from None' inside an except sets __suppress_context__ True on the from_exception capture, and chained format shows 'RuntimeError: chained'"""
import traceback

try:
    try:
        raise ValueError("orig")
    except ValueError:
        raise RuntimeError("chained") from None
except RuntimeError as e:
    suppressed = traceback.TracebackException.from_exception(e)
    _formatted = "".join(suppressed.format())
assert suppressed.__suppress_context__ is True, \
    f"from-None suppress = {suppressed.__suppress_context__!r}"
assert "RuntimeError: chained" in _formatted, f"chained: {_formatted!r}"

print("tracebackexception_raise_from_none_suppresses_context OK")
"###);
    assert_output(&out, r###"tracebackexception_raise_from_none_suppresses_context OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/walk_stack_returns_iterator.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_walk_stack_returns_iterator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "walk_stack_returns_iterator"
# subject = "traceback.walk_stack"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.walk_stack: walk_stack(None) starts from the current frame and returns an iterable (has __iter__)"""
import traceback

_gen = traceback.walk_stack(None)
assert hasattr(_gen, "__iter__"), f"walk_stack iterable: {_gen!r}"

print("walk_stack_returns_iterator OK")
"###);
    assert_output(&out, r###"walk_stack_returns_iterator OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/traceback/walk_tb_yields_one_pair_per_level.py`.
#[test]
fn test_gen_behavior_std_libs_traceback_walk_tb_yields_one_pair_per_level() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "walk_tb_yields_one_pair_per_level"
# subject = "traceback.walk_tb"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.walk_tb: walk_tb yields one (frame, lineno) pair per traceback level: a single-frame ZeroDivisionError traceback has exactly one entry"""
import traceback

try:
    1 / 0
except ZeroDivisionError as e:
    _tb = e.__traceback__
assert len(list(traceback.walk_tb(_tb))) == 1, "walk_tb single-frame"

print("walk_tb_yields_one_pair_per_level OK")
"###);
    assert_output(&out, r###"walk_tb_yields_one_pair_per_level OK
"###);
}
