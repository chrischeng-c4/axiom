# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_ordering__test_overwriting_order"
# subject = "cpython.__init__.TestOrdering.test_overwriting_order"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestOrdering::test_overwriting_order
"""Auto-ported test: TestOrdering::test_overwriting_order (CPython 3.12 oracle)."""


from dataclasses import *
import abc
import io
import pickle
import inspect
import builtins
import types
import weakref
import traceback
import unittest
from unittest.mock import Mock
from typing import ClassVar, Any, List, Union, Tuple, Dict, Generic, TypeVar, Optional, Protocol, DefaultDict
from typing import get_type_hints
from collections import deque, OrderedDict, namedtuple, defaultdict
from copy import deepcopy
from functools import total_ordering
import typing
import dataclasses
from test import support


class CustomError(Exception):
    pass

ByMakeDataClass = make_dataclass('ByMakeDataClass', [('x', int)])

ManualModuleMakeDataClass = make_dataclass('ManualModuleMakeDataClass', [('x', int)], module=__name__)

WrongNameMakeDataclass = make_dataclass('Wrong', [('x', int)])

WrongModuleMakeDataclass = make_dataclass('WrongModuleMakeDataclass', [('x', int)], module='custom')


# --- test body ---
try:

    @dataclass(order=True)
    class C:
        x: int

        def __lt__(self):
            pass
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('Cannot overwrite attribute __lt__.*using functools.total_ordering', str(_aR_e))
try:

    @dataclass(order=True)
    class C:
        x: int

        def __le__(self):
            pass
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('Cannot overwrite attribute __le__.*using functools.total_ordering', str(_aR_e))
try:

    @dataclass(order=True)
    class C:
        x: int

        def __gt__(self):
            pass
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('Cannot overwrite attribute __gt__.*using functools.total_ordering', str(_aR_e))
try:

    @dataclass(order=True)
    class C:
        x: int

        def __ge__(self):
            pass
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('Cannot overwrite attribute __ge__.*using functools.total_ordering', str(_aR_e))
print("TestOrdering::test_overwriting_order: ok")
