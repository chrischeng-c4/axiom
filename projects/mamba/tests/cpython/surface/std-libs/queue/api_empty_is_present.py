# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "surface"
# case = "api_empty_is_present"
# subject = "queue.Empty"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""queue.Empty: api_empty_is_present (surface)."""
import queue

assert hasattr(queue, "Empty")
print("api_empty_is_present OK")
