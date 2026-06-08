# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_extension_dotted_string"
# subject = "mimetypes.guess_extension"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_extension: guess_extension returns a single dotted extension string for a known type (image/png)"""
import mimetypes

ext = mimetypes.guess_extension("image/png")
assert ext is not None, "image/png has extension"
assert isinstance(ext, str), f"extension is str: {type(ext)!r}"
assert ext.startswith("."), f"starts with dot: {ext!r}"
print("guess_extension_dotted_string OK")
