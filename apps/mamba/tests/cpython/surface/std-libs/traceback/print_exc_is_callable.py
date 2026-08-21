# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "print_exc_is_callable"
# subject = "traceback.print_exc"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.print_exc: print_exc_is_callable (surface)."""
import traceback

assert callable(traceback.print_exc)
print("print_exc_is_callable OK")
