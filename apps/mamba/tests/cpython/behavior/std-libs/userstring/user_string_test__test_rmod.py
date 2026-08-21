# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "userstring"
# dimension = "behavior"
# case = "user_string_test__test_rmod"
# subject = "cpython.test_userstring.UserStringTest.test_rmod"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_userstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_userstring.py::UserStringTest::test_rmod
"""Auto-ported test: UserStringTest::test_rmod (CPython 3.12 oracle)."""


import unittest
from test import string_tests
from collections import UserString


# --- test body ---
type2test = UserString

class ustr2(UserString):
    pass

class ustr3(ustr2):

    def __rmod__(self, other):
        return super().__rmod__(other)
fmt2 = ustr2('value is %s')
str3 = ustr3('TEST')

assert fmt2 % str3 == 'value is TEST'
print("UserStringTest::test_rmod: ok")
