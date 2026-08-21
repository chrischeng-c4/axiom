# Operational AssertionPass seed for f-string format spec corners
# not covered by `lang_fstring_format_spec`, `lang_fstring_padding_
# conversion`, `lang_fstring_expressions`, `lang_format_specs`,
# `test_format_alignment_sign_percent_ops`, or
# `test_str_format_advanced_ops`. This seed asserts: PEP 515
# underscore thousands separator on plain integers; binary
# zero-padding (`{x:08b}`); uppercase hex alt-form (`{x:#X}`);
# scientific uppercase exponent (`{x:.3E}`); space-sign for
# positive ints; percent with explicit precision; combined
# width-align-comma-precision; zero-padding on floats; the default
# alignment direction (str left, int/float right); generic-g for
# floats; octal zero-padding.
_ledger: list[int] = []

# PEP 515 underscore separator — plain int
assert f"{1000000:_}" == "1_000_000"; _ledger.append(1)
assert f"{1234567:_}" == "1_234_567"; _ledger.append(1)
assert f"{0:_}" == "0"; _ledger.append(1)
assert f"{-1234567:_}" == "-1_234_567"; _ledger.append(1)
assert f"{999:_}" == "999"; _ledger.append(1)

# Binary zero-padding
assert f"{5:08b}" == "00000101"; _ledger.append(1)
assert f"{255:08b}" == "11111111"; _ledger.append(1)
assert f"{0:08b}" == "00000000"; _ledger.append(1)
assert f"{1:04b}" == "0001"; _ledger.append(1)

# Hex zero-padding
assert f"{255:08x}" == "000000ff"; _ledger.append(1)
assert f"{255:04X}" == "00FF"; _ledger.append(1)
assert f"{0:04x}" == "0000"; _ledger.append(1)

# Octal zero-padding
assert f"{8:04o}" == "0010"; _ledger.append(1)
assert f"{0:03o}" == "000"; _ledger.append(1)

# Hex alt-form lowercase / uppercase
assert f"{255:#x}" == "0xff"; _ledger.append(1)
assert f"{255:#X}" == "0XFF"; _ledger.append(1)
assert f"{15:#x}" == "0xf"; _ledger.append(1)
assert f"{15:#X}" == "0XF"; _ledger.append(1)

# Octal alt-form
assert f"{8:#o}" == "0o10"; _ledger.append(1)

# Scientific notation lowercase / uppercase exponent
assert f"{12345.6789:.3e}" == "1.235e+04"; _ledger.append(1)
assert f"{12345.6789:.3E}" == "1.235E+04"; _ledger.append(1)
assert f"{0.000123:.2e}" == "1.23e-04"; _ledger.append(1)
assert f"{1.5:.0e}" == "2e+00"; _ledger.append(1)

# Space-sign for positive ints (sign='' = leading space)
assert f"{5: d}" == " 5"; _ledger.append(1)
assert f"{-5: d}" == "-5"; _ledger.append(1)
assert f"{0: d}" == " 0"; _ledger.append(1)

# Plus-sign sign character
assert f"{5:+d}" == "+5"; _ledger.append(1)
assert f"{-5:+d}" == "-5"; _ledger.append(1)

# Percent with various precisions
assert f"{0.5:%}" == "50.000000%"; _ledger.append(1)
assert f"{0.5:.0%}" == "50%"; _ledger.append(1)
assert f"{0.5:.1%}" == "50.0%"; _ledger.append(1)
assert f"{0.123456:.2%}" == "12.35%"; _ledger.append(1)
assert f"{1.0:.0%}" == "100%"; _ledger.append(1)
assert f"{0.0:.0%}" == "0%"; _ledger.append(1)

# Comma separator combined with width + align + precision
assert f"{1234567:,}" == "1,234,567"; _ledger.append(1)
assert f"{1234.5678:,.2f}" == "1,234.57"; _ledger.append(1)
assert f"{1234.5:>15,.2f}" == "       1,234.50"; _ledger.append(1)
assert f"{1234.5:<15,.2f}" == "1,234.50       "; _ledger.append(1)
assert f"{1234.5:^15,.2f}" == "   1,234.50    "; _ledger.append(1)

# Zero-padding on floats
assert f"{3.14:08.2f}" == "00003.14"; _ledger.append(1)
assert f"{42:05d}" == "00042"; _ledger.append(1)
assert f"{-42:05d}" == "-0042"; _ledger.append(1)

# Generic-g for floats
assert f"{1234.5678:g}" == "1234.57"; _ledger.append(1)
assert f"{0.000123:.2g}" == "0.00012"; _ledger.append(1)

# Default alignment: str defaults left, int/float defaults right
assert f"{'x':5}" == "x    "; _ledger.append(1)
assert f"{'ab':6}" == "ab    "; _ledger.append(1)
assert f"{5:5}" == "    5"; _ledger.append(1)
assert f"{42:6}" == "    42"; _ledger.append(1)
assert f"{3.14:8}" == "    3.14"; _ledger.append(1)

# Width-fill custom character (PEP 3101)
assert f"{42:*>6}" == "****42"; _ledger.append(1)
assert f"{42:*<6}" == "42****"; _ledger.append(1)
assert f"{42:*^6}" == "**42**"; _ledger.append(1)
assert f"{'x':-^9}" == "----x----"; _ledger.append(1)

# Float fixed precision corners
assert f"{3.14159:.2f}" == "3.14"; _ledger.append(1)
assert f"{3.14159:.0f}" == "3"; _ledger.append(1)
assert f"{0.5:.4f}" == "0.5000"; _ledger.append(1)
assert f"{0.0:.2f}" == "0.00"; _ledger.append(1)

# Combined width + sign on int
assert f"{42:+5d}" == "  +42"; _ledger.append(1)
assert f"{-42:+5d}" == "  -42"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_fstring_format_spec_advanced_ops {sum(_ledger)} asserts")
