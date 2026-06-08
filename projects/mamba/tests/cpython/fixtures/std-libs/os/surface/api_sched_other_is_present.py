# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_sched_other_is_present"
# subject = "os.SCHED_OTHER"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.SCHED_OTHER: api_sched_other_is_present (surface)."""
import os

assert hasattr(os, "SCHED_OTHER")
print("api_sched_other_is_present OK")
