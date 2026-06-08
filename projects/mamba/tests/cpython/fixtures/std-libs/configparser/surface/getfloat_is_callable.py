# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "getfloat_is_callable"
# subject = "configparser.ConfigParser.getfloat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.getfloat: getfloat_is_callable (surface)."""
import configparser

assert callable(configparser.ConfigParser.getfloat)
print("getfloat_is_callable OK")
