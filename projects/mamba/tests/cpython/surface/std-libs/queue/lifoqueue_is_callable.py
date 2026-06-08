# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "lifoqueue_is_callable"
# subject = "queue.LifoQueue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""queue.LifoQueue: lifoqueue_is_callable (surface)."""
import queue

assert callable(queue.LifoQueue)
print("lifoqueue_is_callable OK")
