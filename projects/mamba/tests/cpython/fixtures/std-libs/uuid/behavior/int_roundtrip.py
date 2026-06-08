# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "int_roundtrip"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUID(int=u.int) reconstructs an equal UUID (128-bit int round-trip)"""
import uuid

u = uuid.uuid4()
assert uuid.UUID(int=u.int) == u, "UUID int round-trip"
print("int_roundtrip OK")
