# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "errors"
# case = "result_after_set_exception_raises"
# subject = "concurrent.futures.Future.result"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.result: result_after_set_exception_raises (errors)."""
from concurrent.futures import Future
_f = Future()
_f.set_exception(ValueError('inner'))

_raised = False
try:
    _f.result()
except ValueError:
    _raised = True
assert _raised, "result_after_set_exception_raises: expected ValueError"
print("result_after_set_exception_raises OK")
