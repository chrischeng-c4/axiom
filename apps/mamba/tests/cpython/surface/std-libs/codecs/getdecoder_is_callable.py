# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "getdecoder_is_callable"
# subject = "codecs.getdecoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""codecs.getdecoder: getdecoder_is_callable (surface)."""
import codecs

assert callable(codecs.getdecoder)
print("getdecoder_is_callable OK")
