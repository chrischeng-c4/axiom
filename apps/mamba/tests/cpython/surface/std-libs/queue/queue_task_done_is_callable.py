# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "queue_task_done_is_callable"
# subject = "queue.Queue().task_done"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""queue.Queue().task_done: queue_task_done_is_callable (surface)."""
import queue

assert callable(queue.Queue().task_done)
print("queue_task_done_is_callable OK")
