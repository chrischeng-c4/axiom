# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "configparser_read_string_rejects_int_argument"
# subject = "configparser.ConfigParser.read_string"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""configparser.ConfigParser.read_string: configparser_read_string_rejects_int_argument (errors)."""
import configparser

try:
    result = configparser.ConfigParser().read_string(1)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
