# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_getprofile_is_present"
# subject = "threading.getprofile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.getprofile: api_getprofile_is_present (surface)."""
import threading

assert hasattr(threading, "getprofile")
print("api_getprofile_is_present OK")
