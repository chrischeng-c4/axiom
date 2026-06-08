# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_list_empty_returns_empty_list"
# subject = "traceback.format_list"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_list: format_list([]) over an empty stack returns an empty list (no lines)"""
import traceback

_res = traceback.format_list([])
assert _res == [], f"empty format_list = {_res!r}"

print("format_list_empty_returns_empty_list OK")
