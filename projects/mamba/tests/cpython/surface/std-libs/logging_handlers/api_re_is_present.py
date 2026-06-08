# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "surface"
# case = "api_re_is_present"
# subject = "logging.handlers.re"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.handlers.re: api_re_is_present (surface)."""
import logging.handlers

assert hasattr(logging.handlers, "re")
print("api_re_is_present OK")
