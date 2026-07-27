use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/file/c_auto_file_tests__test_attributes.py`.
#[test]
fn test_gen_behavior_std_libs_file_c_auto_file_tests__test_attributes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "c_auto_file_tests__test_attributes"
# subject = "cpython.test_file.CAutoFileTests.testAttributes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_file.py::CAutoFileTests::testAttributes
"""Auto-ported test: CAutoFileTests::testAttributes (CPython 3.12 oracle)."""


import sys
import os
import unittest
from array import array
from weakref import proxy
import io
import _pyio as pyio
from test.support import gc_collect
from test.support.os_helper import TESTFN
from test.support import os_helper
from test.support import warnings_helper
from collections import UserList


# --- test body ---
open = io.open
self_f = open(TESTFN, 'wb')
f = self_f
f.name
f.mode
f.closed
print("CAutoFileTests::testAttributes: ok")
"###);
    assert_output(&out, r###"CAutoFileTests::testAttributes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/file/c_auto_file_tests__test_writelines_user_list.py`.
#[test]
fn test_gen_behavior_std_libs_file_c_auto_file_tests__test_writelines_user_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "c_auto_file_tests__test_writelines_user_list"
# subject = "cpython.test_file.CAutoFileTests.testWritelinesUserList"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_file.py::CAutoFileTests::testWritelinesUserList
"""Auto-ported test: CAutoFileTests::testWritelinesUserList (CPython 3.12 oracle)."""


import sys
import os
import unittest
from array import array
from weakref import proxy
import io
import _pyio as pyio
from test.support import gc_collect
from test.support.os_helper import TESTFN
from test.support import os_helper
from test.support import warnings_helper
from collections import UserList


# --- test body ---
open = io.open
self_f = open(TESTFN, 'wb')
l = UserList([b'1', b'2'])
self_f.writelines(l)
self_f.close()
self_f = open(TESTFN, 'rb')
buf = self_f.read()

