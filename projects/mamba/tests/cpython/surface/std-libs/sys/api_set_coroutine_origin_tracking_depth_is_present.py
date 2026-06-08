# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_set_coroutine_origin_tracking_depth_is_present"
# subject = "sys.set_coroutine_origin_tracking_depth"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.set_coroutine_origin_tracking_depth: api_set_coroutine_origin_tracking_depth_is_present (surface)."""
import sys

assert hasattr(sys, "set_coroutine_origin_tracking_depth")
print("api_set_coroutine_origin_tracking_depth_is_present OK")
