# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_keyword_args__test_kw_only_twice"
# subject = "cpython.__init__.TestKeywordArgs.test_KW_ONLY_twice"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestKeywordArgs::test_KW_ONLY_twice
"""Auto-ported test: TestKeywordArgs::test_KW_ONLY_twice (CPython 3.12 oracle)."""


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
msg = "'Y' is KW_ONLY, but KW_ONLY has already been specified"
try:

    @dataclass
    class A:
        a: int
        X: KW_ONLY
        Y: KW_ONLY
        b: int
        c: int
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(msg, str(_aR_e))
try:

    @dataclass
    class A:
        a: int
        X: KW_ONLY
        b: int
        Y: KW_ONLY
        c: int
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(msg, str(_aR_e))
try:

    @dataclass
    class A:
        a: int
        X: KW_ONLY
        b: int
        c: int
        Y: KW_ONLY
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(msg, str(_aR_e))

@dataclass
class A:
    a: int
    _: KW_ONLY
    b: int
    c: int = field(kw_only=True)

@dataclass
class A:
    a: int
    _: KW_ONLY
    b: int
    c: int

@dataclass
class B(A):
    _: KW_ONLY
    d: int
try:

    @dataclass
    class A:
        a: int
        _: KW_ONLY
        b: int
        c: int

    @dataclass
    class B(A):
        X: KW_ONLY
        d: int
        Y: KW_ONLY
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(msg, str(_aR_e))
print("TestKeywordArgs::test_KW_ONLY_twice: ok")
