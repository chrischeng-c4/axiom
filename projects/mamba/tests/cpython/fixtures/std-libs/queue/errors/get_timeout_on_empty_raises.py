# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "errors"
# case = "get_timeout_on_empty_raises"
# subject = "queue.Queue.get"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue.get: get_timeout_on_empty_raises (errors)."""
import queue

_raised = False
try:
    queue.Queue().get(timeout=0.001)
except queue.Empty:
    _raised = True
assert _raised, "get_timeout_on_empty_raises: expected queue.Empty"
print("get_timeout_on_empty_raises OK")
