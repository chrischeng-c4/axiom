# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "api_named_temporary_file_is_present"
# subject = "tempfile.NamedTemporaryFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: api_named_temporary_file_is_present (surface)."""
import tempfile

assert hasattr(tempfile, "NamedTemporaryFile")
print("api_named_temporary_file_is_present OK")
