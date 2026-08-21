# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "keys_are_case_insensitive"
# subject = "configparser.ConfigParser.get"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: option keys are case-folded by the default optionxform, so MyKey is retrievable as mykey or MYKEY"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[s]\nMyKey = hello\n")

assert cp.get("s", "mykey") == "hello", "lowercase key lookup"
assert cp.get("s", "MYKEY") == "hello", "uppercase key lookup"
# The stored key is folded to lowercase.
assert cp.options("s") == ["mykey"], f"options = {cp.options('s')!r}"

print("keys_are_case_insensitive OK")
