# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "types_map_values_roundtrip"
# subject = "mimetypes.guess_extension"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_extension: every value in the default types_map round-trips: each registered MIME type has at least one guess_extension result"""
import mimetypes

for mime in mimetypes.types_map.values():
    assert mimetypes.guess_extension(mime) is not None, f"no extension for {mime!r}"
print("types_map_values_roundtrip OK")
