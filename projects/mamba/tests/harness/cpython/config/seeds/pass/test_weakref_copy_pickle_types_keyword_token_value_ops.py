# Atomic 289 pass conformance — weakref module (hasattr ref/proxy/
# WeakValueDictionary/WeakKeyDictionary/WeakSet/WeakMethod/finalize/
# ReferenceType/ProxyType/getweakrefcount/getweakrefs) + copy module
# (hasattr copy/deepcopy/Error + copy([1,2,3])==[1,2,3] + copy({'a'
# :1})=={'a':1} + deepcopy nested + copy tuple + copy set + pickle
# round-trip int/list/str) + pickle module (hasattr dumps/loads/
# dump/load/Pickler/Unpickler/PickleError/PicklingError/Unpickling
# Error/HIGHEST_PROTOCOL/DEFAULT_PROTOCOL + dumps returns bytes +
# loads(dumps(1))==1) + types module (hasattr FunctionType/
# MethodType/ModuleType/GeneratorType/CoroutineType/AsyncGenerator
# Type/BuiltinFunctionType/BuiltinMethodType/LambdaType/MappingProxy
# Type/TracebackType/FrameType/CodeType/CellType/UnionType/Generic
# Alias) + keyword module (hasattr iskeyword/kwlist/issoftkeyword/
# softkwlist + iskeyword 'if'/'foo'/'class'/'def'/'lambda' + kwlist
# list + 'if'/'def'/'class' in kwlist + len > 20) + token module
# (hasattr NAME/NUMBER/STRING/NEWLINE/OP/ENDMARKER/INDENT/DEDENT/
# tok_name/ISTERMINAL + NAME==1/NUMBER==2/STRING==3 + tok_name
# dict).
# All asserts match between CPython 3.12 and mamba.
import weakref
import copy
import pickle
import types
import keyword
import token


_ledger: list[int] = []

# 1) weakref — hasattr core surface
assert hasattr(weakref, "ref") == True; _ledger.append(1)
assert hasattr(weakref, "proxy") == True; _ledger.append(1)
assert hasattr(weakref, "WeakValueDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakKeyDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakSet") == True; _ledger.append(1)
assert hasattr(weakref, "WeakMethod") == True; _ledger.append(1)
assert hasattr(weakref, "finalize") == True; _ledger.append(1)
assert hasattr(weakref, "ReferenceType") == True; _ledger.append(1)
assert hasattr(weakref, "ProxyType") == True; _ledger.append(1)
assert hasattr(weakref, "getweakrefcount") == True; _ledger.append(1)
assert hasattr(weakref, "getweakrefs") == True; _ledger.append(1)

# 2) copy — hasattr surface
assert hasattr(copy, "copy") == True; _ledger.append(1)
assert hasattr(copy, "deepcopy") == True; _ledger.append(1)
assert hasattr(copy, "Error") == True; _ledger.append(1)

# 3) copy — value contracts
assert copy.copy([1, 2, 3]) == [1, 2, 3]; _ledger.append(1)
assert copy.copy({"a": 1}) == {"a": 1}; _ledger.append(1)
assert copy.deepcopy([[1], [2]]) == [[1], [2]]; _ledger.append(1)
assert copy.deepcopy({"a": [1, 2]}) == {"a": [1, 2]}; _ledger.append(1)
assert copy.copy((1, 2, 3)) == (1, 2, 3); _ledger.append(1)
assert copy.copy({1, 2, 3}) == {1, 2, 3}; _ledger.append(1)

