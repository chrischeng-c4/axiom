# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "bytes_roundtrip"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUID(bytes=u.bytes) reconstructs an equal UUID (16-byte round-trip)"""
import uuid

u = uuid.uuid4()
assert uuid.UUID(bytes=u.bytes) == u, "UUID bytes round-trip"
print("bytes_roundtrip OK")
