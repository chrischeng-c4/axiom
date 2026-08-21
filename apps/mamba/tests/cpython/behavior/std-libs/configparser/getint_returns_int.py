# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "getint_returns_int"
# subject = "configparser.ConfigParser.getint"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.getint: getint coerces a numeric option string to a Python int (key2 = 42 -> 42, type int)"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[section1]\nkey2 = 42\n")

i = cp.getint("section1", "key2")
assert i == 42, f"getint = {i!r}"
assert isinstance(i, int), f"getint type = {type(i)!r}"

print("getint_returns_int OK")
