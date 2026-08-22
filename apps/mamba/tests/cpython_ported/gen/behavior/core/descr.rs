use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/descr/class_properties_and_methods__test_attr_raise_through_property.py`.
#[test]
fn test_gen_behavior_core_descr_class_properties_and_methods__test_attr_raise_through_property() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_attr_raise_through_property"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_attr_raise_through_property"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_attr_raise_through_property
"""Auto-ported test: ClassPropertiesAndMethods::test_attr_raise_through_property (CPython 3.12 oracle)."""


import builtins
import copyreg
import gc
import itertools
import math
import pickle
import random
import string
import sys
import types
import unittest
import warnings
import weakref
from copy import deepcopy
from contextlib import redirect_stdout
from test import support
from test.support.testcase import ExtraAssertions


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import xxsubtype
except ImportError:
    xxsubtype = None

class DebugHelperMeta(type):
    """
    Sets default __doc__ and simplifies repr() output.
    """

    def __new__(mcls, name, bases, attrs):
        if attrs.get('__doc__') is None:
            attrs['__doc__'] = name
        return type.__new__(mcls, name, bases, attrs)

    def __repr__(cls):
        return repr(cls.__name__)


# --- test body ---
class A:

    def __getattr__(self, name):
        raise ValueError('FOO')

    @property
    def foo(self):
        return self.__getattr__('asdf')
try:
    A().foo
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('FOO', str(_aR_e))

class B:

    @property
    def __getattr__(self, name):
        raise ValueError('FOO')

    @property
    def foo(self):
        raise NotImplementedError('BAR')
try:
    B().foo
    raise AssertionError('expected NotImplementedError')
except NotImplementedError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('BAR', str(_aR_e))
print("ClassPropertiesAndMethods::test_attr_raise_through_property: ok")
"###);
    assert_output(&out, r###"ClassPropertiesAndMethods::test_attr_raise_through_property: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/descr/class_properties_and_methods__test_deepcopy_recursive.py`.
#[test]
fn test_gen_behavior_core_descr_class_properties_and_methods__test_deepcopy_recursive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_deepcopy_recursive"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_deepcopy_recursive"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_deepcopy_recursive
"""Auto-ported test: ClassPropertiesAndMethods::test_deepcopy_recursive (CPython 3.12 oracle)."""


import builtins
import copyreg
import gc
import itertools
import math
import pickle
import random
import string
import sys
import types
import unittest
import warnings
import weakref
from copy import deepcopy
from contextlib import redirect_stdout
from test import support
from test.support.testcase import ExtraAssertions


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import xxsubtype
except ImportError:
    xxsubtype = None

class DebugHelperMeta(type):
    """
    Sets default __doc__ and simplifies repr() output.
    """

    def __new__(mcls, name, bases, attrs):
        if attrs.get('__doc__') is None:
            attrs['__doc__'] = name
        return type.__new__(mcls, name, bases, attrs)

    def __repr__(cls):
        return repr(cls.__name__)


# --- test body ---
class Node:
    pass
