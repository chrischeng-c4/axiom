# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "cast_returns_value_unchanged"
# subject = "typing.cast"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.cast: cast is a runtime no-op: cast(int, 'not an int') returns the original string object unchanged, doing no conversion or validation"""
import typing

result = typing.cast(int, "not an int")
assert result == "not an int", "cast must return the value unchanged"
assert type(result) is str, "cast does no conversion: the object is still a str"
print("cast_returns_value_unchanged OK")
