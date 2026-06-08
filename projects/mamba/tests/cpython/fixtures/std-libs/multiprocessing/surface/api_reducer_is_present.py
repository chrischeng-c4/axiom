# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_reducer_is_present"
# subject = "multiprocessing.reducer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.reducer: api_reducer_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "reducer")
print("api_reducer_is_present OK")
