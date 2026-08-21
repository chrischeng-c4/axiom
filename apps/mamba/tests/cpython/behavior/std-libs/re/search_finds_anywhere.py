# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "search_finds_anywhere"
# subject = "re.search"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.search: re.search finds the first hit anywhere: r'\\d+' on 'abc123def' yields group '123'"""
import re

m = re.search(r"\d+", "abc123def")
assert m is not None, "search found"
assert m.group() == "123", f"group = {m.group()!r}"

print("search_finds_anywhere OK")
