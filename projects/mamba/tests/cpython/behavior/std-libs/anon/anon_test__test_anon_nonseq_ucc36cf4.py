# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "anon"
# dimension = "behavior"
# case = "anon_test__test_anon_nonseq_ucc36cf4"
# subject = "cpython.test_anon.AnonTest.test_anon_nonseq"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_anon.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
try:
    (lambda: type(Structure)('Name', (Structure,), {'_fields_': [], '_anonymous_': 42}))()
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("AnonTest::test_anon_nonseq: ok")
