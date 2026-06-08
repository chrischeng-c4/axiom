# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_string_annotations__test_classvar_module_level_import"
# subject = "cpython.__init__.TestStringAnnotations.test_classvar_module_level_import"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestStringAnnotations::test_classvar_module_level_import
"""Auto-ported test: TestStringAnnotations::test_classvar_module_level_import (CPython 3.12 oracle)."""


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
from test.test_dataclasses import dataclass_module_1
from test.test_dataclasses import dataclass_module_1_str
from test.test_dataclasses import dataclass_module_2
from test.test_dataclasses import dataclass_module_2_str
for m in (dataclass_module_1, dataclass_module_1_str, dataclass_module_2, dataclass_module_2_str):
    if m.USING_STRINGS:
        c = m.CV(10)
    else:
        c = m.CV()

    assert c.cv0 == 20
    c = m.IV(0, 1, 2, 3, 4)
    for field_name in ('iv0', 'iv1', 'iv2', 'iv3'):
        try:
            getattr(c, field_name)
            raise AssertionError('expected AttributeError')
        except AttributeError as _aR_e:
            import re as _re_aR
            assert _re_aR.search(f"object has no attribute '{field_name}'", str(_aR_e))
    if m.USING_STRINGS:

        assert 'not_iv4' in c.__dict__

        assert c.not_iv4 == 4
    else:

        assert 'not_iv4' not in c.__dict__
print("TestStringAnnotations::test_classvar_module_level_import: ok")
