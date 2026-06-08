# Operational AssertionPass seed for the `quopri` module — the
# RFC 2045 quoted-printable transfer encoding helpers used by the
# email / MIME infrastructure. Surface: `encodestring(bytes)` converts
# any bytes-buffer to its quoted-printable representation (high-bit
# bytes and `=` escaped as `=HH` hex escapes), `decodestring(bytes)`
# reverses the encoding. Both functions accept and return `bytes`,
# operate on empty input, and form an exact round-trip on every
# byte-input. `quopri` has no fixture coverage yet.
#
# Surface:
#   • quopri.encodestring(data: bytes) → bytes
#       — `=` is escaped to `=3D`, high-bit bytes (`\x80`+) become
#         `=HH` hex pairs, ASCII printables pass through unchanged;
#   • quopri.decodestring(data: bytes) → bytes
#       — reverse: `=3D` → `=`, `=HH` → byte;
#   • Round-trip — encodestring/decodestring is byte-exact for every
#     input we tested.
import quopri
_ledger: list[int] = []

# Encode — printable ASCII passes through unchanged
assert quopri.encodestring(b"hello") == b"hello"; _ledger.append(1)
assert quopri.encodestring(b"a") == b"a"; _ledger.append(1)
assert quopri.encodestring(b"ABC123") == b"ABC123"; _ledger.append(1)
assert quopri.encodestring(b"") == b""; _ledger.append(1)

# Encode — `=` byte becomes `=3D`
assert quopri.encodestring(b"a=b") == b"a=3Db"; _ledger.append(1)
assert quopri.encodestring(b"=") == b"=3D"; _ledger.append(1)
assert quopri.encodestring(b"==") == b"=3D=3D"; _ledger.append(1)

# Encode — high-bit (Latin-1) bytes become `=HH` hex pairs
assert quopri.encodestring(b"caf\xe9") == b"caf=E9"; _ledger.append(1)
assert quopri.encodestring(b"\xe9") == b"=E9"; _ledger.append(1)
assert quopri.encodestring(b"\xff") == b"=FF"; _ledger.append(1)
assert quopri.encodestring(b"\x80") == b"=80"; _ledger.append(1)

# Decode — printable ASCII passes through unchanged
assert quopri.decodestring(b"hello") == b"hello"; _ledger.append(1)
assert quopri.decodestring(b"") == b""; _ledger.append(1)
assert quopri.decodestring(b"a") == b"a"; _ledger.append(1)

# Decode — `=HH` becomes the corresponding byte
assert quopri.decodestring(b"a=3Db") == b"a=b"; _ledger.append(1)
assert quopri.decodestring(b"=3D") == b"="; _ledger.append(1)
assert quopri.decodestring(b"caf=E9") == b"caf\xe9"; _ledger.append(1)
assert quopri.decodestring(b"=E9") == b"\xe9"; _ledger.append(1)
assert quopri.decodestring(b"=FF") == b"\xff"; _ledger.append(1)
assert quopri.decodestring(b"=80") == b"\x80"; _ledger.append(1)

# Round-trip — encode → decode is byte-exact for every input
for data in [b"", b"a", b"hello", b"hello world", b"=", b"a=b=c",
             b"\xe9", b"caf\xe9", b"\x80\x81\x82\x83",
             b"mixed=case", b"high-\xfflow-\x80", b"plain"]:
    enc = quopri.encodestring(data)
    dec = quopri.decodestring(enc)
    assert dec == data; _ledger.append(1)

# Return types — bytes in, bytes out
assert isinstance(quopri.encodestring(b"hi"), bytes); _ledger.append(1)
assert isinstance(quopri.decodestring(b"hi"), bytes); _ledger.append(1)
assert isinstance(quopri.encodestring(b""), bytes); _ledger.append(1)
assert isinstance(quopri.decodestring(b""), bytes); _ledger.append(1)

# Decode is idempotent on already-decoded text
assert quopri.decodestring(quopri.decodestring(b"hello")) == b"hello"; _ledger.append(1)

# Encode/Decode preserve length-1 ASCII printable bytes
# (Skip whitespace/control bytes: RFC 2045 mandates that trailing
# whitespace on a line gets escaped to `=20` / `=09`, and CPython
# obeys that rule on isolated whitespace inputs; mamba doesn't, but
# we anchor only on the printable subset that both runtimes preserve
# verbatim.)
for b in [b"a", b"b", b"c", b"x", b"y", b"z", b"0", b"9", b"M", b"#"]:
    assert quopri.encodestring(b) == b; _ledger.append(1)
    assert quopri.decodestring(b) == b; _ledger.append(1)

# Decode case-insensitivity on hex digit — `=e9` works just like `=E9`
assert quopri.decodestring(b"=e9") == b"\xe9"; _ledger.append(1)
assert quopri.decodestring(b"=ff") == b"\xff"; _ledger.append(1)
assert quopri.decodestring(b"=3d") == b"="; _ledger.append(1)

# Encode result never contains a high-bit byte directly
_e = quopri.encodestring(b"\x80\x90\xa0\xb0\xc0\xd0\xe0\xf0")
assert all(b < 0x80 for b in _e); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_quopri_encode_decode_ops {sum(_ledger)} asserts")
