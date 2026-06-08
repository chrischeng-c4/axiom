# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "getincrementaldecoder_is_callable"
# subject = "codecs.getincrementaldecoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""codecs.getincrementaldecoder: getincrementaldecoder_is_callable (surface)."""
import codecs

assert callable(codecs.getincrementaldecoder)
print("getincrementaldecoder_is_callable OK")