a = Node()
b = Node()
a.b = b
b.a = a
z = deepcopy(a)
print("ClassPropertiesAndMethods::test_deepcopy_recursive: ok")
"###);
    assert_output(&out, r###"ClassPropertiesAndMethods::test_deepcopy_recursive: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/descr/class_properties_and_methods__test_evil_type_name.py`.
#[test]
fn test_gen_behavior_core_descr_class_properties_and_methods__test_evil_type_name() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_evil_type_name"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_evil_type_name"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_evil_type_name
"""Auto-ported test: ClassPropertiesAndMethods::test_evil_type_name (CPython 3.12 oracle)."""


import builtins
import copyreg
import gc
import itertools
import math
import pickle
import random
import string
import sys
import types
import unittest
import warnings
import weakref
from copy import deepcopy
from contextlib import redirect_stdout
from test import support
from test.support.testcase import ExtraAssertions


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import xxsubtype
except ImportError:
    xxsubtype = None

class DebugHelperMeta(type):
    """
    Sets default __doc__ and simplifies repr() output.
    """

    def __new__(mcls, name, bases, attrs):
        if attrs.get('__doc__') is None:
            attrs['__doc__'] = name
        return type.__new__(mcls, name, bases, attrs)

    def __repr__(cls):
        return repr(cls.__name__)


# --- test body ---
class Nasty(str):

    def __del__(self):
        C.__name__ = 'other'

class C:
    pass
C.__name__ = Nasty('abc')
C.__name__ = 'normal'
print("ClassPropertiesAndMethods::test_evil_type_name: ok")
"###);
    assert_output(&out, r###"ClassPropertiesAndMethods::test_evil_type_name: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/descr/class_properties_and_methods__test_ipow_returns_not_implemented.py`.
#[test]
fn test_gen_behavior_core_descr_class_properties_and_methods__test_ipow_returns_not_implemented() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_ipow_returns_not_implemented"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_ipow_returns_not_implemented"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_ipow_returns_not_implemented
"""Auto-ported test: ClassPropertiesAndMethods::test_ipow_returns_not_implemented (CPython 3.12 oracle)."""


import builtins
import copyreg
import gc
import itertools
import math
import pickle
import random
import string
import sys
import types
import unittest
import warnings
import weakref
from copy import deepcopy
from contextlib import redirect_stdout
from test import support
from test.support.testcase import ExtraAssertions


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import xxsubtype
except ImportError:
    xxsubtype = None

class DebugHelperMeta(type):
    """
    Sets default __doc__ and simplifies repr() output.
    """

    def __new__(mcls, name, bases, attrs):
        if attrs.get('__doc__') is None:
            attrs['__doc__'] = name
        return type.__new__(mcls, name, bases, attrs)

    def __repr__(cls):
        return repr(cls.__name__)


# --- test body ---
class A:

    def __ipow__(self, other):
        return NotImplemented

class B(A):

    def __rpow__(self, other):
        return 1

class C(A):

    def __pow__(self, other):
        return 2
a = A()
b = B()
c = C()
a **= b

assert a == 1
c **= b

assert c == 2
print("ClassPropertiesAndMethods::test_ipow_returns_not_implemented: ok")
"###);
    assert_output(&out, r###"ClassPropertiesAndMethods::test_ipow_returns_not_implemented: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/descr/class_properties_and_methods__test_no_ipow.py`.
#[test]
fn test_gen_behavior_core_descr_class_properties_and_methods__test_no_ipow() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_no_ipow"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_no_ipow"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_no_ipow
"""Auto-ported test: ClassPropertiesAndMethods::test_no_ipow (CPython 3.12 oracle)."""


import builtins
import copyreg
import gc
import itertools
import math
import pickle
import random
import string
import sys
import types
import unittest
import warnings
import weakref
from copy import deepcopy
from contextlib import redirect_stdout
from test import support
from test.support.testcase import ExtraAssertions


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import xxsubtype
except ImportError:
    xxsubtype = None

class DebugHelperMeta(type):
    """
    Sets default __doc__ and simplifies repr() output.
    """

    def __new__(mcls, name, bases, attrs):
        if attrs.get('__doc__') is None:
            attrs['__doc__'] = name
        return type.__new__(mcls, name, bases, attrs)

    def __repr__(cls):
        return repr(cls.__name__)


# --- test body ---
class B:

    def __rpow__(self, other):
        return 1
a = object()
b = B()
a **= b

assert a == 1
print("ClassPropertiesAndMethods::test_no_ipow: ok")
"###);
    assert_output(&out, r###"ClassPropertiesAndMethods::test_no_ipow: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/descr/class_properties_and_methods__test_repr_with_module_str_subclass.py`.
#[test]
fn test_gen_behavior_core_descr_class_properties_and_methods__test_repr_with_module_str_subclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_repr_with_module_str_subclass"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_repr_with_module_str_subclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_repr_with_module_str_subclass
"""Auto-ported test: ClassPropertiesAndMethods::test_repr_with_module_str_subclass (CPython 3.12 oracle)."""


import builtins
import copyreg
import gc
import itertools
import math
import pickle
import random
import string
import sys
import types
import unittest
import warnings
import weakref
from copy import deepcopy
from contextlib import redirect_stdout
from test import support
from test.support.testcase import ExtraAssertions


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import xxsubtype
except ImportError:
    xxsubtype = None

class DebugHelperMeta(type):
    """
    Sets default __doc__ and simplifies repr() output.
    """

    def __new__(mcls, name, bases, attrs):
        if attrs.get('__doc__') is None:
            attrs['__doc__'] = name
        return type.__new__(mcls, name, bases, attrs)

    def __repr__(cls):
        return repr(cls.__name__)


# --- test body ---
class StrSub(str):
    pass

class Some:
    pass
Some.__module__ = StrSub('example')

assert isinstance(repr(Some), str)

assert isinstance(repr(Some()), str)
print("ClassPropertiesAndMethods::test_repr_with_module_str_subclass: ok")
"###);
    assert_output(&out, r###"ClassPropertiesAndMethods::test_repr_with_module_str_subclass: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/descr/class_properties_and_methods__test_rmul.py`.
#[test]
fn test_gen_behavior_core_descr_class_properties_and_methods__test_rmul() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_rmul"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_rmul"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_rmul
"""Auto-ported test: ClassPropertiesAndMethods::test_rmul (CPython 3.12 oracle)."""


import builtins
import copyreg
import gc
import itertools
import math
import pickle
import random
import string
import sys
import types
import unittest
import warnings
import weakref
from copy import deepcopy
from contextlib import redirect_stdout
from test import support
from test.support.testcase import ExtraAssertions


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import xxsubtype
except ImportError:
    xxsubtype = None

class DebugHelperMeta(type):
    """
    Sets default __doc__ and simplifies repr() output.
    """

    def __new__(mcls, name, bases, attrs):
        if attrs.get('__doc__') is None:
            attrs['__doc__'] = name
        return type.__new__(mcls, name, bases, attrs)

    def __repr__(cls):
        return repr(cls.__name__)


# --- test body ---
class C(object):

    def __mul__(self, other):
        return 'mul'

    def __rmul__(self, other):
        return 'rmul'
a = C()

assert a * 2 == 'mul'

assert a * 2.2 == 'mul'

assert 2 * a == 'rmul'

assert 2.2 * a == 'rmul'
print("ClassPropertiesAndMethods::test_rmul: ok")
"###);
    assert_output(&out, r###"ClassPropertiesAndMethods::test_rmul: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/descr/class_properties_and_methods__test_testcapi_no_segfault.py`.
#[test]
fn test_gen_behavior_core_descr_class_properties_and_methods__test_testcapi_no_segfault() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_testcapi_no_segfault"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_testcapi_no_segfault"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_testcapi_no_segfault
"""Auto-ported test: ClassPropertiesAndMethods::test_testcapi_no_segfault (CPython 3.12 oracle)."""


import builtins
import copyreg
import gc
import itertools
import math
import pickle
import random
import string
import sys
import types
import unittest
import warnings
import weakref
from copy import deepcopy
from contextlib import redirect_stdout
from test import support
from test.support.testcase import ExtraAssertions


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import xxsubtype
except ImportError:
    xxsubtype = None

class DebugHelperMeta(type):
    """
    Sets default __doc__ and simplifies repr() output.
    """

    def __new__(mcls, name, bases, attrs):
        if attrs.get('__doc__') is None:
            attrs['__doc__'] = name
        return type.__new__(mcls, name, bases, attrs)

    def __repr__(cls):
        return repr(cls.__name__)


# --- test body ---
try:
    import _testcapi
except ImportError:
    pass
else:

    class X(object):
        p = property(_testcapi.test_with_docstring)
print("ClassPropertiesAndMethods::test_testcapi_no_segfault: ok")
"###);
    assert_output(&out, r###"ClassPropertiesAndMethods::test_testcapi_no_segfault: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/descr/class_properties_and_methods__test_weakref_segfault.py`.
#[test]
fn test_gen_behavior_core_descr_class_properties_and_methods__test_weakref_segfault() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_weakref_segfault"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_weakref_segfault"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_weakref_segfault
"""Auto-ported test: ClassPropertiesAndMethods::test_weakref_segfault (CPython 3.12 oracle)."""


import builtins
import copyreg
import gc
import itertools
import math
import pickle
import random
import string
import sys
import types
import unittest
import warnings
import weakref
from copy import deepcopy
from contextlib import redirect_stdout
from test import support
from test.support.testcase import ExtraAssertions


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import xxsubtype
except ImportError:
    xxsubtype = None

class DebugHelperMeta(type):
    """
    Sets default __doc__ and simplifies repr() output.
    """

    def __new__(mcls, name, bases, attrs):
        if attrs.get('__doc__') is None:
            attrs['__doc__'] = name
        return type.__new__(mcls, name, bases, attrs)

    def __repr__(cls):
        return repr(cls.__name__)


# --- test body ---
import weakref

class Provoker:

    def __init__(self, referrent):
        self.ref = weakref.ref(referrent)

    def __del__(self):
        x = self.ref()

class Oops(object):
    pass
o = Oops()
o.whatever = Provoker(o)
del o
print("ClassPropertiesAndMethods::test_weakref_segfault: ok")
"###);
    assert_output(&out, r###"ClassPropertiesAndMethods::test_weakref_segfault: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/descr/class_properties_and_methods__test_wrapper_segfault.py`.
#[test]
fn test_gen_behavior_core_descr_class_properties_and_methods__test_wrapper_segfault() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_wrapper_segfault"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_wrapper_segfault"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_wrapper_segfault
"""Auto-ported test: ClassPropertiesAndMethods::test_wrapper_segfault (CPython 3.12 oracle)."""


import builtins
import copyreg
import gc
import itertools
import math
import pickle
import random
import string
import sys
import types
import unittest
import warnings
import weakref
from copy import deepcopy
from contextlib import redirect_stdout
from test import support
from test.support.testcase import ExtraAssertions


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import xxsubtype
except ImportError:
    xxsubtype = None

class DebugHelperMeta(type):
    """
    Sets default __doc__ and simplifies repr() output.
    """

    def __new__(mcls, name, bases, attrs):
        if attrs.get('__doc__') is None:
            attrs['__doc__'] = name
        return type.__new__(mcls, name, bases, attrs)

    def __repr__(cls):
        return repr(cls.__name__)


# --- test body ---
f = lambda: None
for i in range(1000000):
    f = f.__call__
f = None
print("ClassPropertiesAndMethods::test_wrapper_segfault: ok")
"###);
    assert_output(&out, r###"ClassPropertiesAndMethods::test_wrapper_segfault: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/descr/dict_proxy_tests__test_dict_type_with_metaclass.py`.
#[test]
fn test_gen_behavior_core_descr_dict_proxy_tests__test_dict_type_with_metaclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "dict_proxy_tests__test_dict_type_with_metaclass"
# subject = "cpython.test_descr.DictProxyTests.test_dict_type_with_metaclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_descr.py::DictProxyTests::test_dict_type_with_metaclass
"""Auto-ported test: DictProxyTests::test_dict_type_with_metaclass (CPython 3.12 oracle)."""


import builtins
import copyreg
import gc
import itertools
import math
import pickle
import random
import string
import sys
import types
import unittest
import warnings
import weakref
from copy import deepcopy
from contextlib import redirect_stdout
from test import support
from test.support.testcase import ExtraAssertions


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import xxsubtype
except ImportError:
    xxsubtype = None

class DebugHelperMeta(type):
    """
    Sets default __doc__ and simplifies repr() output.
    """

    def __new__(mcls, name, bases, attrs):
        if attrs.get('__doc__') is None:
            attrs['__doc__'] = name
        return type.__new__(mcls, name, bases, attrs)

    def __repr__(cls):
        return repr(cls.__name__)


# --- test body ---
class C(object):

    def meth(self):
        pass
self_C = C

class B(object):
    pass

class M(type):
    pass

class C(metaclass=M):
    pass

assert type(C.__dict__) == type(B.__dict__)
print("DictProxyTests::test_dict_type_with_metaclass: ok")
"###);
    assert_output(&out, r###"DictProxyTests::test_dict_type_with_metaclass: ok
"###);
}
