# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "surface"
# case = "quote_is_callable"
# subject = "shlex.quote"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shlex.quote: quote_is_callable (surface)."""
import shlex

assert callable(shlex.quote)
print("quote_is_callable OK")
