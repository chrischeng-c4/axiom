# Operational AssertionPass seed for the `format(value, spec)`
# builtin and equivalent f-string format-spec surface.
# Surface: integer base conversion (x, X, o, b), float fixed-width
# (.Nf), thousands separator (,), alt-form prefix (#x), precision in
# f-strings. Complements test_format_ops.py which covers the str
# `.format(...)` method.
_ledger: list[int] = []
# Integer base conversion
assert format(255, "x") == "ff"; _ledger.append(1)
assert format(255, "X") == "FF"; _ledger.append(1)
assert format(255, "o") == "377"; _ledger.append(1)
assert format(255, "b") == "11111111"; _ledger.append(1)
# Float fixed precision
assert format(3.14159, ".2f") == "3.14"; _ledger.append(1)
# Thousands grouping
assert format(1000000, ",") == "1,000,000"; _ledger.append(1)
# Alt-form prefix in an f-string
assert f"{255:#x}" == "0xff"; _ledger.append(1)
# Float precision via f-string
assert f"{3.14159:.3f}" == "3.142"; _ledger.append(1)
# Zero-padded width
assert format(42, "06") == "000042"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_format_builtin_ops {sum(_ledger)} asserts")
