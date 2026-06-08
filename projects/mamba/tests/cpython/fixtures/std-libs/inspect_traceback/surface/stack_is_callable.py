# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "surface"
# case = "stack_is_callable"
# subject = "inspect.stack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.stack: stack_is_callable (surface)."""
import inspect

assert callable(inspect.stack)
print("stack_is_callable OK")
