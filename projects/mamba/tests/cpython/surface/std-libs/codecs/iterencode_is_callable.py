# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "iterencode_is_callable"
# subject = "codecs.iterencode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""codecs.iterencode: iterencode_is_callable (surface)."""
import codecs

assert callable(codecs.iterencode)
print("iterencode_is_callable OK")
