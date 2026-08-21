# Atomic 246 pass conformance — typing surface (Any/Optional/Union/List/
# Dict/Tuple/Set/FrozenSet/Callable/Iterator/Generator/TypeVar/Generic/
# Protocol/ClassVar/Final/Literal/cast/get_type_hints/NamedTuple/TypedDict)
# + cast(int, 5) / contextlib basic surface (contextmanager/suppress/
# nullcontext) / abc surface (ABC/ABCMeta/abstractmethod/abstractproperty/
# abstractclassmethod/abstractstaticmethod/get_cache_token/
# update_abstractmethods) / bisect surface (bisect/bisect_left/
# bisect_right/insort/insort_left/insort_right) + bisect_left/right
# value ops + insort mutation / weakref class surface (ref/WeakSet/
# WeakValueDictionary/WeakKeyDictionary/proxy/getweakrefcount/
# getweakrefs/finalize/ReferenceType/ProxyType) / collections.abc surface
# (Iterable/Iterator/Mapping/MutableMapping/Sequence/MutableSequence/
# Set/MutableSet/Container/Hashable/Sized/Callable/Awaitable/Coroutine/
# AsyncIterable/AsyncIterator/Generator) / gc partial surface (collect/
# disable/enable/isenabled/get_count/get_threshold/set_threshold/
# is_tracked/DEBUG_LEAK) + collect/isenabled type ops / types class
# surface (FunctionType/MethodType/ModuleType/GeneratorType/
# CoroutineType/LambdaType/BuiltinFunctionType/BuiltinMethodType/
# SimpleNamespace/MappingProxyType/TracebackType/FrameType/CodeType/
# CellType/NoneType/GenericAlias/UnionType) that match between
# CPython 3.12 and mamba.
import typing
import contextlib
import abc
import bisect
import weakref
import collections.abc as cabc
import gc
import types


_ledger: list[int] = []

# 1) typing surface — basic generic aliases
assert hasattr(typing, "Any") == True; _ledger.append(1)
assert hasattr(typing, "Optional") == True; _ledger.append(1)
assert hasattr(typing, "Union") == True; _ledger.append(1)
assert hasattr(typing, "List") == True; _ledger.append(1)
assert hasattr(typing, "Dict") == True; _ledger.append(1)
assert hasattr(typing, "Tuple") == True; _ledger.append(1)
assert hasattr(typing, "Set") == True; _ledger.append(1)
assert hasattr(typing, "FrozenSet") == True; _ledger.append(1)
assert hasattr(typing, "Callable") == True; _ledger.append(1)

# 2) typing surface — iterator / generator
assert hasattr(typing, "Iterator") == True; _ledger.append(1)
assert hasattr(typing, "Generator") == True; _ledger.append(1)

# 3) typing surface — TypeVar / Generic / Protocol / ClassVar / Final / Literal
assert hasattr(typing, "TypeVar") == True; _ledger.append(1)
assert hasattr(typing, "Generic") == True; _ledger.append(1)
assert hasattr(typing, "Protocol") == True; _ledger.append(1)
assert hasattr(typing, "ClassVar") == True; _ledger.append(1)
assert hasattr(typing, "Final") == True; _ledger.append(1)
assert hasattr(typing, "Literal") == True; _ledger.append(1)

# 4) typing surface — call-site helpers
assert hasattr(typing, "cast") == True; _ledger.append(1)
assert hasattr(typing, "get_type_hints") == True; _ledger.append(1)
assert hasattr(typing, "NamedTuple") == True; _ledger.append(1)
assert hasattr(typing, "TypedDict") == True; _ledger.append(1)

# 5) typing.cast — runtime identity
assert typing.cast(int, 5) == 5; _ledger.append(1)

# 6) contextlib basic surface
assert hasattr(contextlib, "contextmanager") == True; _ledger.append(1)
assert hasattr(contextlib, "suppress") == True; _ledger.append(1)
assert hasattr(contextlib, "nullcontext") == True; _ledger.append(1)

# 7) abc surface
assert hasattr(abc, "ABC") == True; _ledger.append(1)
assert hasattr(abc, "ABCMeta") == True; _ledger.append(1)
assert hasattr(abc, "abstractmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractproperty") == True; _ledger.append(1)
assert hasattr(abc, "abstractclassmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractstaticmethod") == True; _ledger.append(1)
assert hasattr(abc, "get_cache_token") == True; _ledger.append(1)
assert hasattr(abc, "update_abstractmethods") == True; _ledger.append(1)

# 8) bisect surface
assert hasattr(bisect, "bisect") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_left") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_right") == True; _ledger.append(1)
assert hasattr(bisect, "insort") == True; _ledger.append(1)
assert hasattr(bisect, "insort_left") == True; _ledger.append(1)
assert hasattr(bisect, "insort_right") == True; _ledger.append(1)

