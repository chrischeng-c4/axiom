# Operational AssertionPass seed for the `types` module — the
# string-identity surface of every runtime-type token that `types`
# exposes as a class attribute (ModuleType / FunctionType /
# MethodType / MappingProxyType / GeneratorType / CoroutineType /
# AsyncGeneratorType / TracebackType / FrameType / CodeType /
# BuiltinFunctionType / LambdaType / GetSetDescriptorType /
# MemberDescriptorType / WrapperDescriptorType / MethodWrapperType /
# BuiltinMethodType / ClassMethodDescriptorType / MethodDescriptorType
# / NotImplementedType / NoneType / UnionType / GenericAlias).
#
# `types` is the runtime registry of internal type objects that CPython
# constructs on demand. On mamba these are stub `<type instance>`
# placeholders that have correct `.__name__` strings but no class
# identity, no usable constructor, and no isinstance integration. The
# matching subset between mamba and CPython is therefore: each token's
# `.__name__` is the documented type-name string, and `hasattr` is
# True. That is exactly what every introspection helper that wants to
# *name* a runtime type uses, so it is a real ABI surface even when
# the constructor / isinstance hooks diverge.
#
# Surface in this fixture:
#   • types.ModuleType.__name__       == "module";
#   • types.FunctionType.__name__     == "function";
#   • types.MethodType.__name__       == "method";
#   • types.MappingProxyType.__name__ == "mappingproxy";
#   • types.GeneratorType.__name__    == "generator";
#   • types.CoroutineType.__name__    == "coroutine";
#   • types.AsyncGeneratorType.__name__ == "async_generator";
#   • types.TracebackType.__name__    == "traceback";
#   • types.FrameType.__name__        == "frame";
#   • types.CodeType.__name__         == "code";
#   • types.BuiltinFunctionType.__name__ == "builtin_function_or_method";
#   • types.LambdaType.__name__       == "function";
#   • types.GetSetDescriptorType.__name__ == "getset_descriptor";
#   • types.MemberDescriptorType.__name__ == "member_descriptor";
#   • types.WrapperDescriptorType.__name__ == "wrapper_descriptor";
#   • types.MethodWrapperType.__name__ == "method-wrapper";
#   • types.BuiltinMethodType.__name__ == "builtin_function_or_method";
#   • types.ClassMethodDescriptorType.__name__ == "classmethod_descriptor";
#   • types.MethodDescriptorType.__name__ == "method_descriptor";
#   • types.NotImplementedType.__name__ == "NotImplementedType";
#   • types.NoneType.__name__         == "NoneType";
#   • types.UnionType.__name__        == "UnionType";
#   • types.GenericAlias.__name__     == "GenericAlias";
#   • every `.__name__` accessor returns `str`;
#   • `hasattr(types, X)` is True for every documented type token.
#
# Behavioral edges that DIVERGE on mamba (SimpleNamespace / ModuleType
# / MappingProxyType / NoneType constructors, types.NoneType is type(
# None), types.LambdaType is types.FunctionType, generator expressions
# typing as list, isinstance against types.GeneratorType /
# FunctionType / LambdaType, types.resolve_bases callable) are covered
# in `lang_types_constructor_isinstance_simplenamespace_silent.py`.
import types

_ledger: list[int] = []

# 1) Module / function / method / mapping-proxy
assert types.ModuleType.__name__ == "module"; _ledger.append(1)
assert types.FunctionType.__name__ == "function"; _ledger.append(1)
assert types.MethodType.__name__ == "method"; _ledger.append(1)
assert types.MappingProxyType.__name__ == "mappingproxy"; _ledger.append(1)

# 2) Generator family
assert types.GeneratorType.__name__ == "generator"; _ledger.append(1)
assert types.CoroutineType.__name__ == "coroutine"; _ledger.append(1)
assert types.AsyncGeneratorType.__name__ == "async_generator"; _ledger.append(1)

