# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "real_world"
# case = "ascii_transport_round_trip"
# subject = "base64.b64encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b64encode: end-user scenario: serialise a small binary blob to base64 for an ASCII-only transport, then decode it back, also via the urlsafe alphabet"""
import base64

# A short payload (length 26 — one byte short of a 3-byte boundary) so the
# encoded result carries the single-'=' padding case.
payload = b"Mamba base64 round-trip!\x00\x01"

encoded = base64.b64encode(payload)
assert isinstance(encoded, bytes), type(encoded).__name__
assert encoded.endswith(b"="), encoded
assert base64.b64decode(encoded) == payload, "round-trip mismatch"

# URL-safe variant — same payload, no '+' or '/' in the output.
urlsafe = base64.urlsafe_b64encode(payload)
assert b"+" not in urlsafe and b"/" not in urlsafe, urlsafe
assert base64.urlsafe_b64decode(urlsafe) == payload, "urlsafe round-trip mismatch"
print("ascii_transport_round_trip OK")
