# Operational AssertionPass seed for f-string format-spec
# mini-language surfaces beyond lang_fstring_expressions /
# lang_pep701_fstring.
# Surface: width (`{x:5d}`); zero-pad (`{x:05d}`); explicit sign
# (`{x:+d}` for positive and negative); thousands separator
# (`{x:,}`); float fixed-point with precision (`{x:.2f}`); float
# scientific notation (`{x:.2e}`); float percent (`{x:.0%}`); float
# width+precision (`{x:10.2f}`); explicit alignment with fill chars
# (`{x:*^10}`, `{x:->10}`); conversion flags (`!r` / `!s`); int
# alternate-form (`{x:#x}`).
_ledger: list[int] = []

# width and zero-pad on integers
assert f"{42:5d}" == "   42"; _ledger.append(1)
assert f"{42:05d}" == "00042"; _ledger.append(1)
# width 5 of a 5-char value is unchanged
assert f"{12345:5d}" == "12345"; _ledger.append(1)

# Explicit sign — positive gets +, negative still gets -
assert f"{42:+d}" == "+42"; _ledger.append(1)
assert f"{-42:+d}" == "-42"; _ledger.append(1)
assert f"{0:+d}" == "+0"; _ledger.append(1)

# Thousands separator on integers
assert f"{1234567:,}" == "1,234,567"; _ledger.append(1)
# Small numbers under the separator threshold are unchanged
assert f"{999:,}" == "999"; _ledger.append(1)

# Fixed-point float with precision
assert f"{3.14159:.2f}" == "3.14"; _ledger.append(1)
assert f"{3.14159:.4f}" == "3.1416"; _ledger.append(1)
# Precision 0 truncates the fractional part (rounds)
assert f"{2.71:.0f}" == "3"; _ledger.append(1)

# Width + precision on float
assert f"{3.14:10.2f}" == "      3.14"; _ledger.append(1)

# Scientific notation
assert f"{1000000.0:.2e}" == "1.00e+06"; _ledger.append(1)

# Percent
assert f"{0.5:.0%}" == "50%"; _ledger.append(1)
assert f"{0.123:.1%}" == "12.3%"; _ledger.append(1)

# Alignment without an explicit fill (default = space) — also
# covered in lang_fstring_expressions for `^`, here we focus on
# explicit fill chars
# Center with `*` fill
assert f"{'x':*^10}" == "****x*****"; _ledger.append(1)
# Right-align with `-` fill
assert f"{'x':->10}" == "---------x"; _ledger.append(1)
# Left-align with `.` fill
assert f"{'x':.<10}" == "x........."; _ledger.append(1)

# Conversion flags: !r calls repr(), !s calls str()
assert f"{'hello'!r}" == "'hello'"; _ledger.append(1)
assert f"{'hello'!s}" == "hello"; _ledger.append(1)
# !r on int and list
assert f"{42!r}" == "42"; _ledger.append(1)
assert f"{[1, 2]!r}" == "[1, 2]"; _ledger.append(1)

# Integer bases without alternate form
assert f"{255:x}" == "ff"; _ledger.append(1)
assert f"{8:o}" == "10"; _ledger.append(1)
assert f"{10:b}" == "1010"; _ledger.append(1)

# Integer bases WITH alternate form (`#` prefix)
assert f"{255:#x}" == "0xff"; _ledger.append(1)
assert f"{8:#o}" == "0o10"; _ledger.append(1)
assert f"{10:#b}" == "0b1010"; _ledger.append(1)
# Uppercase hex
assert f"{255:X}" == "FF"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_fstring_format_spec {sum(_ledger)} asserts")
