# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "real_world"
# case = "stable_record_id_from_natural_key"
# subject = "uuid"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid: a deduplication pipeline derives stable record ids by uuid5-hashing each natural key under a fixed namespace, so identical inputs map to one id (dedup) while distinct inputs stay distinct, and ids round-trip through their canonical string form"""
import uuid

# Fixed application namespace (any UUID works as a seed).
NAMESPACE = uuid.UUID("6ba7b810-9dad-11d1-80b4-00c04fd430c8")

# An inbound stream with a duplicate natural key ("alice@example.com" twice).
incoming = [
    "alice@example.com",
    "bob@example.com",
    "carol@example.com",
    "alice@example.com",
]

# Derive a stable record id per natural key.
record_ids = {key: uuid.uuid5(NAMESPACE, key) for key in incoming}

# Dedup: 4 inputs, 3 distinct keys -> 3 distinct ids.
assert len(record_ids) == 3, f"distinct ids = {len(record_ids)!r}"

# Stability: re-deriving an id for the same key yields the identical UUID.
assert uuid.uuid5(NAMESPACE, "alice@example.com") == record_ids["alice@example.com"], \
    "uuid5 not stable for the same key"

# Distinct keys produce distinct ids.
assert record_ids["bob@example.com"] != record_ids["carol@example.com"], \
    "distinct keys collided"

# Every id round-trips through its canonical string form (e.g. persisted to a DB).
for key, rid in record_ids.items():
    assert uuid.UUID(str(rid)) == rid, f"round-trip failed for {key}"
    assert rid.version == 5, f"version = {rid.version!r}"

print("stable_record_id_from_natural_key OK:", len(record_ids), "distinct")
