# Operational AssertionPass seed for advanced str.format / format-spec
# surfaces beyond positional/keyword + simple alignment.
# Surface: indexed positional ({0} {1}), format_map, literal {{ }}
# escape, left/center alignment, alt-form thousands separator,
# scientific notation, percent format, sign-aware zero pad.
_ledger: list[int] = []

# Indexed positional — reorders args and reuses them
assert "{1} {0}".format("a", "b") == "b a"; _ledger.append(1)
assert "{0}{1}{0}".format("a", "b") == "aba"; _ledger.append(1)

# format_map takes a mapping and binds by name
assert "{a}+{b}".format_map({"a": 1, "b": 2}) == "1+2"; _ledger.append(1)

# Literal braces — `{{` and `}}` escape to a single brace each
assert "{{}}".format() == "{}"; _ledger.append(1)
assert "before {{ {x} }} after".format(x=42) == "before { 42 } after"; _ledger.append(1)

# Left/center alignment specifiers
assert "{:<5}".format("x") == "x    "; _ledger.append(1)
assert "{:^5}".format("x") == "  x  "; _ledger.append(1)

# Thousands separator on int
assert "{:,}".format(1234567) == "1,234,567"; _ledger.append(1)
# Thousands separator on float
assert "{:,.2f}".format(1234.5) == "1,234.50"; _ledger.append(1)

# Scientific notation
assert "{:.3e}".format(12345.6789) == "1.235e+04"; _ledger.append(1)
# Percent format
assert "{:.1%}".format(0.5) == "50.0%"; _ledger.append(1)

# Sign-aware zero-pad for ints
assert "{:05d}".format(42) == "00042"; _ledger.append(1)
# Explicit sign for positive numbers
assert "{:+d}".format(42) == "+42"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_str_format_advanced_ops {sum(_ledger)} asserts")
