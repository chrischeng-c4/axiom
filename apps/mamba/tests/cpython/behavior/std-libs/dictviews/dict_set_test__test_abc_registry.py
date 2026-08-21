# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_abc_registry"
# subject = "cpython.test_dictviews.DictSetTest.test_abc_registry"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_abc_registry
"""Auto-ported test: DictSetTest::test_abc_registry (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
d = dict(a=1)

assert isinstance(d.keys(), collections.abc.KeysView)

assert isinstance(d.keys(), collections.abc.MappingView)

assert isinstance(d.keys(), collections.abc.Set)

assert isinstance(d.keys(), collections.abc.Sized)

assert isinstance(d.keys(), collections.abc.Iterable)

assert isinstance(d.keys(), collections.abc.Container)

assert isinstance(d.values(), collections.abc.ValuesView)

assert isinstance(d.values(), collections.abc.MappingView)

assert isinstance(d.values(), collections.abc.Sized)

assert isinstance(d.values(), collections.abc.Collection)

assert isinstance(d.values(), collections.abc.Iterable)

assert isinstance(d.values(), collections.abc.Container)

assert isinstance(d.items(), collections.abc.ItemsView)

assert isinstance(d.items(), collections.abc.MappingView)

assert isinstance(d.items(), collections.abc.Set)

assert isinstance(d.items(), collections.abc.Sized)

assert isinstance(d.items(), collections.abc.Iterable)

assert isinstance(d.items(), collections.abc.Container)
print("DictSetTest::test_abc_registry: ok")
