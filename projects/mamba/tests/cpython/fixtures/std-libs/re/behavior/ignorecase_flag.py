# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "ignorecase_flag"
# subject = "re.IGNORECASE"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.IGNORECASE: re.IGNORECASE makes the match case-insensitive: re.search(r'hello','HELLO',re.IGNORECASE) is not None"""
import re

assert re.search(r"hello", "HELLO", re.IGNORECASE) is not None, "IGNORECASE match"
assert re.search(r"hello", "HELLO") is None, "no flag -> no match"

print("ignorecase_flag OK")
