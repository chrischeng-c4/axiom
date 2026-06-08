# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "api_tempdir_is_present"
# subject = "tempfile.tempdir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tempfile.tempdir: api_tempdir_is_present (surface)."""
import tempfile

assert hasattr(tempfile, "tempdir")
print("api_tempdir_is_present OK")
