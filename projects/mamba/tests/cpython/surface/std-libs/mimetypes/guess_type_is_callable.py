# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "guess_type_is_callable"
# subject = "mimetypes.guess_type"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.guess_type: guess_type_is_callable (surface)."""
import mimetypes

assert callable(mimetypes.guess_type)
print("guess_type_is_callable OK")
