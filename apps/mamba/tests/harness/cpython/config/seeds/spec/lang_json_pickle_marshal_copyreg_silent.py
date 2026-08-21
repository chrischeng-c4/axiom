# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `json.loads('1.5')` (the documented
# "JSON number 1.5 decodes to the Python float 1.5" — mamba returns
# 4609434218613702656, the raw 64-bit pattern), `pickle.loads(pickle.
# dumps(5))` (the documented "int pickle roundtrip returns the same
# int" — mamba returns None), `pickle.loads(pickle.dumps('hello'))`
# (the documented "str pickle roundtrip" — mamba returns None),
# `pickle.loads(pickle.dumps([1, 2, 3]))` (the documented "list
# pickle roundtrip" — mamba returns None), `marshal.loads(marshal.
# dumps(5))` (the documented "int marshal roundtrip" — mamba
# returns None), `marshal.loads(marshal.dumps('hello'))` (the
# documented "str marshal roundtrip" — mamba returns None),
# `marshal.loads(marshal.dumps([1, 2, 3]))` (the documented "list
# marshal roundtrip" — mamba returns None), `marshal.loads(marshal.
# dumps((1, 2, 3)))` (the documented "tuple marshal roundtrip" —
# mamba returns None), `hasattr(copyreg, 'pickle')` (the documented
# "copyreg exposes the pickle registration helper" — mamba returns
# False), and `hasattr(copyreg, 'dispatch_table')` (the documented
# "copyreg exposes the dispatch_table mapping" — mamba returns
# False).
# Ten-pack pinned to atomic 271.
#
# Behavioral edges that CONFORM on mamba (json — hasattr loads/
# dumps/load/dump/JSONEncoder/JSONDecoder/JSONDecodeError + loads
# true/false/null/int/str/list/dict/nested + dumps int/str/list/
# dict/None/True/False/indent/sort_keys/separators. pickle —
# hasattr dumps/loads/dump/load/HIGHEST_PROTOCOL/DEFAULT_PROTOCOL/
# Pickler/Unpickler/PickleError/UnpicklingError + HIGHEST_PROTOCOL
# is int / >=2. marshal — hasattr dumps/loads/dump/load/version +
# None roundtrip + version is int) are covered in the matching
# pass fixture `test_json_pickle_marshal_copyreg_value_ops`.
import json
import pickle
import marshal
import copyreg


_ledger: list[int] = []

# 1) json.loads('1.5') — JSON float decodes to Python float
#    (mamba: returns 4609434218613702656 — raw 64-bit pattern)
assert json.loads("1.5") == 1.5; _ledger.append(1)

# 2) pickle int roundtrip (mamba: returns None)
assert pickle.loads(pickle.dumps(5)) == 5; _ledger.append(1)

# 3) pickle str roundtrip (mamba: returns None)
assert pickle.loads(pickle.dumps("hello")) == "hello"; _ledger.append(1)

# 4) pickle list roundtrip (mamba: returns None)
assert pickle.loads(pickle.dumps([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)

# 5) marshal int roundtrip (mamba: returns None)
assert marshal.loads(marshal.dumps(5)) == 5; _ledger.append(1)

# 6) marshal str roundtrip (mamba: returns None)
assert marshal.loads(marshal.dumps("hello")) == "hello"; _ledger.append(1)

# 7) marshal list roundtrip (mamba: returns None)
assert marshal.loads(marshal.dumps([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)

# 8) marshal tuple roundtrip (mamba: returns None)
assert marshal.loads(marshal.dumps((1, 2, 3))) == (1, 2, 3); _ledger.append(1)

# 9) hasattr(copyreg, 'pickle') — registration helper
#    (mamba: returns False)
assert hasattr(copyreg, "pickle") == True; _ledger.append(1)

# 10) hasattr(copyreg, 'dispatch_table') — dispatch mapping
#     (mamba: returns False)
assert hasattr(copyreg, "dispatch_table") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_json_pickle_marshal_copyreg_silent {sum(_ledger)} asserts")
