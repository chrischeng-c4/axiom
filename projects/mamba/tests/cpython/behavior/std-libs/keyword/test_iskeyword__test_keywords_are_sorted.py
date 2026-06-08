# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_keywords_are_sorted"
# subject = "cpython.test_keyword.Test_iskeyword.test_keywords_are_sorted"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_keywords_are_sorted
"""Auto-ported test: Test_iskeyword::test_keywords_are_sorted (CPython 3.12 oracle)."""


import keyword


assert sorted(keyword.kwlist) == keyword.kwlist

print("Test_iskeyword::test_keywords_are_sorted: ok")
