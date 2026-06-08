# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "options_is_callable"
# subject = "configparser.ConfigParser.options"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.options: options_is_callable (surface)."""
import configparser

assert callable(configparser.ConfigParser.options)
print("options_is_callable OK")
