# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "test_iskeyword__test_changing_the_softkwlist_does_not_affect_issoftkeyword"
# subject = "cpython.test_keyword.Test_iskeyword.test_changing_the_softkwlist_does_not_affect_issoftkeyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_keyword.py::Test_iskeyword::test_changing_the_softkwlist_does_not_affect_issoftkeyword
"""Auto-ported test: Test_iskeyword::test_changing_the_softkwlist_does_not_affect_issoftkeyword (CPython 3.12 oracle)."""


import keyword
import unittest


# --- test body ---
oldlist = keyword.softkwlist
pass
keyword.softkwlist = ['foo', 'bar', 'spam', 'egs', 'case']

assert not keyword.issoftkeyword('spam')
print("Test_iskeyword::test_changing_the_softkwlist_does_not_affect_issoftkeyword: ok")
