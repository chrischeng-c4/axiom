# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_stop_listening_is_present"
# subject = "logging.config.stopListening"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.stopListening: api_stop_listening_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "stopListening")
print("api_stop_listening_is_present OK")
