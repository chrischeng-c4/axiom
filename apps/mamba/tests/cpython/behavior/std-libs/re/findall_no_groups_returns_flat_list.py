# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "findall_no_groups_returns_flat_list"
# subject = "re.findall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.findall: findall with no capture groups returns a flat list of whole matches: r'\\w+' on 'hello world foo' is ['hello','world','foo']"""
import re

assert re.findall(r"\w+", "hello world foo") == ["hello", "world", "foo"]

print("findall_no_groups_returns_flat_list OK")
