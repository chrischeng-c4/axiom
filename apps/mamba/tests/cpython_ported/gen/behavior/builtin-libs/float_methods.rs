use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/builtin-libs/float_methods/format_test_case__test_issue35560.py`.
#[test]
fn test_gen_behavior_builtin_libs_float_methods_format_test_case__test_issue35560() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "format_test_case__test_issue35560"
# subject = "cpython.test_float.FormatTestCase.test_issue35560"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_float.py::FormatTestCase::test_issue35560
"""Auto-ported test: FormatTestCase::test_issue35560 (CPython 3.12 oracle)."""


import fractions
import operator
import os
import random
import sys
import struct
import time
import unittest
from test import support
from test.support.testcase import FloatsAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from math import isinf, isnan, copysign, ldexp
import math


try:
    import _testcapi
except ImportError:
    _testcapi = None

INF = float('inf')

NAN = float('nan')

test_dir = os.path.dirname(__file__) or os.curdir

format_testfile = os.path.join(test_dir, 'formatfloat_testcases.txt')

class FloatSubclass(float):
    pass

class OtherFloatSubclass(float):
    pass

BE_DOUBLE_INF = b'\x7f\xf0\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_INF = bytes(reversed(BE_DOUBLE_INF))

BE_DOUBLE_NAN = b'\x7f\xf8\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_NAN = bytes(reversed(BE_DOUBLE_NAN))

BE_FLOAT_INF = b'\x7f\x80\x00\x00'

LE_FLOAT_INF = bytes(reversed(BE_FLOAT_INF))

BE_FLOAT_NAN = b'\x7f\xc0\x00\x00'

LE_FLOAT_NAN = bytes(reversed(BE_FLOAT_NAN))

fromHex = float.fromhex

toHex = float.hex


# --- test body ---

assert format(123.0, '00') == '123.0'

assert format(123.34, '00f') == '123.340000'

assert format(123.34, '00e') == '1.233400e+02'

assert format(123.34, '00g') == '123.34'

assert format(123.34, '00.10f') == '123.3400000000'

assert format(123.34, '00.10e') == '1.2334000000e+02'

assert format(123.34, '00.10g') == '123.34'

assert format(123.34, '01f') == '123.340000'

assert format(-123.0, '00') == '-123.0'

assert format(-123.34, '00f') == '-123.340000'

assert format(-123.34, '00e') == '-1.233400e+02'

assert format(-123.34, '00g') == '-123.34'

assert format(-123.34, '00.10f') == '-123.3400000000'

assert format(-123.34, '00.10f') == '-123.3400000000'

assert format(-123.34, '00.10e') == '-1.2334000000e+02'

assert format(-123.34, '00.10g') == '-123.34'
print("FormatTestCase::test_issue35560: ok")
"###);
    assert_output(&out, r###"FormatTestCase::test_issue35560: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/float_methods/general_float_cases__test_is_integer.py`.
#[test]
fn test_gen_behavior_builtin_libs_float_methods_general_float_cases__test_is_integer() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "general_float_cases__test_is_integer"
# subject = "cpython.test_float.GeneralFloatCases.test_is_integer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_float.py::GeneralFloatCases::test_is_integer
"""Auto-ported test: GeneralFloatCases::test_is_integer (CPython 3.12 oracle)."""


import fractions
import operator
import os
import random
import sys
import struct
import time
import unittest
from test import support
from test.support.testcase import FloatsAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from math import isinf, isnan, copysign, ldexp
import math


try:
    import _testcapi
except ImportError:
    _testcapi = None

INF = float('inf')

NAN = float('nan')

test_dir = os.path.dirname(__file__) or os.curdir

format_testfile = os.path.join(test_dir, 'formatfloat_testcases.txt')

class FloatSubclass(float):
    pass

class OtherFloatSubclass(float):
    pass

BE_DOUBLE_INF = b'\x7f\xf0\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_INF = bytes(reversed(BE_DOUBLE_INF))

BE_DOUBLE_NAN = b'\x7f\xf8\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_NAN = bytes(reversed(BE_DOUBLE_NAN))

BE_FLOAT_INF = b'\x7f\x80\x00\x00'

LE_FLOAT_INF = bytes(reversed(BE_FLOAT_INF))

BE_FLOAT_NAN = b'\x7f\xc0\x00\x00'

LE_FLOAT_NAN = bytes(reversed(BE_FLOAT_NAN))

fromHex = float.fromhex

toHex = float.hex


