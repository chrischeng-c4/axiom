# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "guess_all_extensions_is_callable"
# subject = "mimetypes.guess_all_extensions"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.guess_all_extensions: guess_all_extensions_is_callable (surface)."""
import mimetypes

assert callable(mimetypes.guess_all_extensions)
print("guess_all_extensions_is_callable OK")
