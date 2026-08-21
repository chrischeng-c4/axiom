# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_all_soft_keywords_can_be_used_as_names"
# subject = "cpython.test_keyword.Test_iskeyword.test_all_soft_keywords_can_be_used_as_names"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_all_soft_keywords_can_be_used_as_names
"""Auto-ported test: Test_iskeyword::test_all_soft_keywords_can_be_used_as_names (CPython 3.12 oracle)."""


import keyword
import unittest


# --- test body ---
for key in keyword.softkwlist:
    exec(f'{key} = 42')
print("Test_iskeyword::test_all_soft_keywords_can_be_used_as_names: ok")
