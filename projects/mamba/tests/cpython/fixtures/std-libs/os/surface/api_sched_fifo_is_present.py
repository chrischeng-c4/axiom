# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_sched_fifo_is_present"
# subject = "os.SCHED_FIFO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.SCHED_FIFO: api_sched_fifo_is_present (surface)."""
import os

assert hasattr(os, "SCHED_FIFO")
print("api_sched_fifo_is_present OK")
