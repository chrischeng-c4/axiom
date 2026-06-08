# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_freeze_support_is_present"
# subject = "multiprocessing.freeze_support"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.freeze_support: api_freeze_support_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "freeze_support")
print("api_freeze_support_is_present OK")
