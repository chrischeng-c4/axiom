# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "safe_uuid_enum_members"
# subject = "uuid.SafeUUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.SafeUUID: SafeUUID is an Enum with members safe=0, unsafe=-1, unknown=None; value lookup and identity hold, and a plain UUID reports is_safe is SafeUUID.unknown"""
import enum
import uuid

assert issubclass(uuid.SafeUUID, enum.Enum), "SafeUUID is not an Enum"
assert uuid.SafeUUID.safe.value == 0, f"safe = {uuid.SafeUUID.safe.value!r}"
assert uuid.SafeUUID.unsafe.value == -1, f"unsafe = {uuid.SafeUUID.unsafe.value!r}"
assert uuid.SafeUUID.unknown.value is None, f"unknown = {uuid.SafeUUID.unknown.value!r}"
assert uuid.SafeUUID.safe is not uuid.SafeUUID.unsafe, "members not distinct"
assert uuid.SafeUUID(0) is uuid.SafeUUID.safe, "lookup by value failed"

plain = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert plain.is_safe is uuid.SafeUUID.unknown, f"plain.is_safe = {plain.is_safe!r}"
print("safe_uuid_enum_members OK")
