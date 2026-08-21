# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "append_overflow_overflowerror"
# subject = "array.array.append"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array.append: append_overflow_overflowerror (errors)."""
import array

_raised = False
try:
    array.array("b").append(1000)
except OverflowError:
    _raised = True
assert _raised, "append_overflow_overflowerror: expected OverflowError"
print("append_overflow_overflowerror OK")
