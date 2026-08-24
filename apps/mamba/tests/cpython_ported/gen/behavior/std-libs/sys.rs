use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/sys/active_exception_tests__test_exc_info_no_exception.py`.
#[test]
fn test_gen_behavior_std_libs_sys_active_exception_tests__test_exc_info_no_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "active_exception_tests__test_exc_info_no_exception"
# subject = "cpython.test_sys.ActiveExceptionTests.test_exc_info_no_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_sys.py::ActiveExceptionTests::test_exc_info_no_exception
"""Auto-ported test: ActiveExceptionTests::test_exc_info_no_exception (CPython 3.12 oracle)."""


import builtins
import codecs
import gc
import io
import locale
import operator
import os
import random
import struct
import subprocess
import sys
import sysconfig
import test.support
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, assert_python_failure
from test.support import threading_helper
from test.support import import_helper
import textwrap
import unittest
import warnings


try:
    from test.support import interpreters
except ImportError:
    interpreters = None

def requires_subinterpreters(func):
    deco = unittest.skipIf(interpreters is None, 'Test requires subinterpreters')
    return deco(func)

DICT_KEY_STRUCT_FORMAT = 'n2BI2n'


# --- test body ---

assert sys.exc_info() == (None, None, None)
print("ActiveExceptionTests::test_exc_info_no_exception: ok")
"###);
    assert_output(&out, r###"ActiveExceptionTests::test_exc_info_no_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/active_exception_tests__test_sys_exception_no_exception.py`.
#[test]
fn test_gen_behavior_std_libs_sys_active_exception_tests__test_sys_exception_no_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "active_exception_tests__test_sys_exception_no_exception"
# subject = "cpython.test_sys.ActiveExceptionTests.test_sys_exception_no_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_sys.py::ActiveExceptionTests::test_sys_exception_no_exception
"""Auto-ported test: ActiveExceptionTests::test_sys_exception_no_exception (CPython 3.12 oracle)."""


import builtins
import codecs
import gc
import io
import locale
import operator
import os
import random
import struct
import subprocess
import sys
import sysconfig
import test.support
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, assert_python_failure
from test.support import threading_helper
from test.support import import_helper
import textwrap
import unittest
import warnings


try:
    from test.support import interpreters
except ImportError:
    interpreters = None

def requires_subinterpreters(func):
    deco = unittest.skipIf(interpreters is None, 'Test requires subinterpreters')
    return deco(func)

DICT_KEY_STRUCT_FORMAT = 'n2BI2n'


# --- test body ---

