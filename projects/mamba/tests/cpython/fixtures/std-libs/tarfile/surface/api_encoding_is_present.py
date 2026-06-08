# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_encoding_is_present"
# subject = "tarfile.ENCODING"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.ENCODING: api_encoding_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "ENCODING")
print("api_encoding_is_present OK")
