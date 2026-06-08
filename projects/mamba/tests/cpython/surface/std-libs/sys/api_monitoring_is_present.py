# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_monitoring_is_present"
# subject = "sys.monitoring"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.monitoring: api_monitoring_is_present (surface)."""
import sys

assert hasattr(sys, "monitoring")
print("api_monitoring_is_present OK")
