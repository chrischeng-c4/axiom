# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "api_priority_queue_is_present"
# subject = "queue.PriorityQueue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""queue.PriorityQueue: api_priority_queue_is_present (surface)."""
import queue

assert hasattr(queue, "PriorityQueue")
print("api_priority_queue_is_present OK")
