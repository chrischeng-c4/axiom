# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "class_creation_tests__test_new_class_with_mro_entry_genericalias"
# subject = "cpython.test_types.ClassCreationTests.test_new_class_with_mro_entry_genericalias"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_types.py::ClassCreationTests::test_new_class_with_mro_entry_genericalias
"""Auto-ported test: ClassCreationTests::test_new_class_with_mro_entry_genericalias (CPython 3.12 oracle)."""


from test.support import run_with_locale, cpython_only, iter_builtin_types, iter_slot_wrappers, MISSING_C_DOCSTRINGS
from test.test_import import no_rerun
import collections.abc
from collections import namedtuple
import copy
import gc
import inspect
import pickle
import locale
import sys
import textwrap
import types
import unittest.mock
import weakref
import typing


T = typing.TypeVar('T')

class Example:
    pass

class Forward:
    ...

def clear_typing_caches():
    for f in typing._cleanups:
        f()


# --- test body ---
L1 = types.new_class('L1', (typing.List[int],), {})

assert L1.__bases__ == (list, typing.Generic)

assert L1.__orig_bases__ == (typing.List[int],)

assert L1.__mro__ == (L1, list, typing.Generic, object)
L2 = types.new_class('L2', (list[int],), {})

assert L2.__bases__ == (list,)

assert L2.__orig_bases__ == (list[int],)

assert L2.__mro__ == (L2, list, object)
print("ClassCreationTests::test_new_class_with_mro_entry_genericalias: ok")
