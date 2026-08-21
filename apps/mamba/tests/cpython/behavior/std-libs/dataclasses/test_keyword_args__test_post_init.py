# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_keyword_args__test_post_init"
# subject = "cpython.__init__.TestKeywordArgs.test_post_init"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestKeywordArgs::test_post_init
"""Auto-ported test: TestKeywordArgs::test_post_init (CPython 3.12 oracle)."""


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
class A:
    a: int
    _: KW_ONLY
    b: InitVar[int]
    c: int
    d: InitVar[int]

    def __post_init__(self, b, d):
        raise CustomError(f'b={b!r} d={d!r}')
try:
    A(1, c=2, b=3, d=4)
    raise AssertionError('expected CustomError')
except CustomError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('b=3 d=4', str(_aR_e))

@dataclass
class B:
    a: int
    _: KW_ONLY
    b: InitVar[int]
    c: int
    d: InitVar[int]

    def __post_init__(self, b, d):
        self.a = b
        self.c = d
b = B(1, c=2, b=3, d=4)

assert asdict(b) == {'a': 3, 'c': 4}
print("TestKeywordArgs::test_post_init: ok")
