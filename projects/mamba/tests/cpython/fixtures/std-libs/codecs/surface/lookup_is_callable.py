# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "lookup_is_callable"
# subject = "codecs.lookup"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""codecs.lookup: lookup_is_callable (surface)."""
import codecs

assert callable(codecs.lookup)
print("lookup_is_callable OK")
