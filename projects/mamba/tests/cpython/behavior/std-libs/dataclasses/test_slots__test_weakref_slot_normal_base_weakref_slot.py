# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_slots__test_weakref_slot_normal_base_weakref_slot"
# subject = "cpython.__init__.TestSlots.test_weakref_slot_normal_base_weakref_slot"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestSlots::test_weakref_slot_normal_base_weakref_slot
"""Auto-ported test: TestSlots::test_weakref_slot_normal_base_weakref_slot (CPython 3.12 oracle)."""


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
    __slots__ = ('__weakref__',)

@dataclass(slots=True, weakref_slot=True)
class A(Base):
    field: int

assert '__weakref__' in Base.__slots__

assert '__weakref__' not in A.__slots__
a = A(1)
a_ref = weakref.ref(a)

assert a.__weakref__ is a_ref
print("TestSlots::test_weakref_slot_normal_base_weakref_slot: ok")
