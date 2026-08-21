# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "surface"
# case = "exit_stack_is_callable"
# subject = "contextlib.ExitStack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextlib.ExitStack: exit_stack_is_callable (surface)."""
import contextlib

assert callable(contextlib.ExitStack)
print("exit_stack_is_callable OK")
