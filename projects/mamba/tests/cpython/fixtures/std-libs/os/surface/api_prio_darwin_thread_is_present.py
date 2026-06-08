# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_prio_darwin_thread_is_present"
# subject = "os.PRIO_DARWIN_THREAD"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.PRIO_DARWIN_THREAD: api_prio_darwin_thread_is_present (surface)."""
import os

assert hasattr(os, "PRIO_DARWIN_THREAD")
print("api_prio_darwin_thread_is_present OK")
