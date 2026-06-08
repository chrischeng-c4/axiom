# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_sched_get_priority_max_is_present"
# subject = "os.sched_get_priority_max"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.sched_get_priority_max: api_sched_get_priority_max_is_present (surface)."""
import os

assert hasattr(os, "sched_get_priority_max")
print("api_sched_get_priority_max_is_present OK")
