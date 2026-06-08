# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "priorityqueue_is_callable"
# subject = "queue.PriorityQueue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""queue.PriorityQueue: priorityqueue_is_callable (surface)."""
import queue

assert callable(queue.PriorityQueue)
print("priorityqueue_is_callable OK")
