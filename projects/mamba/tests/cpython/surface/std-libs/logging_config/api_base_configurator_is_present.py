# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_base_configurator_is_present"
# subject = "logging.config.BaseConfigurator"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.BaseConfigurator: api_base_configurator_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "BaseConfigurator")
print("api_base_configurator_is_present OK")
