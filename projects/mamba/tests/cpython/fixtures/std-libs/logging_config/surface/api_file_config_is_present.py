# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_file_config_is_present"
# subject = "logging.config.fileConfig"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.fileConfig: api_file_config_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "fileConfig")
print("api_file_config_is_present OK")
