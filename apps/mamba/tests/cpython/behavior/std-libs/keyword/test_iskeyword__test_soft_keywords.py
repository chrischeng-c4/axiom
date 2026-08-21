# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_soft_keywords"
# subject = "cpython.test_keyword.Test_iskeyword.test_soft_keywords"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_soft_keywords
"""Auto-ported test: Test_iskeyword::test_soft_keywords (CPython 3.12 oracle)."""


import keyword
import unittest


# --- test body ---

assert 'type' in keyword.softkwlist

assert 'match' in keyword.softkwlist

assert 'case' in keyword.softkwlist

assert '_' in keyword.softkwlist
print("Test_iskeyword::test_soft_keywords: ok")
