# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_reset_error_is_present"
# subject = "logging.config.RESET_ERROR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.RESET_ERROR: api_reset_error_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "RESET_ERROR")
print("api_reset_error_is_present OK")
