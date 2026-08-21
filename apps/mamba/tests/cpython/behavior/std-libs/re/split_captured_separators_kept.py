# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "split_captured_separators_kept"
# subject = "re.split"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.split: a capturing group in the split pattern keeps the separators: re.split(r'(\\d+)','a1b2c3') is ['a','1','b','2','c','3','']"""
import re

assert re.split(r"(\d+)", "a1b2c3") == ["a", "1", "b", "2", "c", "3", ""]

print("split_captured_separators_kept OK")
