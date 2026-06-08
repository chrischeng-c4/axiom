# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "suffix_map_is_dict"
# subject = "mimetypes.suffix_map"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.suffix_map: suffix_map_is_dict (surface)."""
import mimetypes

assert type(mimetypes.suffix_map).__name__ == "dict"
print("suffix_map_is_dict OK")
