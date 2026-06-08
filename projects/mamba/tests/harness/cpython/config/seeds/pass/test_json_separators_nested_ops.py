# Operational AssertionPass seed for json module surfaces beyond
# test_json_ops / test_json_options_ops.
# Surface: dumps with custom (item, key) separators; dumps of
# deeply-nested mixed-type structures; the dumps/loads round-trip
# preserves nested list-of-dict and dict-of-list shapes; the
# separators tuple supports the compact `(",", ":")` form.
import json
_ledger: list[int] = []

# Default separators include spaces after item-sep and key-sep
assert json.dumps({"a": 1, "b": 2}) == '{"a": 1, "b": 2}'; _ledger.append(1)

# Compact separators (no spaces)
assert json.dumps({"a": 1, "b": 2}, separators=(",", ":")) == '{"a":1,"b":2}'; _ledger.append(1)

# Compact separators on a list
assert json.dumps([1, 2, 3], separators=(",", ":")) == "[1,2,3]"; _ledger.append(1)

# Custom (item, key) separators — semicolon and equals
assert json.dumps({"a": 1, "b": 2}, separators=(";", "=")) == '{"a"=1;"b"=2}'; _ledger.append(1)

# Nested list-of-dict round-trips through dumps/loads
records = [{"name": "alice", "age": 30}, {"name": "bob", "age": 25}]
assert json.loads(json.dumps(records)) == records; _ledger.append(1)

# Nested dict-of-list round-trips
groups = {"odds": [1, 3, 5], "evens": [2, 4, 6]}
assert json.loads(json.dumps(groups)) == groups; _ledger.append(1)

# Deeply-nested mixed structure round-trips
deep = {"items": [1, 2, {"nested": True, "list": [10, 20, [30, 40]]}]}
assert json.loads(json.dumps(deep)) == deep; _ledger.append(1)

# Empty list and empty dict round-trip
assert json.loads(json.dumps([])) == []; _ledger.append(1)
assert json.loads(json.dumps({})) == {}; _ledger.append(1)

# Top-level scalars round-trip
assert json.loads(json.dumps(42)) == 42; _ledger.append(1)
assert json.loads(json.dumps("hello")) == "hello"; _ledger.append(1)
assert json.loads(json.dumps(True)) == True; _ledger.append(1)
assert json.loads(json.dumps(None)) is None; _ledger.append(1)

# An all-types mixed list round-trips
mixed = [1, "two", 3.5, True, None, [4, 5], {"k": "v"}]
assert json.loads(json.dumps(mixed)) == mixed; _ledger.append(1)

# json.dumps of nested list always uses [...] not (...)
assert json.dumps([1, [2, [3, [4]]]]) == "[1, [2, [3, [4]]]]"; _ledger.append(1)

# loads ignores leading and trailing whitespace
assert json.loads("  [1, 2, 3]  ") == [1, 2, 3]; _ledger.append(1)
assert json.loads("\n{\"a\": 1}\n") == {"a": 1}; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_json_separators_nested_ops {sum(_ledger)} asserts")
