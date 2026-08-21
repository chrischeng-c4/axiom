# Operational AssertionPass seed for the uuid module's
# deterministic surfaces.
# Surface: uuid4 yields a UUID whose .version is 4; uuid1 yields a
# UUID whose .version is 1; UUID(string) parses a canonical
# 36-character UUID string; .hex strips the dashes giving a
# 32-character lowercase string; str(uuid) renders the 36-char
# canonical form; uuid5 with NAMESPACE_DNS + "python.org" is a
# well-known fixed deterministic UUID (RFC 4122 example); .version
# survives an explicit-string ctor.
import uuid
_ledger: list[int] = []

# uuid4() returns a UUID whose version field is 4
u4 = uuid.uuid4()
assert u4.version == 4; _ledger.append(1)
# Its .hex is 32 lowercase hex chars (no dashes)
assert len(u4.hex) == 32; _ledger.append(1)
# Its str() is 36 chars (32 hex + 4 dashes)
assert len(str(u4)) == 36; _ledger.append(1)
# str() contains four dashes at the canonical positions
s4 = str(u4)
assert s4[8] == "-"; _ledger.append(1)
assert s4[13] == "-"; _ledger.append(1)
assert s4[18] == "-"; _ledger.append(1)
assert s4[23] == "-"; _ledger.append(1)

# uuid1() returns a UUID whose version field is 1
u1 = uuid.uuid1()
assert u1.version == 1; _ledger.append(1)
assert len(u1.hex) == 32; _ledger.append(1)

# UUID(string) parses a canonical 36-char UUID string
u_explicit = uuid.UUID("12345678-1234-5678-9234-567812345678")
assert str(u_explicit) == "12345678-1234-5678-9234-567812345678"; _ledger.append(1)
# The explicit value's hex is the dash-stripped form (lowercase)
assert u_explicit.hex == "12345678123456789234567812345678"; _ledger.append(1)
# The version digit (first nibble of the third group) is 5 -> version 5
assert u_explicit.version == 5; _ledger.append(1)

# uuid5 with NAMESPACE_DNS + "python.org" is a well-known RFC 4122
# example: 886313e1-3b8a-5372-9b90-0c9aee199e5d
u5 = uuid.uuid5(uuid.NAMESPACE_DNS, "python.org")
assert str(u5) == "886313e1-3b8a-5372-9b90-0c9aee199e5d"; _ledger.append(1)
assert u5.version == 5; _ledger.append(1)

# uuid5 is deterministic — two calls with the same namespace + name
# produce the same UUID
u5_again = uuid.uuid5(uuid.NAMESPACE_DNS, "python.org")
assert str(u5_again) == str(u5); _ledger.append(1)

# uuid5 with a different name produces a different UUID
u5_other = uuid.uuid5(uuid.NAMESPACE_DNS, "example.com")
assert str(u5_other) != str(u5); _ledger.append(1)
# But its version is still 5
assert u5_other.version == 5; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_uuid_ops {sum(_ledger)} asserts")
