# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "import_shutil"
# subject = "shutil"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shutil: import_shutil (surface)."""
import shutil

assert hasattr(shutil, "which")
print("import_shutil OK")
