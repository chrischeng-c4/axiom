# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "errors"
# case = "cancelled_error_is_exception"
# subject = "concurrent.futures.CancelledError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.CancelledError: concurrent.futures.CancelledError is a subclass of BaseException (and of Exception in 3.12)"""
import concurrent.futures

assert issubclass(concurrent.futures.CancelledError, BaseException), "CancelledError is a BaseException"
assert issubclass(concurrent.futures.CancelledError, Exception), "CancelledError is an Exception in 3.12"
# It is also raise/catchable as a normal exception.
raised = False
try:
    raise concurrent.futures.CancelledError("cancelled")
except Exception as e:
    raised = isinstance(e, concurrent.futures.CancelledError)
assert raised, "CancelledError caught as Exception"

print("cancelled_error_is_exception OK")