assert buf == b'12'
print("CAutoFileTests::testWritelinesUserList: ok")
"###);
    assert_output(&out, r###"CAutoFileTests::testWritelinesUserList: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/file/c_other_file_tests__test_bad_mode_argument.py`.
#[test]
fn test_gen_behavior_std_libs_file_c_other_file_tests__test_bad_mode_argument() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "c_other_file_tests__test_bad_mode_argument"
# subject = "cpython.test_file.COtherFileTests.testBadModeArgument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_file.py::COtherFileTests::testBadModeArgument
"""Auto-ported test: COtherFileTests::testBadModeArgument (CPython 3.12 oracle)."""


import sys
import os
import unittest
from array import array
from weakref import proxy
import io
import _pyio as pyio
from test.support import gc_collect
from test.support.os_helper import TESTFN
from test.support import os_helper
from test.support import warnings_helper
from collections import UserList


# --- test body ---
open = io.open

def _checkBufferSize(s):
    try:
        f = open(TESTFN, 'wb', s)
        f.write(str(s).encode('ascii'))
        f.close()
        f.close()
        f = open(TESTFN, 'rb', s)
        d = int(f.read().decode('ascii'))
        f.close()
        f.close()
    except OSError as msg:

        raise AssertionError('error setting buffer size %d: %s' % (s, str(msg)))

    assert d == s
bad_mode = 'qwerty'
try:
    f = open(TESTFN, bad_mode)
except ValueError as msg:
    if msg.args[0] != 0:
        s = str(msg)
        if TESTFN in s or bad_mode not in s:

            raise AssertionError('bad error message for invalid mode: %s' % s)
else:
    f.close()

    raise AssertionError('no error for invalid mode: %s' % bad_mode)
print("COtherFileTests::testBadModeArgument: ok")
"###);
    assert_output(&out, r###"COtherFileTests::testBadModeArgument: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/file/c_other_file_tests__test_mode_strings.py`.
#[test]
fn test_gen_behavior_std_libs_file_c_other_file_tests__test_mode_strings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "c_other_file_tests__test_mode_strings"
# subject = "cpython.test_file.COtherFileTests.testModeStrings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_file.py::COtherFileTests::testModeStrings
"""Auto-ported test: COtherFileTests::testModeStrings (CPython 3.12 oracle)."""


import sys
import os
import unittest
from array import array
from weakref import proxy
import io
import _pyio as pyio
from test.support import gc_collect
from test.support.os_helper import TESTFN
from test.support import os_helper
from test.support import warnings_helper
from collections import UserList


# --- test body ---
open = io.open

def _checkBufferSize(s):
    try:
        f = open(TESTFN, 'wb', s)
        f.write(str(s).encode('ascii'))
        f.close()
        f.close()
        f = open(TESTFN, 'rb', s)
        d = int(f.read().decode('ascii'))
        f.close()
        f.close()
    except OSError as msg:

        raise AssertionError('error setting buffer size %d: %s' % (s, str(msg)))

    assert d == s
open(TESTFN, 'wb').close()
for mode in ('', 'aU', 'wU+', 'U+', '+U', 'rU+'):
    try:
        f = open(TESTFN, mode)
    except ValueError:
        pass
    else:
        f.close()

        raise AssertionError('%r is an invalid file mode' % mode)
print("COtherFileTests::testModeStrings: ok")
"###);
    assert_output(&out, r###"COtherFileTests::testModeStrings: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/file/py_auto_file_tests__test_attributes.py`.
#[test]
fn test_gen_behavior_std_libs_file_py_auto_file_tests__test_attributes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "py_auto_file_tests__test_attributes"
# subject = "cpython.test_file.PyAutoFileTests.testAttributes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_file.py::PyAutoFileTests::testAttributes
"""Auto-ported test: PyAutoFileTests::testAttributes (CPython 3.12 oracle)."""


import sys
import os
import unittest
from array import array
from weakref import proxy
import io
import _pyio as pyio
from test.support import gc_collect
from test.support.os_helper import TESTFN
from test.support import os_helper
from test.support import warnings_helper
from collections import UserList


# --- test body ---
open = staticmethod(pyio.open)
self_f = open(TESTFN, 'wb')
f = self_f
f.name
f.mode
f.closed
print("PyAutoFileTests::testAttributes: ok")
"###);
    assert_output(&out, r###"PyAutoFileTests::testAttributes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/file/py_auto_file_tests__test_writelines_user_list.py`.
#[test]
fn test_gen_behavior_std_libs_file_py_auto_file_tests__test_writelines_user_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "py_auto_file_tests__test_writelines_user_list"
# subject = "cpython.test_file.PyAutoFileTests.testWritelinesUserList"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_file.py::PyAutoFileTests::testWritelinesUserList
"""Auto-ported test: PyAutoFileTests::testWritelinesUserList (CPython 3.12 oracle)."""


import sys
import os
import unittest
from array import array
from weakref import proxy
import io
import _pyio as pyio
from test.support import gc_collect
from test.support.os_helper import TESTFN
from test.support import os_helper
from test.support import warnings_helper
from collections import UserList


# --- test body ---
open = staticmethod(pyio.open)
self_f = open(TESTFN, 'wb')
l = UserList([b'1', b'2'])
self_f.writelines(l)
self_f.close()
self_f = open(TESTFN, 'rb')
buf = self_f.read()

assert buf == b'12'
print("PyAutoFileTests::testWritelinesUserList: ok")
"###);
    assert_output(&out, r###"PyAutoFileTests::testWritelinesUserList: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/file/py_other_file_tests__test_bad_mode_argument.py`.
#[test]
fn test_gen_behavior_std_libs_file_py_other_file_tests__test_bad_mode_argument() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "py_other_file_tests__test_bad_mode_argument"
# subject = "cpython.test_file.PyOtherFileTests.testBadModeArgument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_file.py::PyOtherFileTests::testBadModeArgument
"""Auto-ported test: PyOtherFileTests::testBadModeArgument (CPython 3.12 oracle)."""


import sys
import os
import unittest
from array import array
from weakref import proxy
import io
import _pyio as pyio
from test.support import gc_collect
from test.support.os_helper import TESTFN
from test.support import os_helper
from test.support import warnings_helper
from collections import UserList


# --- test body ---
open = staticmethod(pyio.open)

def _checkBufferSize(s):
    try:
        f = open(TESTFN, 'wb', s)
        f.write(str(s).encode('ascii'))
        f.close()
        f.close()
        f = open(TESTFN, 'rb', s)
        d = int(f.read().decode('ascii'))
        f.close()
        f.close()
    except OSError as msg:

        raise AssertionError('error setting buffer size %d: %s' % (s, str(msg)))

    assert d == s
bad_mode = 'qwerty'
try:
    f = open(TESTFN, bad_mode)
except ValueError as msg:
    if msg.args[0] != 0:
        s = str(msg)
        if TESTFN in s or bad_mode not in s:

            raise AssertionError('bad error message for invalid mode: %s' % s)
else:
    f.close()

    raise AssertionError('no error for invalid mode: %s' % bad_mode)
print("PyOtherFileTests::testBadModeArgument: ok")
"###);
    assert_output(&out, r###"PyOtherFileTests::testBadModeArgument: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/file/py_other_file_tests__test_mode_strings.py`.
#[test]
fn test_gen_behavior_std_libs_file_py_other_file_tests__test_mode_strings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "file"
# dimension = "behavior"
# case = "py_other_file_tests__test_mode_strings"
# subject = "cpython.test_file.PyOtherFileTests.testModeStrings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_file.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_file.py::PyOtherFileTests::testModeStrings
"""Auto-ported test: PyOtherFileTests::testModeStrings (CPython 3.12 oracle)."""


import sys
import os
import unittest
from array import array
from weakref import proxy
import io
import _pyio as pyio
from test.support import gc_collect
from test.support.os_helper import TESTFN
from test.support import os_helper
from test.support import warnings_helper
from collections import UserList


# --- test body ---
open = staticmethod(pyio.open)

def _checkBufferSize(s):
    try:
        f = open(TESTFN, 'wb', s)
        f.write(str(s).encode('ascii'))
        f.close()
        f.close()
        f = open(TESTFN, 'rb', s)
        d = int(f.read().decode('ascii'))
        f.close()
        f.close()
    except OSError as msg:

        raise AssertionError('error setting buffer size %d: %s' % (s, str(msg)))

    assert d == s
open(TESTFN, 'wb').close()
for mode in ('', 'aU', 'wU+', 'U+', '+U', 'rU+'):
    try:
        f = open(TESTFN, mode)
    except ValueError:
        pass
    else:
        f.close()

        raise AssertionError('%r is an invalid file mode' % mode)
print("PyOtherFileTests::testModeStrings: ok")
"###);
    assert_output(&out, r###"PyOtherFileTests::testModeStrings: ok
"###);
}
