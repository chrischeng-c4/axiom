# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "format_exception_only_renders_type_and_message"
# subject = "traceback.format_exception_only"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception_only: format_exception_only(ValueError, ValueError('bad value')) returns a single line 'ValueError: bad value\\n'"""
import traceback

_lines = traceback.format_exception_only(ValueError, ValueError("bad value"))
assert len(_lines) == 1, f"exception_only lines = {len(_lines)!r}"
assert _lines[0] == "ValueError: bad value\n", f"format = {_lines[0]!r}"

print("format_exception_only_renders_type_and_message OK")
