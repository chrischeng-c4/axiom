# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "behavior"
# case = "quote_empty_string"
# subject = "shlex.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.quote: the empty string quotes to the two-character literal '' so it survives a shell word boundary"""
import shlex

assert shlex.quote("") == "''", "empty string quotes to ''"
print("quote_empty_string OK")
