# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_make_dataclass__test_cannot_be_pickled"
# subject = "cpython.__init__.TestMakeDataclass.test_cannot_be_pickled"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestMakeDataclass::test_cannot_be_pickled
"""Auto-ported test: TestMakeDataclass::test_cannot_be_pickled (CPython 3.12 oracle)."""


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
for klass in [WrongNameMakeDataclass, WrongModuleMakeDataclass]:
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        try:
            pickle.dumps(klass, proto)
            raise AssertionError('expected pickle.PickleError')
        except pickle.PickleError:
            pass
        try:
            pickle.dumps(klass(1), proto)
            raise AssertionError('expected pickle.PickleError')
        except pickle.PickleError:
            pass
print("TestMakeDataclass::test_cannot_be_pickled: ok")
