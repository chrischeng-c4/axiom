# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_cache"
# dimension = "behavior"
# case = "type_cache_tests__test_type_assign_version"
# subject = "cpython.test_type_cache.TypeCacheTests.test_type_assign_version"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_cache.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_cache.py::TypeCacheTests::test_type_assign_version
"""Auto-ported test: TypeCacheTests::test_type_assign_version (CPython 3.12 oracle)."""


import unittest
import dis
from test import support
from test.support import import_helper


' Tests for the internal type cache in CPython. '

try:
    from sys import _clear_type_cache
except ImportError:
    _clear_type_cache = None

_testcapi = import_helper.import_module('_testcapi')

type_get_version = _testcapi.type_get_version

type_assign_specific_version_unsafe = _testcapi.type_assign_specific_version_unsafe

type_assign_version = _testcapi.type_assign_version

type_modified = _testcapi.type_modified


# --- test body ---
class C:
    x = 5

assert type_assign_version(C) == 1
c_ver = type_get_version(C)
C.x = 6

assert type_get_version(C) == 0

assert type_assign_version(C) == 1

assert type_get_version(C) != 0

assert type_get_version(C) != c_ver
print("TypeCacheTests::test_type_assign_version: ok")
