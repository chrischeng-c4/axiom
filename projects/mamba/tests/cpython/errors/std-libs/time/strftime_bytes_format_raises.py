# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "errors"
# case = "strftime_bytes_format_raises"
# subject = "time.strftime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.strftime: strftime_bytes_format_raises (errors)."""
import time

_raised = False
try:
    time.strftime(b'%S', time.gmtime(0))
except TypeError:
    _raised = True
assert _raised, "strftime_bytes_format_raises: expected TypeError"
print("strftime_bytes_format_raises OK")
