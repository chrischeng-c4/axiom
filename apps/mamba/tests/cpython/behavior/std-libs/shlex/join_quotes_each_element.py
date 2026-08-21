# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "behavior"
# case = "join_quotes_each_element"
# subject = "shlex.join"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.join: join applies quote() to every element: safe words pass through, a word with a space is single-quoted, and an empty list joins to the empty string"""
import shlex

assert shlex.join(["a", "b", "c"]) == "a b c", "safe words join with single spaces"
assert shlex.join(["a", "b c", "d"]) == "a 'b c' d", "the spaced element is single-quoted"
assert shlex.join([]) == "", "the empty list joins to the empty string"
print("join_quotes_each_element OK")
