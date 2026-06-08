# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "queue_empty_is_callable"
# subject = "queue.Queue().empty"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""queue.Queue().empty: queue_empty_is_callable (surface)."""
import queue

assert callable(queue.Queue().empty)
print("queue_empty_is_callable OK")
