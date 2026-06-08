# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "errors"
# case = "get_nowait_on_empty_raises"
# subject = "queue.Queue.get_nowait"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue.get_nowait: get_nowait_on_empty_raises (errors)."""
import queue

_raised = False
try:
    queue.Queue().get_nowait()
except queue.Empty:
    _raised = True
assert _raised, "get_nowait_on_empty_raises: expected queue.Empty"
print("get_nowait_on_empty_raises OK")
