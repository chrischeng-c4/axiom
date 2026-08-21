# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "errors"
# case = "queue_put_nowait_full_raises"
# subject = "multiprocessing.Queue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/_test_multiprocessing.py"
# status = "filled"
# ///
"""multiprocessing.Queue: queue_put_nowait_full_raises (errors)."""
import multiprocessing
import queue as _queue
_q = multiprocessing.Queue(maxsize=1)
_q.put_nowait('a')

_raised = False
try:
    _q.put_nowait('b')
except _queue.Full:
    _raised = True
assert _raised, "queue_put_nowait_full_raises: expected _queue.Full"
print("queue_put_nowait_full_raises OK")