# --- test body ---

assert not 1.1.is_integer()

assert 1.0.is_integer()

assert not float('nan').is_integer()

assert not float('inf').is_integer()
print("GeneralFloatCases::test_is_integer: ok")
"###);
    assert_output(&out, r###"GeneralFloatCases::test_is_integer: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/float_methods/general_float_cases__test_noargs.py`.
#[test]
fn test_gen_behavior_builtin_libs_float_methods_general_float_cases__test_noargs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "general_float_cases__test_noargs"
# subject = "cpython.test_float.GeneralFloatCases.test_noargs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_float.py::GeneralFloatCases::test_noargs
"""Auto-ported test: GeneralFloatCases::test_noargs (CPython 3.12 oracle)."""


import fractions
import operator
import os
import random
import sys
import struct
import time
import unittest
from test import support
from test.support.testcase import FloatsAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from math import isinf, isnan, copysign, ldexp
import math


try:
    import _testcapi
except ImportError:
    _testcapi = None

INF = float('inf')

NAN = float('nan')

test_dir = os.path.dirname(__file__) or os.curdir

format_testfile = os.path.join(test_dir, 'formatfloat_testcases.txt')

class FloatSubclass(float):
    pass

class OtherFloatSubclass(float):
    pass

BE_DOUBLE_INF = b'\x7f\xf0\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_INF = bytes(reversed(BE_DOUBLE_INF))

BE_DOUBLE_NAN = b'\x7f\xf8\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_NAN = bytes(reversed(BE_DOUBLE_NAN))

BE_FLOAT_INF = b'\x7f\x80\x00\x00'

LE_FLOAT_INF = bytes(reversed(BE_FLOAT_INF))

BE_FLOAT_NAN = b'\x7f\xc0\x00\x00'

LE_FLOAT_NAN = bytes(reversed(BE_FLOAT_NAN))

fromHex = float.fromhex

toHex = float.hex


# --- test body ---

assert float() == 0.0
print("GeneralFloatCases::test_noargs: ok")
"###);
    assert_output(&out, r###"GeneralFloatCases::test_noargs: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/float_methods/ieee_format_test_case__test_double_specials_do_unpack.py`.
#[test]
fn test_gen_behavior_builtin_libs_float_methods_ieee_format_test_case__test_double_specials_do_unpack() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "ieee_format_test_case__test_double_specials_do_unpack"
# subject = "cpython.test_float.IEEEFormatTestCase.test_double_specials_do_unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_float.py::IEEEFormatTestCase::test_double_specials_do_unpack
"""Auto-ported test: IEEEFormatTestCase::test_double_specials_do_unpack (CPython 3.12 oracle)."""


import fractions
import operator
import os
import random
import sys
import struct
import time
import unittest
from test import support
from test.support.testcase import FloatsAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from math import isinf, isnan, copysign, ldexp
import math


try:
    import _testcapi
except ImportError:
    _testcapi = None

INF = float('inf')

NAN = float('nan')

test_dir = os.path.dirname(__file__) or os.curdir

format_testfile = os.path.join(test_dir, 'formatfloat_testcases.txt')

class FloatSubclass(float):
    pass

class OtherFloatSubclass(float):
    pass

BE_DOUBLE_INF = b'\x7f\xf0\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_INF = bytes(reversed(BE_DOUBLE_INF))

BE_DOUBLE_NAN = b'\x7f\xf8\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_NAN = bytes(reversed(BE_DOUBLE_NAN))

BE_FLOAT_INF = b'\x7f\x80\x00\x00'

LE_FLOAT_INF = bytes(reversed(BE_FLOAT_INF))

BE_FLOAT_NAN = b'\x7f\xc0\x00\x00'

LE_FLOAT_NAN = bytes(reversed(BE_FLOAT_NAN))

fromHex = float.fromhex

toHex = float.hex


# --- test body ---
for fmt, data in [('>d', BE_DOUBLE_INF), ('>d', BE_DOUBLE_NAN), ('<d', LE_DOUBLE_INF), ('<d', LE_DOUBLE_NAN)]:
    struct.unpack(fmt, data)
print("IEEEFormatTestCase::test_double_specials_do_unpack: ok")
"###);
    assert_output(&out, r###"IEEEFormatTestCase::test_double_specials_do_unpack: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/float_methods/ieee_format_test_case__test_float_specials_do_unpack.py`.
#[test]
fn test_gen_behavior_builtin_libs_float_methods_ieee_format_test_case__test_float_specials_do_unpack() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "ieee_format_test_case__test_float_specials_do_unpack"
# subject = "cpython.test_float.IEEEFormatTestCase.test_float_specials_do_unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_float.py::IEEEFormatTestCase::test_float_specials_do_unpack
"""Auto-ported test: IEEEFormatTestCase::test_float_specials_do_unpack (CPython 3.12 oracle)."""


import fractions
import operator
import os
import random
import sys
import struct
import time
import unittest
from test import support
from test.support.testcase import FloatsAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from math import isinf, isnan, copysign, ldexp
import math


try:
    import _testcapi
except ImportError:
    _testcapi = None

INF = float('inf')

NAN = float('nan')

test_dir = os.path.dirname(__file__) or os.curdir

format_testfile = os.path.join(test_dir, 'formatfloat_testcases.txt')

class FloatSubclass(float):
    pass

class OtherFloatSubclass(float):
    pass

BE_DOUBLE_INF = b'\x7f\xf0\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_INF = bytes(reversed(BE_DOUBLE_INF))

BE_DOUBLE_NAN = b'\x7f\xf8\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_NAN = bytes(reversed(BE_DOUBLE_NAN))

BE_FLOAT_INF = b'\x7f\x80\x00\x00'

LE_FLOAT_INF = bytes(reversed(BE_FLOAT_INF))

BE_FLOAT_NAN = b'\x7f\xc0\x00\x00'

LE_FLOAT_NAN = bytes(reversed(BE_FLOAT_NAN))

fromHex = float.fromhex

toHex = float.hex


# --- test body ---
for fmt, data in [('>f', BE_FLOAT_INF), ('>f', BE_FLOAT_NAN), ('<f', LE_FLOAT_INF), ('<f', LE_FLOAT_NAN)]:
    struct.unpack(fmt, data)
print("IEEEFormatTestCase::test_float_specials_do_unpack: ok")
"###);
    assert_output(&out, r###"IEEEFormatTestCase::test_float_specials_do_unpack: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/float_methods/inf_nan_test__test_inf_as_str.py`.
#[test]
fn test_gen_behavior_builtin_libs_float_methods_inf_nan_test__test_inf_as_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "inf_nan_test__test_inf_as_str"
# subject = "cpython.test_float.InfNanTest.test_inf_as_str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_float.py::InfNanTest::test_inf_as_str
"""Auto-ported test: InfNanTest::test_inf_as_str (CPython 3.12 oracle)."""


import fractions
import operator
import os
import random
import sys
import struct
import time
import unittest
from test import support
from test.support.testcase import FloatsAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from math import isinf, isnan, copysign, ldexp
import math


try:
    import _testcapi
except ImportError:
    _testcapi = None

INF = float('inf')

NAN = float('nan')

test_dir = os.path.dirname(__file__) or os.curdir

format_testfile = os.path.join(test_dir, 'formatfloat_testcases.txt')

class FloatSubclass(float):
    pass

class OtherFloatSubclass(float):
    pass

BE_DOUBLE_INF = b'\x7f\xf0\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_INF = bytes(reversed(BE_DOUBLE_INF))

BE_DOUBLE_NAN = b'\x7f\xf8\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_NAN = bytes(reversed(BE_DOUBLE_NAN))

BE_FLOAT_INF = b'\x7f\x80\x00\x00'

LE_FLOAT_INF = bytes(reversed(BE_FLOAT_INF))

BE_FLOAT_NAN = b'\x7f\xc0\x00\x00'

LE_FLOAT_NAN = bytes(reversed(BE_FLOAT_NAN))

fromHex = float.fromhex

toHex = float.hex


# --- test body ---

assert repr(1e+300 * 1e+300) == 'inf'

assert repr(-1e+300 * 1e+300) == '-inf'

assert str(1e+300 * 1e+300) == 'inf'

assert str(-1e+300 * 1e+300) == '-inf'
print("InfNanTest::test_inf_as_str: ok")
"###);
    assert_output(&out, r###"InfNanTest::test_inf_as_str: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/float_methods/inf_nan_test__test_inf_from_str.py`.
#[test]
fn test_gen_behavior_builtin_libs_float_methods_inf_nan_test__test_inf_from_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "inf_nan_test__test_inf_from_str"
# subject = "cpython.test_float.InfNanTest.test_inf_from_str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_float.py::InfNanTest::test_inf_from_str
"""Auto-ported test: InfNanTest::test_inf_from_str (CPython 3.12 oracle)."""


import fractions
import operator
import os
import random
import sys
import struct
import time
import unittest
from test import support
from test.support.testcase import FloatsAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from math import isinf, isnan, copysign, ldexp
import math


try:
    import _testcapi
except ImportError:
    _testcapi = None

INF = float('inf')

NAN = float('nan')

test_dir = os.path.dirname(__file__) or os.curdir

format_testfile = os.path.join(test_dir, 'formatfloat_testcases.txt')

class FloatSubclass(float):
    pass

class OtherFloatSubclass(float):
    pass

BE_DOUBLE_INF = b'\x7f\xf0\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_INF = bytes(reversed(BE_DOUBLE_INF))

BE_DOUBLE_NAN = b'\x7f\xf8\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_NAN = bytes(reversed(BE_DOUBLE_NAN))

BE_FLOAT_INF = b'\x7f\x80\x00\x00'

LE_FLOAT_INF = bytes(reversed(BE_FLOAT_INF))

BE_FLOAT_NAN = b'\x7f\xc0\x00\x00'

LE_FLOAT_NAN = bytes(reversed(BE_FLOAT_NAN))

fromHex = float.fromhex

toHex = float.hex


# --- test body ---

assert isinf(float('inf'))

assert isinf(float('+inf'))

assert isinf(float('-inf'))

assert isinf(float('infinity'))

assert isinf(float('+infinity'))

assert isinf(float('-infinity'))

assert repr(float('inf')) == 'inf'

assert repr(float('+inf')) == 'inf'

assert repr(float('-inf')) == '-inf'

assert repr(float('infinity')) == 'inf'

assert repr(float('+infinity')) == 'inf'

assert repr(float('-infinity')) == '-inf'

assert repr(float('INF')) == 'inf'

assert repr(float('+Inf')) == 'inf'

assert repr(float('-iNF')) == '-inf'

assert repr(float('Infinity')) == 'inf'

assert repr(float('+iNfInItY')) == 'inf'

assert repr(float('-INFINITY')) == '-inf'

assert str(float('inf')) == 'inf'

assert str(float('+inf')) == 'inf'

assert str(float('-inf')) == '-inf'

assert str(float('infinity')) == 'inf'

assert str(float('+infinity')) == 'inf'

assert str(float('-infinity')) == '-inf'

try:
    float('info')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('+info')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('-info')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('in')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('+in')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('-in')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('infinit')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('+Infin')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('-INFI')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('infinitys')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('++Inf')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('-+inf')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('+-infinity')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('--Infinity')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("InfNanTest::test_inf_from_str: ok")
"###);
    assert_output(&out, r###"InfNanTest::test_inf_from_str: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/float_methods/inf_nan_test__test_inf_signs.py`.
#[test]
fn test_gen_behavior_builtin_libs_float_methods_inf_nan_test__test_inf_signs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "inf_nan_test__test_inf_signs"
# subject = "cpython.test_float.InfNanTest.test_inf_signs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_float.py::InfNanTest::test_inf_signs
"""Auto-ported test: InfNanTest::test_inf_signs (CPython 3.12 oracle)."""


import fractions
import operator
import os
import random
import sys
import struct
import time
import unittest
from test import support
from test.support.testcase import FloatsAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from math import isinf, isnan, copysign, ldexp
import math


try:
    import _testcapi
except ImportError:
    _testcapi = None

INF = float('inf')

NAN = float('nan')

test_dir = os.path.dirname(__file__) or os.curdir

format_testfile = os.path.join(test_dir, 'formatfloat_testcases.txt')

class FloatSubclass(float):
    pass

class OtherFloatSubclass(float):
    pass

BE_DOUBLE_INF = b'\x7f\xf0\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_INF = bytes(reversed(BE_DOUBLE_INF))

BE_DOUBLE_NAN = b'\x7f\xf8\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_NAN = bytes(reversed(BE_DOUBLE_NAN))

BE_FLOAT_INF = b'\x7f\x80\x00\x00'

LE_FLOAT_INF = bytes(reversed(BE_FLOAT_INF))

BE_FLOAT_NAN = b'\x7f\xc0\x00\x00'

LE_FLOAT_NAN = bytes(reversed(BE_FLOAT_NAN))

fromHex = float.fromhex

toHex = float.hex


# --- test body ---

assert copysign(1.0, float('inf')) == 1.0

assert copysign(1.0, float('-inf')) == -1.0
print("InfNanTest::test_inf_signs: ok")
"###);
    assert_output(&out, r###"InfNanTest::test_inf_signs: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/float_methods/inf_nan_test__test_nan_as_str.py`.
#[test]
fn test_gen_behavior_builtin_libs_float_methods_inf_nan_test__test_nan_as_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "inf_nan_test__test_nan_as_str"
# subject = "cpython.test_float.InfNanTest.test_nan_as_str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_float.py::InfNanTest::test_nan_as_str
"""Auto-ported test: InfNanTest::test_nan_as_str (CPython 3.12 oracle)."""


import fractions
import operator
import os
import random
import sys
import struct
import time
import unittest
from test import support
from test.support.testcase import FloatsAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from math import isinf, isnan, copysign, ldexp
import math


try:
    import _testcapi
except ImportError:
    _testcapi = None

INF = float('inf')

NAN = float('nan')

test_dir = os.path.dirname(__file__) or os.curdir

format_testfile = os.path.join(test_dir, 'formatfloat_testcases.txt')

class FloatSubclass(float):
    pass

class OtherFloatSubclass(float):
    pass

BE_DOUBLE_INF = b'\x7f\xf0\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_INF = bytes(reversed(BE_DOUBLE_INF))

BE_DOUBLE_NAN = b'\x7f\xf8\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_NAN = bytes(reversed(BE_DOUBLE_NAN))

BE_FLOAT_INF = b'\x7f\x80\x00\x00'

LE_FLOAT_INF = bytes(reversed(BE_FLOAT_INF))

BE_FLOAT_NAN = b'\x7f\xc0\x00\x00'

LE_FLOAT_NAN = bytes(reversed(BE_FLOAT_NAN))

fromHex = float.fromhex

toHex = float.hex


# --- test body ---

assert repr(1e+300 * 1e+300 * 0) == 'nan'

assert repr(-1e+300 * 1e+300 * 0) == 'nan'

assert str(1e+300 * 1e+300 * 0) == 'nan'

assert str(-1e+300 * 1e+300 * 0) == 'nan'
print("InfNanTest::test_nan_as_str: ok")
"###);
    assert_output(&out, r###"InfNanTest::test_nan_as_str: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/float_methods/inf_nan_test__test_nan_from_str.py`.
#[test]
fn test_gen_behavior_builtin_libs_float_methods_inf_nan_test__test_nan_from_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "inf_nan_test__test_nan_from_str"
# subject = "cpython.test_float.InfNanTest.test_nan_from_str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_float.py::InfNanTest::test_nan_from_str
"""Auto-ported test: InfNanTest::test_nan_from_str (CPython 3.12 oracle)."""


import fractions
import operator
import os
import random
import sys
import struct
import time
import unittest
from test import support
from test.support.testcase import FloatsAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from math import isinf, isnan, copysign, ldexp
import math


try:
    import _testcapi
except ImportError:
    _testcapi = None

INF = float('inf')

NAN = float('nan')

test_dir = os.path.dirname(__file__) or os.curdir

format_testfile = os.path.join(test_dir, 'formatfloat_testcases.txt')

class FloatSubclass(float):
    pass

class OtherFloatSubclass(float):
    pass

BE_DOUBLE_INF = b'\x7f\xf0\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_INF = bytes(reversed(BE_DOUBLE_INF))

BE_DOUBLE_NAN = b'\x7f\xf8\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_NAN = bytes(reversed(BE_DOUBLE_NAN))

BE_FLOAT_INF = b'\x7f\x80\x00\x00'

LE_FLOAT_INF = bytes(reversed(BE_FLOAT_INF))

BE_FLOAT_NAN = b'\x7f\xc0\x00\x00'

LE_FLOAT_NAN = bytes(reversed(BE_FLOAT_NAN))

fromHex = float.fromhex

toHex = float.hex


# --- test body ---

assert isnan(float('nan'))

assert isnan(float('+nan'))

assert isnan(float('-nan'))

assert repr(float('nan')) == 'nan'

assert repr(float('+nan')) == 'nan'

assert repr(float('-nan')) == 'nan'

assert repr(float('NAN')) == 'nan'

assert repr(float('+NAn')) == 'nan'

assert repr(float('-NaN')) == 'nan'

assert str(float('nan')) == 'nan'

assert str(float('+nan')) == 'nan'

assert str(float('-nan')) == 'nan'

try:
    float('nana')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('+nana')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('-nana')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('na')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('+na')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('-na')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('++nan')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('-+NAN')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('+-NaN')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('--nAn')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("InfNanTest::test_nan_from_str: ok")
"###);
    assert_output(&out, r###"InfNanTest::test_nan_from_str: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/float_methods/round_test_case__test_inf_nan_ndigits.py`.
#[test]
fn test_gen_behavior_builtin_libs_float_methods_round_test_case__test_inf_nan_ndigits() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "round_test_case__test_inf_nan_ndigits"
# subject = "cpython.test_float.RoundTestCase.test_inf_nan_ndigits"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_float.py::RoundTestCase::test_inf_nan_ndigits
"""Auto-ported test: RoundTestCase::test_inf_nan_ndigits (CPython 3.12 oracle)."""


import fractions
import operator
import os
import random
import sys
import struct
import time
import unittest
from test import support
from test.support.testcase import FloatsAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from math import isinf, isnan, copysign, ldexp
import math


try:
    import _testcapi
except ImportError:
    _testcapi = None

INF = float('inf')

NAN = float('nan')

test_dir = os.path.dirname(__file__) or os.curdir

format_testfile = os.path.join(test_dir, 'formatfloat_testcases.txt')

class FloatSubclass(float):
    pass

class OtherFloatSubclass(float):
    pass

BE_DOUBLE_INF = b'\x7f\xf0\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_INF = bytes(reversed(BE_DOUBLE_INF))

BE_DOUBLE_NAN = b'\x7f\xf8\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_NAN = bytes(reversed(BE_DOUBLE_NAN))

BE_FLOAT_INF = b'\x7f\x80\x00\x00'

LE_FLOAT_INF = bytes(reversed(BE_FLOAT_INF))

BE_FLOAT_NAN = b'\x7f\xc0\x00\x00'

LE_FLOAT_NAN = bytes(reversed(BE_FLOAT_NAN))

fromHex = float.fromhex

toHex = float.hex


# --- test body ---

assert round(INF, 0) == INF

assert round(-INF, 0) == -INF

assert math.isnan(round(NAN, 0))
print("RoundTestCase::test_inf_nan_ndigits: ok")
"###);
    assert_output(&out, r###"RoundTestCase::test_inf_nan_ndigits: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/float_methods/round_test_case__test_none_ndigits.py`.
#[test]
fn test_gen_behavior_builtin_libs_float_methods_round_test_case__test_none_ndigits() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "float_methods"
# dimension = "behavior"
# case = "round_test_case__test_none_ndigits"
# subject = "cpython.test_float.RoundTestCase.test_None_ndigits"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_float.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_float.py::RoundTestCase::test_None_ndigits
"""Auto-ported test: RoundTestCase::test_None_ndigits (CPython 3.12 oracle)."""


import fractions
import operator
import os
import random
import sys
import struct
import time
import unittest
from test import support
from test.support.testcase import FloatsAreIdenticalMixin
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from math import isinf, isnan, copysign, ldexp
import math


try:
    import _testcapi
except ImportError:
    _testcapi = None

INF = float('inf')

NAN = float('nan')

test_dir = os.path.dirname(__file__) or os.curdir

format_testfile = os.path.join(test_dir, 'formatfloat_testcases.txt')

class FloatSubclass(float):
    pass

class OtherFloatSubclass(float):
    pass

BE_DOUBLE_INF = b'\x7f\xf0\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_INF = bytes(reversed(BE_DOUBLE_INF))

BE_DOUBLE_NAN = b'\x7f\xf8\x00\x00\x00\x00\x00\x00'

LE_DOUBLE_NAN = bytes(reversed(BE_DOUBLE_NAN))

BE_FLOAT_INF = b'\x7f\x80\x00\x00'

LE_FLOAT_INF = bytes(reversed(BE_FLOAT_INF))

BE_FLOAT_NAN = b'\x7f\xc0\x00\x00'

LE_FLOAT_NAN = bytes(reversed(BE_FLOAT_NAN))

fromHex = float.fromhex

toHex = float.hex


# --- test body ---
for x in (round(1.23), round(1.23, None), round(1.23, ndigits=None)):

    assert x == 1

    assert isinstance(x, int)
for x in (round(1.78), round(1.78, None), round(1.78, ndigits=None)):

    assert x == 2

    assert isinstance(x, int)
print("RoundTestCase::test_None_ndigits: ok")
"###);
    assert_output(&out, r###"RoundTestCase::test_None_ndigits: ok
"###);
}
