# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_listen_is_present"
# subject = "logging.config.listen"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.listen: api_listen_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "listen")
print("api_listen_is_present OK")
