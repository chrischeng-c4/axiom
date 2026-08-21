# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "typecodes_lists_core_codes"
# subject = "array.typecodes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.typecodes: array.typecodes is a str containing every core integer and float code in 'bBhHiIlLqQfd', and each advertised code constructs an empty array of that code"""
import array

assert isinstance(array.typecodes, str), f"typecodes type = {type(array.typecodes)!r}"
for code in "bBhHiIlLqQfd":
    assert code in array.typecodes, f"{code!r} in typecodes"
# Every advertised typecode constructs an empty array of that code.
for code in array.typecodes:
    ac = array.array(code)
    assert ac.typecode == code, f"constructed typecode = {ac.typecode!r}"

print("typecodes_lists_core_codes OK")
