# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "guess_extension_is_callable"
# subject = "mimetypes.guess_extension"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.guess_extension: guess_extension_is_callable (surface)."""
import mimetypes

assert callable(mimetypes.guess_extension)
print("guess_extension_is_callable OK")
