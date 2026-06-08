# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "remove_option_and_section"
# subject = "configparser.ConfigParser.remove_option"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.remove_option: remove_option deletes one key leaving siblings intact; remove_section deletes the whole section so has_section/has_option go False"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[s]\nk1=v1\nk2=v2\n")

cp.remove_option("s", "k1")
assert not cp.has_option("s", "k1"), "option removed"
assert cp.has_option("s", "k2"), "other option still there"

cp.remove_section("s")
assert not cp.has_section("s"), "section removed"
assert not cp.has_option("s", "k2"), "options gone with the section"

print("remove_option_and_section OK")
