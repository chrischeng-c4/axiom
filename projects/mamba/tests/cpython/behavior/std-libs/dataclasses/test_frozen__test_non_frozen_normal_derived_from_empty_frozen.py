# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_frozen__test_non_frozen_normal_derived_from_empty_frozen"
# subject = "cpython.__init__.TestFrozen.test_non_frozen_normal_derived_from_empty_frozen"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestFrozen::test_non_frozen_normal_derived_from_empty_frozen
"""Auto-ported test: TestFrozen::test_non_frozen_normal_derived_from_empty_frozen (CPython 3.12 oracle)."""


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
class D:
    pass

class S(D):
    pass
s = S()

assert not hasattr(s, 'x')
s.x = 5

assert s.x == 5
del s.x

assert not hasattr(s, 'x')
try:
    del s.x
    raise AssertionError('expected AttributeError')
except AttributeError as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert not isinstance(cm.exception, FrozenInstanceError)
print("TestFrozen::test_non_frozen_normal_derived_from_empty_frozen: ok")
