# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_set_forkserver_preload_is_present"
# subject = "multiprocessing.set_forkserver_preload"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.set_forkserver_preload: api_set_forkserver_preload_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "set_forkserver_preload")
print("api_set_forkserver_preload_is_present OK")
