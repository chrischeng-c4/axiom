# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "decomposition_is_callable"
# subject = "unicodedata.decomposition"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unicodedata.decomposition: decomposition_is_callable (surface)."""
import unicodedata

assert callable(unicodedata.decomposition)
print("decomposition_is_callable OK")
