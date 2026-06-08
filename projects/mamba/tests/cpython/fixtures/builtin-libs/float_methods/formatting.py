# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""float __format__ presentation types and spec flags (CPython 3.12 oracle)."""

INF = float("inf")
NAN = float("nan")

# Empty spec mirrors str(); 'f' is fixed-point with 6 default digits.
assert format(0.0, "") == "0.0"
assert format(0.01, "") == "0.01"
assert format(0.0, "f") == "0.000000"
assert format(1.0, "f") == "1.000000"
assert format(-1.0, "f") == "-1.000000"

# Sign flags: space pads positives, '+' forces a sign, '%' scales by 100.
assert format(1.0, " f") == " 1.000000"
assert format(-1.0, " f") == "-1.000000"
assert format(1.0, "+f") == "+1.000000"
assert format(-1.0, "%") == "-100.000000%"

# Precision after '.' rounds and may switch to exponential ('g'-like '.4').
assert format(123.456, ".4") == "123.5"
assert format(1234.56, ".4") == "1.235e+03"
assert format(12345.6, ".4") == "1.235e+04"

# Leading '0' enables zero-padding / minimum-width handling (gh-35560).
assert format(123.34, "00f") == "123.340000"
assert format(123.34, "00e") == "1.233400e+02"
assert format(123.34, "00g") == "123.34"
assert format(-123.34, "00.10f") == "-123.3400000000"

# inf / nan render with case following the presentation letter.
assert format(NAN, "f") == "nan"
assert format(NAN, "F") == "NAN"
assert format(INF, "f") == "inf"
assert format(INF, "F") == "INF"

# Integer-only presentation types are invalid for float and raise ValueError.
for spec in "sbcdoxX":
    for value in (0.0, 1.0, -1.0, 1e100):
        try:
            format(value, spec)
            raise AssertionError("expected ValueError for spec %r" % spec)
        except ValueError:
            pass

print("formatting OK")
