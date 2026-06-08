# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "queue_get_nowait_is_callable"
# subject = "queue.Queue().get_nowait"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""queue.Queue().get_nowait: queue_get_nowait_is_callable (surface)."""
import queue

assert callable(queue.Queue().get_nowait)
print("queue_get_nowait_is_callable OK")