# 9) bisect_left / bisect_right / bisect alias value ops
assert bisect.bisect_left([1, 2, 3, 5, 7], 4) == 3; _ledger.append(1)
assert bisect.bisect_right([1, 2, 3, 5, 7], 3) == 3; _ledger.append(1)
assert bisect.bisect([1, 2, 3, 5, 7], 3) == 3; _ledger.append(1)

# 10) bisect.insort — mutates list in place
_L = [1, 3, 5, 7]
bisect.insort(_L, 4)
assert _L == [1, 3, 4, 5, 7]; _ledger.append(1)

# 11) weakref class surface
assert hasattr(weakref, "ref") == True; _ledger.append(1)
assert hasattr(weakref, "WeakSet") == True; _ledger.append(1)
assert hasattr(weakref, "WeakValueDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakKeyDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "proxy") == True; _ledger.append(1)
assert hasattr(weakref, "getweakrefcount") == True; _ledger.append(1)
assert hasattr(weakref, "getweakrefs") == True; _ledger.append(1)
assert hasattr(weakref, "finalize") == True; _ledger.append(1)
assert hasattr(weakref, "ReferenceType") == True; _ledger.append(1)
assert hasattr(weakref, "ProxyType") == True; _ledger.append(1)

# 12) collections.abc surface
assert hasattr(cabc, "Iterable") == True; _ledger.append(1)
assert hasattr(cabc, "Iterator") == True; _ledger.append(1)
assert hasattr(cabc, "Mapping") == True; _ledger.append(1)
assert hasattr(cabc, "MutableMapping") == True; _ledger.append(1)
assert hasattr(cabc, "Sequence") == True; _ledger.append(1)
assert hasattr(cabc, "MutableSequence") == True; _ledger.append(1)
assert hasattr(cabc, "Set") == True; _ledger.append(1)
assert hasattr(cabc, "MutableSet") == True; _ledger.append(1)
assert hasattr(cabc, "Container") == True; _ledger.append(1)
assert hasattr(cabc, "Hashable") == True; _ledger.append(1)
assert hasattr(cabc, "Sized") == True; _ledger.append(1)
assert hasattr(cabc, "Callable") == True; _ledger.append(1)
assert hasattr(cabc, "Awaitable") == True; _ledger.append(1)
assert hasattr(cabc, "Coroutine") == True; _ledger.append(1)
assert hasattr(cabc, "AsyncIterable") == True; _ledger.append(1)
assert hasattr(cabc, "AsyncIterator") == True; _ledger.append(1)
assert hasattr(cabc, "Generator") == True; _ledger.append(1)

# 13) gc partial surface
assert hasattr(gc, "collect") == True; _ledger.append(1)
assert hasattr(gc, "disable") == True; _ledger.append(1)
assert hasattr(gc, "enable") == True; _ledger.append(1)
assert hasattr(gc, "isenabled") == True; _ledger.append(1)
assert hasattr(gc, "get_count") == True; _ledger.append(1)
assert hasattr(gc, "get_threshold") == True; _ledger.append(1)
assert hasattr(gc, "set_threshold") == True; _ledger.append(1)
assert hasattr(gc, "is_tracked") == True; _ledger.append(1)
assert hasattr(gc, "DEBUG_LEAK") == True; _ledger.append(1)
assert type(gc.collect()).__name__ == "int"; _ledger.append(1)
assert type(gc.isenabled()).__name__ == "bool"; _ledger.append(1)

# 14) types class surface
assert hasattr(types, "FunctionType") == True; _ledger.append(1)
assert hasattr(types, "MethodType") == True; _ledger.append(1)
assert hasattr(types, "ModuleType") == True; _ledger.append(1)
assert hasattr(types, "GeneratorType") == True; _ledger.append(1)
assert hasattr(types, "CoroutineType") == True; _ledger.append(1)
assert hasattr(types, "LambdaType") == True; _ledger.append(1)
assert hasattr(types, "BuiltinFunctionType") == True; _ledger.append(1)
assert hasattr(types, "BuiltinMethodType") == True; _ledger.append(1)
assert hasattr(types, "SimpleNamespace") == True; _ledger.append(1)
assert hasattr(types, "MappingProxyType") == True; _ledger.append(1)
assert hasattr(types, "TracebackType") == True; _ledger.append(1)
assert hasattr(types, "FrameType") == True; _ledger.append(1)
assert hasattr(types, "CodeType") == True; _ledger.append(1)
assert hasattr(types, "CellType") == True; _ledger.append(1)
assert hasattr(types, "NoneType") == True; _ledger.append(1)
assert hasattr(types, "GenericAlias") == True; _ledger.append(1)
assert hasattr(types, "UnionType") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_typing_contextlib_abc_bisect_weakref_cabc_gc_types_value_ops {sum(_ledger)} asserts")
