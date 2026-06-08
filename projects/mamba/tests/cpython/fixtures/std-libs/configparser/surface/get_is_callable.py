# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "get_is_callable"
# subject = "configparser.ConfigParser.get"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.get: get_is_callable (surface)."""
import configparser

assert callable(configparser.ConfigParser.get)
print("get_is_callable OK")
