# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "behavior"
# case = "quote_wraps_whitespace"
# subject = "shlex.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.quote: a value containing whitespace is wrapped in single quotes, e.g. quote('hello world') == "'hello world'" """
import shlex

assert shlex.quote("hello world") == "'hello world'", "space forces single-quoting"
assert shlex.quote("test file name") == "'test file name'", "multiple spaces still one quoted token"
print("quote_wraps_whitespace OK")
