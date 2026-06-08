# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_thread_info_is_present"
# subject = "sys.thread_info"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.thread_info: api_thread_info_is_present (surface)."""
import sys

assert hasattr(sys, "thread_info")
print("api_thread_info_is_present OK")
