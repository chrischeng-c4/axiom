# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_sched_rr_is_present"
# subject = "os.SCHED_RR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.SCHED_RR: api_sched_rr_is_present (surface)."""
import os

assert hasattr(os, "SCHED_RR")
print("api_sched_rr_is_present OK")
