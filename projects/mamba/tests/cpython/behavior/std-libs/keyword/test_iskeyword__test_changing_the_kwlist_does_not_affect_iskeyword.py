# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_changing_the_kwlist_does_not_affect_iskeyword"
# subject = "cpython.test_keyword.Test_iskeyword.test_changing_the_kwlist_does_not_affect_iskeyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_changing_the_kwlist_does_not_affect_iskeyword
"""Auto-ported test: Test_iskeyword::test_changing_the_kwlist_does_not_affect_iskeyword (CPython 3.12 oracle)."""


import keyword
import unittest


# --- test body ---
oldlist = keyword.kwlist
pass
keyword.kwlist = ['its', 'all', 'eggs', 'beans', 'and', 'a', 'slice']

assert not keyword.iskeyword('eggs')
print("Test_iskeyword::test_changing_the_kwlist_does_not_affect_iskeyword: ok")
