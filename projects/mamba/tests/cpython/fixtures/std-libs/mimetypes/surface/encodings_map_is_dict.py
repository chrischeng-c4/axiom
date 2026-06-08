# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "encodings_map_is_dict"
# subject = "mimetypes.encodings_map"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.encodings_map: encodings_map_is_dict (surface)."""
import mimetypes

assert type(mimetypes.encodings_map).__name__ == "dict"
print("encodings_map_is_dict OK")
