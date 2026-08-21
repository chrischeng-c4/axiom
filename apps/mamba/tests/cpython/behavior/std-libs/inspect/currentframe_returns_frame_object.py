# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "currentframe_returns_frame_object"
# subject = "inspect.currentframe"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.currentframe: currentframe() returns a frame object exposing f_code, f_locals, and f_lineno"""
import inspect

_frame = inspect.currentframe()
assert _frame is not None, "currentframe not None"
assert hasattr(_frame, "f_code"), "frame has f_code"
assert hasattr(_frame, "f_locals"), "frame has f_locals"
assert hasattr(_frame, "f_lineno"), "frame has f_lineno"

print("currentframe_returns_frame_object OK")
