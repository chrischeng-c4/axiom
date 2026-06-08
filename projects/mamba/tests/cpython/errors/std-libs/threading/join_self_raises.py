# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "join_self_raises"
# subject = "threading.current_thread"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.current_thread: join_self_raises (errors)."""
import threading

_raised = False
try:
    threading.current_thread().join()
except RuntimeError:
    _raised = True
assert _raised, "join_self_raises: expected RuntimeError"
print("join_self_raises OK")
