# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_string_annotations__test_initvar"
# subject = "cpython.__init__.TestStringAnnotations.test_initvar"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestStringAnnotations::test_initvar
"""Auto-ported test: TestStringAnnotations::test_initvar (CPython 3.12 oracle)."""


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
for typestr in ('InitVar[int]', 'InitVar [int] InitVar [int]', 'InitVar', ' InitVar ', 'dataclasses.InitVar[int]', 'dataclasses.InitVar[str]', ' dataclasses.InitVar[str]', 'dataclasses .InitVar[str]', 'dataclasses. InitVar[str]', 'dataclasses.InitVar [str]', 'dataclasses.InitVar [ str]', 'dataclasses.InitVar.[int]', 'dataclasses.InitVar+'):

    @dataclass
    class C:
        x: typestr
    try:
        C(1).x
        raise AssertionError('expected AttributeError')
    except AttributeError as _aR_e:
        import re as _re_aR
        assert _re_aR.search("object has no attribute 'x'", str(_aR_e))
print("TestStringAnnotations::test_initvar: ok")
