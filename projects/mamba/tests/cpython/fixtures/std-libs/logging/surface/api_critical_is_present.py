# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_critical_is_present"
# subject = "logging.CRITICAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.CRITICAL: api_critical_is_present (surface)."""
import logging

assert hasattr(logging, "CRITICAL")
print("api_critical_is_present OK")
