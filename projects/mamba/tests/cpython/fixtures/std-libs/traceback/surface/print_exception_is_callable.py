# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "print_exception_is_callable"
# subject = "traceback.print_exception"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.print_exception: print_exception_is_callable (surface)."""
import traceback

assert callable(traceback.print_exception)
print("print_exception_is_callable OK")
