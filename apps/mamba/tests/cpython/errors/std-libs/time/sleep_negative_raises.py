# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "errors"
# case = "sleep_negative_raises"
# subject = "time.sleep"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.sleep: sleep_negative_raises (errors)."""
import time

_raised = False
try:
    time.sleep(-1)
except ValueError:
    _raised = True
assert _raised, "sleep_negative_raises: expected ValueError"
print("sleep_negative_raises OK")
