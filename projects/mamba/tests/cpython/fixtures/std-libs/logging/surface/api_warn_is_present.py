# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_warn_is_present"
# subject = "logging.WARN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.WARN: api_warn_is_present (surface)."""
import logging

assert hasattr(logging, "WARN")
print("api_warn_is_present OK")
