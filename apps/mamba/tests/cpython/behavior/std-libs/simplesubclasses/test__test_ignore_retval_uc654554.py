# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "simplesubclasses"
# dimension = "behavior"
# case = "test__test_ignore_retval_uc654554"
# subject = "cpython.test_simplesubclasses.Test.test_ignore_retval"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_simplesubclasses.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
proto = CFUNCTYPE(None)

def func():
    return (1, 'abc', None)
cb = proto(func)
assert None == cb()

print("Test::test_ignore_retval: ok")
