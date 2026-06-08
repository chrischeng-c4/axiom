# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "import_zipfile"
# subject = "zipfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile: import_zipfile (surface)."""
import zipfile

assert hasattr(zipfile, "ZipFile")
print("import_zipfile OK")
