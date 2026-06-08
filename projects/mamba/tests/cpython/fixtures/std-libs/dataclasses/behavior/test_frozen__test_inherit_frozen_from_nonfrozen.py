# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_frozen__test_inherit_frozen_from_nonfrozen"
# subject = "cpython.__init__.TestFrozen.test_inherit_frozen_from_nonfrozen"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestFrozen::test_inherit_frozen_from_nonfrozen
"""Auto-ported test: TestFrozen::test_inherit_frozen_from_nonfrozen (CPython 3.12 oracle)."""


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
for intermediate_class in [True, False]:

    @dataclass
    class C:
        i: int
    if intermediate_class:

        class I(C):
            pass
    else:
        I = C
    try:

        @dataclass(frozen=True)
        class D(I):
            pass
        raise AssertionError('expected TypeError')
    except TypeError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('cannot inherit frozen dataclass from a non-frozen one', str(_aR_e))
print("TestFrozen::test_inherit_frozen_from_nonfrozen: ok")
