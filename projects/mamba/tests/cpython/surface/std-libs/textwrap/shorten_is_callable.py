# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "surface"
# case = "shorten_is_callable"
# subject = "textwrap.shorten"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.shorten: shorten_is_callable (surface)."""
import textwrap

assert callable(textwrap.shorten)
print("shorten_is_callable OK")
