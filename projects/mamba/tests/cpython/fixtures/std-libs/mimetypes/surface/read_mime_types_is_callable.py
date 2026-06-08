# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "read_mime_types_is_callable"
# subject = "mimetypes.read_mime_types"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.read_mime_types: read_mime_types_is_callable (surface)."""
import mimetypes

assert callable(mimetypes.read_mime_types)
print("read_mime_types_is_callable OK")
