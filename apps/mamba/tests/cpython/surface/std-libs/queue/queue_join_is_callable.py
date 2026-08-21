# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "queue_join_is_callable"
# subject = "queue.Queue().join"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""queue.Queue().join: queue_join_is_callable (surface)."""
import queue

assert callable(queue.Queue().join)
print("queue_join_is_callable OK")
