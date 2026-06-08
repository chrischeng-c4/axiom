# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_valid_ident_is_present"
# subject = "logging.config.valid_ident"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.valid_ident: api_valid_ident_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "valid_ident")
print("api_valid_ident_is_present OK")
