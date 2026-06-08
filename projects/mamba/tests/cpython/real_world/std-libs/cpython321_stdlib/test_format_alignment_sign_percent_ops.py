# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_format_alignment_sign_percent_ops"
# subject = "cpython321.test_format_alignment_sign_percent_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_format_alignment_sign_percent_ops.py"
# status = "filled"
# ///
"""cpython321.test_format_alignment_sign_percent_ops: execute CPython 3.12 seed test_format_alignment_sign_percent_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `format(value, spec)` surface
# not covered by `test_format_builtin_ops`. That seed asserts base
# conversion (x, X, o, b), float .Nf, thousands ",", alt-form #x,
# zero-padded width "06". This seed asserts:
#   * Alignment: ">", "<", "^" on int and str, with and without
#     custom fill characters;
#   * Percent format: "%", ".0%", ".2%" for fractional values;
#   * Sign: "+" forces leading sign for positives, " " uses a space
#     for positives, both pass negatives through;
#   * Scientific: "e", ".Ne" emits `mantissa e±exp` with precision.
_ledger: list[int] = []

# Alignment on ints (default right for numbers)
assert format(42, "5") == "   42"; _ledger.append(1)
assert format(42, ">5") == "   42"; _ledger.append(1)
assert format(42, "<5") == "42   "; _ledger.append(1)
assert format(42, "^5") == " 42  "; _ledger.append(1)

# Alignment on strings (default left for strings)
assert format("hi", "<5") == "hi   "; _ledger.append(1)
assert format("hi", ">5") == "   hi"; _ledger.append(1)
assert format("hi", "^5") == " hi  "; _ledger.append(1)

# Custom fill character ("fill"+"align"+width)
assert format("hi", "*^6") == "**hi**"; _ledger.append(1)
assert format("hi", "*<6") == "hi****"; _ledger.append(1)
assert format("hi", "*>6") == "****hi"; _ledger.append(1)
assert format("hi", "-^10") == "----hi----"; _ledger.append(1)
assert format("hi", "0^6") == "00hi00"; _ledger.append(1)

# Percent format on floats
# Default precision is 6 decimals → "50.000000%"
assert format(0.5, "%") == "50.000000%"; _ledger.append(1)
# Explicit zero-precision drops the decimal point
assert format(0.5, ".0%") == "50%"; _ledger.append(1)
# Explicit 2-decimal precision
assert format(0.5, ".2%") == "50.00%"; _ledger.append(1)
# 100% boundary
assert format(1.0, ".1%") == "100.0%"; _ledger.append(1)
# Quarter
assert format(0.25, ".0%") == "25%"; _ledger.append(1)

# Sign — "+" forces "+" for positives
assert format(5, "+") == "+5"; _ledger.append(1)
assert format(-5, "+") == "-5"; _ledger.append(1)
assert format(0, "+") == "+0"; _ledger.append(1)
# " " — space for positives (alignment with negatives)
assert format(5, " ") == " 5"; _ledger.append(1)
assert format(-5, " ") == "-5"; _ledger.append(1)

# Scientific — "e" emits mantissa+e±exp
assert format(1234.5, "e") == "1.234500e+03"; _ledger.append(1)
assert format(1234.5, ".2e") == "1.23e+03"; _ledger.append(1)
assert format(0.000123, ".3e") == "1.230e-04"; _ledger.append(1)

# Identity on empty spec
assert format(42, "") == "42"; _ledger.append(1)
assert format("hi", "") == "hi"; _ledger.append(1)
assert format(3.14, "") == "3.14"; _ledger.append(1)

# Return-type invariants — always a str
assert isinstance(format(42, ">5"), str); _ledger.append(1)
assert isinstance(format(0.5, "%"), str); _ledger.append(1)
assert isinstance(format("hi", "*^6"), str); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_format_alignment_sign_percent_ops {sum(_ledger)} asserts")
