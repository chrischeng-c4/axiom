# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/complex: format() / __format__ spec handling (3.12 oracle)."""

INF = float("inf")
NAN = float("nan")

# Empty / fill-only specs match str(); parentheses appear only when the
# value is padded as a whole (no precision/type given).
assert format(1 + 3j, "") == str(1 + 3j)
assert format(3j, "") == "3j"

# General format 'g': no parentheses, sign control via '+', ' ', '-'.
assert format(1 + 3j, "g") == "1+3j"
assert format(3j, "g") == "0+3j"
assert format(1.5 + 3.5j, "+g") == "+1.5+3.5j"
assert format(1.5 - 3.5j, "+g") == "+1.5-3.5j"
assert format(1.5 + 3.5j, " g") == " 1.5+3.5j"
assert format(-1.5 + 3.5j, " g") == "-1.5+3.5j"

# Fixed / exponential presentation types and precision.
assert format(-1.5 - 3.5j, "f") == "-1.500000-3.500000j"
assert format(-1.5 - 3.5j, "F") == "-1.500000-3.500000j"
assert format(-1.5 - 3.5j, ".2e") == "-1.50e+00-3.50e+00j"
assert format(-1.5 - 3.5j, ".2E") == "-1.50E+00-3.50E+00j"

# Width, fill and alignment. Without a type, the whole value is wrapped
# in parentheses; with a type, the two parts are aligned bare.
assert format(1.5 + 3j, "<20g") == "1.5+3j              "
assert format(1.5 + 3j, "*<20g") == "1.5+3j**************"
assert format(1.5 + 3j, "^20g") == "       1.5+3j       "
assert format(1.5 + 3j, "<20") == "(1.5+3j)            "
assert format(1.5 + 3j, "^20") == "      (1.5+3j)      "

# Thousands separators and alternate ('#') form.
assert format(1.5e21 + 3000j, ",.2f") == "1,500,000,000,000,000,000,000.00+3,000.00j"
assert format(1 + 1j, "#.0f") == "1.+1.j"
assert format(1.1 + 1.1j, "#g") == "1.10000+1.10000j"

# str.format() routes through __format__ too.
assert "*{0:.3f}*".format(3.14159 + 2.71828j) == "*3.142+2.718j*"

# Non-finite parts format per the case of the type letter.
assert format(complex(NAN, NAN), "f") == "nan+nanj"
assert format(complex(NAN, NAN), "F") == "NAN+NANj"
assert format(complex(INF, -1), "f") == "inf-1.000000j"
assert format(complex(INF, INF), "F") == "INF+INFj"

# Zero-padding ('0'), '=' alignment, and integer presentation types are
# all rejected for complex with ValueError.
for spec in ("010f", "=20", "b", "d", "o", "x", "X"):
    try:
        format(1.5 + 0.5j, spec)
        raise AssertionError("expected ValueError for spec %r" % spec)
    except ValueError:
        pass

print("format OK")
