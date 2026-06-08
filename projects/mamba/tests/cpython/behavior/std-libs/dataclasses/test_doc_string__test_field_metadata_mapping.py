# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_doc_string__test_field_metadata_mapping"
# subject = "cpython.__init__.TestDocString.test_field_metadata_mapping"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestDocString::test_field_metadata_mapping
"""Auto-ported test: TestDocString::test_field_metadata_mapping (CPython 3.12 oracle)."""


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
try:

    @dataclass
    class C:
        i: int = field(metadata=0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
d = {}

@dataclass
class C:
    i: int = field(metadata=d)

assert not fields(C)[0].metadata

assert len(fields(C)[0].metadata) == 0
d['foo'] = 1

assert len(fields(C)[0].metadata) == 1

assert fields(C)[0].metadata['foo'] == 1
try:
    fields(C)[0].metadata['test'] = 3
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('does not support item assignment', str(_aR_e))
d = {'test': 10, 'bar': '42', 3: 'three'}

@dataclass
class C:
    i: int = field(metadata=d)

assert len(fields(C)[0].metadata) == 3

assert fields(C)[0].metadata['test'] == 10

assert fields(C)[0].metadata['bar'] == '42'

assert fields(C)[0].metadata[3] == 'three'
d['foo'] = 1

assert len(fields(C)[0].metadata) == 4

assert fields(C)[0].metadata['foo'] == 1
try:
    fields(C)[0].metadata['baz']
    raise AssertionError('expected KeyError')
except KeyError:
    pass
try:
    fields(C)[0].metadata['test'] = 3
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('does not support item assignment', str(_aR_e))
print("TestDocString::test_field_metadata_mapping: ok")
