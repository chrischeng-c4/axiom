# Operational AssertionPass seed for the `format()` builtin / f-string
# format-spec mini-language: alt-form `#` prefix across all integer
# bases, scientific notation `e`/`E` with explicit precision, percent
# `%` formatting, sign options (`+` / `-` / space), general-format `g`,
# PEP 515 underscore `_` grouping on decimal, and bool format
# passthrough. Existing format seeds (test_format_ops,
# test_format_builtin_ops, test_format_alignment_sign_percent_ops,
# test_str_format_advanced_ops) cover the basic-spec / alignment /
# str-`.format(...)` surface, but skip the `#`-alt-form on bases
# other than hex, the scientific/percent/general-format triplet, the
# sign-options triplet, the underscore-grouping (PEP 515) on decimal,
# and bool-format passthrough. mamba 0.3.60 supports every probed
# form below.
#
# Surface:
#   • `#` alt-form prefix for all integer bases (`#x` / `#X` / `#b` /
#     `#o`);
#   • scientific notation `e`/`E` (default precision and `.Ne`/`.NE`);
#   • percent `%` for proportions and `.N%` for fixed-precision %;
#   • sign options: `+`, ` ` (space-for-positive), `-`;
#   • general format `g` (default + explicit precision);
#   • PEP 515 underscore `_` grouping for decimal (`_` / `_d`);
#   • bool format passthrough (`True` / `False` text).
_ledger: list[int] = []

# Alt-form `#` prefix — hex / HEX / bin / oct
assert format(255, "#x") == "0xff"; _ledger.append(1)
assert format(255, "#X") == "0XFF"; _ledger.append(1)
assert format(255, "#b") == "0b11111111"; _ledger.append(1)
assert format(255, "#o") == "0o377"; _ledger.append(1)

# Alt-form `#` via f-string for the same bases
assert f"{255:#x}" == "0xff"; _ledger.append(1)
assert f"{255:#X}" == "0XFF"; _ledger.append(1)
assert f"{255:#b}" == "0b11111111"; _ledger.append(1)
assert f"{255:#o}" == "0o377"; _ledger.append(1)

# Scientific notation `e` / `E` — default 6-digit precision
assert format(1.5, "e") == "1.500000e+00"; _ledger.append(1)
assert format(1.5, "E") == "1.500000E+00"; _ledger.append(1)
assert format(123456789, "e") == "1.234568e+08"; _ledger.append(1)

# Scientific with explicit `.N` precision
assert format(0.0001, ".2e") == "1.00e-04"; _ledger.append(1)
assert format(1234.567, ".3e") == "1.235e+03"; _ledger.append(1)

# Percent `%` formatting — value × 100, then suffix `%`
assert format(0.5, "%") == "50.000000%"; _ledger.append(1)
assert format(0.123, ".1%") == "12.3%"; _ledger.append(1)
assert format(1.0, ".0%") == "100%"; _ledger.append(1)

# Sign options — `+` always shows, ` ` shows space for positive, `-` default
assert format(5, "+") == "+5"; _ledger.append(1)
assert format(-5, "+") == "-5"; _ledger.append(1)
assert format(5, " ") == " 5"; _ledger.append(1)
assert format(-5, " ") == "-5"; _ledger.append(1)
assert format(5, "-") == "5"; _ledger.append(1)
assert format(-5, "-") == "-5"; _ledger.append(1)

# Sign + width combo via f-string
assert f"{5:+05}" == "+0005"; _ledger.append(1)
assert f"{-5:+05}" == "-0005"; _ledger.append(1)

# General `g` — chooses fixed or scientific based on magnitude
assert format(1.5, "g") == "1.5"; _ledger.append(1)
assert format(1.5, ".10g") == "1.5"; _ledger.append(1)
assert format(0.0001, "g") == "0.0001"; _ledger.append(1)

# PEP 515 — underscore `_` grouping for decimal
assert format(1000000, "_") == "1_000_000"; _ledger.append(1)
assert format(1234567890, "_d") == "1_234_567_890"; _ledger.append(1)

# Underscore grouping via f-string
assert f"{1000000:_}" == "1_000_000"; _ledger.append(1)
assert f"{1234567:_d}" == "1_234_567"; _ledger.append(1)

# Bool format passthrough — empty spec yields "True"/"False" text
assert format(True, "") == "True"; _ledger.append(1)
assert format(False, "") == "False"; _ledger.append(1)

# Zero-padded with sign
assert format(5, "+06") == "+00005"; _ledger.append(1)

# Combined — sign + thousands + precision
assert format(1234567.89, "+,.2f") == "+1,234,567.89"; _ledger.append(1)

# Combined — alt-form + width + zero pad on hex
assert format(255, "#010x") == "0x000000ff"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_format_spec_minilang_ops {sum(_ledger)} asserts")
