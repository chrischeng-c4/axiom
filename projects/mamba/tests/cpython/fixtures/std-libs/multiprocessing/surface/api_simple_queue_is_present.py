# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_simple_queue_is_present"
# subject = "multiprocessing.SimpleQueue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.SimpleQueue: api_simple_queue_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "SimpleQueue")
print("api_simple_queue_is_present OK")
