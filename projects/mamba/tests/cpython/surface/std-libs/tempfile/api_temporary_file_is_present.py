# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "api_temporary_file_is_present"
# subject = "tempfile.TemporaryFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tempfile.TemporaryFile: api_temporary_file_is_present (surface)."""
import tempfile

assert hasattr(tempfile, "TemporaryFile")
print("api_temporary_file_is_present OK")
