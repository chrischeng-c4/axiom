# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "default_section_inherited_by_all"
# subject = "configparser.ConfigParser.get"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: options in the DEFAULT section are inherited by every other section (color=blue in DEFAULT is visible from s1 and s2)"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[DEFAULT]\ncolor = blue\n[s1]\nname = alice\n[s2]\nname = bob\n")

assert cp.get("s1", "color") == "blue", "s1 inherits DEFAULT"
assert cp.get("s2", "color") == "blue", "s2 inherits DEFAULT"
# The section's own option still wins over the inherited default.
assert cp.get("s1", "name") == "alice", "s1 own option"

print("default_section_inherited_by_all OK")
