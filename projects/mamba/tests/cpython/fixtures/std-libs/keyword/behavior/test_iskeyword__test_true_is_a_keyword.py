# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_true_is_a_keyword"
# subject = "cpython.test_keyword.Test_iskeyword.test_true_is_a_keyword"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_true_is_a_keyword
"""Auto-ported test: Test_iskeyword::test_true_is_a_keyword (CPython 3.12 oracle)."""


import keyword
import unittest


# --- test body ---

assert keyword.iskeyword('True')
print("Test_iskeyword::test_true_is_a_keyword: ok")
