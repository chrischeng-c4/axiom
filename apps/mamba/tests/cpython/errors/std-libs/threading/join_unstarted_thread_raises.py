# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "join_unstarted_thread_raises"
# subject = "threading.Thread"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: join_unstarted_thread_raises (errors)."""
import threading

_raised = False
try:
    threading.Thread().join()
except RuntimeError:
    _raised = True
assert _raised, "join_unstarted_thread_raises: expected RuntimeError"
print("join_unstarted_thread_raises OK")
