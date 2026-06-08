# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "queue_qsize_is_callable"
# subject = "queue.Queue().qsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""queue.Queue().qsize: queue_qsize_is_callable (surface)."""
import queue

assert callable(queue.Queue().qsize)
print("queue_qsize_is_callable OK")
