# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cast"
# dimension = "behavior"
# case = "test__test_p2a_objects_uc1d3dda"
# subject = "cpython.test_cast.Test.test_p2a_objects"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_cast.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import sys
array = (c_char_p * 5)()
assert array._objects == None
array[0] = b'foo bar'
assert array._objects == {'0': b'foo bar'}
p = cast(array, POINTER(c_char_p))
assert p._objects is array._objects
assert array._objects == {'0': b'foo bar', id(array): array}
p[0] = b'spam spam'
assert p._objects == {'0': b'spam spam', id(array): array}
assert array._objects is p._objects
p[1] = b'foo bar'
assert p._objects == {'1': b'foo bar', '0': b'spam spam', id(array): array}
assert array._objects is p._objects

print("Test::test_p2a_objects: ok")
