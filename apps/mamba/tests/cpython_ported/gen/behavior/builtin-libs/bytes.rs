use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/assorted_bytes_test__test_compare_bytes_to_bytearray.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_assorted_bytes_test__test_compare_bytes_to_bytearray() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "assorted_bytes_test__test_compare_bytes_to_bytearray"
# subject = "cpython.test_bytes.AssortedBytesTest.test_compare_bytes_to_bytearray"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::AssortedBytesTest::test_compare_bytes_to_bytearray
"""Auto-ported test: AssortedBytesTest::test_compare_bytes_to_bytearray (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---

assert (b'abc' == bytes(b'abc')) == True

assert (b'ab' != bytes(b'abc')) == True

assert (b'ab' <= bytes(b'abc')) == True

assert (b'ab' < bytes(b'abc')) == True

assert (b'abc' >= bytes(b'ab')) == True

assert (b'abc' > bytes(b'ab')) == True

assert (b'abc' != bytes(b'abc')) == False

assert (b'ab' == bytes(b'abc')) == False

assert (b'ab' > bytes(b'abc')) == False

assert (b'ab' >= bytes(b'abc')) == False

assert (b'abc' < bytes(b'ab')) == False

assert (b'abc' <= bytes(b'ab')) == False

assert (bytes(b'abc') == b'abc') == True

assert (bytes(b'ab') != b'abc') == True

assert (bytes(b'ab') <= b'abc') == True

assert (bytes(b'ab') < b'abc') == True

assert (bytes(b'abc') >= b'ab') == True

assert (bytes(b'abc') > b'ab') == True

assert (bytes(b'abc') != b'abc') == False

assert (bytes(b'ab') == b'abc') == False

assert (bytes(b'ab') > b'abc') == False

assert (bytes(b'ab') >= b'abc') == False

assert (bytes(b'abc') < b'ab') == False

assert (bytes(b'abc') <= b'ab') == False
print("AssortedBytesTest::test_compare_bytes_to_bytearray: ok")
"###);
    assert_output(&out, r###"AssortedBytesTest::test_compare_bytes_to_bytearray: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/assorted_bytes_test__test_repr_str.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_assorted_bytes_test__test_repr_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "assorted_bytes_test__test_repr_str"
# subject = "cpython.test_bytes.AssortedBytesTest.test_repr_str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::AssortedBytesTest::test_repr_str
"""Auto-ported test: AssortedBytesTest::test_repr_str (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
for f in (str, repr):

    assert f(bytearray()) == "bytearray(b'')"

    assert f(bytearray([0])) == "bytearray(b'\\x00')"

    assert f(bytearray([0, 1, 254, 255])) == "bytearray(b'\\x00\\x01\\xfe\\xff')"

    assert f(b'abc') == "b'abc'"

    assert f(b"'") == 'b"\'"'

    assert f(b'\'"') == 'b\'\\\'"\''
print("AssortedBytesTest::test_repr_str: ok")
"###);
    assert_output(&out, r###"AssortedBytesTest::test_repr_str: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/assorted_bytes_test__test_return_self.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_assorted_bytes_test__test_return_self() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "assorted_bytes_test__test_return_self"
# subject = "cpython.test_bytes.AssortedBytesTest.test_return_self"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::AssortedBytesTest::test_return_self
"""Auto-ported test: AssortedBytesTest::test_return_self (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
b = bytearray()

assert b.replace(b'', b'') is not b
print("AssortedBytesTest::test_return_self: ok")
"###);
    assert_output(&out, r###"AssortedBytesTest::test_return_self: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/assorted_bytes_test__test_to_str.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_assorted_bytes_test__test_to_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "assorted_bytes_test__test_to_str"
# subject = "cpython.test_bytes.AssortedBytesTest.test_to_str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::AssortedBytesTest::test_to_str
"""Auto-ported test: AssortedBytesTest::test_to_str (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---

assert str(b'') == "b''"

assert str(b'x') == "b'x'"

assert str(b'\x80') == "b'\\x80'"

assert str(bytearray(b'')) == "bytearray(b'')"

assert str(bytearray(b'x')) == "bytearray(b'x')"

assert str(bytearray(b'\x80')) == "bytearray(b'\\x80')"
print("AssortedBytesTest::test_to_str: ok")
"###);
    assert_output(&out, r###"AssortedBytesTest::test_to_str: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_bytearray_api.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_bytearray_api() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_bytearray_api"
# subject = "cpython.test_bytes.ByteArrayTest.test_bytearray_api"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_bytearray_api
"""Auto-ported test: ByteArrayTest::test_bytearray_api (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
short_sample = b'Hello world\n'
sample = short_sample + b'\x00' * (20 - len(short_sample))
tfn = tempfile.mktemp()
try:
    with open(tfn, 'wb') as f:
        f.write(short_sample)
    with open(tfn, 'rb') as f:
        b = bytearray(20)
        n = f.readinto(b)

    assert n == len(short_sample)

    assert list(b) == list(sample)
    with open(tfn, 'wb') as f:
        f.write(b)
    with open(tfn, 'rb') as f:

        assert f.read() == sample
finally:
    try:
        os.remove(tfn)
    except OSError:
        pass
print("ByteArrayTest::test_bytearray_api: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_bytearray_api: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_center.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_center() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_center"
# subject = "cpython.test_bytes.ByteArrayTest.test_center"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_center
"""Auto-ported test: ByteArrayTest::test_center (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'abc')
for fill_type in (bytes, bytearray):

    assert b.center(7, fill_type(b'-')) == type2test(b'--abc--')
print("ByteArrayTest::test_center: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_center: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_clear.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_clear() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_clear"
# subject = "cpython.test_bytes.ByteArrayTest.test_clear"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_clear
"""Auto-ported test: ByteArrayTest::test_clear (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = bytearray(b'python')
b.clear()

assert b == b''
b = bytearray(b'')
b.clear()

assert b == b''
b = bytearray(b'')
b.append(ord('r'))
b.clear()
b.append(ord('p'))

assert b == b'p'
print("ByteArrayTest::test_clear: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_clear: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_compare.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_compare() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_compare"
# subject = "cpython.test_bytes.ByteArrayTest.test_compare"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_compare
"""Auto-ported test: ByteArrayTest::test_compare (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b1 = type2test([1, 2, 3])
b2 = type2test([1, 2, 3])
b3 = type2test([1, 3])

assert b1 == b2

assert b2 != b3

assert b1 <= b2

assert b1 <= b3

assert b1 < b3

assert b1 >= b2

assert b3 >= b2

assert b3 > b2

assert not b1 != b2

assert not b2 == b3

assert not b1 > b2

assert not b1 > b3

assert not b1 >= b3

assert not b1 < b2

assert not b3 < b2

assert not b3 <= b2
print("ByteArrayTest::test_compare: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_compare: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_compare_to_str.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_compare_to_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_compare_to_str"
# subject = "cpython.test_bytes.ByteArrayTest.test_compare_to_str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_compare_to_str
"""Auto-ported test: ByteArrayTest::test_compare_to_str (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected

assert (type2test(b'\x00a\x00b\x00c') == 'abc') == False

assert (type2test(b'\x00\x00\x00a\x00\x00\x00b\x00\x00\x00c') == 'abc') == False

assert (type2test(b'a\x00b\x00c\x00') == 'abc') == False

assert (type2test(b'a\x00\x00\x00b\x00\x00\x00c\x00\x00\x00') == 'abc') == False

assert (type2test() == str()) == False

assert (type2test() != str()) == True
print("ByteArrayTest::test_compare_to_str: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_compare_to_str: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_concat.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_concat() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_concat"
# subject = "cpython.test_bytes.ByteArrayTest.test_concat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_concat
"""Auto-ported test: ByteArrayTest::test_concat (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b1 = type2test(b'abc')
b2 = type2test(b'def')

assert b1 + b2 == b'abcdef'

assert b1 + bytes(b'def') == b'abcdef'

assert bytes(b'def') + b1 == b'defabc'

try:
    (lambda: b1 + 'def')()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    (lambda: 'abc' + b2)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ByteArrayTest::test_concat: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_concat: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_constructor_value_errors.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_constructor_value_errors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_constructor_value_errors"
# subject = "cpython.test_bytes.ByteArrayTest.test_constructor_value_errors"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_constructor_value_errors
"""Auto-ported test: ByteArrayTest::test_constructor_value_errors (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected

try:
    type2test([-1])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([-sys.maxsize])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([-sys.maxsize - 1])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([-sys.maxsize - 2])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([-10 ** 100])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([256])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([257])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([sys.maxsize])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([sys.maxsize + 1])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([10 ** 100])
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("ByteArrayTest::test_constructor_value_errors: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_constructor_value_errors: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_contains.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_contains() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_contains"
# subject = "cpython.test_bytes.ByteArrayTest.test_contains"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_contains
"""Auto-ported test: ByteArrayTest::test_contains (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'abc')

assert ord('a') in b

assert int(ord('a')) in b

assert 200 not in b

try:
    (lambda: 300 in b)()
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    (lambda: -1 in b)()
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    (lambda: sys.maxsize + 1 in b)()
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    (lambda: None in b)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    (lambda: float(ord('a')) in b)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    (lambda: 'a' in b)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
for f in (bytes, bytearray):

    assert f(b'') in b

    assert f(b'a') in b

    assert f(b'b') in b

    assert f(b'c') in b

    assert f(b'ab') in b

    assert f(b'bc') in b

    assert f(b'abc') in b

    assert f(b'ac') not in b

    assert f(b'd') not in b

    assert f(b'dab') not in b

    assert f(b'abd') not in b
print("ByteArrayTest::test_contains: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_contains: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_copied.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_copied() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_copied"
# subject = "cpython.test_bytes.ByteArrayTest.test_copied"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_copied
"""Auto-ported test: ByteArrayTest::test_copied (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = bytearray(b'abc')

assert b is not b.replace(b'abc', b'cde', 0)
t = bytearray([i for i in range(256)])
x = bytearray(b'')

assert x is not x.translate(t)
print("ByteArrayTest::test_copied: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_copied: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_del_expand.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_del_expand() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_del_expand"
# subject = "cpython.test_bytes.ByteArrayTest.test_del_expand"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_del_expand
"""Auto-ported test: ByteArrayTest::test_del_expand (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = bytearray(10)
size = sys.getsizeof(b)
del b[:1]

assert sys.getsizeof(b) <= size
print("ByteArrayTest::test_del_expand: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_del_expand: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_fifo_overrun.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_fifo_overrun() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_fifo_overrun"
# subject = "cpython.test_bytes.ByteArrayTest.test_fifo_overrun"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_fifo_overrun
"""Auto-ported test: ByteArrayTest::test_fifo_overrun (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = bytearray(10)
b.pop()
del b[:1]
b += bytes(2)
del b
print("ByteArrayTest::test_fifo_overrun: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_fifo_overrun: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_hex_separator_five_bytes.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_hex_separator_five_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_hex_separator_five_bytes"
# subject = "cpython.test_bytes.ByteArrayTest.test_hex_separator_five_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_hex_separator_five_bytes
"""Auto-ported test: ByteArrayTest::test_hex_separator_five_bytes (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
five_bytes = type2test(range(90, 95))

assert five_bytes.hex() == '5a5b5c5d5e'
print("ByteArrayTest::test_hex_separator_five_bytes: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_hex_separator_five_bytes: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_hex_separator_six_bytes.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_hex_separator_six_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_hex_separator_six_bytes"
# subject = "cpython.test_bytes.ByteArrayTest.test_hex_separator_six_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_hex_separator_six_bytes
"""Auto-ported test: ByteArrayTest::test_hex_separator_six_bytes (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
six_bytes = type2test((x * 3 for x in range(1, 7)))

assert six_bytes.hex() == '0306090c0f12'

assert six_bytes.hex('.', 1) == '03.06.09.0c.0f.12'

assert six_bytes.hex(' ', 2) == '0306 090c 0f12'

assert six_bytes.hex('-', 3) == '030609-0c0f12'

assert six_bytes.hex(':', 4) == '0306:090c0f12'

assert six_bytes.hex(':', 5) == '03:06090c0f12'

assert six_bytes.hex(':', 6) == '0306090c0f12'

assert six_bytes.hex(':', 95) == '0306090c0f12'

assert six_bytes.hex('_', -3) == '030609_0c0f12'

assert six_bytes.hex(':', -4) == '0306090c:0f12'

assert six_bytes.hex(b'@', -5) == '0306090c0f@12'

assert six_bytes.hex(':', -6) == '0306090c0f12'

assert six_bytes.hex(' ', -95) == '0306090c0f12'
print("ByteArrayTest::test_hex_separator_six_bytes: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_hex_separator_six_bytes: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_index.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_index() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_index"
# subject = "cpython.test_bytes.ByteArrayTest.test_index"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_index
"""Auto-ported test: ByteArrayTest::test_index (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'mississippi')
i = 105
w = 119

assert b.index(b'ss') == 2

try:
    b.index(b'w')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    b.index(b'mississippian')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert b.index(i) == 1

try:
    b.index(w)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert b.index(b'ss', 3) == 5

assert b.index(b'ss', 1, 7) == 2

try:
    b.index(b'ss', 1, 3)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert b.index(i, 6) == 7

assert b.index(i, 1, 3) == 1

try:
    b.index(w, 1, 3)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("ByteArrayTest::test_index: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_index: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_ljust.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_ljust() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_ljust"
# subject = "cpython.test_bytes.ByteArrayTest.test_ljust"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_ljust
"""Auto-ported test: ByteArrayTest::test_ljust (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'abc')
for fill_type in (bytes, bytearray):

    assert b.ljust(7, fill_type(b'-')) == type2test(b'abc----')
print("ByteArrayTest::test_ljust: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_ljust: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_none_arguments.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_none_arguments() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_none_arguments"
# subject = "cpython.test_bytes.ByteArrayTest.test_none_arguments"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_none_arguments
"""Auto-ported test: ByteArrayTest::test_none_arguments (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'hello')
l = type2test(b'l')
h = type2test(b'h')
x = type2test(b'x')
o = type2test(b'o')

assert 2 == b.find(l, None)

assert 3 == b.find(l, -2, None)

assert 2 == b.find(l, None, -2)

assert 0 == b.find(h, None, None)

assert 3 == b.rfind(l, None)

assert 3 == b.rfind(l, -2, None)

assert 2 == b.rfind(l, None, -2)

assert 0 == b.rfind(h, None, None)

assert 2 == b.index(l, None)

assert 3 == b.index(l, -2, None)

assert 2 == b.index(l, None, -2)

assert 0 == b.index(h, None, None)

assert 3 == b.rindex(l, None)

assert 3 == b.rindex(l, -2, None)

assert 2 == b.rindex(l, None, -2)

assert 0 == b.rindex(h, None, None)

assert 2 == b.count(l, None)

assert 1 == b.count(l, -2, None)

assert 1 == b.count(l, None, -2)

assert 0 == b.count(x, None, None)

assert True == b.endswith(o, None)

assert True == b.endswith(o, -2, None)

assert True == b.endswith(l, None, -2)

assert False == b.endswith(x, None, None)

assert True == b.startswith(h, None)

assert True == b.startswith(l, -2, None)

assert True == b.startswith(h, None, -2)

assert False == b.startswith(x, None, None)
print("ByteArrayTest::test_none_arguments: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_none_arguments: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_nosort.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_nosort() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_nosort"
# subject = "cpython.test_bytes.ByteArrayTest.test_nosort"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_nosort
"""Auto-ported test: ByteArrayTest::test_nosort (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected

try:
    (lambda: bytearray().sort())()
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
print("ByteArrayTest::test_nosort: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_nosort: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_partition.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_partition() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_partition"
# subject = "cpython.test_bytes.ByteArrayTest.test_partition"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_partition
"""Auto-ported test: ByteArrayTest::test_partition (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'mississippi')

assert b.partition(b'ss') == (b'mi', b'ss', b'issippi')

assert b.partition(b'w') == (b'mississippi', b'', b'')
print("ByteArrayTest::test_partition: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_partition: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_partition_bytearray_doesnt_share_nullstring.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_partition_bytearray_doesnt_share_nullstring() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_partition_bytearray_doesnt_share_nullstring"
# subject = "cpython.test_bytes.ByteArrayTest.test_partition_bytearray_doesnt_share_nullstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_partition_bytearray_doesnt_share_nullstring
"""Auto-ported test: ByteArrayTest::test_partition_bytearray_doesnt_share_nullstring (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
a, b, c = bytearray(b'x').partition(b'y')

assert b == b''

assert c == b''

assert b is not c
b += b'!'

assert c == b''
a, b, c = bytearray(b'x').partition(b'y')

assert b == b''

assert c == b''
b, c, a = bytearray(b'x').rpartition(b'y')

assert b == b''

assert c == b''

assert b is not c
b += b'!'

assert c == b''
c, b, a = bytearray(b'x').rpartition(b'y')

assert b == b''

assert c == b''
print("ByteArrayTest::test_partition_bytearray_doesnt_share_nullstring: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_partition_bytearray_doesnt_share_nullstring: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_pickling.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_pickling() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_pickling"
# subject = "cpython.test_bytes.ByteArrayTest.test_pickling"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_pickling
"""Auto-ported test: ByteArrayTest::test_pickling (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    for b in (b'', b'a', b'abc', b'\xffab\x80', b'\x00\x00\xff\x00\x00'):
        b = type2test(b)
        ps = pickle.dumps(b, proto)
        q = pickle.loads(ps)

        assert b == q
print("ByteArrayTest::test_pickling: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_pickling: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_replace.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_replace() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_replace"
# subject = "cpython.test_bytes.ByteArrayTest.test_replace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_replace
"""Auto-ported test: ByteArrayTest::test_replace (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'mississippi')

assert b.replace(b'i', b'a') == b'massassappa'

assert b.replace(b'ss', b'x') == b'mixixippi'
print("ByteArrayTest::test_replace: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_replace: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_reverse.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_reverse() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_reverse"
# subject = "cpython.test_bytes.ByteArrayTest.test_reverse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_reverse
"""Auto-ported test: ByteArrayTest::test_reverse (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = bytearray(b'hello')

assert b.reverse() == None

assert b == b'olleh'
b = bytearray(b'hello1')
b.reverse()

assert b == b'1olleh'
b = bytearray()
b.reverse()

assert not b
print("ByteArrayTest::test_reverse: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_reverse: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_rfind.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_rfind() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_rfind"
# subject = "cpython.test_bytes.ByteArrayTest.test_rfind"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_rfind
"""Auto-ported test: ByteArrayTest::test_rfind (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'mississippi')
i = 105
w = 119

assert b.rfind(b'ss') == 5

assert b.rfind(b'w') == -1

assert b.rfind(b'mississippian') == -1

assert b.rfind(i) == 10

assert b.rfind(w) == -1

assert b.rfind(b'ss', 3) == 5

assert b.rfind(b'ss', 0, 6) == 2

assert b.rfind(i, 1, 3) == 1

assert b.rfind(i, 3, 9) == 7

assert b.rfind(w, 1, 3) == -1
print("ByteArrayTest::test_rfind: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_rfind: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_rindex.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_rindex() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_rindex"
# subject = "cpython.test_bytes.ByteArrayTest.test_rindex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_rindex
"""Auto-ported test: ByteArrayTest::test_rindex (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'mississippi')
i = 105
w = 119

assert b.rindex(b'ss') == 5

try:
    b.rindex(b'w')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    b.rindex(b'mississippian')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert b.rindex(i) == 10

try:
    b.rindex(w)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert b.rindex(b'ss', 3) == 5

assert b.rindex(b'ss', 0, 6) == 2

assert b.rindex(i, 1, 3) == 1

assert b.rindex(i, 3, 9) == 7

try:
    b.rindex(w, 1, 3)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("ByteArrayTest::test_rindex: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_rindex: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_rjust.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_rjust() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_rjust"
# subject = "cpython.test_bytes.ByteArrayTest.test_rjust"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_rjust
"""Auto-ported test: ByteArrayTest::test_rjust (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'abc')
for fill_type in (bytes, bytearray):

    assert b.rjust(7, fill_type(b'-')) == type2test(b'----abc')
print("ByteArrayTest::test_rjust: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_rjust: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/byte_array_test__test_rpartition.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_byte_array_test__test_rpartition() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_rpartition"
# subject = "cpython.test_bytes.ByteArrayTest.test_rpartition"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::ByteArrayTest::test_rpartition
"""Auto-ported test: ByteArrayTest::test_rpartition (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytearray
test_exhausted_iterator = test.list_tests.CommonTest.test_exhausted_iterator

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'mississippi')

assert b.rpartition(b'ss') == (b'missi', b'ss', b'ippi')

assert b.rpartition(b'i') == (b'mississipp', b'i', b'')

assert b.rpartition(b'w') == (b'', b'', b'mississippi')
print("ByteArrayTest::test_rpartition: ok")
"###);
    assert_output(&out, r###"ByteArrayTest::test_rpartition: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_center.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_center() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_center"
# subject = "cpython.test_bytes.BytesTest.test_center"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_center
"""Auto-ported test: BytesTest::test_center (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'abc')
for fill_type in (bytes, bytearray):

    assert b.center(7, fill_type(b'-')) == type2test(b'--abc--')
print("BytesTest::test_center: ok")
"###);
    assert_output(&out, r###"BytesTest::test_center: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_compare.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_compare() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_compare"
# subject = "cpython.test_bytes.BytesTest.test_compare"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_compare
"""Auto-ported test: BytesTest::test_compare (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b1 = type2test([1, 2, 3])
b2 = type2test([1, 2, 3])
b3 = type2test([1, 3])

assert b1 == b2

assert b2 != b3

assert b1 <= b2

assert b1 <= b3

assert b1 < b3

assert b1 >= b2

assert b3 >= b2

assert b3 > b2

assert not b1 != b2

assert not b2 == b3

assert not b1 > b2

assert not b1 > b3

assert not b1 >= b3

assert not b1 < b2

assert not b3 < b2

assert not b3 <= b2
print("BytesTest::test_compare: ok")
"###);
    assert_output(&out, r###"BytesTest::test_compare: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_compare_to_str.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_compare_to_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_compare_to_str"
# subject = "cpython.test_bytes.BytesTest.test_compare_to_str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_compare_to_str
"""Auto-ported test: BytesTest::test_compare_to_str (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected

assert (type2test(b'\x00a\x00b\x00c') == 'abc') == False

assert (type2test(b'\x00\x00\x00a\x00\x00\x00b\x00\x00\x00c') == 'abc') == False

assert (type2test(b'a\x00b\x00c\x00') == 'abc') == False

assert (type2test(b'a\x00\x00\x00b\x00\x00\x00c\x00\x00\x00') == 'abc') == False

assert (type2test() == str()) == False

assert (type2test() != str()) == True
print("BytesTest::test_compare_to_str: ok")
"###);
    assert_output(&out, r###"BytesTest::test_compare_to_str: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_concat.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_concat() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_concat"
# subject = "cpython.test_bytes.BytesTest.test_concat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_concat
"""Auto-ported test: BytesTest::test_concat (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b1 = type2test(b'abc')
b2 = type2test(b'def')

assert b1 + b2 == b'abcdef'

assert b1 + bytes(b'def') == b'abcdef'

assert bytes(b'def') + b1 == b'defabc'

try:
    (lambda: b1 + 'def')()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    (lambda: 'abc' + b2)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("BytesTest::test_concat: ok")
"###);
    assert_output(&out, r###"BytesTest::test_concat: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_constructor_value_errors.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_constructor_value_errors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_constructor_value_errors"
# subject = "cpython.test_bytes.BytesTest.test_constructor_value_errors"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_constructor_value_errors
"""Auto-ported test: BytesTest::test_constructor_value_errors (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected

try:
    type2test([-1])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([-sys.maxsize])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([-sys.maxsize - 1])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([-sys.maxsize - 2])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([-10 ** 100])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([256])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([257])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([sys.maxsize])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([sys.maxsize + 1])
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    type2test([10 ** 100])
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("BytesTest::test_constructor_value_errors: ok")
"###);
    assert_output(&out, r###"BytesTest::test_constructor_value_errors: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_contains.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_contains() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_contains"
# subject = "cpython.test_bytes.BytesTest.test_contains"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_contains
"""Auto-ported test: BytesTest::test_contains (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'abc')

assert ord('a') in b

assert int(ord('a')) in b

assert 200 not in b

try:
    (lambda: 300 in b)()
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    (lambda: -1 in b)()
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    (lambda: sys.maxsize + 1 in b)()
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    (lambda: None in b)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    (lambda: float(ord('a')) in b)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    (lambda: 'a' in b)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
for f in (bytes, bytearray):

    assert f(b'') in b

    assert f(b'a') in b

    assert f(b'b') in b

    assert f(b'c') in b

    assert f(b'ab') in b

    assert f(b'bc') in b

    assert f(b'abc') in b

    assert f(b'ac') not in b

    assert f(b'd') not in b

    assert f(b'dab') not in b

    assert f(b'abd') not in b
print("BytesTest::test_contains: ok")
"###);
    assert_output(&out, r###"BytesTest::test_contains: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_copy.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_copy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_copy"
# subject = "cpython.test_bytes.BytesTest.test_copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_copy
"""Auto-ported test: BytesTest::test_copy (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
a = type2test(b'abcd')
for copy_method in (copy.copy, copy.deepcopy):
    b = copy_method(a)

    assert a == b

    assert type(a) == type(b)
print("BytesTest::test_copy: ok")
"###);
    assert_output(&out, r###"BytesTest::test_copy: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_hex_separator_five_bytes.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_hex_separator_five_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_hex_separator_five_bytes"
# subject = "cpython.test_bytes.BytesTest.test_hex_separator_five_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_hex_separator_five_bytes
"""Auto-ported test: BytesTest::test_hex_separator_five_bytes (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
five_bytes = type2test(range(90, 95))

assert five_bytes.hex() == '5a5b5c5d5e'
print("BytesTest::test_hex_separator_five_bytes: ok")
"###);
    assert_output(&out, r###"BytesTest::test_hex_separator_five_bytes: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_hex_separator_six_bytes.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_hex_separator_six_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_hex_separator_six_bytes"
# subject = "cpython.test_bytes.BytesTest.test_hex_separator_six_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_hex_separator_six_bytes
"""Auto-ported test: BytesTest::test_hex_separator_six_bytes (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
six_bytes = type2test((x * 3 for x in range(1, 7)))

assert six_bytes.hex() == '0306090c0f12'

assert six_bytes.hex('.', 1) == '03.06.09.0c.0f.12'

assert six_bytes.hex(' ', 2) == '0306 090c 0f12'

assert six_bytes.hex('-', 3) == '030609-0c0f12'

assert six_bytes.hex(':', 4) == '0306:090c0f12'

assert six_bytes.hex(':', 5) == '03:06090c0f12'

assert six_bytes.hex(':', 6) == '0306090c0f12'

assert six_bytes.hex(':', 95) == '0306090c0f12'

assert six_bytes.hex('_', -3) == '030609_0c0f12'

assert six_bytes.hex(':', -4) == '0306090c:0f12'

assert six_bytes.hex(b'@', -5) == '0306090c0f@12'

assert six_bytes.hex(':', -6) == '0306090c0f12'

assert six_bytes.hex(' ', -95) == '0306090c0f12'
print("BytesTest::test_hex_separator_six_bytes: ok")
"###);
    assert_output(&out, r###"BytesTest::test_hex_separator_six_bytes: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_index.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_index() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_index"
# subject = "cpython.test_bytes.BytesTest.test_index"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_index
"""Auto-ported test: BytesTest::test_index (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'mississippi')
i = 105
w = 119

assert b.index(b'ss') == 2

try:
    b.index(b'w')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    b.index(b'mississippian')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert b.index(i) == 1

try:
    b.index(w)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert b.index(b'ss', 3) == 5

assert b.index(b'ss', 1, 7) == 2

try:
    b.index(b'ss', 1, 3)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert b.index(i, 6) == 7

assert b.index(i, 1, 3) == 1

try:
    b.index(w, 1, 3)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("BytesTest::test_index: ok")
"###);
    assert_output(&out, r###"BytesTest::test_index: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_ljust.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_ljust() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_ljust"
# subject = "cpython.test_bytes.BytesTest.test_ljust"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_ljust
"""Auto-ported test: BytesTest::test_ljust (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'abc')
for fill_type in (bytes, bytearray):

    assert b.ljust(7, fill_type(b'-')) == type2test(b'abc----')
print("BytesTest::test_ljust: ok")
"###);
    assert_output(&out, r###"BytesTest::test_ljust: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_none_arguments.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_none_arguments() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_none_arguments"
# subject = "cpython.test_bytes.BytesTest.test_none_arguments"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_none_arguments
"""Auto-ported test: BytesTest::test_none_arguments (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'hello')
l = type2test(b'l')
h = type2test(b'h')
x = type2test(b'x')
o = type2test(b'o')

assert 2 == b.find(l, None)

assert 3 == b.find(l, -2, None)

assert 2 == b.find(l, None, -2)

assert 0 == b.find(h, None, None)

assert 3 == b.rfind(l, None)

assert 3 == b.rfind(l, -2, None)

assert 2 == b.rfind(l, None, -2)

assert 0 == b.rfind(h, None, None)

assert 2 == b.index(l, None)

assert 3 == b.index(l, -2, None)

assert 2 == b.index(l, None, -2)

assert 0 == b.index(h, None, None)

assert 3 == b.rindex(l, None)

assert 3 == b.rindex(l, -2, None)

assert 2 == b.rindex(l, None, -2)

assert 0 == b.rindex(h, None, None)

assert 2 == b.count(l, None)

assert 1 == b.count(l, -2, None)

assert 1 == b.count(l, None, -2)

assert 0 == b.count(x, None, None)

assert True == b.endswith(o, None)

assert True == b.endswith(o, -2, None)

assert True == b.endswith(l, None, -2)

assert False == b.endswith(x, None, None)

assert True == b.startswith(h, None)

assert True == b.startswith(l, -2, None)

assert True == b.startswith(h, None, -2)

assert False == b.startswith(x, None, None)
print("BytesTest::test_none_arguments: ok")
"###);
    assert_output(&out, r###"BytesTest::test_none_arguments: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_partition.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_partition() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_partition"
# subject = "cpython.test_bytes.BytesTest.test_partition"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_partition
"""Auto-ported test: BytesTest::test_partition (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'mississippi')

assert b.partition(b'ss') == (b'mi', b'ss', b'issippi')

assert b.partition(b'w') == (b'mississippi', b'', b'')
print("BytesTest::test_partition: ok")
"###);
    assert_output(&out, r###"BytesTest::test_partition: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_pickling.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_pickling() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_pickling"
# subject = "cpython.test_bytes.BytesTest.test_pickling"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_pickling
"""Auto-ported test: BytesTest::test_pickling (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    for b in (b'', b'a', b'abc', b'\xffab\x80', b'\x00\x00\xff\x00\x00'):
        b = type2test(b)
        ps = pickle.dumps(b, proto)
        q = pickle.loads(ps)

        assert b == q
print("BytesTest::test_pickling: ok")
"###);
    assert_output(&out, r###"BytesTest::test_pickling: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_repeat_1char.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_repeat_1char() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_repeat_1char"
# subject = "cpython.test_bytes.BytesTest.test_repeat_1char"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_repeat_1char
"""Auto-ported test: BytesTest::test_repeat_1char (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected

assert type2test(b'x') * 100 == type2test([ord('x')] * 100)
print("BytesTest::test_repeat_1char: ok")
"###);
    assert_output(&out, r###"BytesTest::test_repeat_1char: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_replace.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_replace() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_replace"
# subject = "cpython.test_bytes.BytesTest.test_replace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_replace
"""Auto-ported test: BytesTest::test_replace (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'mississippi')

assert b.replace(b'i', b'a') == b'massassappa'

assert b.replace(b'ss', b'x') == b'mixixippi'
print("BytesTest::test_replace: ok")
"###);
    assert_output(&out, r###"BytesTest::test_replace: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_rfind.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_rfind() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_rfind"
# subject = "cpython.test_bytes.BytesTest.test_rfind"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_rfind
"""Auto-ported test: BytesTest::test_rfind (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'mississippi')
i = 105
w = 119

assert b.rfind(b'ss') == 5

assert b.rfind(b'w') == -1

assert b.rfind(b'mississippian') == -1

assert b.rfind(i) == 10

assert b.rfind(w) == -1

assert b.rfind(b'ss', 3) == 5

assert b.rfind(b'ss', 0, 6) == 2

assert b.rfind(i, 1, 3) == 1

assert b.rfind(i, 3, 9) == 7

assert b.rfind(w, 1, 3) == -1
print("BytesTest::test_rfind: ok")
"###);
    assert_output(&out, r###"BytesTest::test_rfind: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_rindex.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_rindex() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_rindex"
# subject = "cpython.test_bytes.BytesTest.test_rindex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_rindex
"""Auto-ported test: BytesTest::test_rindex (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'mississippi')
i = 105
w = 119

assert b.rindex(b'ss') == 5

try:
    b.rindex(b'w')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    b.rindex(b'mississippian')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert b.rindex(i) == 10

try:
    b.rindex(w)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert b.rindex(b'ss', 3) == 5

assert b.rindex(b'ss', 0, 6) == 2

assert b.rindex(i, 1, 3) == 1

assert b.rindex(i, 3, 9) == 7

try:
    b.rindex(w, 1, 3)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("BytesTest::test_rindex: ok")
"###);
    assert_output(&out, r###"BytesTest::test_rindex: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_rjust.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_rjust() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_rjust"
# subject = "cpython.test_bytes.BytesTest.test_rjust"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_rjust
"""Auto-ported test: BytesTest::test_rjust (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'abc')
for fill_type in (bytes, bytearray):

    assert b.rjust(7, fill_type(b'-')) == type2test(b'----abc')
print("BytesTest::test_rjust: ok")
"###);
    assert_output(&out, r###"BytesTest::test_rjust: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/bytes/bytes_test__test_rpartition.py`.
#[test]
fn test_gen_behavior_builtin_libs_bytes_bytes_test__test_rpartition() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_rpartition"
# subject = "cpython.test_bytes.BytesTest.test_rpartition"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bytes.py::BytesTest::test_rpartition
"""Auto-ported test: BytesTest::test_rpartition (CPython 3.12 oracle)."""


import array
import os
import re
import sys
import copy
import functools
import operator
import pickle
import tempfile
import textwrap
import unittest
import test.support
from test.support import import_helper
from test.support import warnings_helper
import test.string_tests
import test.list_tests
from test.support import bigaddrspacetest, MAX_Py_ssize_t
from test.support.script_helper import assert_python_failure


'Unit tests for the bytes and bytearray types.\n\nXXX This is a mess.  Common tests should be unified with string_tests.py (and\nthe latter should be modernized).\n'

if sys.flags.bytes_warning:

    def check_bytes_warnings(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            with warnings_helper.check_warnings(('', BytesWarning)):
                return func(*args, **kw)
        return wrapper
else:

    def check_bytes_warnings(func):
        return func

class Indexable:

    def __init__(self, value=0):
        self.value = value

    def __index__(self):
        return self.value

class FixedStringTest(test.string_tests.BaseTest):

    def fixtype(self, obj):
        if isinstance(obj, str):
            return self.type2test(obj.encode('utf-8'))
        return super().fixtype(obj)
    contains_bytes = True

class ByteArraySubclass(bytearray):
    pass

class ByteArraySubclassWithSlots(bytearray):
    __slots__ = ('x', 'y', '__dict__')

class BytesSubclass(bytes):
    pass

class OtherBytesSubclass(bytes):
    pass

class WithBytes:

    def __init__(self, value):
        self.value = value

    def __bytes__(self):
        return self.value


# --- test body ---
type2test = bytes

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected
b = type2test(b'mississippi')

assert b.rpartition(b'ss') == (b'missi', b'ss', b'ippi')

assert b.rpartition(b'i') == (b'mississipp', b'i', b'')

assert b.rpartition(b'w') == (b'', b'', b'mississippi')
print("BytesTest::test_rpartition: ok")
"###);
    assert_output(&out, r###"BytesTest::test_rpartition: ok
"###);
}
