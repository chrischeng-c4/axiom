# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "release_unheld_rlock_raises"
# subject = "threading.RLock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.RLock: release_unheld_rlock_raises (errors)."""
import threading

_raised = False
try:
    threading.RLock().release()
except RuntimeError:
    _raised = True
assert _raised, "release_unheld_rlock_raises: expected RuntimeError"
print("release_unheld_rlock_raises OK")
