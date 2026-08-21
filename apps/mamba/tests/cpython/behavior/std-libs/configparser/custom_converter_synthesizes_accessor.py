# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "custom_converter_synthesizes_accessor"
# subject = "configparser.ConfigParser.converters"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.converters: registering converters['decimal'] synthesizes a getdecimal() accessor on both the parser and each section proxy, returning the converted value"""
import configparser
import decimal

parser = configparser.ConfigParser()
parser.converters["decimal"] = decimal.Decimal
parser.read_string("[s1]\none = 1\n[s2]\ntwo = 2\n")

assert "decimal" in parser.converters, "converter registered"

# The accessor exists on the parser ...
assert parser.getdecimal("s1", "one") == decimal.Decimal("1"), "parser.getdecimal s1"
assert parser.getdecimal("s2", "two") == decimal.Decimal("2"), "parser.getdecimal s2"

# ... and on each section proxy.
assert parser["s1"].getdecimal("one") == decimal.Decimal("1"), "proxy getdecimal s1"
assert parser["s2"].getdecimal("two") == decimal.Decimal("2"), "proxy getdecimal s2"

print("custom_converter_synthesizes_accessor OK")
