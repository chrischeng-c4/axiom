# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "api_spooled_temporary_file_is_present"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: api_spooled_temporary_file_is_present (surface)."""
import tempfile

assert hasattr(tempfile, "SpooledTemporaryFile")
print("api_spooled_temporary_file_is_present OK")
