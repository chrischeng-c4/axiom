# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "tempdir_present"
# subject = "tempfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile: tempdir_present (surface)."""
import tempfile

assert hasattr(tempfile, "tempdir")
print("tempdir_present OK")
