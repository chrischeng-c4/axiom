# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "walk_stack_is_callable"
# subject = "traceback.walk_stack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.walk_stack: walk_stack_is_callable (surface)."""
import traceback

assert callable(traceback.walk_stack)
print("walk_stack_is_callable OK")
