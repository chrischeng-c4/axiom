# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "behavior"
# case = "currentframe_yields_frame"
# subject = "inspect.currentframe"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
"""inspect.currentframe: inspect.currentframe() returns the caller's frame object (not None) with a positive f_lineno"""
import inspect

frame = inspect.currentframe()
assert frame is not None
assert frame.f_lineno > 0, frame.f_lineno

print("currentframe_yields_frame OK")
