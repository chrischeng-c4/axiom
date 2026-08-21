# Operational AssertionPass seed for `json` surfaces beyond the
# functional dumps/loads/sort_keys/separators/indent territory
# already covered by test_json_ops, test_json_separators_nested_ops,
# test_json_numeric_indent_ops, and test_json_options_ops. This
# seed asserts: the class-based `json.JSONEncoder()` / `json.JSONDecoder()`
# round-trip API (`encoder.encode(obj)` == `json.dumps(obj)` and
# `decoder.decode(text)` == `json.loads(text)`); JSON's escape
# escape vocabulary in `dumps` output (`\"`, `\\`); decoding of
# the JSON-spec escapes (`\n`, `\t`, `\\`, `\"`) and `\u00xx`
# unicode escape sequences;
# roundtrip preservation of None / True / False / int / float
# through `loads(dumps(...))`; nested structures roundtrip
# identically; `loads` accepts whitespace around values; primitive
# scalars (null/true/false/int/float/str) decode at the top level.
import json
_ledger: list[int] = []

# JSONEncoder() class-based API mirrors json.dumps
enc = json.JSONEncoder()
assert enc.encode([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)
assert enc.encode({"a": 1}) == '{"a": 1}'; _ledger.append(1)
assert enc.encode(None) == "null"; _ledger.append(1)
assert enc.encode(True) == "true"; _ledger.append(1)
assert enc.encode(False) == "false"; _ledger.append(1)
assert enc.encode("hello") == '"hello"'; _ledger.append(1)
assert enc.encode(42) == "42"; _ledger.append(1)

# JSONDecoder() class-based API mirrors json.loads
dec = json.JSONDecoder()
assert dec.decode("[1, 2, 3]") == [1, 2, 3]; _ledger.append(1)
assert dec.decode('{"a": 1}') == {"a": 1}; _ledger.append(1)
assert dec.decode("null") is None; _ledger.append(1)
assert dec.decode("true") == True; _ledger.append(1)
assert dec.decode("false") == False; _ledger.append(1)
assert dec.decode('"hello"') == "hello"; _ledger.append(1)
assert dec.decode("42") == 42; _ledger.append(1)

# Escape vocabulary in dumps output — quote and backslash always escaped
assert json.dumps('"') == '"\\""'; _ledger.append(1)
assert json.dumps("\\") == '"\\\\"'; _ledger.append(1)
assert json.dumps('"hello"') == '"\\"hello\\""'; _ledger.append(1)
assert json.dumps("a\\b") == '"a\\\\b"'; _ledger.append(1)
assert json.dumps('say "hi"') == '"say \\"hi\\""'; _ledger.append(1)

# Decoding \u00xx unicode escape sequences
assert json.loads('"\\u0041"') == "A"; _ledger.append(1)
assert json.loads('"\\u0042"') == "B"; _ledger.append(1)
assert json.loads('"\\u0030"') == "0"; _ledger.append(1)
assert json.loads('"\\u00ff"') == "ÿ"; _ledger.append(1)

# Standard escape decoding
assert json.loads('"\\n"') == "\n"; _ledger.append(1)
assert json.loads('"\\t"') == "\t"; _ledger.append(1)
assert json.loads('"\\\\"') == "\\"; _ledger.append(1)
assert json.loads('"\\""') == '"'; _ledger.append(1)

# Primitive scalar decoding
assert json.loads("0") == 0; _ledger.append(1)
assert json.loads("-1") == -1; _ledger.append(1)
assert json.loads("0.5") == 0.5; _ledger.append(1)
assert json.loads("-3.14") == -3.14; _ledger.append(1)
assert json.loads('""') == ""; _ledger.append(1)
assert json.loads("[]") == []; _ledger.append(1)
assert json.loads("{}") == {}; _ledger.append(1)

# Roundtrip preservation through loads(dumps(...))
assert json.loads(json.dumps(None)) is None; _ledger.append(1)
assert json.loads(json.dumps(True)) == True; _ledger.append(1)
assert json.loads(json.dumps(False)) == False; _ledger.append(1)
assert json.loads(json.dumps(0)) == 0; _ledger.append(1)
assert json.loads(json.dumps(42)) == 42; _ledger.append(1)
assert json.loads(json.dumps(-1)) == -1; _ledger.append(1)
assert json.loads(json.dumps(3.14)) == 3.14; _ledger.append(1)
assert json.loads(json.dumps("hello")) == "hello"; _ledger.append(1)
assert json.loads(json.dumps([])) == []; _ledger.append(1)
assert json.loads(json.dumps({})) == {}; _ledger.append(1)
assert json.loads(json.dumps([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
assert json.loads(json.dumps({"a": 1, "b": 2})) == {"a": 1, "b": 2}; _ledger.append(1)

# Nested structures roundtrip identically
data = {"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}], "count": 2}
assert json.loads(json.dumps(data)) == data; _ledger.append(1)

nested = [[1, 2], [3, [4, 5, [6]]]]
assert json.loads(json.dumps(nested)) == nested; _ledger.append(1)

# loads tolerates whitespace around values
assert json.loads("  42  ") == 42; _ledger.append(1)
assert json.loads("\n[1, 2]\n") == [1, 2]; _ledger.append(1)
assert json.loads("  {\"a\": 1}  ") == {"a": 1}; _ledger.append(1)

# Empty containers
assert json.dumps([]) == "[]"; _ledger.append(1)
assert json.dumps({}) == "{}"; _ledger.append(1)
assert json.dumps([[]]) == "[[]]"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_json_encoder_decoder_escape_ops {sum(_ledger)} asserts")
