# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "condition_wait_without_lock_raises"
# subject = "threading.Condition"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Condition: condition_wait_without_lock_raises (errors)."""
import threading

_raised = False
try:
    threading.Condition().wait(timeout=0.001)
except RuntimeError:
    _raised = True
assert _raised, "condition_wait_without_lock_raises: expected RuntimeError"
print("condition_wait_without_lock_raises OK")
