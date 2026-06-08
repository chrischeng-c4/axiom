# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "queue_full_is_callable"
# subject = "queue.Queue().full"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""queue.Queue().full: queue_full_is_callable (surface)."""
import queue

assert callable(queue.Queue().full)
print("queue_full_is_callable OK")
