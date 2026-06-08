# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "api_notset_is_present"
# subject = "logging.NOTSET"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.NOTSET: api_notset_is_present (surface)."""
import logging

assert hasattr(logging, "NOTSET")
print("api_notset_is_present OK")
