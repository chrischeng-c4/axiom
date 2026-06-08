# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""int() base-parameter parsing: base=0 prefix detection, base limits,
indexable bases, and same-value round-trip across every supported base.
"""

# base=0 auto-detects the literal prefix.
assert int("0o123", 0) == 83
assert int("0x123", 0) == 291
assert int("0b100", 0) == 4
assert int("000", 0) == 0
# Mixed case + surrounding whitespace still detected under base=0.
assert int(" 0O123   ", 0) == 83
assert int(" 0X123  ", 0) == 291
assert int(" 0B100 ", 0) == 4
print("base0_prefix: ok")

# Under base=0 a non-zero number may not have a leading zero.
try:
    int("010", 0)
    print("leading_zero: no_raise")
except ValueError:
    print("leading_zero: ValueError")

# An explicit base must match the prefix and the digits.
assert int("0x123", 16) == 291
assert int("0o123", 8) == 83
assert int("0b100", 2) == 4
for bad in ("0b2", "0o8", "0xg"):
    try:
        int(bad, int(bad[1] == "b") + 2 if bad[1] == "b" else 0)
    except ValueError:
        pass

# Same numeric value parses identically in every base from 2 to 36.
TABLE = {
    2: "100000000000000000000000000000000",
    8: "40000000000", 10: "4294967296", 16: "100000000", 36: "1z141z4",
}
for base, text in TABLE.items():
    assert int(text, base) == 4294967296, base
print("base_roundtrip: ok")

# Base must be 0 or 2..36; anything else is a ValueError.
assert int("0", 5) == 0
for base in (1, 37, -909):
    try:
        int("0", base)
        print("base_limit: no_raise", base)
        break
    except ValueError:
        pass
else:
    print("base_limit: ValueError")

# A non-integer base is a TypeError, even if it is a whole float.
for bad_base in (5.5, 5.0):
    try:
        int("0", bad_base)
        print("bad_base_type: no_raise")
        break
    except TypeError:
        pass
else:
    print("bad_base_type: TypeError")

# The base may be supplied through __index__.
class Indexable:
    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value


assert int("101", base=Indexable(2)) == 5
assert int("101", base=Indexable(10)) == 101
assert int("ff", Indexable(16)) == 255
print("indexable_base: ok")

print("int_base_parsing OK")
