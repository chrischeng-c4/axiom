# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_identifier_is_present"
# subject = "logging.config.IDENTIFIER"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.IDENTIFIER: api_identifier_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "IDENTIFIER")
print("api_identifier_is_present OK")
