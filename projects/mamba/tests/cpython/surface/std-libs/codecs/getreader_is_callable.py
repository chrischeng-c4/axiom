# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "getreader_is_callable"
# subject = "codecs.getreader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""codecs.getreader: getreader_is_callable (surface)."""
import codecs

assert callable(codecs.getreader)
print("getreader_is_callable OK")
