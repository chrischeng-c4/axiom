# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "import_urllib_parse"
# subject = "urllib.parse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse: import_urllib_parse (surface)."""
import urllib.parse

assert hasattr(urllib.parse, "quote")
print("import_urllib_parse OK")
