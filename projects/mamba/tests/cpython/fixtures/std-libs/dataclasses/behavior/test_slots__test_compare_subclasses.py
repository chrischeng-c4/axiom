# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_slots__test_compare_subclasses"
# subject = "cpython.__init__.TestSlots.test_compare_subclasses"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestSlots::test_compare_subclasses
"""Auto-ported test: TestSlots::test_compare_subclasses (CPython 3.12 oracle)."""


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
@dataclass
class B:
    i: int

@dataclass
class C(B):
    pass
for idx, (fn, expected) in enumerate([(lambda a, b: a == b, False), (lambda a, b: a != b, True)]):

    assert fn(B(0), C(0)) == expected
for idx, fn in enumerate([lambda a, b: a < b, lambda a, b: a <= b, lambda a, b: a > b, lambda a, b: a >= b]):
    try:
        fn(B(0), C(0))
        raise AssertionError('expected TypeError')
    except TypeError as _aR_e:
        import re as _re_aR
        assert _re_aR.search("not supported between instances of 'B' and 'C'", str(_aR_e))
print("TestSlots::test_compare_subclasses: ok")
