# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "surface"
# case = "issoftkeyword_is_callable"
# subject = "keyword.issoftkeyword"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""keyword.issoftkeyword: issoftkeyword_is_callable (surface)."""
import keyword

assert callable(keyword.issoftkeyword)
print("issoftkeyword_is_callable OK")
