# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "surface"
# case = "clear_is_callable"
# subject = "atexit._clear"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""atexit._clear: clear_is_callable (surface)."""
import atexit

assert callable(atexit._clear)
print("clear_is_callable OK")
