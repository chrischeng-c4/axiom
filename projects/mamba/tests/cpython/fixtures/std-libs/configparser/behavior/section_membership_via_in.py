# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "section_membership_via_in"
# subject = "configparser.ConfigParser.__contains__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.__contains__: the parser supports `name in parser` section membership; a parsed section is a member, DEFAULT is always a member, and an absent section is not"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[s]\nk = v\n")

assert "s" in cp, "section membership via in"
assert "DEFAULT" in cp, "DEFAULT always a member"
assert "absent" not in cp, "absent section not a member"

print("section_membership_via_in OK")
