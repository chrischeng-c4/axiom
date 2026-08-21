# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "behavior"
# case = "stack_top_is_module"
# subject = "inspect.stack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
"""inspect.stack: inspect.stack() returns a non-empty list of FrameInfo records; the top frame's .function is the running scope name ('<module>' at top level)"""
import inspect

st = inspect.stack()
assert isinstance(st, list)
assert len(st) > 0
top = st[0]
assert isinstance(top.function, str)
assert top.function == "<module>", top.function

print("stack_top_is_module OK")
