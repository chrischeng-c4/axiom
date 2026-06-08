# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "queue_put_nowait_is_callable"
# subject = "queue.Queue().put_nowait"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""queue.Queue().put_nowait: queue_put_nowait_is_callable (surface)."""
import queue

assert callable(queue.Queue().put_nowait)
print("queue_put_nowait_is_callable OK")
