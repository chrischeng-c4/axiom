# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_slots__test_dataclass_slot_dict_ctype"
# subject = "cpython.__init__.TestSlots.test_dataclass_slot_dict_ctype"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestSlots::test_dataclass_slot_dict_ctype
"""Auto-ported test: TestSlots::test_dataclass_slot_dict_ctype (CPython 3.12 oracle)."""


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
from test.support import import_helper
_testcapi = import_helper.import_module('_testcapi')

@dataclass(slots=True)
class HasDictOffset(_testcapi.HeapCTypeWithDict):
    __dict__: dict = {}

assert _testcapi.HeapCTypeWithDict.__dictoffset__ != 0

assert HasDictOffset.__slots__ == ()

@dataclass(slots=True)
class DoesNotHaveDictOffset(_testcapi.HeapCTypeWithWeakref):
    __dict__: dict = {}

assert _testcapi.HeapCTypeWithWeakref.__dictoffset__ == 0

assert DoesNotHaveDictOffset.__slots__ == ('__dict__',)
print("TestSlots::test_dataclass_slot_dict_ctype: ok")
