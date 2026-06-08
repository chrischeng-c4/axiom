# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "surface"
# case = "api_st_ino_is_present"
# subject = "logging.handlers.ST_INO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.handlers.ST_INO: api_st_ino_is_present (surface)."""
import logging.handlers

assert hasattr(logging.handlers, "ST_INO")
print("api_st_ino_is_present OK")
