use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/builtin-libs/int_methods/int_test_cases__test_invalid_signs.py`.
#[test]
fn test_gen_behavior_builtin_libs_int_methods_int_test_cases__test_invalid_signs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "int_methods"
# dimension = "behavior"
# case = "int_test_cases__test_invalid_signs"
# subject = "cpython.test_int.IntTestCases.test_invalid_signs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_int.py::IntTestCases::test_invalid_signs
"""Auto-ported test: IntTestCases::test_invalid_signs (CPython 3.12 oracle)."""


import sys
import time
import unittest
from unittest import mock
from test import support
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS


try:
    import _pylong
except ImportError:
    _pylong = None

L = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), ('Ȁ', ValueError)]

class IntSubclass(int):
    pass


# --- test body ---
try:
    int('+')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    int('-')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    int('- 1')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    int('+ 1')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    int(' + 1 ')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("IntTestCases::test_invalid_signs: ok")
"###);
    assert_output(&out, r###"IntTestCases::test_invalid_signs: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/int_methods/int_test_cases__test_issue31619.py`.
#[test]
fn test_gen_behavior_builtin_libs_int_methods_int_test_cases__test_issue31619() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "int_methods"
# dimension = "behavior"
# case = "int_test_cases__test_issue31619"
# subject = "cpython.test_int.IntTestCases.test_issue31619"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_int.py::IntTestCases::test_issue31619
"""Auto-ported test: IntTestCases::test_issue31619 (CPython 3.12 oracle)."""


import sys
import time
import unittest
from unittest import mock
from test import support
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS


try:
    import _pylong
except ImportError:
    _pylong = None

L = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), ('Ȁ', ValueError)]

class IntSubclass(int):
    pass


# --- test body ---

assert int('1_0_1_0_1_0_1_0_1_0_1_0_1_0_1_0_1_0_1_0_1_0_1_0_1_0_1_0_1_0_1', 2) == 1431655765

assert int('1_2_3_4_5_6_7_0_1_2_3', 8) == 1402433619

assert int('1_2_3_4_5_6_7_8_9', 16) == 4886718345

assert int('1_2_3_4_5_6_7', 32) == 1144132807
print("IntTestCases::test_issue31619: ok")
"###);
    assert_output(&out, r###"IntTestCases::test_issue31619: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/int_methods/int_test_cases__test_no_args.py`.
#[test]
fn test_gen_behavior_builtin_libs_int_methods_int_test_cases__test_no_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "int_methods"
# dimension = "behavior"
# case = "int_test_cases__test_no_args"
# subject = "cpython.test_int.IntTestCases.test_no_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_int.py::IntTestCases::test_no_args
"""Auto-ported test: IntTestCases::test_no_args (CPython 3.12 oracle)."""


import sys
import time
import unittest
from unittest import mock
from test import support
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS


try:
    import _pylong
except ImportError:
    _pylong = None

L = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), ('Ȁ', ValueError)]

class IntSubclass(int):
    pass


# --- test body ---

assert int() == 0
print("IntTestCases::test_no_args: ok")
"###);
    assert_output(&out, r###"IntTestCases::test_no_args: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/int_methods/int_test_cases__test_string_float.py`.
#[test]
fn test_gen_behavior_builtin_libs_int_methods_int_test_cases__test_string_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "int_methods"
# dimension = "behavior"
# case = "int_test_cases__test_string_float"
# subject = "cpython.test_int.IntTestCases.test_string_float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_int.py::IntTestCases::test_string_float
"""Auto-ported test: IntTestCases::test_string_float (CPython 3.12 oracle)."""


import sys
import time
import unittest
from unittest import mock
from test import support
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS


try:
    import _pylong
except ImportError:
    _pylong = None

L = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), ('Ȁ', ValueError)]

class IntSubclass(int):
    pass


# --- test body ---

try:
    int('1.2')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("IntTestCases::test_string_float: ok")
"###);
    assert_output(&out, r###"IntTestCases::test_string_float: ok
"###);
}
