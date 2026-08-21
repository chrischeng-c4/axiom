# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "interpolation_raw_bypass"
# subject = "configparser.ConfigParser.get"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: get(..., raw=True) returns the uninterpolated template string verbatim (%(base)s/bob), bypassing substitution"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[s]\nbase = /home\nfull = %(base)s/bob\n")

assert cp.get("s", "full") == "/home/bob", "interpolated value"
assert cp.get("s", "full", raw=True) == "%(base)s/bob", "raw bypasses interpolation"

print("interpolation_raw_bypass OK")
