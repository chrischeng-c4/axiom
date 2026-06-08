# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "codecinfo_lookup_type"
# subject = "codecs.lookup('utf-8')"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""codecs.lookup('utf-8'): codecinfo_lookup_type (surface)."""
import codecs

assert type(codecs.lookup('utf-8')).__name__ == "CodecInfo"
print("codecinfo_lookup_type OK")
