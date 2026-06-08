# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_frozen__test_overwrite_hash"
# subject = "cpython.__init__.TestFrozen.test_overwrite_hash"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestFrozen::test_overwrite_hash
"""Auto-ported test: TestFrozen::test_overwrite_hash (CPython 3.12 oracle)."""


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
@dataclass(frozen=True)
class C:
    x: int

    def __hash__(self):
        return 301

assert hash(C(100)) == 301

@dataclass(frozen=True)
class C:
    x: int

    def __eq__(self, other):
        return False

assert hash(C(100)) == hash((100,))
try:

    @dataclass(unsafe_hash=True)
    class C:

        def __hash__(self):
            pass
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('Cannot overwrite attribute __hash__', str(_aR_e))

@dataclass(unsafe_hash=True)
class C:
    x: int

    def __eq__(self):
        pass

assert hash(C(10)) == hash((10,))
try:

    @dataclass(unsafe_hash=True)
    class C:
        x: int

        def __eq__(self):
            pass

        def __hash__(self):
            pass
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('Cannot overwrite attribute __hash__', str(_aR_e))
print("TestFrozen::test_overwrite_hash: ok")
