# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "errors"
# case = "task_done_too_many_raises"
# subject = "queue.Queue.task_done"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_queue.py"
# status = "filled"
# ///
"""queue.Queue.task_done: task_done_too_many_raises (errors)."""
import queue

_raised = False
try:
    (lambda q: (q.put('x'), q.get(), q.task_done(), q.task_done()))(queue.Queue())
except ValueError:
    _raised = True
assert _raised, "task_done_too_many_raises: expected ValueError"
print("task_done_too_many_raises OK")
