# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_get_native_id_is_present"
# subject = "threading.get_native_id"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.get_native_id: api_get_native_id_is_present (surface)."""
import threading

assert hasattr(threading, "get_native_id")
print("api_get_native_id_is_present OK")
