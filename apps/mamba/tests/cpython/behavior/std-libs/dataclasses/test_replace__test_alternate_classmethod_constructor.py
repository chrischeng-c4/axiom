# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_replace__test_alternate_classmethod_constructor"
# subject = "cpython.__init__.TestReplace.test_alternate_classmethod_constructor"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestReplace::test_alternate_classmethod_constructor
"""Auto-ported test: TestReplace::test_alternate_classmethod_constructor (CPython 3.12 oracle)."""


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
class C:
    x: int

    @classmethod
    def from_file(cls, filename):
        value_in_file = 20
        return cls(value_in_file)

assert C.from_file('filename').x == 20
print("TestReplace::test_alternate_classmethod_constructor: ok")
