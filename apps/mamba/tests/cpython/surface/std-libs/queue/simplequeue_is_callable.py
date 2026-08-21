# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "simplequeue_is_callable"
# subject = "queue.SimpleQueue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""queue.SimpleQueue: simplequeue_is_callable (surface)."""
import queue

assert callable(queue.SimpleQueue)
print("simplequeue_is_callable OK")
