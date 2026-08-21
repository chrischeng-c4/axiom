# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "decode_is_callable"
# subject = "codecs.decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""codecs.decode: decode_is_callable (surface)."""
import codecs

assert callable(codecs.decode)
print("decode_is_callable OK")
