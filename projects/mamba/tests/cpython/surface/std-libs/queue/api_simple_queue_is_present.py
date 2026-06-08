# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "api_simple_queue_is_present"
# subject = "queue.SimpleQueue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""queue.SimpleQueue: api_simple_queue_is_present (surface)."""
import queue

assert hasattr(queue, "SimpleQueue")
print("api_simple_queue_is_present OK")
