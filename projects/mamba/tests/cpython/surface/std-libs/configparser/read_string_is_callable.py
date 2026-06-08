# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "read_string_is_callable"
# subject = "configparser.ConfigParser.read_string"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.read_string: read_string_is_callable (surface)."""
import configparser

assert callable(configparser.ConfigParser.read_string)
print("read_string_is_callable OK")
