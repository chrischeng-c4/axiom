# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_converting_tuple_is_present"
# subject = "logging.config.ConvertingTuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.ConvertingTuple: api_converting_tuple_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "ConvertingTuple")
print("api_converting_tuple_is_present OK")
