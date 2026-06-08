# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "findall_no_match_returns_empty_list"
# subject = "re.findall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.findall: findall returns an empty list (never None) when nothing matches: r'\\d+' on 'no digits' and on '' are both []"""
import re

assert re.findall(r"\d+", "no digits") == []
assert re.findall(r"\d+", "") == []

print("findall_no_match_returns_empty_list OK")
