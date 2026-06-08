# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "surface"
# case = "iskeyword_is_callable"
# subject = "keyword.iskeyword"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""keyword.iskeyword: iskeyword_is_callable (surface)."""
import keyword

assert callable(keyword.iskeyword)
print("iskeyword_is_callable OK")
