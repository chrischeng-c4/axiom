# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "userstring"
# dimension = "behavior"
# case = "user_string_test__test_encode_explicit_none_args"
# subject = "cpython.test_userstring.UserStringTest.test_encode_explicit_none_args"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_userstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: UserStringTest::test_encode_explicit_none_args (CPython 3.12 oracle)."""

from collections import UserString


assert UserString("hello").encode(None, None) == b"hello"
assert UserString("\U00023456").encode(None, None) == b"\xf0\xa3\x91\x96"

try:
    UserString("\ud800").encode(None, None)
except UnicodeError as exc:
    assert str(exc) != ""
else:
    raise AssertionError("UserString.encode(None, None) strict errors did not raise")

print("UserStringTest::test_encode_explicit_none_args: ok")
