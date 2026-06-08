# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_defaultsect_is_present"
# subject = "configparser.DEFAULTSECT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.DEFAULTSECT: api_defaultsect_is_present (surface)."""
import configparser

assert hasattr(configparser, "DEFAULTSECT")
print("api_defaultsect_is_present OK")
