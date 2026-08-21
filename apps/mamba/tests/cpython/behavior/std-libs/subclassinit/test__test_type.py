# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_type"
# subject = "cpython.test_subclassinit.Test.test_type"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_subclassinit.py::Test::test_type
"""Auto-ported test: Test::test_type (CPython 3.12 oracle)."""


import types
import unittest


# --- test body ---
t = type('NewClass', (object,), {})

assert isinstance(t, type)

assert t.__name__ == 'NewClass'
try:
    type(name='NewClass', bases=(object,), dict={})
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("Test::test_type: ok")
