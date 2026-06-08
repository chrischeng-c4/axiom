# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "str_startswith_endswith"
# subject = "str.startswith"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.startswith: startswith/endswith report prefix/suffix membership as bools: 'hello'.startswith('hel') is True, 'hello'.endswith('xyz') is False"""
import builtins  # noqa: F401

assert "hello".startswith("hel") == True, "startswith match"
assert "hello".startswith("xyz") == False, "startswith no match"
assert "hello".endswith("llo") == True, "endswith match"
assert "hello".endswith("xyz") == False, "endswith no match"
print("str_startswith_endswith OK")
