# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_keywords"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_keywords"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_keywords
"""Auto-ported test: ClassPropertiesAndMethods::test_keywords (CPython 3.12 oracle)."""


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
    int(x=1)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('keyword argument', str(_aR_e))
try:
    float(x=2)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('keyword argument', str(_aR_e))
try:
    bool(x=2)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('keyword argument', str(_aR_e))

assert complex(imag=42, real=666) == complex(666, 42)

assert str(object=500) == '500'

assert str(object=b'abc', errors='strict') == 'abc'
try:
    tuple(sequence=range(3))
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('keyword argument', str(_aR_e))
try:
    list(sequence=(0, 1, 2))
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('keyword argument', str(_aR_e))
for constructor in (int, float, int, complex, str, str, tuple, list):
    try:
        constructor(bogus_keyword_arg=1)
    except TypeError:
        pass
    else:

        raise AssertionError('expected TypeError from bogus keyword argument to %r' % constructor)
print("ClassPropertiesAndMethods::test_keywords: ok")
