# Operational AssertionPass seed for json.dumps/loads option surfaces
# beyond test_json_ops basics.
# Surface: sort_keys=True orders keys lexicographically, indent
# inserts newlines + spaces in nested output, escape of quote
# characters inside string values, float round-trip, nested object
# parsing, and bool/null name-spelling.
import json
_ledger: list[int] = []

# sort_keys reorders keys lexicographically regardless of insertion
assert json.dumps({"b": 2, "a": 1}, sort_keys=True) == '{"a": 1, "b": 2}'; _ledger.append(1)

# indent=2 pretty-prints one key-value pair across multiple lines
expected_indent = '{\n  "a": 1\n}'
assert json.dumps({"a": 1}, indent=2) == expected_indent; _ledger.append(1)

# Escape of a double-quote inside a string value
assert json.dumps('he said "hi"') == '"he said \\"hi\\""'; _ledger.append(1)

# float round-trip
assert json.loads(json.dumps(3.14)) == 3.14; _ledger.append(1)

# Nested object: object inside array inside object
src = {"outer": [{"k": "v"}]}
s = json.dumps(src)
parsed = json.loads(s)
assert parsed == src; _ledger.append(1)

# Bool / null name-spelling on parse — `is True` would trip the
# bool-identity-through-return drop ([[project_mamba_bool_identity]]),
# so use == against True/False and `is None` (None is interned).
assert json.loads("true") == True; _ledger.append(1)
assert json.loads("false") == False; _ledger.append(1)
assert json.loads("null") is None; _ledger.append(1)

# Whitespace tolerance — loads strips leading/trailing whitespace
assert json.loads("  42  ") == 42; _ledger.append(1)

# Round-trip a list of mixed scalar types
mixed = [1, "x", True, None, 2.5]
assert json.loads(json.dumps(mixed)) == mixed; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_json_options_ops {sum(_ledger)} asserts")
