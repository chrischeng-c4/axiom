# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "normalize_encoding_folds_separators"
# subject = "encodings.normalize_encoding"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""encodings.normalize_encoding: encodings.normalize_encoding folds whitespace runs to a single underscore while preserving case and dots: 'utf   8' -> 'utf_8', 'UTF 8' -> 'UTF_8', 'utf.8' -> 'utf.8'"""
import encodings

_normalize = encodings.normalize_encoding
assert _normalize("utf_8") == "utf_8"
assert _normalize("utf   8") == "utf_8"
assert _normalize("UTF 8") == "UTF_8"  # case is preserved
assert _normalize("utf.8") == "utf.8"  # dots are kept

print("normalize_encoding_folds_separators OK")
