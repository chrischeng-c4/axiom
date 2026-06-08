# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "exception_none_outside_live_inside"
# subject = "sys.exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.exception: sys.exception() is None outside a handler and is the caught instance inside an except ValueError block (3.11+)"""
import sys


def _raise():
    raise ValueError(42)


assert sys.exception() is None, f"exception() outside = {sys.exception()!r}"
try:
    _raise()
except ValueError as _e2:
    assert sys.exception() is _e2, "exception() returns the caught instance"
print("exception_none_outside_live_inside OK")
