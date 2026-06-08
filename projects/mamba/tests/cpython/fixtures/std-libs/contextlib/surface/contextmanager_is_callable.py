# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "surface"
# case = "contextmanager_is_callable"
# subject = "contextlib.contextmanager"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextlib.contextmanager: contextmanager_is_callable (surface)."""
import contextlib

assert callable(contextlib.contextmanager)
print("contextmanager_is_callable OK")
