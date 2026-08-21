# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "getwriter_is_callable"
# subject = "codecs.getwriter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""codecs.getwriter: getwriter_is_callable (surface)."""
import codecs

assert callable(codecs.getwriter)
print("getwriter_is_callable OK")
