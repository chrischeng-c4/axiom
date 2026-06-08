# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "errors"
# case = "set_result_twice_raises"
# subject = "concurrent.futures.Future.set_result"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.set_result: set_result_twice_raises (errors)."""
from concurrent.futures import Future, InvalidStateError
_f = Future()
_f.set_result(1)

_raised = False
try:
    _f.set_result(2)
except InvalidStateError:
    _raised = True
assert _raised, "set_result_twice_raises: expected InvalidStateError"
print("set_result_twice_raises OK")
