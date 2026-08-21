# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "behavior"
# case = "quote_embedded_single_quote"
# subject = "shlex.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.quote: an embedded single quote is escaped with the classic '"'"' splice, e.g. quote("it's") == '\\'it\\'"\\'"\\'s\\''"""
import shlex

# The whole word is single-quoted; the inner ' is broken out as '"'"' so the
# shell rejoins it as a literal single quote.
assert shlex.quote("it's") == "'it'\"'\"'s'", "embedded single quote uses the '\"'\"' splice"
print("quote_embedded_single_quote OK")
