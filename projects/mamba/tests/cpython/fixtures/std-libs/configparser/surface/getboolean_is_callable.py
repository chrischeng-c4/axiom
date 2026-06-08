# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "getboolean_is_callable"
# subject = "configparser.ConfigParser.getboolean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.getboolean: getboolean_is_callable (surface)."""
import configparser

assert callable(configparser.ConfigParser.getboolean)
print("getboolean_is_callable OK")
