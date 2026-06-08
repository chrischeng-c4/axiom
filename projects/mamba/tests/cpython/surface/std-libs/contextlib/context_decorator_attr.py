# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "surface"
# case = "context_decorator_attr"
# subject = "contextlib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextlib: context_decorator_attr (surface)."""
import contextlib

assert hasattr(contextlib, "ContextDecorator")
print("context_decorator_attr OK")
