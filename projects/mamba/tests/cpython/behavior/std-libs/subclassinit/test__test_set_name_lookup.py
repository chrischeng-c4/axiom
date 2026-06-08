# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_set_name_lookup"
# subject = "cpython.test_subclassinit.Test.test_set_name_lookup"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_subclassinit.py::Test::test_set_name_lookup
"""Auto-ported test: Test::test_set_name_lookup (CPython 3.12 oracle)."""


import types
import unittest


# --- test body ---
resolved = []

class NonDescriptor:

    def __getattr__(self, name):
        resolved.append(name)

class A:
    d = NonDescriptor()

assert '__set_name__' not in resolved
print("Test::test_set_name_lookup: ok")
