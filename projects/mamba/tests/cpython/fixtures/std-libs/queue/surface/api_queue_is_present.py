# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "api_queue_is_present"
# subject = "queue.Queue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""queue.Queue: api_queue_is_present (surface)."""
import queue

assert hasattr(queue, "Queue")
print("api_queue_is_present OK")
