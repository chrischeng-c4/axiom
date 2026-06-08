# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "api_temporary_directory_is_present"
# subject = "tempfile.TemporaryDirectory"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tempfile.TemporaryDirectory: api_temporary_directory_is_present (surface)."""
import tempfile

assert hasattr(tempfile, "TemporaryDirectory")
print("api_temporary_directory_is_present OK")
