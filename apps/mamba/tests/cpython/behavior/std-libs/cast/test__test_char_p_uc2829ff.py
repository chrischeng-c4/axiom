# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cast"
# dimension = "behavior"
# case = "test__test_char_p_uc2829ff"
# subject = "cpython.test_cast.Test.test_char_p"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_cast.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import sys
s = c_char_p(b'hiho')
assert cast(cast(s, c_void_p), c_char_p).value == b'hiho'

print("Test::test_char_p: ok")
