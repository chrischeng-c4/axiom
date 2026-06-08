# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "none_exception_renders_sentinel_across_apis"
# subject = "traceback.format_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception: a None exception renders 'NoneType: None\\n' across format_exc(None), format_exception(None)/(None,None,None), and format_exception_only(None)/(None,None)"""
import traceback

_NONE = "NoneType: None\n"
assert traceback.format_exc(None) == _NONE, "format_exc(None)"
assert traceback.format_exception(None) == [_NONE], "format_exception(None)"
assert traceback.format_exception(None, None, None) == [_NONE], "format_exception(None,None,None)"
assert traceback.format_exception_only(None) == [_NONE], "format_exception_only(None)"
assert traceback.format_exception_only(None, None) == [_NONE], "format_exception_only(None,None)"

print("none_exception_renders_sentinel_across_apis OK")
