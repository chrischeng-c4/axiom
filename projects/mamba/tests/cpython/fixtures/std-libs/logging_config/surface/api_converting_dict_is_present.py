# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_converting_dict_is_present"
# subject = "logging.config.ConvertingDict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.ConvertingDict: api_converting_dict_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "ConvertingDict")
print("api_converting_dict_is_present OK")
