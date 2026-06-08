# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_setprofile_is_present"
# subject = "threading.setprofile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.setprofile: api_setprofile_is_present (surface)."""
import threading

assert hasattr(threading, "setprofile")
print("api_setprofile_is_present OK")
