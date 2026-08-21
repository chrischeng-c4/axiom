# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "badzipfile_present"
# subject = "zipfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile: badzipfile_present (surface)."""
import zipfile

assert hasattr(zipfile, "BadZipFile")
print("badzipfile_present OK")
