# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_make_dataclass__test_invalid_type_specification"
# subject = "cpython.__init__.TestMakeDataclass.test_invalid_type_specification"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestMakeDataclass::test_invalid_type_specification
"""Auto-ported test: TestMakeDataclass::test_invalid_type_specification (CPython 3.12 oracle)."""


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
for bad_field in [(), (1, 2, 3, 4)]:
    try:
        make_dataclass('C', ['a', bad_field])
        raise AssertionError('expected TypeError')
    except TypeError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('Invalid field: ', str(_aR_e))
for bad_field in [float, lambda x: x]:
    try:
        make_dataclass('C', ['a', bad_field])
        raise AssertionError('expected TypeError')
    except TypeError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('has no len\\(\\)', str(_aR_e))
print("TestMakeDataclass::test_invalid_type_specification: ok")
