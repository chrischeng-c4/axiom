# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_quopri_ops"
# subject = "cpython321.test_quopri_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_quopri_ops.py"
# status = "filled"
# ///
"""cpython321.test_quopri_ops: execute CPython 3.12 seed test_quopri_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `quopri` quoted-printable codec
# (RFC 2045 transport encoding used in MIME email bodies).
# Surface: encodestring quotes bytes that need escaping (= → =3D);
# decodestring is the inverse; round-trip is identity for ASCII
# payloads.
# Companion to stub/test_quopri.py — vendored unittest seed.
import quopri
_ledger: list[int] = []
# Equals sign gets quoted as =3D (its hex byte)
assert quopri.encodestring(b"hello = world") == b"hello =3D world"; _ledger.append(1)
# Decode is the inverse of encode
assert quopri.decodestring(b"hello =3D world") == b"hello = world"; _ledger.append(1)
# Round-trip identity for ASCII payload
assert quopri.decodestring(quopri.encodestring(b"abc")) == b"abc"; _ledger.append(1)
# Round-trip for a longer ASCII payload
payload = b"mamba-quopri-roundtrip-payload"
assert quopri.decodestring(quopri.encodestring(payload)) == payload; _ledger.append(1)
# Empty bytes is identity
assert quopri.encodestring(b"") == b""; _ledger.append(1)
assert quopri.decodestring(b"") == b""; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_quopri_ops {sum(_ledger)} asserts")
