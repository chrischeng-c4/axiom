# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_string_annotations__test_two_fields_one_default"
# subject = "cpython.__init__.TestStringAnnotations.test_two_fields_one_default"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestStringAnnotations::test_two_fields_one_default
"""Auto-ported test: TestStringAnnotations::test_two_fields_one_default (CPython 3.12 oracle)."""


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
class C:
    x: int
    y: int = 0
o = C(3)

assert (o.x, o.y) == (3, 0)
try:

    @dataclass
    class C:
        x: int = 0
        y: int
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search("non-default argument 'y' follows default argument", str(_aR_e))
try:

    @dataclass
    class B:
        x: int = 0

    @dataclass
    class C(B):
        y: int
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search("non-default argument 'y' follows default argument", str(_aR_e))
try:

    @dataclass
    class B:
        x: int
        y: int

    @dataclass
    class C(B):
        x: int = 0
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search("non-default argument 'y' follows default argument", str(_aR_e))
print("TestStringAnnotations::test_two_fields_one_default: ok")
