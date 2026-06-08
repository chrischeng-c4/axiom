# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "getfloat_returns_float"
# subject = "configparser.ConfigParser.getfloat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.getfloat: getfloat coerces a numeric option string to a Python float (key3 = 3.14 -> 3.14, type float)"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[section1]\nkey3 = 3.14\n")

f = cp.getfloat("section1", "key3")
assert abs(f - 3.14) < 0.001, f"getfloat = {f!r}"
assert isinstance(f, float), f"getfloat type = {type(f)!r}"

print("getfloat_returns_float OK")
