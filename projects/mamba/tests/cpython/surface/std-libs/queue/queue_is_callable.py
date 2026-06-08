# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "queue_is_callable"
# subject = "queue.Queue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""queue.Queue: queue_is_callable (surface)."""
import queue

assert callable(queue.Queue)
print("queue_is_callable OK")
