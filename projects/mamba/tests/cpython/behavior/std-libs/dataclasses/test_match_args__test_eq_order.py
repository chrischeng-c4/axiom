# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "test_match_args__test_eq_order"
# subject = "cpython.__init__.TestMatchArgs.test_eq_order"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dataclasses/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestMatchArgs::test_eq_order
"""Auto-ported test: TestMatchArgs::test_eq_order (CPython 3.12 oracle)."""


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
for eq, order, result in [(False, False, 'neither'), (False, True, 'exception'), (True, False, 'eq_only'), (True, True, 'both')]:
    if result == 'exception':
        try:

            @dataclass(eq=eq, order=order)
            class C:
                pass
            raise AssertionError('expected ValueError')
        except ValueError as _aR_e:
            import re as _re_aR
            assert _re_aR.search('eq must be true if order is true', str(_aR_e))
    else:

        @dataclass(eq=eq, order=order)
        class C:
            pass
        if result == 'neither':

            assert '__eq__' not in C.__dict__

            assert '__lt__' not in C.__dict__

            assert '__le__' not in C.__dict__

            assert '__gt__' not in C.__dict__

            assert '__ge__' not in C.__dict__
        elif result == 'both':

            assert '__eq__' in C.__dict__

            assert '__lt__' in C.__dict__

            assert '__le__' in C.__dict__

            assert '__gt__' in C.__dict__

            assert '__ge__' in C.__dict__
        elif result == 'eq_only':

            assert '__eq__' in C.__dict__

            assert '__lt__' not in C.__dict__

            assert '__le__' not in C.__dict__

            assert '__gt__' not in C.__dict__

            assert '__ge__' not in C.__dict__
        else:
            assert False, f'unknown result {result!r}'
print("TestMatchArgs::test_eq_order: ok")
