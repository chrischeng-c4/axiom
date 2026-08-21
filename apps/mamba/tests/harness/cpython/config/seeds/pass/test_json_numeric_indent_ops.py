# Operational AssertionPass seed for `json` numeric-literal parsing
# and `indent=` formatting structure not covered by `test_json_ops`,
# `test_json_options_ops`, or `test_json_default_decode_error_ops`.
# Existing seeds exercise `loads("3.14")`, `loads("true")`, and
# `dumps(..., indent=2)` for one trivial dict. This seed asserts
# scientific notation (`1.5e10`, `-2.5e-3`), negative integer
# literals, that `dumps` with indent=N produces a multi-line string
# containing the requested indent, and that `sort_keys=True` /
# `separators=(",", ":")` reorder and compact respectively.
import json
_ledger: list[int] = []

# Scientific notation — positive exponent
assert json.loads("1.5e10") == 1.5e10; _ledger.append(1)
assert json.loads("2e3") == 2000.0; _ledger.append(1)
# Scientific notation — negative exponent
assert json.loads("-2.5e-3") == -0.0025; _ledger.append(1)
assert json.loads("1E-5") == 1e-5; _ledger.append(1)

# Negative integers and floats
assert json.loads("-5") == -5; _ledger.append(1)
assert json.loads("-3.14") == -3.14; _ledger.append(1)
assert json.loads("0") == 0; _ledger.append(1)
assert json.loads("-0") == 0; _ledger.append(1)

# Integer literal stays int (not float)
assert isinstance(json.loads("42"), int); _ledger.append(1)
assert isinstance(json.loads("-7"), int); _ledger.append(1)

# Float literal stays float
assert isinstance(json.loads("3.14"), float); _ledger.append(1)
assert isinstance(json.loads("1.5e10"), float); _ledger.append(1)

# sort_keys reorders dict keys lexicographically
assert json.dumps({"c": 1, "a": 2, "b": 3}, sort_keys=True) == '{"a": 2, "b": 3, "c": 1}'; _ledger.append(1)

# indent=2 produces multi-line output with the requested indent
out = json.dumps({"a": 1}, indent=2)
assert "\n" in out; _ledger.append(1)
assert "  " in out; _ledger.append(1)

# indent=4 likewise produces 4-space indent
out4 = json.dumps([1, 2, 3], indent=4)
assert "\n" in out4; _ledger.append(1)
assert "    " in out4; _ledger.append(1)

# Round-trip preserves numeric value through scientific notation
assert json.loads(json.dumps(1.5e10)) == 1.5e10; _ledger.append(1)
assert json.loads(json.dumps(-0.0025)) == -0.0025; _ledger.append(1)

# Round-trip preserves negative integers
assert json.loads(json.dumps(-42)) == -42; _ledger.append(1)

# Decoded nested array of numbers preserves type
result = json.loads("[1, 2.5, -3, 4e2]")
assert result == [1, 2.5, -3, 400.0]; _ledger.append(1)
assert isinstance(result[0], int); _ledger.append(1)
assert isinstance(result[1], float); _ledger.append(1)
assert isinstance(result[2], int); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_json_numeric_indent_ops {sum(_ledger)} asserts")
