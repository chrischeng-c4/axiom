# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_excepthook_is_present"
# subject = "threading.excepthook"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.excepthook: api_excepthook_is_present (surface)."""
import threading

assert hasattr(threading, "excepthook")
print("api_excepthook_is_present OK")
