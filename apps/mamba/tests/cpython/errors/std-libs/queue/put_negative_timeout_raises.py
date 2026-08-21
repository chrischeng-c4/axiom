# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "errors"
# case = "put_negative_timeout_raises"
# subject = "queue.Queue.put"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue.put: put_negative_timeout_raises (errors)."""
import queue

_raised = False
try:
    queue.Queue(maxsize=1).put(1, timeout=-1)
except ValueError:
    _raised = True
assert _raised, "put_negative_timeout_raises: expected ValueError"
print("put_negative_timeout_raises OK")
