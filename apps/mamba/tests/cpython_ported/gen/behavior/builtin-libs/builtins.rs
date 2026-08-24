use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_bug_27936.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_bug_27936() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_bug_27936"
# subject = "cpython.test_builtin.BuiltinTest.test_bug_27936"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_bug_27936
"""Auto-ported test: BuiltinTest::test_bug_27936 (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1
for x in [1234, 1234.56, decimal.Decimal('1234.56'), fractions.Fraction(123456, 100)]:

    assert round(x, None) == round(x)

    assert type(round(x, None)) == type(round(x))
print("BuiltinTest::test_bug_27936: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_bug_27936: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_cmp.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_cmp() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_cmp"
# subject = "cpython.test_builtin.BuiltinTest.test_cmp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_cmp
"""Auto-ported test: BuiltinTest::test_cmp (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1

assert not hasattr(builtins, 'cmp')
print("BuiltinTest::test_cmp: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_cmp: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_compile_top_level_await_no_coro.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_compile_top_level_await_no_coro() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_compile_top_level_await_no_coro"
# subject = "cpython.test_builtin.BuiltinTest.test_compile_top_level_await_no_coro"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_compile_top_level_await_no_coro
"""Auto-ported test: BuiltinTest::test_compile_top_level_await_no_coro (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1
'Make sure top level non-await codes get the correct coroutine flags'
modes = ('single', 'exec')
code_samples = ['def f():pass\n', '[x for x in l]', '{x for x in l}', '(x for x in l)', '{x:x for x in l}']
for mode, code_sample in product(modes, code_samples):
    source = dedent(code_sample)
    co = compile(source, '?', mode, flags=ast.PyCF_ALLOW_TOP_LEVEL_AWAIT)

    assert co.co_flags & CO_COROUTINE != CO_COROUTINE
print("BuiltinTest::test_compile_top_level_await_no_coro: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_compile_top_level_await_no_coro: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_exec_redirected.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_exec_redirected() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_exec_redirected"
# subject = "cpython.test_builtin.BuiltinTest.test_exec_redirected"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_exec_redirected
"""Auto-ported test: BuiltinTest::test_exec_redirected (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1
savestdout = sys.stdout
sys.stdout = None
try:
    exec('a')
except NameError:
    pass
finally:
    sys.stdout = savestdout
print("BuiltinTest::test_exec_redirected: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_exec_redirected: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_filter_dealloc.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_filter_dealloc() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_filter_dealloc"
# subject = "cpython.test_builtin.BuiltinTest.test_filter_dealloc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_filter_dealloc
"""Auto-ported test: BuiltinTest::test_filter_dealloc (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1
max_iters = 1000000
i = filter(bool, range(max_iters))
for _ in range(max_iters):
    i = filter(bool, i)
del i
gc.collect()
print("BuiltinTest::test_filter_dealloc: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_filter_dealloc: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_filter_pickle.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_filter_pickle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_filter_pickle"
# subject = "cpython.test_builtin.BuiltinTest.test_filter_pickle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_filter_pickle
"""Auto-ported test: BuiltinTest::test_filter_pickle (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1

def check_iter_pickle(it, seq, proto):
    itorg = it
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert type(itorg) == type(it)

    assert list(it) == seq
    it = pickle.loads(d)
    try:
        next(it)
    except StopIteration:
        return
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert list(it) == seq[1:]

def get_vars_f0():
    return vars()

def get_vars_f2():
    BuiltinTest.get_vars_f0()
    a = 1
    b = 2
    return vars()

def iter_error(iterable, error):
    """Collect `iterable` into a list, catching an expected `error`."""
    items = []
    try:
        for item in iterable:
            items.append(item)
        raise AssertionError('expected error')
    except error:
        pass
    return items

def write_testfile():
    fp = open(TESTFN, 'w', encoding='utf-8')
    pass
    with fp:
        fp.write('1+1\n')
        fp.write('The quick brown fox jumps over the lazy dog')
        fp.write('.\n')
        fp.write('Dear John\n')
        fp.write('XXX' * 100)
        fp.write('YYY' * 100)
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    f1 = filter(filter_char, 'abcdeabcde')
    f2 = filter(filter_char, 'abcdeabcde')
    check_iter_pickle(f1, list(f2), proto)
print("BuiltinTest::test_filter_pickle: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_filter_pickle: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_id.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_id() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_id"
# subject = "cpython.test_builtin.BuiltinTest.test_id"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_id
"""Auto-ported test: BuiltinTest::test_id (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1
id(None)
id(1)
id(1.0)
id('spam')
id((0, 1, 2, 3))
id([0, 1, 2, 3])
id({'spam': 1, 'eggs': 2, 'ham': 3})
print("BuiltinTest::test_id: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_id: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_map_pickle.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_map_pickle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_map_pickle"
# subject = "cpython.test_builtin.BuiltinTest.test_map_pickle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_map_pickle
"""Auto-ported test: BuiltinTest::test_map_pickle (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1

def check_iter_pickle(it, seq, proto):
    itorg = it
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert type(itorg) == type(it)

    assert list(it) == seq
    it = pickle.loads(d)
    try:
        next(it)
    except StopIteration:
        return
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert list(it) == seq[1:]

def get_vars_f0():
    return vars()

def get_vars_f2():
    BuiltinTest.get_vars_f0()
    a = 1
    b = 2
    return vars()

def iter_error(iterable, error):
    """Collect `iterable` into a list, catching an expected `error`."""
    items = []
    try:
        for item in iterable:
            items.append(item)
        raise AssertionError('expected error')
    except error:
        pass
    return items

def write_testfile():
    fp = open(TESTFN, 'w', encoding='utf-8')
    pass
    with fp:
        fp.write('1+1\n')
        fp.write('The quick brown fox jumps over the lazy dog')
        fp.write('.\n')
        fp.write('Dear John\n')
        fp.write('XXX' * 100)
        fp.write('YYY' * 100)
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    m1 = map(map_char, 'Is this the real life?')
    m2 = map(map_char, 'Is this the real life?')
    check_iter_pickle(m1, list(m2), proto)
print("BuiltinTest::test_map_pickle: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_map_pickle: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_neg.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_neg() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_neg"
# subject = "cpython.test_builtin.BuiltinTest.test_neg"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_neg
"""Auto-ported test: BuiltinTest::test_neg (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1
x = -sys.maxsize - 1

assert isinstance(x, int)

assert -x == sys.maxsize + 1
print("BuiltinTest::test_neg: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_neg: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_next.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_next() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_next"
# subject = "cpython.test_builtin.BuiltinTest.test_next"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_next
"""Auto-ported test: BuiltinTest::test_next (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1
it = iter(range(2))

assert next(it) == 0

assert next(it) == 1

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert next(it, 42) == 42

class Iter(object):

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration
it = iter(Iter())

assert next(it, 42) == 42

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

def gen():
    yield 1
    return
it = gen()

assert next(it) == 1

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert next(it, 42) == 42
print("BuiltinTest::test_next: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_next: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_open.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_open() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_open"
# subject = "cpython.test_builtin.BuiltinTest.test_open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_open
"""Auto-ported test: BuiltinTest::test_open (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1

def check_iter_pickle(it, seq, proto):
    itorg = it
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert type(itorg) == type(it)

    assert list(it) == seq
    it = pickle.loads(d)
    try:
        next(it)
    except StopIteration:
        return
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert list(it) == seq[1:]

def get_vars_f0():
    return vars()

def get_vars_f2():
    BuiltinTest.get_vars_f0()
    a = 1
    b = 2
    return vars()

def iter_error(iterable, error):
    """Collect `iterable` into a list, catching an expected `error`."""
    items = []
    try:
        for item in iterable:
            items.append(item)
        raise AssertionError('expected error')
    except error:
        pass
    return items

def write_testfile():
    fp = open(TESTFN, 'w', encoding='utf-8')
    pass
    with fp:
        fp.write('1+1\n')
        fp.write('The quick brown fox jumps over the lazy dog')
        fp.write('.\n')
        fp.write('Dear John\n')
        fp.write('XXX' * 100)
        fp.write('YYY' * 100)
write_testfile()
fp = open(TESTFN, encoding='utf-8')
with fp:

    assert fp.readline(4) == '1+1\n'

    assert fp.readline() == 'The quick brown fox jumps over the lazy dog.\n'

    assert fp.readline(4) == 'Dear'

    assert fp.readline(100) == ' John\n'

    assert fp.read(300) == 'XXX' * 100

    assert fp.read(1000) == 'YYY' * 100

try:
    open('a\x00b')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    open(b'a\x00b')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("BuiltinTest::test_open: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_open: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_type.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_type() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_type"
# subject = "cpython.test_builtin.BuiltinTest.test_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_type
"""Auto-ported test: BuiltinTest::test_type (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1

assert type('') == type('123')

assert type('') != type(())
print("BuiltinTest::test_type: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_type: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_zip_bad_iterable.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_zip_bad_iterable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_zip_bad_iterable"
# subject = "cpython.test_builtin.BuiltinTest.test_zip_bad_iterable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_zip_bad_iterable
"""Auto-ported test: BuiltinTest::test_zip_bad_iterable (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1
exception = TypeError()

class BadIterable:

    def __iter__(self):
        raise exception
try:
    zip(BadIterable())
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert cm.exception is exception
print("BuiltinTest::test_zip_bad_iterable: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_zip_bad_iterable: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_zip_pickle.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_zip_pickle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_zip_pickle"
# subject = "cpython.test_builtin.BuiltinTest.test_zip_pickle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_zip_pickle
"""Auto-ported test: BuiltinTest::test_zip_pickle (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1

def check_iter_pickle(it, seq, proto):
    itorg = it
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert type(itorg) == type(it)

    assert list(it) == seq
    it = pickle.loads(d)
    try:
        next(it)
    except StopIteration:
        return
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert list(it) == seq[1:]

def get_vars_f0():
    return vars()

def get_vars_f2():
    BuiltinTest.get_vars_f0()
    a = 1
    b = 2
    return vars()

def iter_error(iterable, error):
    """Collect `iterable` into a list, catching an expected `error`."""
    items = []
    try:
        for item in iterable:
            items.append(item)
        raise AssertionError('expected error')
    except error:
        pass
    return items

def write_testfile():
    fp = open(TESTFN, 'w', encoding='utf-8')
    pass
    with fp:
        fp.write('1+1\n')
        fp.write('The quick brown fox jumps over the lazy dog')
        fp.write('.\n')
        fp.write('Dear John\n')
        fp.write('XXX' * 100)
        fp.write('YYY' * 100)
a = (1, 2, 3)
b = (4, 5, 6)
t = [(1, 4), (2, 5), (3, 6)]
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    z1 = zip(a, b)
    check_iter_pickle(z1, t, proto)
print("BuiltinTest::test_zip_pickle: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_zip_pickle: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_zip_pickle_strict.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_zip_pickle_strict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_zip_pickle_strict"
# subject = "cpython.test_builtin.BuiltinTest.test_zip_pickle_strict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_zip_pickle_strict
"""Auto-ported test: BuiltinTest::test_zip_pickle_strict (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1

def check_iter_pickle(it, seq, proto):
    itorg = it
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert type(itorg) == type(it)

    assert list(it) == seq
    it = pickle.loads(d)
    try:
        next(it)
    except StopIteration:
        return
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert list(it) == seq[1:]

def get_vars_f0():
    return vars()

def get_vars_f2():
    BuiltinTest.get_vars_f0()
    a = 1
    b = 2
    return vars()

def iter_error(iterable, error):
    """Collect `iterable` into a list, catching an expected `error`."""
    items = []
    try:
        for item in iterable:
            items.append(item)
        raise AssertionError('expected error')
    except error:
        pass
    return items

def write_testfile():
    fp = open(TESTFN, 'w', encoding='utf-8')
    pass
    with fp:
        fp.write('1+1\n')
        fp.write('The quick brown fox jumps over the lazy dog')
        fp.write('.\n')
        fp.write('Dear John\n')
        fp.write('XXX' * 100)
        fp.write('YYY' * 100)
a = (1, 2, 3)
b = (4, 5, 6)
t = [(1, 4), (2, 5), (3, 6)]
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    z1 = zip(a, b, strict=True)
    check_iter_pickle(z1, t, proto)
print("BuiltinTest::test_zip_pickle_strict: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_zip_pickle_strict: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_zip_strict.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_zip_strict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_zip_strict"
# subject = "cpython.test_builtin.BuiltinTest.test_zip_strict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_zip_strict
"""Auto-ported test: BuiltinTest::test_zip_strict (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1

assert tuple(zip((1, 2, 3), 'abc', strict=True)) == ((1, 'a'), (2, 'b'), (3, 'c'))

try:
    tuple(zip((1, 2, 3, 4), 'abc', strict=True))
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    tuple(zip((1, 2), 'abc', strict=True))
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    tuple(zip((1, 2), (1, 2), 'abc', strict=True))
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("BuiltinTest::test_zip_strict: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_zip_strict: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_zip_strict_error_handling_stopiteration.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_zip_strict_error_handling_stopiteration() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_zip_strict_error_handling_stopiteration"
# subject = "cpython.test_builtin.BuiltinTest.test_zip_strict_error_handling_stopiteration"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_zip_strict_error_handling_stopiteration
"""Auto-ported test: BuiltinTest::test_zip_strict_error_handling_stopiteration (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1

def check_iter_pickle(it, seq, proto):
    itorg = it
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert type(itorg) == type(it)

    assert list(it) == seq
    it = pickle.loads(d)
    try:
        next(it)
    except StopIteration:
        return
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert list(it) == seq[1:]

def get_vars_f0():
    return vars()

def get_vars_f2():
    BuiltinTest.get_vars_f0()
    a = 1
    b = 2
    return vars()

def iter_error(iterable, error):
    """Collect `iterable` into a list, catching an expected `error`."""
    items = []
    try:
        for item in iterable:
            items.append(item)
        raise AssertionError('expected error')
    except error:
        pass
    return items

def write_testfile():
    fp = open(TESTFN, 'w', encoding='utf-8')
    pass
    with fp:
        fp.write('1+1\n')
        fp.write('The quick brown fox jumps over the lazy dog')
        fp.write('.\n')
        fp.write('Dear John\n')
        fp.write('XXX' * 100)
        fp.write('YYY' * 100)

class Iter:

    def __init__(self, size):
        self.size = size

    def __iter__(self):
        return self

    def __next__(self):
        self.size -= 1
        if self.size < 0:
            raise StopIteration
        return self.size
l1 = iter_error(zip('AB', Iter(1), strict=True), ValueError)

assert l1 == [('A', 0)]
l2 = iter_error(zip('AB', Iter(2), 'A', strict=True), ValueError)

assert l2 == [('A', 1, 'A')]
l3 = iter_error(zip('AB', Iter(2), 'ABC', strict=True), ValueError)

assert l3 == [('A', 1, 'A'), ('B', 0, 'B')]
l4 = iter_error(zip('AB', Iter(3), strict=True), ValueError)

assert l4 == [('A', 2), ('B', 1)]
l5 = iter_error(zip(Iter(1), 'AB', strict=True), ValueError)

assert l5 == [(0, 'A')]
l6 = iter_error(zip(Iter(2), 'A', strict=True), ValueError)

assert l6 == [(1, 'A')]
l7 = iter_error(zip(Iter(2), 'ABC', strict=True), ValueError)

assert l7 == [(1, 'A'), (0, 'B')]
l8 = iter_error(zip(Iter(3), 'AB', strict=True), ValueError)

assert l8 == [(2, 'A'), (1, 'B')]
print("BuiltinTest::test_zip_strict_error_handling_stopiteration: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_zip_strict_error_handling_stopiteration: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_zip_strict_iterators.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_builtin_test__test_zip_strict_iterators() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "builtin_test__test_zip_strict_iterators"
# subject = "cpython.test_builtin.BuiltinTest.test_zip_strict_iterators"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::BuiltinTest::test_zip_strict_iterators
"""Auto-ported test: BuiltinTest::test_zip_strict_iterators (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
linux_alpha = platform.system().startswith('Linux') and platform.machine().startswith('alpha')
system_round_bug = round(5000000000000000.0 + 1) != 5000000000000000.0 + 1
x = iter(range(5))
y = [0]
z = iter(range(5))

try:
    list(zip(x, y, z, strict=True))
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert next(x) == 2

assert next(z) == 1
print("BuiltinTest::test_zip_strict_iterators: ok")
"###);
    assert_output(&out, r###"BuiltinTest::test_zip_strict_iterators: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/chr_lone_surrogate_roundtrip.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_chr_lone_surrogate_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "chr_lone_surrogate_roundtrip"
# subject = "builtins.chr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Lone surrogate strings created by chr() round-trip like CPython."""

s = chr(0xD800)

assert len(s) == 1
assert ord(s) == 0xD800
assert repr(s) == "'\\ud800'"
assert ascii(s) == "'\\ud800'"
assert s == chr(0xD800)
assert hash(s) == hash(chr(0xD800))

d = {s: "surrogate"}
assert d[chr(0xD800)] == "surrogate"
assert list(d.keys())[0] == s

print("chr_lone_surrogate_roundtrip OK")
"###);
    assert_output(&out, r###"chr_lone_surrogate_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/memoryview_cast_format_shape_byte_backed.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_memoryview_cast_format_shape_byte_backed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "memoryview_cast_format_shape_byte_backed"
# subject = "builtins.memoryview.cast format and shape metadata"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""memoryview.cast preserves format, itemsize, shape, strides, and logical reads."""


raw = memoryview(b"\x01\x00\x02\x00")
shorts = raw.cast("H")
assert shorts.format == "H"
assert shorts.itemsize == 2
assert shorts.nbytes == 4
assert shorts.ndim == 1
assert shorts.shape == (2,)
assert shorts.strides == (2,)
assert shorts[0] == 1
assert shorts[1] == 2
assert shorts.tolist() == [1, 2]

back = shorts.cast("B")
assert back.format == "B"
assert back.itemsize == 1
assert back.shape == (4,)
assert back.tobytes() == b"\x01\x00\x02\x00"

grid = memoryview(bytearray(6)).cast("B", shape=[2, 3])
assert len(grid) == 2
assert grid.ndim == 2
assert grid.shape == (2, 3)
assert grid.strides == (3, 1)
assert grid.tolist() == [[0, 0, 0], [0, 0, 0]]
try:
    grid[0]
    assert False
except NotImplementedError:
    pass

nested = memoryview(shorts)
assert nested.format == "H"
assert nested.itemsize == 2
assert nested.shape == (2,)
assert nested[0] == 1
assert nested.tolist() == [1, 2]

print("memoryview_cast_format_shape_byte_backed: ok")
"###);
    assert_output(&out, r###"memoryview_cast_format_shape_byte_backed: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/builtins/test_sorted__test_basic.py`.
#[test]
fn test_gen_behavior_builtin_libs_builtins_test_sorted__test_basic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "test_sorted__test_basic"
# subject = "cpython.test_builtin.TestSorted.test_basic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_builtin.py::TestSorted::test_basic
"""Auto-ported test: TestSorted::test_basic (CPython 3.12 oracle)."""


import ast
import asyncio
import builtins
import collections
import decimal
import fractions
import gc
import io
import locale
import math
import os
import pickle
import platform
import random
import re
import sys
import traceback
import types
import typing
import unittest
import warnings
from contextlib import ExitStack
from functools import partial
from inspect import CO_COROUTINE
from itertools import product
from textwrap import dedent
from types import AsyncGeneratorType, FunctionType, CellType
from operator import neg
from test import support
from test.support import cpython_only, swap_attr, maybe_get_event_loop_policy
from test.support.os_helper import EnvironmentVarGuard, TESTFN, unlink
from test.support.script_helper import assert_python_ok
from test.support.warnings_helper import check_warnings
from test.support import requires_IEEE_754
from unittest.mock import MagicMock, patch


try:
    import pty, signal
except ImportError:
    pty = signal = None

x, y = (1e+16, 2.9999)

HAVE_DOUBLE_ROUNDING = x + y == 1e+16 + 4

class Squares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(n * n)
            n += 1
        return self.sofar[i]

class StrSquares:

    def __init__(self, max):
        self.max = max
        self.sofar = []

    def __len__(self):
        return len(self.sofar)

    def __getitem__(self, i):
        if not 0 <= i < self.max:
            raise IndexError
        n = len(self.sofar)
        while n <= i:
            self.sofar.append(str(n * n))
            n += 1
        return self.sofar[i]

class BitBucket:

    def write(self, line):
        pass

test_conv_no_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

test_conv_sign = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', ValueError), ('314 ', 314), ('  \t\t  314  \t\t  ', ValueError), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', ValueError), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), (str(b'\\u0663\\u0661\\u0664 ', 'raw-unicode-escape'), 314), (chr(512), ValueError)]

def filter_char(arg):
    return ord(arg) > ord('d')

def map_char(arg):
    return chr(ord(arg) + 1)

def load_tests(loader, tests, pattern):
    from doctest import DocTestSuite
    tests.addTest(DocTestSuite(builtins))
    return tests


# --- test body ---
data = list(range(100))
copy = data[:]
random.shuffle(copy)

assert data == sorted(copy)

assert data != copy
data.reverse()
random.shuffle(copy)

assert data == sorted(copy, key=lambda x: -x)

assert data != copy
random.shuffle(copy)

assert data == sorted(copy, reverse=True)

assert data != copy
print("TestSorted::test_basic: ok")
"###);
    assert_output(&out, r###"TestSorted::test_basic: ok
"###);
}
