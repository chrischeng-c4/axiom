# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sched"
# dimension = "surface"
# case = "api_scheduler_is_present"
# subject = "sched.scheduler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sched.scheduler: api_scheduler_is_present (surface)."""
import sched

assert hasattr(sched, "scheduler")
print("api_scheduler_is_present OK")
