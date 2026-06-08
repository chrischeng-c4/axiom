# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_config_parser_is_present"
# subject = "configparser.ConfigParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.ConfigParser: api_config_parser_is_present (surface)."""
import configparser

assert hasattr(configparser, "ConfigParser")
print("api_config_parser_is_present OK")
