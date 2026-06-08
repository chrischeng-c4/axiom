# Atomic 268 pass conformance — typing module (hasattr Any/List/Dict/
# Tuple/Set/Optional/Union/Callable/Type/Iterator/Generator/TYPE_
# CHECKING/cast/get_type_hints/Protocol/Final/Literal/NamedTuple/
# TypedDict/ClassVar/TypeVar + TYPE_CHECKING==False + cast(int, 5)==5)
# + abc module (hasattr ABC/ABCMeta/abstractmethod/abstractproperty/
# abstractclassmethod/abstractstaticmethod/update_abstractmethods) +
# weakref module (hasattr ref/proxy/WeakSet/WeakValueDictionary/
# WeakKeyDictionary/finalize/getweakrefcount/getweakrefs/
# ReferenceType) + contextlib module (hasattr contextmanager/suppress/
# nullcontext + suppress callable).
# All asserts match between CPython 3.12 and mamba.
import typing
import abc
import weakref
import contextlib


_ledger: list[int] = []

# 1) typing — hasattr generic-type surface
assert hasattr(typing, "Any") == True; _ledger.append(1)
assert hasattr(typing, "List") == True; _ledger.append(1)
assert hasattr(typing, "Dict") == True; _ledger.append(1)
assert hasattr(typing, "Tuple") == True; _ledger.append(1)
assert hasattr(typing, "Set") == True; _ledger.append(1)
assert hasattr(typing, "Optional") == True; _ledger.append(1)
assert hasattr(typing, "Union") == True; _ledger.append(1)
assert hasattr(typing, "Callable") == True; _ledger.append(1)
assert hasattr(typing, "Type") == True; _ledger.append(1)
assert hasattr(typing, "Iterator") == True; _ledger.append(1)
assert hasattr(typing, "Generator") == True; _ledger.append(1)

# 2) typing — hasattr utility surface
assert hasattr(typing, "TYPE_CHECKING") == True; _ledger.append(1)
assert hasattr(typing, "cast") == True; _ledger.append(1)
assert hasattr(typing, "get_type_hints") == True; _ledger.append(1)

# 3) typing — hasattr special-form surface
assert hasattr(typing, "Protocol") == True; _ledger.append(1)
assert hasattr(typing, "Final") == True; _ledger.append(1)
assert hasattr(typing, "Literal") == True; _ledger.append(1)
assert hasattr(typing, "NamedTuple") == True; _ledger.append(1)
assert hasattr(typing, "TypedDict") == True; _ledger.append(1)
assert hasattr(typing, "ClassVar") == True; _ledger.append(1)
assert hasattr(typing, "TypeVar") == True; _ledger.append(1)

# 4) typing — TYPE_CHECKING is False at runtime
assert typing.TYPE_CHECKING == False; _ledger.append(1)

# 5) typing — cast returns its second argument unchanged
assert typing.cast(int, 5) == 5; _ledger.append(1)

# 6) abc — hasattr surface
assert hasattr(abc, "ABC") == True; _ledger.append(1)
assert hasattr(abc, "ABCMeta") == True; _ledger.append(1)
assert hasattr(abc, "abstractmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractproperty") == True; _ledger.append(1)
assert hasattr(abc, "abstractclassmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractstaticmethod") == True; _ledger.append(1)
assert hasattr(abc, "update_abstractmethods") == True; _ledger.append(1)

# 7) weakref — hasattr surface
assert hasattr(weakref, "ref") == True; _ledger.append(1)
assert hasattr(weakref, "proxy") == True; _ledger.append(1)
assert hasattr(weakref, "WeakSet") == True; _ledger.append(1)
assert hasattr(weakref, "WeakValueDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakKeyDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "finalize") == True; _ledger.append(1)
assert hasattr(weakref, "getweakrefcount") == True; _ledger.append(1)
assert hasattr(weakref, "getweakrefs") == True; _ledger.append(1)
assert hasattr(weakref, "ReferenceType") == True; _ledger.append(1)

# 8) contextlib — hasattr surface that both runtimes agree on
assert hasattr(contextlib, "contextmanager") == True; _ledger.append(1)
assert hasattr(contextlib, "suppress") == True; _ledger.append(1)
assert hasattr(contextlib, "nullcontext") == True; _ledger.append(1)

# 9) contextlib — suppress is callable
assert callable(contextlib.suppress) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_typing_abc_weakref_contextlib_value_ops {sum(_ledger)} asserts")
