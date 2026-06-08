# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exception_three_arg_returns_str_list"
# subject = "traceback.format_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception: the 3-arg form format_exception(type, value, tb) of a live IndexError returns a list of str whose join contains 'IndexError: idx'"""
import sys
import traceback

try:
    raise IndexError("idx")
except IndexError:
    _exc_type, _exc_val, _exc_tb = sys.exc_info()
    _parts = traceback.format_exception(_exc_type, _exc_val, _exc_tb)
assert isinstance(_parts, list), f"format_exception type = {type(_parts)!r}"
_combined = "".join(_parts)
assert "IndexError: idx" in _combined, f"IndexError in format_exception: {_combined!r}"

print("format_exception_three_arg_returns_str_list OK")
