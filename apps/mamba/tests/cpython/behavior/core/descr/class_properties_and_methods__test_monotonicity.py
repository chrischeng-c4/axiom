# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_monotonicity"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_monotonicity"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_monotonicity
"""Auto-ported test: ClassPropertiesAndMethods::test_monotonicity (CPython 3.12 oracle)."""


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
class Boat(object):
    pass

class DayBoat(Boat):
    pass

class WheelBoat(Boat):
    pass

class EngineLess(DayBoat):
    pass

class SmallMultihull(DayBoat):
    pass

class PedalWheelBoat(EngineLess, WheelBoat):
    pass

class SmallCatamaran(SmallMultihull):
    pass

class Pedalo(PedalWheelBoat, SmallCatamaran):
    pass

assert PedalWheelBoat.__mro__ == (PedalWheelBoat, EngineLess, DayBoat, WheelBoat, Boat, object)

assert SmallCatamaran.__mro__ == (SmallCatamaran, SmallMultihull, DayBoat, Boat, object)

assert Pedalo.__mro__ == (Pedalo, PedalWheelBoat, EngineLess, SmallCatamaran, SmallMultihull, DayBoat, WheelBoat, Boat, object)
print("ClassPropertiesAndMethods::test_monotonicity: ok")