assert sys.exception() == None
print("ActiveExceptionTests::test_sys_exception_no_exception: ok")
"###);
    assert_output(&out, r###"ActiveExceptionTests::test_sys_exception_no_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/asyncgen_hooks_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_sys_asyncgen_hooks_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "asyncgen_hooks_roundtrip"
# subject = "sys.set_asyncgen_hooks"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.set_asyncgen_hooks: asyncgen hooks default to None and round-trip through set_asyncgen_hooks(firstiter=)/(finalizer=) and get_asyncgen_hooks(), then restore"""
import sys

old = sys.get_asyncgen_hooks()
assert old.firstiter is None and old.finalizer is None, "asyncgen hooks start None"
_first = lambda *a: None
_final = lambda *a: None
try:
    sys.set_asyncgen_hooks(firstiter=_first)
    h = sys.get_asyncgen_hooks()
    assert h.firstiter is _first and h[0] is _first, "firstiter set"
    assert h.finalizer is None and h[1] is None, "finalizer untouched"
    sys.set_asyncgen_hooks(finalizer=_final)
    h = sys.get_asyncgen_hooks()
    assert h.firstiter is _first and h.finalizer is _final, "both hooks set"
finally:
    sys.set_asyncgen_hooks(*old)
assert sys.get_asyncgen_hooks().firstiter is None, "asyncgen hooks restored"
print("asyncgen_hooks_roundtrip OK")
"###);
    assert_output(&out, r###"asyncgen_hooks_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/byteorder_consistent_with_struct.py`.
#[test]
fn test_gen_behavior_std_libs_sys_byteorder_consistent_with_struct() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "byteorder_consistent_with_struct"
# subject = "sys.byteorder"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.byteorder: sys.byteorder is 'little' or 'big' and packing 1 as a 4-byte unsigned int with the matching endian prefix yields bytes"""
import sys
import struct

assert sys.byteorder in ("little", "big"), f"byteorder = {sys.byteorder!r}"
if sys.byteorder == "little":
    _packed = struct.pack("<I", 1)
else:
    _packed = struct.pack(">I", 1)
assert isinstance(_packed, bytes), "struct pack consistent with byteorder"
print("byteorder_consistent_with_struct OK")
"###);
    assert_output(&out, r###"byteorder_consistent_with_struct OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/display_hook_test__test_gh130163.py`.
#[test]
fn test_gen_behavior_std_libs_sys_display_hook_test__test_gh130163() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "display_hook_test__test_gh130163"
# subject = "cpython.test_sys.DisplayHookTest.test_gh130163"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_sys.py::DisplayHookTest::test_gh130163
"""Auto-ported test: DisplayHookTest::test_gh130163 (CPython 3.12 oracle)."""


import builtins
import codecs
import gc
import io
import locale
import operator
import os
import random
import struct
import subprocess
import sys
import sysconfig
import test.support
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, assert_python_failure
from test.support import threading_helper
from test.support import import_helper
import textwrap
import unittest
import warnings


try:
    from test.support import interpreters
except ImportError:
    interpreters = None

def requires_subinterpreters(func):
    deco = unittest.skipIf(interpreters is None, 'Test requires subinterpreters')
    return deco(func)

DICT_KEY_STRUCT_FORMAT = 'n2BI2n'


# --- test body ---
class X:

    def __repr__(self):
        sys.stdout = io.StringIO()
        support.gc_collect()
        return 'foo'
with support.swap_attr(sys, 'stdout', None):
    sys.stdout = io.StringIO()
    sys.displayhook(X())
print("DisplayHookTest::test_gh130163: ok")
"###);
    assert_output(&out, r###"DisplayHookTest::test_gh130163: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/displayhook_none_writes_nothing.py`.
#[test]
fn test_gen_behavior_std_libs_sys_displayhook_none_writes_nothing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "displayhook_none_writes_nothing"
# subject = "sys.__displayhook__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__displayhook__: __displayhook__(None) writes nothing and leaves builtins._ unset"""
import builtins
import io
import sys
from contextlib import redirect_stdout

if hasattr(builtins, "_"):
    del builtins._
buf = io.StringIO()
with redirect_stdout(buf):
    sys.__displayhook__(None)
assert buf.getvalue() == "", f"displayhook(None) wrote {buf.getvalue()!r}"
assert not hasattr(builtins, "_"), "displayhook(None) left builtins._ unset"
print("displayhook_none_writes_nothing OK")
"###);
    assert_output(&out, r###"displayhook_none_writes_nothing OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/displayhook_prints_repr_binds_underscore.py`.
#[test]
fn test_gen_behavior_std_libs_sys_displayhook_prints_repr_binds_underscore() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "displayhook_prints_repr_binds_underscore"
# subject = "sys.__displayhook__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__displayhook__: __displayhook__(42) writes '42\\n' to stdout and binds builtins._ to 42 for a non-None value"""
import builtins
import io
import sys
from contextlib import redirect_stdout

buf = io.StringIO()
with redirect_stdout(buf):
    sys.__displayhook__(42)
assert buf.getvalue() == "42\n", f"displayhook(42) wrote {buf.getvalue()!r}"
assert builtins._ == 42, f"builtins._ = {builtins._!r}"
del builtins._
print("displayhook_prints_repr_binds_underscore OK")
"###);
    assert_output(&out, r###"displayhook_prints_repr_binds_underscore OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/exc_info_inside_handler.py`.
#[test]
fn test_gen_behavior_std_libs_sys_exc_info_inside_handler() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "exc_info_inside_handler"
# subject = "sys.exc_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.exc_info: inside an except ValueError block, exc_info()[0] is ValueError and str(exc_info()[1]) is the raised message"""
import sys

try:
    raise ValueError("test_error")
except ValueError:
    _et, _ev, _etb = sys.exc_info()
    assert _et is ValueError, f"exc_type in handler = {_et!r}"
    assert str(_ev) == "test_error", f"exc_val = {str(_ev)!r}"
print("exc_info_inside_handler OK")
"###);
    assert_output(&out, r###"exc_info_inside_handler OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/exc_info_links_traceback.py`.
#[test]
fn test_gen_behavior_std_libs_sys_exc_info_links_traceback() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "exc_info_links_traceback"
# subject = "sys.exc_info"
# kind = "semantic"
# xfail = "mamba e.__traceback__ is None / exc_info linkage incomplete (repo-memory project_mamba_module_exec_del_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.exc_info: exc_info() returns (ValueError, the caught instance, the instance's __traceback__) for a live exception"""
import sys


def _raise():
    raise ValueError(42)


try:
    _raise()
except ValueError as _e:
    _t, _v, _tb = sys.exc_info()
    assert _t is ValueError, f"exc_info[0] = {_t!r}"
    assert _v is _e, "exc_info[1] is the caught instance"
    assert _tb is _e.__traceback__, "exc_info[2] is e.__traceback__"
print("exc_info_links_traceback OK")
"###);
    assert_output(&out, r###"exc_info_links_traceback OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/exc_info_none_outside_handler.py`.
#[test]
fn test_gen_behavior_std_libs_sys_exc_info_none_outside_handler() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "exc_info_none_outside_handler"
# subject = "sys.exc_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.exc_info: outside any except block, exc_info() returns (None, None, None)"""
import sys

_exc_type, _exc_val, _exc_tb = sys.exc_info()
assert _exc_type is None, f"exc_type outside = {_exc_type!r}"
assert _exc_val is None, f"exc_val outside = {_exc_val!r}"
assert _exc_tb is None, f"exc_tb outside = {_exc_tb!r}"
print("exc_info_none_outside_handler OK")
"###);
    assert_output(&out, r###"exc_info_none_outside_handler OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/excepthook_formats_active_exception.py`.
#[test]
fn test_gen_behavior_std_libs_sys_excepthook_formats_active_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "excepthook_formats_active_exception"
# subject = "sys.__excepthook__"
# kind = "semantic"
# xfail = "mamba exc_info / traceback formatting incomplete (repo-memory project_mamba_traceback_format_exc_stub)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__excepthook__: __excepthook__(*exc_info()) for an active ValueError(42) writes a traceback to stderr ending in 'ValueError: 42\\n'"""
import io
import sys
from contextlib import redirect_stderr

try:
    raise ValueError(42)
except ValueError:
    err = io.StringIO()
    with redirect_stderr(err):
        sys.__excepthook__(*sys.exc_info())
assert err.getvalue().endswith("ValueError: 42\n"), \
    f"excepthook tail = {err.getvalue()[-40:]!r}"
print("excepthook_formats_active_exception OK")
"###);
    assert_output(&out, r###"excepthook_formats_active_exception OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/exception_none_outside_live_inside.py`.
#[test]
fn test_gen_behavior_std_libs_sys_exception_none_outside_live_inside() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "exception_none_outside_live_inside"
# subject = "sys.exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.exception: sys.exception() is None outside a handler and is the caught instance inside an except ValueError block (3.11+)"""
import sys


def _raise():
    raise ValueError(42)


assert sys.exception() is None, f"exception() outside = {sys.exception()!r}"
try:
    _raise()
except ValueError as _e2:
    assert sys.exception() is _e2, "exception() returns the caught instance"
print("exception_none_outside_live_inside OK")
"###);
    assert_output(&out, r###"exception_none_outside_live_inside OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/file_attr_omitted.py`.
#[test]
fn test_gen_behavior_std_libs_sys_file_attr_omitted() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "file_attr_omitted"
# subject = "sys.__file__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__file__: sys has no __file__ attribute at all (a frozen/statically-linked builtin), matching CPython"""
import sys

assert not hasattr(sys, "__file__")
print("file_attr_omitted OK")
"###);
    assert_output(&out, r###"file_attr_omitted OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/flags_documented_fields.py`.
#[test]
fn test_gen_behavior_std_libs_sys_flags_documented_fields() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "flags_documented_fields"
# subject = "sys.flags"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.flags: every documented sys.flags field exists with the right type (bool for dev_mode/safe_path, int otherwise) and utf8_mode is 0/1/2"""
import sys

flag_names = (
    "debug", "inspect", "interactive", "optimize", "dont_write_bytecode",
    "no_user_site", "no_site", "ignore_environment", "verbose",
    "bytes_warning", "quiet", "hash_randomization", "isolated",
    "dev_mode", "utf8_mode", "warn_default_encoding", "safe_path",
    "int_max_str_digits",
)
for name in flag_names:
    assert hasattr(sys.flags, name), f"sys.flags missing {name}"
    expected = bool if name in ("dev_mode", "safe_path") else int
    assert type(getattr(sys.flags, name)) is expected, \
        f"flag {name} type = {type(getattr(sys.flags, name))!r}"
assert sys.flags.utf8_mode in (0, 1, 2), f"utf8_mode = {sys.flags.utf8_mode!r}"
print("flags_documented_fields OK")
"###);
    assert_output(&out, r###"flags_documented_fields OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/float_info_shape.py`.
#[test]
fn test_gen_behavior_std_libs_sys_float_info_shape() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "float_info_shape"
# subject = "sys.float_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.float_info: float_info has 11 fields, radix 2, and a positive max"""
import sys

assert len(sys.float_info) == 11, f"float_info len = {len(sys.float_info)!r}"
assert sys.float_info.radix == 2, f"float_info.radix = {sys.float_info.radix!r}"
assert sys.float_info.max > 0, "float_info.max positive"
print("float_info_shape OK")
"###);
    assert_output(&out, r###"float_info_shape OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/getdefaultencoding_is_utf8.py`.
#[test]
fn test_gen_behavior_std_libs_sys_getdefaultencoding_is_utf8() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "getdefaultencoding_is_utf8"
# subject = "sys.getdefaultencoding"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.getdefaultencoding: sys.getdefaultencoding() returns 'utf-8' on CPython 3.12"""
import sys

assert sys.getdefaultencoding() == "utf-8", \
    f"getdefaultencoding = {sys.getdefaultencoding()!r}"
print("getdefaultencoding_is_utf8 OK")
"###);
    assert_output(&out, r###"getdefaultencoding_is_utf8 OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/getsizeof_default_fallback.py`.
#[test]
fn test_gen_behavior_std_libs_sys_getsizeof_default_fallback() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "getsizeof_default_fallback"
# subject = "sys.getsizeof"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.getsizeof: the second positional arg is a default size: ignored for a normal object (real size returned), but returned when the object's __sizeof__ raises"""
import sys


class _NoSize:
    def __sizeof__(self):
        raise TypeError("boom")


# A normal object reports its real size; the default arg is ignored.
_real = sys.getsizeof(object(), 1234)
assert isinstance(_real, int) and _real > 0 and _real != 1234, \
    f"normal object size = {_real!r}"
# When __sizeof__ raises, getsizeof returns the supplied default instead.
assert sys.getsizeof(_NoSize(), 1234) == 1234, "default returned on __sizeof__ failure"
print("getsizeof_default_fallback OK")
"###);
    assert_output(&out, r###"getsizeof_default_fallback OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/getsizeof_positive_int.py`.
#[test]
fn test_gen_behavior_std_libs_sys_getsizeof_positive_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "getsizeof_positive_int"
# subject = "sys.getsizeof"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.getsizeof: getsizeof returns a positive int for both a small int and an empty list"""
import sys

_sz_int = sys.getsizeof(0)
_sz_list = sys.getsizeof([])
assert _sz_int > 0 and isinstance(_sz_int, int), f"int size = {_sz_int!r}"
assert _sz_list > 0 and isinstance(_sz_list, int), f"list size = {_sz_list!r}"
print("getsizeof_positive_int OK")
"###);
    assert_output(&out, r###"getsizeof_positive_int OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/hash_info_shape.py`.
#[test]
fn test_gen_behavior_std_libs_sys_hash_info_shape() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "hash_info_shape"
# subject = "sys.hash_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.hash_info: hash_info has 9 fields, a modulus that fits in 'width' bits, and a known algorithm name (fnv / siphash13 / siphash24)"""
import sys

assert len(sys.hash_info) == 9, f"hash_info len = {len(sys.hash_info)!r}"
assert sys.hash_info.modulus < 2 ** sys.hash_info.width, \
    "modulus fits within width bits"
assert sys.hash_info.algorithm in ("fnv", "siphash13", "siphash24"), \
    f"hash algorithm = {sys.hash_info.algorithm!r}"
print("hash_info_shape OK")
"###);
    assert_output(&out, r###"hash_info_shape OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/implementation_describes_interpreter.py`.
#[test]
fn test_gen_behavior_std_libs_sys_implementation_describes_interpreter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "implementation_describes_interpreter"
# subject = "sys.implementation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.implementation: sys.implementation has a lowercase name, a version whose first two fields index as (major, minor), an int hexversion, and a str cache_tag"""
import sys

assert sys.implementation.name == sys.implementation.name.lower(), \
    f"implementation.name = {sys.implementation.name!r}"
_iv = sys.implementation.version
assert _iv[:2] == (_iv.major, _iv.minor), "implementation.version indexing"
assert isinstance(sys.implementation.hexversion, int), "hexversion is int"
assert isinstance(sys.implementation.cache_tag, str), "cache_tag is str"
print("implementation_describes_interpreter OK")
"###);
    assert_output(&out, r###"implementation_describes_interpreter OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/int_info_shape.py`.
#[test]
fn test_gen_behavior_std_libs_sys_int_info_shape() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "int_info_shape"
# subject = "sys.int_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.int_info: int_info has 4 fields, bits_per_digit a multiple of 5, sizeof_digit >= 1, and default_max_str_digits above the str_digits_check_threshold"""
import sys

assert len(sys.int_info) == 4, f"int_info len = {len(sys.int_info)!r}"
assert sys.int_info.bits_per_digit % 5 == 0, \
    f"bits_per_digit = {sys.int_info.bits_per_digit!r}"
assert sys.int_info.sizeof_digit >= 1, "sizeof_digit >= 1"
assert sys.int_info.default_max_str_digits > sys.int_info.str_digits_check_threshold, \
    "default_max_str_digits exceeds the check threshold"
print("int_info_shape OK")
"###);
    assert_output(&out, r###"int_info_shape OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/intern_returns_same_object.py`.
#[test]
fn test_gen_behavior_std_libs_sys_intern_returns_same_object() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "intern_returns_same_object"
# subject = "sys.intern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.intern: interning the same string twice returns the identical object (a is b)"""
import sys

_a = sys.intern("hello_world_unique_key")
_b = sys.intern("hello_world_unique_key")
assert _a is _b, "interned strings are same object"
print("intern_returns_same_object OK")
"###);
    assert_output(&out, r###"intern_returns_same_object OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/is_finalizing_false_during_run.py`.
#[test]
fn test_gen_behavior_std_libs_sys_is_finalizing_false_during_run() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "is_finalizing_false_during_run"
# subject = "sys.is_finalizing"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.is_finalizing: sys.is_finalizing() is False during normal (non-shutdown) execution"""
import sys

assert sys.is_finalizing() is False, f"is_finalizing = {sys.is_finalizing()!r}"
print("is_finalizing_false_during_run OK")
"###);
    assert_output(&out, r###"is_finalizing_false_during_run OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/maxsize_platform_word.py`.
#[test]
fn test_gen_behavior_std_libs_sys_maxsize_platform_word() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "maxsize_platform_word"
# subject = "sys.maxsize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.maxsize: sys.maxsize is the platform pointer-word signed max: 2**63-1 on 64-bit (struct.calcsize('P')*8 == 64), else 2**31-1"""
import sys
import struct

_bits = struct.calcsize("P") * 8
if _bits == 64:
    assert sys.maxsize == 2**63 - 1, f"64-bit maxsize = {sys.maxsize!r}"
else:
    assert sys.maxsize == 2**31 - 1, f"32-bit maxsize = {sys.maxsize!r}"
print("maxsize_platform_word OK")
"###);
    assert_output(&out, r###"maxsize_platform_word OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/meta_path_unique_finders.py`.
#[test]
fn test_gen_behavior_std_libs_sys_meta_path_unique_finders() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "meta_path_unique_finders"
# subject = "sys.meta_path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.meta_path: sys.meta_path is a list of import finders with no duplicate entries"""
import sys

assert isinstance(sys.meta_path, list), f"meta_path type = {type(sys.meta_path)!r}"
assert len(sys.meta_path) == len(set(sys.meta_path)), "meta_path has no duplicates"
print("meta_path_unique_finders OK")
"###);
    assert_output(&out, r###"meta_path_unique_finders OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/modules_caches_imported.py`.
#[test]
fn test_gen_behavior_std_libs_sys_modules_caches_imported() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "modules_caches_imported"
# subject = "sys.modules"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.modules: sys.modules caches imported modules by name: sys.modules['math'] is the math object and sys.modules['os'] is the os object"""
import sys
import math
import os

assert sys.modules["math"] is math, "sys.modules[math] is math"
assert sys.modules["os"] is os, "sys.modules[os] is os"
print("modules_caches_imported OK")
"###);
    assert_output(&out, r###"modules_caches_imported OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/path_entries_are_str.py`.
#[test]
fn test_gen_behavior_std_libs_sys_path_entries_are_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "path_entries_are_str"
# subject = "sys.path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.path: every entry of sys.path is a str"""
import sys

assert all(isinstance(p, str) for p in sys.path), "all path entries are str"
print("path_entries_are_str OK")
"###);
    assert_output(&out, r###"path_entries_are_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/platform_darwin_on_macos.py`.
#[test]
fn test_gen_behavior_std_libs_sys_platform_darwin_on_macos() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "platform_darwin_on_macos"
# subject = "sys.platform"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.platform: macOS reports CPython's historical darwin platform tag."""
import platform
import sys

if platform.system() == "Darwin":
    assert sys.platform == "darwin", sys.platform
else:
    assert isinstance(sys.platform, str) and len(sys.platform) > 0
print("platform_darwin_on_macos OK")
"###);
    assert_output(&out, r###"platform_darwin_on_macos OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/recursionlimit_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_sys_recursionlimit_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "recursionlimit_roundtrip"
# subject = "sys.setrecursionlimit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.setrecursionlimit: setrecursionlimit(500) is observable via getrecursionlimit(), then restored to the original limit"""
import sys

_orig = sys.getrecursionlimit()
sys.setrecursionlimit(500)
assert sys.getrecursionlimit() == 500, f"set to 500: {sys.getrecursionlimit()!r}"
sys.setrecursionlimit(_orig)  # restore
assert sys.getrecursionlimit() == _orig, f"restored: {sys.getrecursionlimit()!r}"
print("recursionlimit_roundtrip OK")
"###);
    assert_output(&out, r###"recursionlimit_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/stdlib_module_names_str_set.py`.
#[test]
fn test_gen_behavior_std_libs_sys_stdlib_module_names_str_set() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "stdlib_module_names_str_set"
# subject = "sys.stdlib_module_names"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.stdlib_module_names: stdlib_module_names is a frozenset of str naming bundled modules and contains 'sys'"""
import sys

assert isinstance(sys.stdlib_module_names, frozenset), \
    f"stdlib_module_names type = {type(sys.stdlib_module_names)!r}"
assert all(isinstance(n, str) for n in sys.stdlib_module_names), \
    "stdlib_module_names entries are all str"
assert "sys" in sys.stdlib_module_names, "sys names itself as stdlib"
print("stdlib_module_names_str_set OK")
"###);
    assert_output(&out, r###"stdlib_module_names_str_set OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/switchinterval_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_sys_switchinterval_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "switchinterval_roundtrip"
# subject = "sys.setswitchinterval"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.setswitchinterval: the switch interval is a small positive float (< 0.5) that survives a setswitchinterval(0.05)/getswitchinterval round trip, then restores"""
import sys

orig = sys.getswitchinterval()
assert orig < 0.5, f"default switchinterval = {orig!r}"
try:
    sys.setswitchinterval(0.05)
    assert abs(sys.getswitchinterval() - 0.05) < 1e-7, "switchinterval round trip"
finally:
    sys.setswitchinterval(orig)
assert abs(sys.getswitchinterval() - orig) < 1e-7, "switchinterval restored"
print("switchinterval_roundtrip OK")
"###);
    assert_output(&out, r###"switchinterval_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/sys_module_test__test_getfilesystemencoding.py`.
#[test]
fn test_gen_behavior_std_libs_sys_sys_module_test__test_getfilesystemencoding() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_getfilesystemencoding"
# subject = "cpython.test_sys.SysModuleTest.test_getfilesystemencoding"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_getfilesystemencoding
"""Auto-ported test: SysModuleTest::test_getfilesystemencoding (CPython 3.12 oracle)."""


import builtins
import codecs
import gc
import io
import locale
import operator
import os
import random
import struct
import subprocess
import sys
import sysconfig
import test.support
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, assert_python_failure
from test.support import threading_helper
from test.support import import_helper
import textwrap
import unittest
import warnings


try:
    from test.support import interpreters
except ImportError:
    interpreters = None

def requires_subinterpreters(func):
    deco = unittest.skipIf(interpreters is None, 'Test requires subinterpreters')
    return deco(func)

DICT_KEY_STRUCT_FORMAT = 'n2BI2n'


# --- test body ---
def assert_raise_on_new_sys_type(sys_attr):
    arg = sys_attr
    attr_type = type(sys_attr)
    try:
        attr_type(arg)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        attr_type.__new__(attr_type, arg)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def c_locale_get_error_handler(locale, isolated=False, encoding=None):
    env = os.environ.copy()
    env['LC_ALL'] = locale
    env['PYTHONCOERCECLOCALE'] = '0'
    code = '\n'.join(('import sys', 'def dump(name):', '    std = getattr(sys, name)', '    print("%s: %s" % (name, std.errors))', 'dump("stdin")', 'dump("stdout")', 'dump("stderr")'))
    args = [sys.executable, '-X', 'utf8=0', '-c', code]
    if isolated:
        args.append('-I')
    if encoding is not None:
        env['PYTHONIOENCODING'] = encoding
    else:
        env.pop('PYTHONIOENCODING', None)
    p = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, env=env, universal_newlines=True)
    stdout, stderr = p.communicate()
    return stdout

def check_fsencoding(fs_encoding, expected=None):

    assert fs_encoding is not None
    codecs.lookup(fs_encoding)
    if expected:

        assert fs_encoding == expected

def check_locale_surrogateescape(locale):
    out = c_locale_get_error_handler(locale, isolated=True)

    assert out == 'stdin: surrogateescape\nstdout: surrogateescape\nstderr: backslashreplace\n'
    out = c_locale_get_error_handler(locale, encoding=':ignore')

    assert out == 'stdin: ignore\nstdout: ignore\nstderr: backslashreplace\n'
    out = c_locale_get_error_handler(locale, encoding='iso8859-1')

    assert out == 'stdin: strict\nstdout: strict\nstderr: backslashreplace\n'
    out = c_locale_get_error_handler(locale, encoding='iso8859-1:')

    assert out == 'stdin: strict\nstdout: strict\nstderr: backslashreplace\n'
    out = c_locale_get_error_handler(locale, encoding=':')

    assert out == 'stdin: surrogateescape\nstdout: surrogateescape\nstderr: backslashreplace\n'
    out = c_locale_get_error_handler(locale, encoding='')

    assert out == 'stdin: surrogateescape\nstdout: surrogateescape\nstderr: backslashreplace\n'
fs_encoding = sys.getfilesystemencoding()
if sys.platform == 'darwin':
    expected = 'utf-8'
else:
    expected = None
check_fsencoding(fs_encoding, expected)
print("SysModuleTest::test_getfilesystemencoding: ok")
"###);
    assert_output(&out, r###"SysModuleTest::test_getfilesystemencoding: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/sys_module_test__test_module_names.py`.
#[test]
fn test_gen_behavior_std_libs_sys_sys_module_test__test_module_names() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_module_names"
# subject = "cpython.test_sys.SysModuleTest.test_module_names"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_module_names
"""Auto-ported test: SysModuleTest::test_module_names (CPython 3.12 oracle)."""


import builtins
import codecs
import gc
import io
import locale
import operator
import os
import random
import struct
import subprocess
import sys
import sysconfig
import test.support
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, assert_python_failure
from test.support import threading_helper
from test.support import import_helper
import textwrap
import unittest
import warnings


try:
    from test.support import interpreters
except ImportError:
    interpreters = None

def requires_subinterpreters(func):
    deco = unittest.skipIf(interpreters is None, 'Test requires subinterpreters')
    return deco(func)

DICT_KEY_STRUCT_FORMAT = 'n2BI2n'


# --- test body ---

assert isinstance(sys.stdlib_module_names, frozenset)
for name in sys.stdlib_module_names:

    assert isinstance(name, str)
print("SysModuleTest::test_module_names: ok")
"###);
    assert_output(&out, r###"SysModuleTest::test_module_names: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/sys_module_test__test_no_duplicates_in_meta_path.py`.
#[test]
fn test_gen_behavior_std_libs_sys_sys_module_test__test_no_duplicates_in_meta_path() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_no_duplicates_in_meta_path"
# subject = "cpython.test_sys.SysModuleTest.test_no_duplicates_in_meta_path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_no_duplicates_in_meta_path
"""Auto-ported test: SysModuleTest::test_no_duplicates_in_meta_path (CPython 3.12 oracle)."""


import builtins
import codecs
import gc
import io
import locale
import operator
import os
import random
import struct
import subprocess
import sys
import sysconfig
import test.support
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, assert_python_failure
from test.support import threading_helper
from test.support import import_helper
import textwrap
import unittest
import warnings


try:
    from test.support import interpreters
except ImportError:
    interpreters = None

def requires_subinterpreters(func):
    deco = unittest.skipIf(interpreters is None, 'Test requires subinterpreters')
    return deco(func)

DICT_KEY_STRUCT_FORMAT = 'n2BI2n'


# --- test body ---

assert len(sys.meta_path) == len(set(sys.meta_path))
print("SysModuleTest::test_no_duplicates_in_meta_path: ok")
"###);
    assert_output(&out, r###"SysModuleTest::test_no_duplicates_in_meta_path: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/sys_module_test__test_stdlib_dir.py`.
#[test]
fn test_gen_behavior_std_libs_sys_sys_module_test__test_stdlib_dir() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_stdlib_dir"
# subject = "cpython.test_sys.SysModuleTest.test_stdlib_dir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_stdlib_dir
"""Auto-ported test: SysModuleTest::test_stdlib_dir (CPython 3.12 oracle)."""


import builtins
import codecs
import gc
import io
import locale
import operator
import os
import random
import struct
import subprocess
import sys
import sysconfig
import test.support
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, assert_python_failure
from test.support import threading_helper
from test.support import import_helper
import textwrap
import unittest
import warnings


try:
    from test.support import interpreters
except ImportError:
    interpreters = None

def requires_subinterpreters(func):
    deco = unittest.skipIf(interpreters is None, 'Test requires subinterpreters')
    return deco(func)

DICT_KEY_STRUCT_FORMAT = 'n2BI2n'


# --- test body ---
os = import_helper.import_fresh_module('os')
marker = getattr(os, '__file__', None)
if marker and (not os.path.exists(marker)):
    marker = None
expected = os.path.dirname(marker) if marker else None

assert os.path.normpath(sys._stdlib_dir) == os.path.normpath(expected)
print("SysModuleTest::test_stdlib_dir: ok")
"###);
    assert_output(&out, r###"SysModuleTest::test_stdlib_dir: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/sys_module_test__test_thread_info.py`.
#[test]
fn test_gen_behavior_std_libs_sys_sys_module_test__test_thread_info() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_thread_info"
# subject = "cpython.test_sys.SysModuleTest.test_thread_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_thread_info
"""Auto-ported test: SysModuleTest::test_thread_info (CPython 3.12 oracle)."""


import builtins
import codecs
import gc
import io
import locale
import operator
import os
import random
import struct
import subprocess
import sys
import sysconfig
import test.support
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, assert_python_failure
from test.support import threading_helper
from test.support import import_helper
import textwrap
import unittest
import warnings


try:
    from test.support import interpreters
except ImportError:
    interpreters = None

def requires_subinterpreters(func):
    deco = unittest.skipIf(interpreters is None, 'Test requires subinterpreters')
    return deco(func)

DICT_KEY_STRUCT_FORMAT = 'n2BI2n'


# --- test body ---
info = sys.thread_info

assert len(info) == 3

assert info.name in ('nt', 'pthread', 'pthread-stubs', 'solaris', None)

assert info.lock in ('semaphore', 'mutex+cond', None)
if sys.platform.startswith(('linux', 'freebsd')):

    assert info.name == 'pthread'
elif sys.platform == 'win32':

    assert info.name == 'nt'
elif sys.platform == 'emscripten':

    assert info.name in {'pthread', 'pthread-stubs'}
elif sys.platform == 'wasi':

    assert info.name == 'pthread-stubs'
print("SysModuleTest::test_thread_info: ok")
"###);
    assert_output(&out, r###"SysModuleTest::test_thread_info: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/thread_info_shape.py`.
#[test]
fn test_gen_behavior_std_libs_sys_thread_info_shape() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "thread_info_shape"
# subject = "sys.thread_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.thread_info: thread_info is a 3-field struct with a documented name (nt/pthread/pthread-stubs/solaris/None) and lock (semaphore/mutex+cond/None)"""
import sys

ti = sys.thread_info
assert len(ti) == 3, f"thread_info len = {len(ti)!r}"
assert ti.name in ("nt", "pthread", "pthread-stubs", "solaris", None), \
    f"thread_info.name = {ti.name!r}"
assert ti.lock in ("semaphore", "mutex+cond", None), \
    f"thread_info.lock = {ti.lock!r}"
print("thread_info_shape OK")
"###);
    assert_output(&out, r###"thread_info_shape OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/version_info_attributes.py`.
#[test]
fn test_gen_behavior_std_libs_sys_version_info_attributes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "version_info_attributes"
# subject = "sys.version_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.version_info: version_info exposes major==3, non-negative minor/micro, and a valid releaselevel in (alpha, beta, candidate, final)"""
import sys

assert sys.version_info.major == 3, f"major = {sys.version_info.major!r}"
assert sys.version_info.minor >= 0, f"minor = {sys.version_info.minor!r}"
assert sys.version_info.micro >= 0, f"micro = {sys.version_info.micro!r}"
assert sys.version_info.releaselevel in ("alpha", "beta", "candidate", "final"), \
    f"releaselevel = {sys.version_info.releaselevel!r}"
print("version_info_attributes OK")
"###);
    assert_output(&out, r###"version_info_attributes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/sys/version_info_indexable_and_named.py`.
#[test]
fn test_gen_behavior_std_libs_sys_version_info_indexable_and_named() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "version_info_indexable_and_named"
# subject = "sys.version_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.version_info: version_info is a 5-field named tuple: index i equals the named field, it slices to a tuple, and compares as a tuple (> (1, 0, 0))"""
import sys

vi = sys.version_info
assert len(vi) == 5, f"version_info len = {len(vi)!r}"
assert isinstance(vi[:], tuple), "version_info slice is a tuple"
assert vi[0] == vi.major, "vi[0] == major"
assert vi[1] == vi.minor, "vi[1] == minor"
assert vi[2] == vi.micro, "vi[2] == micro"
assert vi[3] == vi.releaselevel, "vi[3] == releaselevel"
assert vi[4] == vi.serial, "vi[4] == serial"
assert vi > (1, 0, 0), "version_info compares as a tuple"
print("version_info_indexable_and_named OK")
"###);
    assert_output(&out, r###"version_info_indexable_and_named OK
"###);
}
