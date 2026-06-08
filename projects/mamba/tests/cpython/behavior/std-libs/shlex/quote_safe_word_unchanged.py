# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "behavior"
# case = "quote_safe_word_unchanged"
# subject = "shlex.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.quote: a word built only from the shell-safe set (letters/digits/@%_-+=:,./) is returned unchanged, no quoting added"""
import shlex

assert shlex.quote("hello_world") == "hello_world", "underscored word is safe"
assert shlex.quote("a-b.c/d:e@f%g=h+i,j") == "a-b.c/d:e@f%g=h+i,j", "every safe punct char passes through"
print("quote_safe_word_unchanged OK")
