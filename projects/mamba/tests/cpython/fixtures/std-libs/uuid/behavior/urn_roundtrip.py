# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "urn_roundtrip"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUID(u.urn) reconstructs an equal UUID (urn:uuid: round-trip through the constructor)"""
import uuid

u = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert uuid.UUID(u.urn) == u, "urn round-trip"
print("urn_roundtrip OK")
