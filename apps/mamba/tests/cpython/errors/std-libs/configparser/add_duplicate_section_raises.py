# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "add_duplicate_section_raises"
# subject = "configparser.ConfigParser.add_section"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.add_section: add_duplicate_section_raises (errors)."""
import configparser
_cp_dup_sec = configparser.ConfigParser()
_cp_dup_sec.read_string('[s1]\nk=v\n')

_raised = False
try:
    _cp_dup_sec.add_section('s1')
except configparser.DuplicateSectionError:
    _raised = True
assert _raised, "add_duplicate_section_raises: expected configparser.DuplicateSectionError"
print("add_duplicate_section_raises OK")
