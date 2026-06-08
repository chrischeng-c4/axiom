# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_pipe_is_present"
# subject = "multiprocessing.Pipe"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.Pipe: api_pipe_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "Pipe")
print("api_pipe_is_present OK")
