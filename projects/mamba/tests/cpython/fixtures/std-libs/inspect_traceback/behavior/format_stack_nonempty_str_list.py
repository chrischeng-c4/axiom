# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "behavior"
# case = "format_stack_nonempty_str_list"
# subject = "traceback.format_stack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
"""traceback.format_stack: traceback.format_stack() at top level returns a non-empty list whose every entry is a str line of formatted stack text"""
import traceback

stack = traceback.format_stack()
assert isinstance(stack, list)
assert len(stack) > 0
assert all(isinstance(line, str) for line in stack)

print("format_stack_nonempty_str_list OK")
