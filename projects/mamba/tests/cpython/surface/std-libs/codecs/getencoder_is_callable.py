# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "getencoder_is_callable"
# subject = "codecs.getencoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""codecs.getencoder: getencoder_is_callable (surface)."""
import codecs

assert callable(codecs.getencoder)
print("getencoder_is_callable OK")
