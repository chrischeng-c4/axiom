# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_compression_error_is_present"
# subject = "tarfile.CompressionError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.CompressionError: api_compression_error_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "CompressionError")
print("api_compression_error_is_present OK")
