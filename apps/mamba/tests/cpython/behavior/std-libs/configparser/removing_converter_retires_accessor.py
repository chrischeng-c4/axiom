# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "removing_converter_retires_accessor"
# subject = "configparser.ConfigParser.converters"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.converters: deleting converters['decimal'] retires the synthesized getdecimal() accessor on both the parser and its section proxies, so a later call raises AttributeError"""
import configparser
import decimal

parser = configparser.ConfigParser()
parser.converters["decimal"] = decimal.Decimal
parser.read_string("[s1]\none = 1\n")
assert parser.getdecimal("s1", "one") == decimal.Decimal("1"), "accessor present"

# Removing the converter retires the accessor.
del parser.converters["decimal"]
assert "decimal" not in parser.converters, "converter removed"

raised = False
try:
    parser.getdecimal("s1", "one")
except AttributeError:
    raised = True
assert raised, "getdecimal gone after converter removed"

raised_proxy = False
try:
    parser["s1"].getdecimal("one")
except AttributeError:
    raised_proxy = True
assert raised_proxy, "proxy getdecimal gone after converter removed"

print("removing_converter_retires_accessor OK")
