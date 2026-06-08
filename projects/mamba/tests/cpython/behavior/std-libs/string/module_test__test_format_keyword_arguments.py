# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_format_keyword_arguments"
# subject = "cpython.test_string.ModuleTest.test_format_keyword_arguments"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_format_keyword_arguments
"""Auto-ported test: ModuleTest::test_format_keyword_arguments (CPython 3.12 oracle)."""


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

assert fmt.format('-{arg}-', arg='test') == '-test-'

try:
    fmt.format('-{arg}-')
    raise AssertionError('expected KeyError')
except KeyError:
    pass

assert fmt.format('-{self}-', self='test') == '-test-'

try:
    fmt.format('-{self}-')
    raise AssertionError('expected KeyError')
except KeyError:
    pass

assert fmt.format('-{format_string}-', format_string='test') == '-test-'

try:
    fmt.format('-{format_string}-')
    raise AssertionError('expected KeyError')
except KeyError:
    pass
try:
    fmt.format(format_string='-{arg}-', arg='test')
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('format_string', str(_aR_e))
print("ModuleTest::test_format_keyword_arguments: ok")
