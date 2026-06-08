# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_cpu_count_is_present"
# subject = "multiprocessing.cpu_count"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.cpu_count: api_cpu_count_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "cpu_count")
print("api_cpu_count_is_present OK")
