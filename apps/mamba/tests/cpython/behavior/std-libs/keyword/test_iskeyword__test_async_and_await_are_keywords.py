# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_async_and_await_are_keywords"
# subject = "cpython.test_keyword.Test_iskeyword.test_async_and_await_are_keywords"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_async_and_await_are_keywords
"""Auto-ported test: Test_iskeyword::test_async_and_await_are_keywords (CPython 3.12 oracle)."""


import keyword
import unittest


# --- test body ---

assert 'async' in keyword.kwlist

assert 'await' in keyword.kwlist
print("Test_iskeyword::test_async_and_await_are_keywords: ok")
