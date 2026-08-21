# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "import_codecs"
# subject = "codecs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""codecs: import_codecs (surface)."""
import codecs

assert hasattr(codecs, "encode")
print("import_codecs OK")
