# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_set_name"
# subject = "cpython.test_subclassinit.Test.test_set_name"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_subclassinit.py::Test::test_set_name
"""Auto-ported test: Test::test_set_name (CPython 3.12 oracle)."""


import types
import unittest


# --- test body ---
class Descriptor:

    def __set_name__(self, owner, name):
        self.owner = owner
        self.name = name

class A:
    d = Descriptor()

assert A.d.name == 'd'

assert A.d.owner is A
print("Test::test_set_name: ok")
