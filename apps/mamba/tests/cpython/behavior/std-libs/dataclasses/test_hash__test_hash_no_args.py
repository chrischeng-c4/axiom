# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_hash__test_hash_no_args"
# subject = "cpython.__init__.TestHash.test_hash_no_args"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestHash::test_hash_no_args
"""Auto-ported test: TestHash::test_hash_no_args (CPython 3.12 oracle)."""


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
class Base:

    def __hash__(self):
        return 301
for frozen, eq, base, expected in [(None, None, object, 'unhashable'), (None, None, Base, 'unhashable'), (None, False, object, 'object'), (None, False, Base, 'base'), (None, True, object, 'unhashable'), (None, True, Base, 'unhashable'), (False, None, object, 'unhashable'), (False, None, Base, 'unhashable'), (False, False, object, 'object'), (False, False, Base, 'base'), (False, True, object, 'unhashable'), (False, True, Base, 'unhashable'), (True, None, object, 'tuple'), (True, None, Base, 'tuple'), (True, False, object, 'object'), (True, False, Base, 'base'), (True, True, object, 'tuple'), (True, True, Base, 'tuple')]:
    if frozen is None and eq is None:

        @dataclass
        class C(base):
            i: int
    elif frozen is None:

        @dataclass(eq=eq)
        class C(base):
            i: int
    elif eq is None:

        @dataclass(frozen=frozen)
        class C(base):
            i: int
    else:

        @dataclass(frozen=frozen, eq=eq)
        class C(base):
            i: int
    if expected == 'unhashable':
        c = C(10)
        try:
            hash(c)
            raise AssertionError('expected TypeError')
        except TypeError as _aR_e:
            import re as _re_aR
            assert _re_aR.search('unhashable type', str(_aR_e))
    elif expected == 'base':

        assert hash(C(10)) == 301
    elif expected == 'object':

        assert C.__hash__ is object.__hash__
    elif expected == 'tuple':

        assert hash(C(42)) == hash((42,))
    else:
        assert False, f'unknown value for expected={expected!r}'
print("TestHash::test_hash_no_args: ok")
