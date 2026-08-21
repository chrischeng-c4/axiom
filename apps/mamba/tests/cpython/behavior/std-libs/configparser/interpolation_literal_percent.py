# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "interpolation_literal_percent"
# subject = "configparser.ConfigParser.get"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: a doubled %% in a value is an escaped literal percent sign (p = 100%% -> 100%)"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[s]\np = 100%%\n")
assert cp.get("s", "p") == "100%", f"literal percent = {cp.get('s', 'p')!r}"

print("interpolation_literal_percent OK")
