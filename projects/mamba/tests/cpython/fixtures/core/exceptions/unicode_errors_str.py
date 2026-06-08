# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""UnicodeEncode/Decode/TranslateError str() formatting (CPython 3.12 oracle)."""

# Encode error spanning a range of characters.
u = UnicodeEncodeError("baz", "xxxxx", 1, 5, "foo")
assert str(u) == "'baz' codec can't encode characters in position 1-4: foo"

# Narrowing the range to a single character changes the wording.
u.end = 2
assert str(u) == "'baz' codec can't encode character '\\x78' in position 1: foo"
print("encode_error: range and single-char forms render")


# Decode error formats bytes rather than characters.
u = UnicodeDecodeError("baz", b"xxxxx", 1, 5, "foo")
assert str(u) == "'baz' codec can't decode bytes in position 1-4: foo"
u.end = 2
assert str(u) == "'baz' codec can't decode byte 0x78 in position 1: foo"
print("decode_error: bytes form renders")


# Translate error has no codec name in its message.
u = UnicodeTranslateError("xxxx", 1, 5, "foo")
assert str(u) == "can't translate characters in position 1-4: foo"
u.end = 2
assert str(u) == "can't translate character '\\x78' in position 1: foo"
print("translate_error: range and single-char forms render")


# Mutating encoding/start after construction is reflected in str().
u = UnicodeEncodeError("baz", "xxxxx", 1, 5, "foo")
u.encoding = "utf-7"
assert str(u) == "'utf-7' codec can't encode characters in position 1-4: foo"


# A freshly allocated UnicodeError with no object renders empty.
for klass in (UnicodeEncodeError, UnicodeDecodeError, UnicodeTranslateError):
    assert str(klass.__new__(klass)) == ""
print("no_object: bare instances render empty")

print("unicode_errors_str OK")
