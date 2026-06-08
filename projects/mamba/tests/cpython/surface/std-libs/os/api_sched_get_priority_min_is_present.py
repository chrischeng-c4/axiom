# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_sched_get_priority_min_is_present"
# subject = "os.sched_get_priority_min"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.sched_get_priority_min: api_sched_get_priority_min_is_present (surface)."""
import os

assert hasattr(os, "sched_get_priority_min")
print("api_sched_get_priority_min_is_present OK")
