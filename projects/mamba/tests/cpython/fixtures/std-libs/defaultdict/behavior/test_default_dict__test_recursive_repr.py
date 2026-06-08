# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "defaultdict"
# dimension = "behavior"
# case = "test_default_dict__test_recursive_repr"
# subject = "cpython.test_defaultdict.TestDefaultDict.test_recursive_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_defaultdict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_defaultdict.py::TestDefaultDict::test_recursive_repr
"""Auto-ported test: TestDefaultDict::test_recursive_repr (CPython 3.12 oracle)."""


import re
from collections import defaultdict


class sub(defaultdict):
    def __init__(self):
        self.default_factory = self._factory

    def _factory(self):
        return []


d = sub()
assert re.search(
    r"sub\(<bound method .*sub\._factory " r"of sub\(\.\.\., \{\}\)>, \{\}\)",
    repr(d),
)

print("TestDefaultDict::test_recursive_repr: ok")
