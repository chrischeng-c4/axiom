# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_keyword_args__test_is_dataclass_when_getattr_always_returns"
# subject = "cpython.__init__.TestKeywordArgs.test_is_dataclass_when_getattr_always_returns"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestKeywordArgs::test_is_dataclass_when_getattr_always_returns
"""Auto-ported test: TestKeywordArgs::test_is_dataclass_when_getattr_always_returns (CPython 3.12 oracle)."""


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
class A:

    def __getattr__(self, key):
        return 0

assert not is_dataclass(A)
a = A()

class B:
    pass
b = B()
b.__dataclass_fields__ = []
for obj in (a, b):

    assert not is_dataclass(obj)
    try:
        asdict(obj)
        raise AssertionError('expected TypeError')
    except TypeError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('should be called on dataclass instances', str(_aR_e))
    try:
        astuple(obj)
        raise AssertionError('expected TypeError')
    except TypeError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('should be called on dataclass instances', str(_aR_e))
    try:
        replace(obj, x=0)
        raise AssertionError('expected TypeError')
    except TypeError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('should be called on dataclass instances', str(_aR_e))
print("TestKeywordArgs::test_is_dataclass_when_getattr_always_returns: ok")
