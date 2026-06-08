# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "findall_one_group_returns_group_text"
# subject = "re.findall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.findall: findall with one capture group returns the group's text, not the whole match: r'(\\d+)' on 'abc123def456' is ['123','456']"""
import re

assert re.findall(r"(\d+)", "abc123def456") == ["123", "456"]

print("findall_one_group_returns_group_text OK")
