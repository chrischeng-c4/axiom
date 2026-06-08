# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "return_type_is_int"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect.bisect_left: bisect_left returns a plain int insertion index"""
import bisect

result = bisect.bisect_left([1, 2, 3], 2)
assert isinstance(result, int), f"return type is int, got {type(result).__name__}"
assert result == 1, f"bisect_left = {result!r}"

print("return_type_is_int OK")
