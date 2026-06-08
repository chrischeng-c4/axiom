# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "fsencode_fsdecode_roundtrip"
# subject = "os.fsencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.fsencode: os.fsencode(str)->bytes and os.fsdecode(bytes)->str round-trip identically for encodable unicode names, and each is a no-op on its own target type"""
import os

# fsencode(str) -> bytes; fsdecode(bytes) -> str.
assert os.fsencode("ascii") == b"ascii", "fsencode ascii"
assert isinstance(os.fsencode("ascii"), bytes), "fsencode returns bytes"
assert os.fsdecode(b"ascii") == "ascii", "fsdecode ascii"
assert isinstance(os.fsdecode("ascii"), str), "fsdecode str passthrough type"

# fsencode is a no-op on bytes; fsdecode is a no-op on str.
assert os.fsencode(b"abc\xff") == b"abc\xff", "fsencode passes bytes through"
assert os.fsdecode("abcŁ") == "abcŁ", "fsdecode passes str through"

# Round-trip identity for encodable unicode names.
for name in ("ascii", "latié", "unicodeŁ"):
    encoded = os.fsencode(name)
    assert isinstance(encoded, bytes), f"fsencode({name!r}) type"
    assert os.fsdecode(encoded) == name, f"round-trip {name!r}"
print("fsencode_fsdecode_roundtrip OK")