# 3) Traceback / frame / code (interpreter-internal types)
assert types.TracebackType.__name__ == "traceback"; _ledger.append(1)
assert types.FrameType.__name__ == "frame"; _ledger.append(1)
assert types.CodeType.__name__ == "code"; _ledger.append(1)

# 4) Builtin function + lambda
assert types.BuiltinFunctionType.__name__ == "builtin_function_or_method"; _ledger.append(1)
assert types.LambdaType.__name__ == "function"; _ledger.append(1)

# 5) Descriptor family
assert types.GetSetDescriptorType.__name__ == "getset_descriptor"; _ledger.append(1)
assert types.MemberDescriptorType.__name__ == "member_descriptor"; _ledger.append(1)
assert types.WrapperDescriptorType.__name__ == "wrapper_descriptor"; _ledger.append(1)
assert types.MethodWrapperType.__name__ == "method-wrapper"; _ledger.append(1)
assert types.BuiltinMethodType.__name__ == "builtin_function_or_method"; _ledger.append(1)
assert types.ClassMethodDescriptorType.__name__ == "classmethod_descriptor"; _ledger.append(1)
assert types.MethodDescriptorType.__name__ == "method_descriptor"; _ledger.append(1)

# 6) Singleton-type accessors
assert types.NotImplementedType.__name__ == "NotImplementedType"; _ledger.append(1)
assert types.NoneType.__name__ == "NoneType"; _ledger.append(1)

# 7) PEP-604 / generic alias type tokens
assert types.UnionType.__name__ == "UnionType"; _ledger.append(1)
assert types.GenericAlias.__name__ == "GenericAlias"; _ledger.append(1)

# 8) Every `.__name__` is a real str
assert isinstance(types.ModuleType.__name__, str); _ledger.append(1)
assert isinstance(types.FunctionType.__name__, str); _ledger.append(1)
assert isinstance(types.MethodType.__name__, str); _ledger.append(1)
assert isinstance(types.GeneratorType.__name__, str); _ledger.append(1)
assert isinstance(types.CodeType.__name__, str); _ledger.append(1)
assert isinstance(types.NoneType.__name__, str); _ledger.append(1)

# 9) hasattr — every documented type token is exposed
assert hasattr(types, "ModuleType"); _ledger.append(1)
assert hasattr(types, "FunctionType"); _ledger.append(1)
assert hasattr(types, "MethodType"); _ledger.append(1)
assert hasattr(types, "MappingProxyType"); _ledger.append(1)
assert hasattr(types, "GeneratorType"); _ledger.append(1)
assert hasattr(types, "CoroutineType"); _ledger.append(1)
assert hasattr(types, "AsyncGeneratorType"); _ledger.append(1)
assert hasattr(types, "TracebackType"); _ledger.append(1)
assert hasattr(types, "FrameType"); _ledger.append(1)
assert hasattr(types, "CodeType"); _ledger.append(1)
assert hasattr(types, "BuiltinFunctionType"); _ledger.append(1)
assert hasattr(types, "LambdaType"); _ledger.append(1)
assert hasattr(types, "GetSetDescriptorType"); _ledger.append(1)
assert hasattr(types, "MemberDescriptorType"); _ledger.append(1)
assert hasattr(types, "NotImplementedType"); _ledger.append(1)
assert hasattr(types, "NoneType"); _ledger.append(1)
assert hasattr(types, "UnionType"); _ledger.append(1)
assert hasattr(types, "GenericAlias"); _ledger.append(1)

# NB: types.SimpleNamespace(a=1).a, types.ModuleType("x"), types.
# MappingProxyType({1:2})[1], types.NoneType() constructor calls, the
# `types.NoneType is type(None)` identity, the `types.LambdaType is
# types.FunctionType` alias identity, the runtime type of generator
# expressions (`type((x for x in [1])).__name__ == "generator"`),
# isinstance against types.GeneratorType / FunctionType / LambdaType,
# and types.resolve_bases being callable all DIVERGE on mamba — moved
# to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_types_class_descriptor_name_ops {sum(_ledger)} asserts")
