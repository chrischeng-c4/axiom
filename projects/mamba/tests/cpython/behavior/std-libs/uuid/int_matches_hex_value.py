# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "int_matches_hex_value"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUID.int equals int(UUID.hex, 16) for a fixed UUID"""
import uuid

u = uuid.UUID("550e8400-e29b-41d4-a716-446655440000")
assert u.int == int(u.hex, 16), f"int {u.int!r} != int(hex, 16) {int(u.hex, 16)!r}"
print("int_matches_hex_value OK")
