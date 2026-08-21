# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_invalid_with_no_lines"
# subject = "cpython.test_string.TestTemplate.test_invalid_with_no_lines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_invalid_with_no_lines
"""Auto-ported test: TestTemplate::test_invalid_with_no_lines (CPython 3.12 oracle)."""


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
class MyTemplate(Template):
    pattern = '\n              (?P<invalid>) |\n              unreachable(\n                (?P<named>)   |\n                (?P<braced>)  |\n                (?P<escaped>)\n              )\n            '
s = MyTemplate('')
try:
    s.substitute({})
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import types as _types_aR
    err = _types_aR.SimpleNamespace(exception=_aR_e)

assert 'line 1, col 1' in str(err.exception)
print("TestTemplate::test_invalid_with_no_lines: ok")
