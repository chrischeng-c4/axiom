# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "mimetypes_class_is_callable"
# subject = "mimetypes.MimeTypes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.MimeTypes: mimetypes_class_is_callable (surface)."""
import mimetypes

assert callable(mimetypes.MimeTypes)
print("mimetypes_class_is_callable OK")
