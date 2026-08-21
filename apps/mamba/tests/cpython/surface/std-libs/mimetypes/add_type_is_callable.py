# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "add_type_is_callable"
# subject = "mimetypes.add_type"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.add_type: add_type_is_callable (surface)."""
import mimetypes

assert callable(mimetypes.add_type)
print("add_type_is_callable OK")
