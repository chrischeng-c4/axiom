# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_frozen__test_no_unhashable_default"
# subject = "cpython.__init__.TestFrozen.test_no_unhashable_default"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestFrozen::test_no_unhashable_default
"""Auto-ported test: TestFrozen::test_no_unhashable_default (CPython 3.12 oracle)."""


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
class Unhashable:
    __hash__ = None
unhashable_re = 'mutable default .* for field a is not allowed'
try:

    @dataclass
    class A:
        a: dict = {}
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(unhashable_re, str(_aR_e))
try:

    @dataclass
    class A:
        a: Any = Unhashable()
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(unhashable_re, str(_aR_e))
try:
    unhashable = Unhashable()
    unhashable.__hash__ = lambda: 0

    @dataclass
    class A:
        a: Any = unhashable
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(unhashable_re, str(_aR_e))
print("TestFrozen::test_no_unhashable_default: ok")
