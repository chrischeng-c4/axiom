# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "hex_roundtrip"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUID(hex=u.hex) reconstructs an equal UUID (32-char hex round-trip)"""
import uuid

u = uuid.uuid4()
assert uuid.UUID(hex=u.hex) == u, "UUID hex round-trip"
print("hex_roundtrip OK")
