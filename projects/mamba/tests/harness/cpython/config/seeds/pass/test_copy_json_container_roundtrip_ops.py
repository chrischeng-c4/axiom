# Operational AssertionPass seed for the matching `copy` (shallow +
# deep) and `json` (non-float roundtrip + dumps formatting) surface.
# There are no existing copy / json seeds.
#
# This fixture pairs two libraries because both define "structured-
# data echo" semantics over the same set of base types — copy moves
# Python objects through the cloning path, json moves them through the
# serialize/deserialize path. The matching subset is the set of
# container shapes that both pipelines render identically across
# CPython and mamba.
#
# Surface in this fixture:
#   • copy.copy / copy.deepcopy on list / dict / nested-list — distinct
#     identity (`a is not b`), structural equality (`a == b`);
#   • shallow copy semantics — mutating the copy does NOT touch the
#     original, but inner references stay shared at depth 1;
#   • deepcopy semantics — mutating the deep clone does NOT affect any
#     part of the original, even at depth 2+;
#   • json.dumps / json.loads round-trip on int / bool / None / str /
#     nested dict / nested list / empty container (FLOAT round-trip is
#     in the divergence-spec fixture);
#   • json.dumps with `indent=N` and `sort_keys=True` formatting flags.
#
# Behavioral edges that DIVERGE on mamba (json bare-float loads,
# non-serializable type errors, copy of immutable types, deepcopy
# shared-ref identity, json.dumps default ensure_ascii) are covered in
# `lang_copy_json_loads_dumps_silent.py`.
import copy
import json

_ledger: list[int] = []

# 1) copy.copy on primitives + containers — exact equality
assert copy.copy(42) == 42; _ledger.append(1)
assert copy.copy(True) == True; _ledger.append(1)
assert copy.copy(None) is None; _ledger.append(1)
assert copy.copy("hello") == "hello"; _ledger.append(1)
assert copy.copy([1, 2, 3]) == [1, 2, 3]; _ledger.append(1)
assert copy.copy({"a": 1, "b": 2}) == {"a": 1, "b": 2}; _ledger.append(1)
assert copy.copy((1, 2, 3)) == (1, 2, 3); _ledger.append(1)
assert copy.copy([]) == []; _ledger.append(1)
assert copy.copy({}) == {}; _ledger.append(1)

# 2) Shallow copy of a list creates a distinct container
_a = [1, 2, 3]
_b = copy.copy(_a)
assert _a is not _b; _ledger.append(1)
assert _a == _b; _ledger.append(1)
# Mutating the copy does NOT touch the original
_b.append(4)
assert _a == [1, 2, 3]; _ledger.append(1)
assert _b == [1, 2, 3, 4]; _ledger.append(1)

# Same for dict
_d = {"a": 1, "b": 2}
_dc = copy.copy(_d)
assert _d is not _dc; _ledger.append(1)
assert _d == _dc; _ledger.append(1)
_dc["c"] = 3
assert _d == {"a": 1, "b": 2}; _ledger.append(1)
assert _dc == {"a": 1, "b": 2, "c": 3}; _ledger.append(1)

# 3) copy.deepcopy on nested mutables
_nested = [[1, 2], [3, 4]]
_dc2 = copy.deepcopy(_nested)
assert _nested == _dc2; _ledger.append(1)
assert _nested is not _dc2; _ledger.append(1)
# The inner lists are also distinct
assert _nested[0] is not _dc2[0]; _ledger.append(1)
assert _nested[1] is not _dc2[1]; _ledger.append(1)
# Mutating the original inner does NOT affect the deep clone
_nested[0].append(99)
assert _dc2[0] == [1, 2]; _ledger.append(1)
# And vice versa
_nested2 = [[1, 2], [3, 4]]
_dc3 = copy.deepcopy(_nested2)
_dc3[0].append(99)
assert _nested2[0] == [1, 2]; _ledger.append(1)

