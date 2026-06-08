# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_default_logging_config_port_is_present"
# subject = "logging.config.DEFAULT_LOGGING_CONFIG_PORT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.DEFAULT_LOGGING_CONFIG_PORT: api_default_logging_config_port_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "DEFAULT_LOGGING_CONFIG_PORT")
print("api_default_logging_config_port_is_present OK")
