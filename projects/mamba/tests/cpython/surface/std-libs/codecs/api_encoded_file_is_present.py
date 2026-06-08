# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_encoded_file_is_present"
# subject = "codecs.EncodedFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.EncodedFile: api_encoded_file_is_present (surface)."""
import codecs

assert hasattr(codecs, "EncodedFile")
print("api_encoded_file_is_present OK")