# deepcopy of dict-of-list
_d2 = {"items": [1, 2, 3], "tags": ["a", "b"]}
_dd = copy.deepcopy(_d2)
assert _d2 == _dd; _ledger.append(1)
assert _d2 is not _dd; _ledger.append(1)
assert _d2["items"] is not _dd["items"]; _ledger.append(1)
_dd["items"].append(99)
assert _d2["items"] == [1, 2, 3]; _ledger.append(1)

# deepcopy of primitives is just equality (identity may or may not
# match per implementation)
assert copy.deepcopy(42) == 42; _ledger.append(1)
assert copy.deepcopy("hello") == "hello"; _ledger.append(1)
assert copy.deepcopy(None) is None; _ledger.append(1)

# 4) json.dumps round-trip for non-float types
assert json.dumps(42) == "42"; _ledger.append(1)
assert json.dumps(-7) == "-7"; _ledger.append(1)
assert json.dumps(0) == "0"; _ledger.append(1)
assert json.dumps(True) == "true"; _ledger.append(1)
assert json.dumps(False) == "false"; _ledger.append(1)
assert json.dumps(None) == "null"; _ledger.append(1)
assert json.dumps("hello") == '"hello"'; _ledger.append(1)
assert json.dumps("") == '""'; _ledger.append(1)
assert json.dumps([]) == "[]"; _ledger.append(1)
assert json.dumps({}) == "{}"; _ledger.append(1)
assert json.dumps([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)
assert json.dumps({"a": 1, "b": 2}) == '{"a": 1, "b": 2}'; _ledger.append(1)

# 5) json.loads round-trip for non-float types
assert json.loads("42") == 42; _ledger.append(1)
assert json.loads("-7") == -7; _ledger.append(1)
assert json.loads("0") == 0; _ledger.append(1)
assert json.loads("true") == True; _ledger.append(1)
assert json.loads("false") == False; _ledger.append(1)
assert json.loads("null") is None; _ledger.append(1)
assert json.loads('"hello"') == "hello"; _ledger.append(1)
assert json.loads('""') == ""; _ledger.append(1)
assert json.loads("[]") == []; _ledger.append(1)
assert json.loads("{}") == {}; _ledger.append(1)
assert json.loads("[1, 2, 3]") == [1, 2, 3]; _ledger.append(1)
assert json.loads('{"a": 1, "b": 2}') == {"a": 1, "b": 2}; _ledger.append(1)

# 6) Nested structures — round-trip preserves shape
_obj = {"name": "test", "values": [1, 2, 3], "nested": {"flag": True, "tags": ["x", "y"]}}
_serialized = json.dumps(_obj)
_decoded = json.loads(_serialized)
assert _decoded == _obj; _ledger.append(1)
assert _decoded["name"] == "test"; _ledger.append(1)
assert _decoded["values"] == [1, 2, 3]; _ledger.append(1)
assert _decoded["nested"] == {"flag": True, "tags": ["x", "y"]}; _ledger.append(1)
assert _decoded["nested"]["flag"] == True; _ledger.append(1)
assert _decoded["nested"]["tags"] == ["x", "y"]; _ledger.append(1)

# 7) json.dumps with indent + sort_keys
assert json.dumps({"a": 1}, indent=2) == '{\n  "a": 1\n}'; _ledger.append(1)
assert json.dumps({"z": 1, "a": 2}, sort_keys=True) == '{"a": 2, "z": 1}'; _ledger.append(1)

# Combined: indent + sort_keys
_combined = json.dumps({"b": 2, "a": 1}, indent=2, sort_keys=True)
assert '"a": 1' in _combined; _ledger.append(1)
assert '"b": 2' in _combined; _ledger.append(1)
assert "\n" in _combined; _ledger.append(1)

# Nested list of ints round-trip
assert json.loads(json.dumps([[1, 2], [3, 4]])) == [[1, 2], [3, 4]]; _ledger.append(1)
# Nested dict round-trip
assert json.loads(json.dumps({"x": {"y": {"z": 1}}})) == {"x": {"y": {"z": 1}}}; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_copy_json_container_roundtrip_ops {sum(_ledger)} asserts")
