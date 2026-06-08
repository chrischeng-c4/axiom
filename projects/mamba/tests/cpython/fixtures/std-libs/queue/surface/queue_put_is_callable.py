# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "queue_put_is_callable"
# subject = "queue.Queue().put"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""queue.Queue().put: queue_put_is_callable (surface)."""
import queue

assert callable(queue.Queue().put)
print("queue_put_is_callable OK")
