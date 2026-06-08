# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""float.hex / float.fromhex round-trips and rejected inputs (CPython 3.12 oracle)."""

# hex() produces the canonical "0x1.<mantissa>p<exp>" form.
assert (1.5).hex() == "0x1.8000000000000p+0"
assert (1.0).hex() == "0x1.0000000000000p+0"
assert (0.0).hex() == "0x0.0p+0"
assert (-0.0).hex() == "-0x0.0p+0"

# fromhex is the exact inverse for finite values, including signs.
for v in (1.5, -1.5, 0.5, 256.0, 0.1, 3.141592653589793, 0.0, -0.0):
    assert float.fromhex(v.hex()) == v

# Digits after "0x" are interpreted as hexadecimal, not decimal.
assert float.fromhex("0x2.8p0") == 2.5
assert float.fromhex("2.5") == 2 + 5 / 16   # "2.5" == 2 + 5/16 in hex
import math
assert math.isinf(float.fromhex("inf"))
assert math.isnan(float.fromhex("nan"))

# Surrounding whitespace is tolerated.
assert float.fromhex("  0x1.8p0  ") == 1.5

# Malformed hex strings raise ValueError.
invalid = ["infi", "++inf", "0xnan", "", " ", "x1.0p0", "0xX1.0p0",
           "0x1 2.0p0", "0x1.0 p0", "0x1.0p 0", "0x1.0.p0", "0x.p0",
           "0x1,p0", "0x1pa", "not_hex"]
for s in invalid:
    try:
        float.fromhex(s)
        raise AssertionError("expected ValueError for %r" % s)
    except ValueError:
        pass

print("fromhex_hex OK")
