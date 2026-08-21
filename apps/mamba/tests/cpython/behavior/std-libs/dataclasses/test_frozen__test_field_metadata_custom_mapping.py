# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_frozen__test_field_metadata_custom_mapping"
# subject = "cpython.__init__.TestFrozen.test_field_metadata_custom_mapping"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestFrozen::test_field_metadata_custom_mapping
"""Auto-ported test: TestFrozen::test_field_metadata_custom_mapping (CPython 3.12 oracle)."""


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
class SimpleNameSpace:

    def __init__(self, **kw):
        self.__dict__.update(kw)

    def __getitem__(self, item):
        if item == 'xyzzy':
            return 'plugh'
        return getattr(self, item)

    def __len__(self):
        return self.__dict__.__len__()

@dataclass
class C:
    i: int = field(metadata=SimpleNameSpace(a=10))

assert len(fields(C)[0].metadata) == 1

assert fields(C)[0].metadata['a'] == 10
try:
    fields(C)[0].metadata['b']
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass

assert fields(C)[0].metadata['xyzzy'] == 'plugh'
print("TestFrozen::test_field_metadata_custom_mapping: ok")
