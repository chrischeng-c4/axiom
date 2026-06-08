# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "sections_is_callable"
# subject = "configparser.ConfigParser.sections"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.sections: sections_is_callable (surface)."""
import configparser

assert callable(configparser.ConfigParser.sections)
print("sections_is_callable OK")
