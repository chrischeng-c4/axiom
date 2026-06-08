# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "errors"
# case = "get_negative_timeout_raises"
# subject = "queue.Queue.get"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue.get: get_negative_timeout_raises (errors)."""
import queue

_raised = False
try:
    queue.Queue().get(timeout=-1)
except ValueError:
    _raised = True
assert _raised, "get_negative_timeout_raises: expected ValueError"
print("get_negative_timeout_raises OK")
