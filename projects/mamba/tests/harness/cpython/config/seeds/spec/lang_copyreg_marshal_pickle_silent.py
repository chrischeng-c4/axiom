# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `type(copyreg).__name__` (the
# documented "copyreg is the copyreg module" — mamba returns
# 'NoneType' — module resolves to None), `hasattr(copyreg, 'pickle')
# ` (the documented "copyreg exposes the pickle registration
# helper" — mamba returns False), `hasattr(copyreg, 'constructor')`
# (the documented "copyreg exposes the constructor registration
# helper" — mamba returns False), `hasattr(copyreg, '__newobj__')`
# (the documented "copyreg exposes the __newobj__ reduce helper" —
# mamba returns False), `hasattr(copyreg, '__newobj_ex__')` (the
# documented "copyreg exposes the __newobj_ex__ reduce helper" —
# mamba returns False), `hasattr(copyreg, '_reconstructor')` (the
# documented "copyreg exposes the _reconstructor private helper" —
# mamba returns False), `hasattr(copyreg, 'dispatch_table')` (the
# documented "copyreg exposes the dispatch_table mapping" — mamba
# returns False), `isinstance(marshal.dumps(1), bytes)` (the
# documented "marshal.dumps returns bytes" — mamba returns str —
# str-encoded output), `marshal.loads(marshal.dumps(1)) == 1` (the
# documented "marshal round-trip preserves int" — mamba returns
# None — loads loses value), and `pickle.dumps(1)[:2] == b'\\x80
# \\x04'` (the documented "pickle.dumps emits protocol-4 header
# 0x80 0x04" — mamba returns b'I1' — protocol-0 ASCII INT opcode).
# Ten-pack pinned to atomic 289.
#
# Behavioral edges that CONFORM on mamba (weakref — hasattr ref/
# proxy/WeakValueDictionary/WeakKeyDictionary/WeakSet/WeakMethod/
# finalize/ReferenceType/ProxyType/getweakrefcount/getweakrefs.
# copy — hasattr copy/deepcopy/Error + copy/deepcopy/tuple/set.
# pickle — hasattr dumps/loads/dump/load/Pickler/Unpickler/Pickle
# Error/PicklingError/UnpicklingError/HIGHEST_PROTOCOL/DEFAULT_
# PROTOCOL + dumps bytes + round-trip 1/[1,2]/'hi'. types — hasattr
# FunctionType/MethodType/ModuleType/GeneratorType/CoroutineType/
# AsyncGeneratorType/BuiltinFunctionType/BuiltinMethodType/Lambda
# Type/MappingProxyType/TracebackType/FrameType/CodeType/CellType/
# UnionType/GenericAlias. keyword — hasattr iskeyword/kwlist/
# issoftkeyword/softkwlist + iskeyword + kwlist list. token —
# hasattr NAME/NUMBER/STRING/NEWLINE/OP/ENDMARKER/INDENT/DEDENT/
# tok_name/ISTERMINAL + NAME=1/NUMBER=2/STRING=3 + tok_name dict)
# are covered in the matching pass fixture `test_weakref_copy_
# pickle_types_keyword_token_value_ops`.
import copyreg
import marshal
import pickle


_ledger: list[int] = []

# 1) type(copyreg).__name__ == 'module' — copyreg is a real module
#    (mamba: returns 'NoneType' — module resolves to None)
assert type(copyreg).__name__ == "module"; _ledger.append(1)

# 2) hasattr(copyreg, 'pickle') — pickle registration helper
#    (mamba: returns False)
assert hasattr(copyreg, "pickle") == True; _ledger.append(1)

# 3) hasattr(copyreg, 'constructor') — constructor registration helper
#    (mamba: returns False)
assert hasattr(copyreg, "constructor") == True; _ledger.append(1)

# 4) hasattr(copyreg, '__newobj__') — __newobj__ reduce helper
#    (mamba: returns False)
assert hasattr(copyreg, "__newobj__") == True; _ledger.append(1)

# 5) hasattr(copyreg, '__newobj_ex__') — __newobj_ex__ reduce helper
#    (mamba: returns False)
assert hasattr(copyreg, "__newobj_ex__") == True; _ledger.append(1)

# 6) hasattr(copyreg, '_reconstructor') — _reconstructor private helper
#    (mamba: returns False)
assert hasattr(copyreg, "_reconstructor") == True; _ledger.append(1)

# 7) hasattr(copyreg, 'dispatch_table') — dispatch_table mapping
#    (mamba: returns False)
assert hasattr(copyreg, "dispatch_table") == True; _ledger.append(1)

# 8) isinstance(marshal.dumps(1), bytes) — marshal emits bytes
#    (mamba: returns str — str-encoded output)
assert isinstance(marshal.dumps(1), bytes) == True; _ledger.append(1)

# 9) marshal.loads(marshal.dumps(1)) == 1 — round-trip preserves int
#    (mamba: returns None — loads loses value)
assert marshal.loads(marshal.dumps(1)) == 1; _ledger.append(1)

# 10) pickle.dumps(1)[:2] == b'\x80\x04' — protocol-4 header
#     (mamba: returns b'I1' — protocol-0 ASCII INT opcode)
assert pickle.dumps(1)[:2] == b"\x80\x04"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_copyreg_marshal_pickle_silent {sum(_ledger)} asserts")
