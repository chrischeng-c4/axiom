# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_raw_config_parser_is_present"
# subject = "configparser.RawConfigParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.RawConfigParser: api_raw_config_parser_is_present (surface)."""
import configparser

assert hasattr(configparser, "RawConfigParser")
print("api_raw_config_parser_is_present OK")
