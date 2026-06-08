# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exc_outside_except_is_none_sentinel"
# subject = "traceback.format_exc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exc: format_exc() with no active exception returns the sentinel 'NoneType: None\\n'"""
import traceback

_outside = traceback.format_exc()
assert _outside == "NoneType: None\n", f"outside = {_outside!r}"

print("format_exc_outside_except_is_none_sentinel OK")
