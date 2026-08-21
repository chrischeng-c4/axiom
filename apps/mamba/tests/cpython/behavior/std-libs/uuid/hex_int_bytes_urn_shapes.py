# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "hex_int_bytes_urn_shapes"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: a v4 UUID exposes .hex (32 chars), .int (positive int), .bytes (16 bytes), and .urn ('urn:uuid:' prefix)"""
import uuid

u = uuid.uuid4()
assert isinstance(u.hex, str) and len(u.hex) == 32, f"hex = {u.hex!r}"
assert isinstance(u.int, int) and u.int > 0, f"int = {u.int!r}"
assert isinstance(u.bytes, bytes) and len(u.bytes) == 16, f"bytes len = {len(u.bytes)!r}"
assert u.urn.startswith("urn:uuid:"), f"urn = {u.urn!r}"
print("hex_int_bytes_urn_shapes OK")
