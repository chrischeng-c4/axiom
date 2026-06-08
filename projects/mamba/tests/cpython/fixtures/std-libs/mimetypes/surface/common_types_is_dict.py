# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "common_types_is_dict"
# subject = "mimetypes.common_types"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.common_types: common_types_is_dict (surface)."""
import mimetypes

assert type(mimetypes.common_types).__name__ == "dict"
print("common_types_is_dict OK")
