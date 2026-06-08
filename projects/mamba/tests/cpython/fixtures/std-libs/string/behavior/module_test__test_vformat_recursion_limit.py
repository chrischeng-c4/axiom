# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_vformat_recursion_limit"
# subject = "cpython.test_string.ModuleTest.test_vformat_recursion_limit"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_vformat_recursion_limit
"""Auto-ported test: ModuleTest::test_vformat_recursion_limit (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
fmt = string.Formatter()
args = ()
kwargs = dict(i=100)
try:
    fmt._vformat('{i}', args, kwargs, set(), -1)
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import types as _types_aR
    err = _types_aR.SimpleNamespace(exception=_aR_e)

assert 'recursion' in str(err.exception)
print("ModuleTest::test_vformat_recursion_limit: ok")
