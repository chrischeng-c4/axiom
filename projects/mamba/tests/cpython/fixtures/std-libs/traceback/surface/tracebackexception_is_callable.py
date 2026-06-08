# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "tracebackexception_is_callable"
# subject = "traceback.TracebackException"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.TracebackException: tracebackexception_is_callable (surface)."""
import traceback

assert callable(traceback.TracebackException)
print("tracebackexception_is_callable OK")
