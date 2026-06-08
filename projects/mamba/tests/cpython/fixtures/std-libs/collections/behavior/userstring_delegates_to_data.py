# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "userstring_delegates_to_data"
# subject = "collections.UserString"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.UserString: UserString wraps a str in .data and forwards string methods, indexing/len, and concatenation while staying a UserString"""
from collections import UserString

us = UserString("hello")
assert us.data == "hello", "payload lives in .data"
assert us.upper() == "HELLO", "string method delegation"
assert us[0] == "h" and len(us) == 5, "indexing/len"
assert str(us + " world") == "hello world", "concatenation"
assert isinstance(us, UserString), "stays a UserString"

print("userstring_delegates_to_data OK")
