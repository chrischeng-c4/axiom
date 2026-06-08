# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "api_lifo_queue_is_present"
# subject = "queue.LifoQueue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""queue.LifoQueue: api_lifo_queue_is_present (surface)."""
import queue

assert hasattr(queue, "LifoQueue")
print("api_lifo_queue_is_present OK")
