# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "errors"
# case = "queue_get_nowait_empty_raises"
# subject = "multiprocessing.Queue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/_test_multiprocessing.py"
# status = "filled"
# ///
"""multiprocessing.Queue: queue_get_nowait_empty_raises (errors)."""
import multiprocessing
import queue as _queue
_q = multiprocessing.Queue()

_raised = False
try:
    _q.get_nowait()
except _queue.Empty:
    _raised = True
assert _raised, "queue_get_nowait_empty_raises: expected _queue.Empty"
print("queue_get_nowait_empty_raises OK")