# 4) pickle — hasattr core surface
assert hasattr(pickle, "dumps") == True; _ledger.append(1)
assert hasattr(pickle, "loads") == True; _ledger.append(1)
assert hasattr(pickle, "dump") == True; _ledger.append(1)
assert hasattr(pickle, "load") == True; _ledger.append(1)
assert hasattr(pickle, "Pickler") == True; _ledger.append(1)
assert hasattr(pickle, "Unpickler") == True; _ledger.append(1)
assert hasattr(pickle, "PickleError") == True; _ledger.append(1)
assert hasattr(pickle, "PicklingError") == True; _ledger.append(1)
assert hasattr(pickle, "UnpicklingError") == True; _ledger.append(1)
assert hasattr(pickle, "HIGHEST_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "DEFAULT_PROTOCOL") == True; _ledger.append(1)

# 5) pickle — value contracts
assert isinstance(pickle.dumps(1), bytes) == True; _ledger.append(1)
assert pickle.loads(pickle.dumps(1)) == 1; _ledger.append(1)
assert pickle.loads(pickle.dumps([1, 2])) == [1, 2]; _ledger.append(1)
assert pickle.loads(pickle.dumps("hi")) == "hi"; _ledger.append(1)

# 6) types — hasattr function/method/module types
assert hasattr(types, "FunctionType") == True; _ledger.append(1)
assert hasattr(types, "MethodType") == True; _ledger.append(1)
assert hasattr(types, "ModuleType") == True; _ledger.append(1)
assert hasattr(types, "GeneratorType") == True; _ledger.append(1)
assert hasattr(types, "CoroutineType") == True; _ledger.append(1)
assert hasattr(types, "AsyncGeneratorType") == True; _ledger.append(1)
assert hasattr(types, "BuiltinFunctionType") == True; _ledger.append(1)
assert hasattr(types, "BuiltinMethodType") == True; _ledger.append(1)
assert hasattr(types, "LambdaType") == True; _ledger.append(1)
assert hasattr(types, "MappingProxyType") == True; _ledger.append(1)
assert hasattr(types, "TracebackType") == True; _ledger.append(1)
assert hasattr(types, "FrameType") == True; _ledger.append(1)
assert hasattr(types, "CodeType") == True; _ledger.append(1)
assert hasattr(types, "CellType") == True; _ledger.append(1)
assert hasattr(types, "UnionType") == True; _ledger.append(1)
assert hasattr(types, "GenericAlias") == True; _ledger.append(1)

# 7) keyword — hasattr + behavior
assert hasattr(keyword, "iskeyword") == True; _ledger.append(1)
assert hasattr(keyword, "kwlist") == True; _ledger.append(1)
assert hasattr(keyword, "issoftkeyword") == True; _ledger.append(1)
assert hasattr(keyword, "softkwlist") == True; _ledger.append(1)
assert keyword.iskeyword("if") == True; _ledger.append(1)
assert keyword.iskeyword("foo") == False; _ledger.append(1)
assert keyword.iskeyword("class") == True; _ledger.append(1)
assert keyword.iskeyword("def") == True; _ledger.append(1)
assert keyword.iskeyword("lambda") == True; _ledger.append(1)
assert isinstance(keyword.kwlist, list) == True; _ledger.append(1)
assert ("if" in keyword.kwlist) == True; _ledger.append(1)
assert ("def" in keyword.kwlist) == True; _ledger.append(1)
assert ("class" in keyword.kwlist) == True; _ledger.append(1)
assert (len(keyword.kwlist) > 20) == True; _ledger.append(1)

# 8) token — hasattr core surface
assert hasattr(token, "NAME") == True; _ledger.append(1)
assert hasattr(token, "NUMBER") == True; _ledger.append(1)
assert hasattr(token, "STRING") == True; _ledger.append(1)
assert hasattr(token, "NEWLINE") == True; _ledger.append(1)
assert hasattr(token, "OP") == True; _ledger.append(1)
assert hasattr(token, "ENDMARKER") == True; _ledger.append(1)
assert hasattr(token, "INDENT") == True; _ledger.append(1)
assert hasattr(token, "DEDENT") == True; _ledger.append(1)
assert hasattr(token, "tok_name") == True; _ledger.append(1)
assert hasattr(token, "ISTERMINAL") == True; _ledger.append(1)

# 9) token — value contracts
assert token.NAME == 1; _ledger.append(1)
assert token.NUMBER == 2; _ledger.append(1)
assert token.STRING == 3; _ledger.append(1)
assert isinstance(token.tok_name, dict) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_weakref_copy_pickle_types_keyword_token_value_ops {sum(_ledger)} asserts")
