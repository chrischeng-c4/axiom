# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "surface"
# case = "import_keyword"
# subject = "keyword"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""keyword: import_keyword (surface)."""
import keyword

assert hasattr(keyword, "iskeyword")
print("import_keyword OK")
