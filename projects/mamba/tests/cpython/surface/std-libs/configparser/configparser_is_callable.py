# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "configparser_is_callable"
# subject = "configparser.ConfigParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser: configparser_is_callable (surface)."""
import configparser

assert callable(configparser.ConfigParser)
print("configparser_is_callable OK")
