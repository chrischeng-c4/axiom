# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "equal_uuids_hash_and_dedup"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: two UUIDs built from the same string hash equal and collapse to one element in a set"""
import uuid

canon = uuid.UUID("12345678-1234-5678-1234-567812345678")
dup = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert hash(dup) == hash(canon), "equal UUIDs hash equal"
assert len({canon, dup}) == 1, "set dedup"
print("equal_uuids_hash_and_dedup OK")
