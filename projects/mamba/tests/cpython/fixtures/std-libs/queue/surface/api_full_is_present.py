# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "api_full_is_present"
# subject = "queue.Full"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""queue.Full: api_full_is_present (surface)."""
import queue

assert hasattr(queue, "Full")
print("api_full_is_present OK")
