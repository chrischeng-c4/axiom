# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: uuid4 uniqueness, uuid5 namespace-deterministic, fields, and
# hex/int/bytes/urn views over uuid4 outputs. (Reverse construction from
# hex=/bytes=/int= keyword args is intentionally NOT exercised — mamba's UUID
# constructor returns the zero UUID for those kwargs; tracked separately.)
import uuid

_ledger: list[int] = []

# uuid4 produces a canonical 8-4-4-4-12 dashed string
u4 = uuid.uuid4()
s = str(u4)
assert len(s) == 36, f"uuid4 string length 36, got {len(s)}"
_ledger.append(1)

assert s[8] == "-" and s[13] == "-" and s[18] == "-" and s[23] == "-", (
    "uuid4 string has dashes at positions 8/13/18/23"
)
_ledger.append(1)

# uuid4 version field is 4
assert u4.version == 4, f"uuid4 version is 4, got {u4.version}"
_ledger.append(1)

# hex view drops the dashes
assert len(u4.hex) == 32, f"uuid4 hex length 32, got {len(u4.hex)}"
_ledger.append(1)

assert "-" not in u4.hex, "uuid4 hex omits dashes"
_ledger.append(1)

# bytes view is 16 bytes
assert len(u4.bytes) == 16, f"uuid4 bytes length 16, got {len(u4.bytes)}"
_ledger.append(1)

# int view is a positive integer
assert isinstance(u4.int, int) and u4.int > 0, "uuid4 int is positive"
_ledger.append(1)

# urn view uses the RFC 4122 prefix
assert u4.urn == "urn:uuid:" + s, f"uuid4 urn is 'urn:uuid:<str>', got {u4.urn}"
_ledger.append(1)

# fields tuple has the documented 6-component shape
assert len(u4.fields) == 6, f"uuid4 fields tuple has 6 components, got {len(u4.fields)}"
_ledger.append(1)

# uuid4 produces unique values across repeated calls
batch = {str(uuid.uuid4()) for _ in range(50)}
assert len(batch) == 50, f"50 uuid4 calls produce 50 distinct values, got {len(batch)}"
_ledger.append(1)

# uuid1 version field is 1
u1 = uuid.uuid1()
assert u1.version == 1, f"uuid1 version is 1, got {u1.version}"
_ledger.append(1)

# uuid5 over the DNS namespace is deterministic
a = uuid.uuid5(uuid.NAMESPACE_DNS, "python.org")
b = uuid.uuid5(uuid.NAMESPACE_DNS, "python.org")
assert str(a) == str(b), "uuid5(NAMESPACE_DNS, 'python.org') is deterministic"
_ledger.append(1)

# uuid5 over the DNS namespace matches the RFC 4122 v5 NIST vector
assert str(a) == "886313e1-3b8a-5372-9b90-0c9aee199e5d", (
    "uuid5(NAMESPACE_DNS,'python.org') == RFC 4122 reference value"
)
_ledger.append(1)

# UUID parsed from a canonical string round-trips through str()
parsed = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert str(parsed) == "12345678-1234-5678-1234-567812345678", (
    "UUID(str) preserves canonical formatting"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_uuid {sum(_ledger)} asserts")
