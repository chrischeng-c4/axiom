# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "uuid4_is_version_4_random"
# subject = "uuid.uuid4"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.uuid4: uuid4() returns a version-4 UUID with the RFC 4122 variant, and two successive calls differ (random)"""
import uuid

a = uuid.uuid4()
b = uuid.uuid4()
assert isinstance(a, uuid.UUID), f"uuid4 type = {type(a)!r}"
assert a.version == 4, f"uuid4 version = {a.version!r}"
assert a.variant == uuid.RFC_4122, f"uuid4 variant = {a.variant!r}"
assert a != b, f"uuid4 not unique: {a} vs {b}"
print("uuid4_is_version_4_random OK")
