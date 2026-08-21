# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keeprefs"
# dimension = "behavior"
# case = "simple_test_case__test_ccharp_uc427ba2"
# subject = "cpython.test_keeprefs.SimpleTestCase.test_ccharp"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_keeprefs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
x = c_char_p()
assert x._objects == None
x.value = b'abc'
assert x._objects == b'abc'
x = c_char_p(b'spam')
assert x._objects == b'spam'

print("SimpleTestCase::test_ccharp: ok")
