# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "uuid1_default_is_version_1"
# subject = "uuid.uuid1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.uuid1: uuid1() with no args is still a version-1 RFC 4122 UUID and accepts the live getnode() value without raising"""
import uuid

node = uuid.getnode()
assert 0 < node < (1 << 48), f"node out of range: {node!r}"
try:
    uuid.uuid1(node=node)
except ValueError as e:
    raise AssertionError(f"uuid1 rejected a valid node: {e}")

d = uuid.uuid1()
assert d.version == 1, f"default version = {d.version!r}"
assert d.variant == uuid.RFC_4122, f"default variant = {d.variant!r}"
print("uuid1_default_is_version_1 OK")
