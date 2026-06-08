# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "rawconfigparser_is_callable"
# subject = "configparser.RawConfigParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.RawConfigParser: rawconfigparser_is_callable (surface)."""
import configparser

assert callable(configparser.RawConfigParser)
print("rawconfigparser_is_callable OK")
