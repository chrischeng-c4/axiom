# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "interpolation_same_section_substitution"
# subject = "configparser.ConfigParser.get"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: basic %(name)s interpolation substitutes another option in the same section (base=/home, full=%(base)s/bob -> /home/bob)"""
import configparser

cp = configparser.ConfigParser()
cp.read_string("[s]\nbase = /home\nfull = %(base)s/bob\n")
assert cp.get("s", "full") == "/home/bob", f"basic = {cp.get('s', 'full')!r}"

print("interpolation_same_section_substitution OK")
