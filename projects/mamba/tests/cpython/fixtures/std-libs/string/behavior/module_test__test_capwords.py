# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_capwords"
# subject = "cpython.test_string.ModuleTest.test_capwords"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_capwords
"""Auto-ported test: ModuleTest::test_capwords (CPython 3.12 oracle)."""


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

assert string.capwords('abc def ghi') == 'Abc Def Ghi'

assert string.capwords('abc\tdef\nghi') == 'Abc Def Ghi'

assert string.capwords('abc\t   def  \nghi') == 'Abc Def Ghi'

assert string.capwords('ABC DEF GHI') == 'Abc Def Ghi'

assert string.capwords('ABC-DEF-GHI', '-') == 'Abc-Def-Ghi'

assert string.capwords('ABC-def DEF-ghi GHI') == 'Abc-def Def-ghi Ghi'

assert string.capwords('   aBc  DeF   ') == 'Abc Def'

assert string.capwords('\taBc\tDeF\t') == 'Abc Def'

assert string.capwords('\taBc\tDeF\t', '\t') == '\tAbc\tDef\t'
print("ModuleTest::test_capwords: ok")
