# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "release_unlocked_lock_raises"
# subject = "threading.Lock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Lock: release_unlocked_lock_raises (errors)."""
import threading

_raised = False
try:
    threading.Lock().release()
except RuntimeError:
    _raised = True
assert _raised, "release_unlocked_lock_raises: expected RuntimeError"
print("release_unlocked_lock_raises OK")
