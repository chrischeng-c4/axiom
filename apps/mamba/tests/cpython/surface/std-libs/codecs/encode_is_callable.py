# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "encode_is_callable"
# subject = "codecs.encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""codecs.encode: encode_is_callable (surface)."""
import codecs

assert callable(codecs.encode)
print("encode_is_callable OK")
