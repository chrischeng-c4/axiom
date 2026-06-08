# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_consistency_with_epg"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_consistency_with_epg"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_consistency_with_epg
"""Auto-ported test: ClassPropertiesAndMethods::test_consistency_with_epg (CPython 3.12 oracle)."""


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
class Pane(object):
    pass

class ScrollingMixin(object):
    pass

class EditingMixin(object):
    pass

class ScrollablePane(Pane, ScrollingMixin):
    pass

class EditablePane(Pane, EditingMixin):
    pass

class EditableScrollablePane(ScrollablePane, EditablePane):
    pass

assert EditableScrollablePane.__mro__ == (EditableScrollablePane, ScrollablePane, EditablePane, Pane, ScrollingMixin, EditingMixin, object)
print("ClassPropertiesAndMethods::test_consistency_with_epg: ok")
