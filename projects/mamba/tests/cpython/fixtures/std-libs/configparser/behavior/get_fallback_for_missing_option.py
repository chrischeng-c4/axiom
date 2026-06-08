# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "get_fallback_for_missing_option"
# subject = "configparser.ConfigParser.get"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ConfigParser.get: get(section, option, fallback=...) returns the fallback instead of raising when the option is absent"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[section1]\nkey1 = value1\n")

fallback = cp.get("section1", "missing_key", fallback="default_val")
assert fallback == "default_val", f"fallback = {fallback!r}"

# A present option ignores the fallback and returns its real value.
present = cp.get("section1", "key1", fallback="default_val")
assert present == "value1", f"present = {present!r}"

print("get_fallback_for_missing_option OK")
