# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "surface"
# case = "api_converting_mixin_is_present"
# subject = "logging.config.ConvertingMixin"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""logging.config.ConvertingMixin: api_converting_mixin_is_present (surface)."""
import logging.config

assert hasattr(logging.config, "ConvertingMixin")
print("api_converting_mixin_is_present OK")
