# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "read_dict_is_callable"
# subject = "configparser.ConfigParser.read_dict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.read_dict: read_dict_is_callable (surface)."""
import configparser

assert callable(configparser.ConfigParser.read_dict)
print("read_dict_is_callable OK")
