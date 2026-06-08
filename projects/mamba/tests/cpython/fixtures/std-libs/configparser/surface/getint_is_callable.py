# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "getint_is_callable"
# subject = "configparser.ConfigParser.getint"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.getint: getint_is_callable (surface)."""
import configparser

assert callable(configparser.ConfigParser.getint)
print("getint_is_callable OK")
