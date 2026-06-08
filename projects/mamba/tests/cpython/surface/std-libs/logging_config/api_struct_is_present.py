# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_struct_is_present"
# subject = "logging.config.struct"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.struct: api_struct_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "struct")
print("api_struct_is_present OK")
