# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "open_is_callable"
# subject = "codecs.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""codecs.open: open_is_callable (surface)."""
import codecs

assert callable(codecs.open)
print("open_is_callable OK")
