# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_joinable_queue_is_present"
# subject = "multiprocessing.JoinableQueue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.JoinableQueue: api_joinable_queue_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "JoinableQueue")
print("api_joinable_queue_is_present OK")
