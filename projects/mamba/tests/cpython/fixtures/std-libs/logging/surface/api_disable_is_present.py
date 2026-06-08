# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_disable_is_present"
# subject = "logging.disable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.disable: api_disable_is_present (surface)."""
import logging

assert hasattr(logging, "disable")
print("api_disable_is_present OK")
