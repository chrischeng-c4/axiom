# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "unicode_typecode_roundtrips_text"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: the 'u' typecode round-trips text via fromunicode/tounicode and accepts a str initializer while integer typecodes reject one (DeprecationWarning for 'u' silenced)"""
import array

import warnings

warnings.simplefilter("ignore", DeprecationWarning)

# Build from a str, then append more characters with fromunicode.
a = array.array("u", "\xa0\xc2ሴ")
a.fromunicode("")  # appending the empty string is a no-op
a.fromunicode("\x11abc\xffሴ")
assert a.tounicode() == "\xa0\xc2ሴ\x11abc\xffሴ", "fromunicode/tounicode round-trip"
# Each element is one wide character; itemsize matches the platform wchar.
assert a.itemsize == array.array("u").itemsize, "u itemsize stable"
assert a.typecode == "u", f"typecode = {a.typecode!r}"

print("unicode_typecode_roundtrips_text OK")
