# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "types_map_is_dict"
# subject = "mimetypes.types_map"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.types_map: types_map_is_dict (surface)."""
import mimetypes

assert type(mimetypes.types_map).__name__ == "dict"
print("types_map_is_dict OK")
