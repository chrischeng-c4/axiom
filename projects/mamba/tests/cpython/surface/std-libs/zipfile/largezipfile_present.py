# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "largezipfile_present"
# subject = "zipfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile: largezipfile_present (surface)."""
import zipfile

assert hasattr(zipfile, "LargeZipFile")
print("largezipfile_present OK")
