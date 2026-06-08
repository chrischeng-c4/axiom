# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "compiled_pattern_reuse"
# subject = "re.Pattern.findall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Pattern.findall: a compiled pattern is reusable across method calls: re.compile(r'[aeiou]').findall('hello world') is ['e','o','o'] and .sub('_','hello') is 'h_ll_'"""
import re

pat = re.compile(r"[aeiou]")
assert pat.findall("hello world") == ["e", "o", "o"], "compiled findall"
assert pat.sub("_", "hello") == "h_ll_", "compiled sub reuse"

print("compiled_pattern_reuse OK")
