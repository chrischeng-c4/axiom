# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "import_stat"
# subject = "stat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat: import_stat (surface)."""
import stat

assert hasattr(stat, "S_ISDIR")
print("import_stat OK")
