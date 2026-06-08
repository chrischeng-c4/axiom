# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "uuid5_is_deterministic"
# subject = "uuid.uuid5"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.uuid5: uuid5(NAMESPACE_DNS, 'example.com') is deterministic: two calls with the same name yield equal version-5 UUIDs"""
import uuid

a = uuid.uuid5(uuid.NAMESPACE_DNS, "example.com")
b = uuid.uuid5(uuid.NAMESPACE_DNS, "example.com")
assert isinstance(a, uuid.UUID), f"uuid5 type = {type(a)!r}"
assert a.version == 5, f"uuid5 version = {a.version!r}"
assert a == b, f"uuid5 not deterministic: {a} vs {b}"
print("uuid5_is_deterministic OK")
