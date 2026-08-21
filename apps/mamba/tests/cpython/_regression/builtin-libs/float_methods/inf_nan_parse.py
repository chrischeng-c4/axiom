# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""float() parsing of inf / nan spellings and their repr (CPython 3.12 oracle)."""

import math

# inf accepts long/short forms, optional sign, any case.
for s in ("inf", "+inf", "infinity", "+infinity", "INF", "+Inf", "iNfInItY"):
    assert math.isinf(float(s))
    assert float(s) > 0
    assert repr(float(s)) == "inf"
    assert str(float(s)) == "inf"

for s in ("-inf", "-infinity", "-iNF", "-INFINITY"):
    assert math.isinf(float(s))
    assert float(s) < 0
    assert repr(float(s)) == "-inf"

# nan likewise; nan is never equal to itself, sign is dropped in repr.
for s in ("nan", "+nan", "-nan", "NAN", "+NAn", "-NaN"):
    v = float(s)
    assert math.isnan(v)
    assert v != v
    assert repr(v) == "nan"
    assert str(v) == "nan"

# Overflowing arithmetic produces inf / nan with the right repr.
big = 1e300 * 1e300
assert repr(big) == "inf"
assert repr(-big) == "-inf"
assert repr(big * 0) == "nan"

# Truncated / doubled-sign / trailing-garbage spellings are rejected.
bad = ["info", "+info", "-in", "infinit", "infinitys", "++Inf", "-+inf",
       "nana", "na", "++nan", "+-NaN", "--nAn"]
for s in bad:
    try:
        float(s)
        raise AssertionError("expected ValueError for %r" % s)
    except ValueError:
        pass

print("inf_nan_parse OK")
