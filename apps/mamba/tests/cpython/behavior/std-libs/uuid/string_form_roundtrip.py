# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "string_form_roundtrip"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUID(str(u)) reconstructs an equal UUID (canonical-string round-trip)"""
import uuid

u = uuid.uuid4()
assert uuid.UUID(str(u)) == u, "UUID str round-trip"
print("string_form_roundtrip OK")
