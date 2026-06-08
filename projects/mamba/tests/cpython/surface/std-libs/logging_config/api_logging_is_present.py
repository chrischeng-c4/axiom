# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_logging_is_present"
# subject = "logging.config.logging"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.logging: api_logging_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "logging")
print("api_logging_is_present OK")
