# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "split_no_groups"
# subject = "re.split"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.split: split breaks on the separator pattern and drops it: re.split(r'\\s+','hello  world  foo') is ['hello','world','foo']"""
import re

assert re.split(r"\s+", "hello  world  foo") == ["hello", "world", "foo"]

print("split_no_groups OK")
