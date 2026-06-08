# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_converting_list_is_present"
# subject = "logging.config.ConvertingList"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.ConvertingList: api_converting_list_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "ConvertingList")
print("api_converting_list_is_present OK")
