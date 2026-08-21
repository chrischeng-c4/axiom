# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "errors"
# case = "put_nowait_on_full_raises"
# subject = "queue.Queue.put_nowait"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue.put_nowait: put_nowait_on_full_raises (errors)."""
import queue

_raised = False
try:
    (lambda q: (q.put_nowait('a'), q.put_nowait('b')))(queue.Queue(maxsize=1))
except queue.Full:
    _raised = True
assert _raised, "put_nowait_on_full_raises: expected queue.Full"
print("put_nowait_on_full_raises OK")
