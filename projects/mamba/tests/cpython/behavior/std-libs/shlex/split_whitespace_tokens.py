# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "behavior"
# case = "split_whitespace_tokens"
# subject = "shlex.split"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.split: whitespace-separated words split into a token list; the empty string yields an empty list"""
import shlex

assert shlex.split("hello world") == ["hello", "world"], 'split("hello world")'
assert shlex.split("foo   bar    bla") == ["foo", "bar", "bla"], "runs of whitespace collapse"
assert shlex.split("") == [], 'split("")'
print("split_whitespace_tokens OK")
