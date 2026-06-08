# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "tmp_max_present"
# subject = "tempfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile: tmp_max_present (surface)."""
import tempfile

assert hasattr(tempfile, "TMP_MAX")
print("tmp_max_present OK")
