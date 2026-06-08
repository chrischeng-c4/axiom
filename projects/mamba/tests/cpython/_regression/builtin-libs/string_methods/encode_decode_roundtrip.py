# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""str <-> bytes: UTF-8 encode/decode + error handlers (CPython 3.12)."""

# --- UTF-8 encode of multi-byte and astral code points ----------------
assert "".encode("utf-8") == b""
assert "€".encode("utf-8") == b"\xe2\x82\xac"          # 3-byte BMP
assert "𐀂".encode("utf-8") == b"\xf0\x90\x80\x82"     # 4-byte astral
assert ("𐀂" * 3).encode("utf-8") == b"\xf0\x90\x80\x82" * 3
print("encode:", "café".encode("utf-8"))

# --- decode of valid UTF-8 byte sequences -----------------------------
valid = [
    (b"\x00", "\x00"),
    (b"a", "a"),
    (b"\xc2\x80", "\x80"),          # 2-byte boundary
    (b"\xdf\xbf", "߿"),        # 2-byte max
    (b"\xe0\xa0\x80", "ࠀ"),    # 3-byte boundary
    (b"\xef\xbf\xbf", "￿"),    # 3-byte max
    (b"\xf0\x90\x80\x80", "\U00010000"),  # 4-byte boundary
    (b"\xf4\x8f\xbf\xbf", "\U0010ffff"),  # 4-byte max
]
for raw, want in valid:
    assert raw.decode("utf-8") == want, raw
print("decode roundtrip:", b"caf\xc3\xa9".decode("utf-8"))

# --- the str() constructor decodes bytes ------------------------------
assert str() == ""
assert str(b"foo") == "b'foo'"                          # no encoding -> repr
assert str(b"caf\xc3\xa9", "utf-8") == "café"
assert str(b"foo", encoding="utf-8") == "foo"
assert str(object="bar") == "bar"

# --- decode error handlers on invalid UTF-8 ---------------------------
bad = b"caf\xff\xfe!"
raised = False
try:
    bad.decode("utf-8")                                 # strict (default)
except UnicodeDecodeError:
    raised = True
assert raised
# replace substitutes U+FFFD per ill-formed unit; ignore drops them.
assert bad.decode("utf-8", "replace") == "caf��!"
assert bad.decode("utf-8", "ignore") == "caf!"
print("error handlers:", b"\x80".decode("utf-8", "replace"))

# --- encode error handlers (ASCII target) -----------------------------
assert "Andr\x82 x".encode("ascii", "ignore") == b"Andr x"
assert "Andr\x82 x".encode("ascii", "replace") == b"Andr? x"
raised = False
try:
    "Andr\x82 x".encode("ascii")
except UnicodeEncodeError:
    raised = True
assert raised

print("encode_decode_roundtrip OK")
