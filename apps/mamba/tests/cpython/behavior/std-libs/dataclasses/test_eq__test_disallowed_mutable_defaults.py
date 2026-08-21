# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_eq__test_disallowed_mutable_defaults"
# subject = "cpython.__init__.TestEq.test_disallowed_mutable_defaults"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestEq::test_disallowed_mutable_defaults
"""Auto-ported test: TestEq::test_disallowed_mutable_defaults (CPython 3.12 oracle)."""


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
for typ, empty, non_empty in [(list, [], [1]), (dict, {}, {0: 1}), (set, set(), set([1]))]:
    try:

        @dataclass
        class Point:
            x: typ = empty
        raise AssertionError('expected ValueError')
    except ValueError as _aR_e:
        import re as _re_aR
        assert _re_aR.search(f'mutable default {typ} for field x is not allowed', str(_aR_e))
    try:

        @dataclass
        class Point:
            y: typ = non_empty
        raise AssertionError('expected ValueError')
    except ValueError as _aR_e:
        import re as _re_aR
        assert _re_aR.search(f'mutable default {typ} for field y is not allowed', str(_aR_e))

    class Subclass(typ):
        pass
    try:

        @dataclass
        class Point:
            z: typ = Subclass()
        raise AssertionError('expected ValueError')
    except ValueError as _aR_e:
        import re as _re_aR
        assert _re_aR.search("mutable default .*Subclass'> for field z is not allowed", str(_aR_e))

    @dataclass
    class C:
        z: ClassVar[typ] = typ()

    @dataclass
    class C:
        x: ClassVar[typ] = Subclass()
print("TestEq::test_disallowed_mutable_defaults: ok")
