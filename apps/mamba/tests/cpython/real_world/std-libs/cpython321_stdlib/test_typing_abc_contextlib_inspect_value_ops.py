# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_typing_abc_contextlib_inspect_value_ops"
# subject = "cpython321.test_typing_abc_contextlib_inspect_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_typing_abc_contextlib_inspect_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_typing_abc_contextlib_inspect_value_ops: execute CPython 3.12 seed test_typing_abc_contextlib_inspect_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 302 pass conformance — typing module (hasattr Any/Optional/
# Union/List/Dict/Tuple/Set/FrozenSet/Callable/TypeVar/Generic/Type/
# Final/ClassVar/Literal/Protocol/NamedTuple/TypedDict/cast/get_type_
# hints + cast pass-through) + abc module (hasattr ABC/ABCMeta/
# abstractmethod/abstractproperty/abstractclassmethod/abstractstatic
# method/get_cache_token/update_abstractmethods) + weakref module
# (hasattr ref/WeakSet/WeakValueDictionary/WeakKeyDictionary/proxy/
# getweakrefcount/getweakrefs/ReferenceType/ProxyType/finalize + type
# weakref.ref(c) == 'ReferenceType') + contextlib module (hasattr
# contextmanager/suppress/nullcontext) + inspect module (hasattr
# signature/isfunction/ismethod/isclass).
# All asserts match between CPython 3.12 and mamba.
import typing
import abc
import weakref
import contextlib
import inspect


_ledger: list[int] = []

# 1) typing — hasattr core surface (conformant subset)
assert hasattr(typing, "Any") == True; _ledger.append(1)
assert hasattr(typing, "Optional") == True; _ledger.append(1)
assert hasattr(typing, "Union") == True; _ledger.append(1)
assert hasattr(typing, "List") == True; _ledger.append(1)
assert hasattr(typing, "Dict") == True; _ledger.append(1)
assert hasattr(typing, "Tuple") == True; _ledger.append(1)
assert hasattr(typing, "Set") == True; _ledger.append(1)
assert hasattr(typing, "FrozenSet") == True; _ledger.append(1)
assert hasattr(typing, "Callable") == True; _ledger.append(1)
assert hasattr(typing, "TypeVar") == True; _ledger.append(1)
assert hasattr(typing, "Generic") == True; _ledger.append(1)
assert hasattr(typing, "Type") == True; _ledger.append(1)
assert hasattr(typing, "Final") == True; _ledger.append(1)
assert hasattr(typing, "ClassVar") == True; _ledger.append(1)
assert hasattr(typing, "Literal") == True; _ledger.append(1)
assert hasattr(typing, "Protocol") == True; _ledger.append(1)
assert hasattr(typing, "NamedTuple") == True; _ledger.append(1)
assert hasattr(typing, "TypedDict") == True; _ledger.append(1)
assert hasattr(typing, "cast") == True; _ledger.append(1)
assert hasattr(typing, "get_type_hints") == True; _ledger.append(1)

# 2) typing — value contracts (conformant subset)
assert typing.cast(int, 5) == 5; _ledger.append(1)

# 3) abc — hasattr core surface
assert hasattr(abc, "ABC") == True; _ledger.append(1)
assert hasattr(abc, "ABCMeta") == True; _ledger.append(1)
assert hasattr(abc, "abstractmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractproperty") == True; _ledger.append(1)
assert hasattr(abc, "abstractclassmethod") == True; _ledger.append(1)
assert hasattr(abc, "abstractstaticmethod") == True; _ledger.append(1)
assert hasattr(abc, "get_cache_token") == True; _ledger.append(1)
assert hasattr(abc, "update_abstractmethods") == True; _ledger.append(1)

# 4) weakref — hasattr core surface
assert hasattr(weakref, "ref") == True; _ledger.append(1)
assert hasattr(weakref, "WeakSet") == True; _ledger.append(1)
assert hasattr(weakref, "WeakValueDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "WeakKeyDictionary") == True; _ledger.append(1)
assert hasattr(weakref, "proxy") == True; _ledger.append(1)
assert hasattr(weakref, "getweakrefcount") == True; _ledger.append(1)
assert hasattr(weakref, "getweakrefs") == True; _ledger.append(1)
assert hasattr(weakref, "ReferenceType") == True; _ledger.append(1)
assert hasattr(weakref, "ProxyType") == True; _ledger.append(1)
assert hasattr(weakref, "finalize") == True; _ledger.append(1)


class W:
    pass


_w = W()
_r = weakref.ref(_w)
# 5) weakref — type echo (conformant subset)
assert type(_r).__name__ == "ReferenceType"; _ledger.append(1)

# 6) contextlib — hasattr core surface (conformant subset)
assert hasattr(contextlib, "contextmanager") == True; _ledger.append(1)
assert hasattr(contextlib, "suppress") == True; _ledger.append(1)
assert hasattr(contextlib, "nullcontext") == True; _ledger.append(1)


# 7) inspect — hasattr core surface (conformant subset)
assert hasattr(inspect, "signature") == True; _ledger.append(1)
assert hasattr(inspect, "isfunction") == True; _ledger.append(1)
assert hasattr(inspect, "ismethod") == True; _ledger.append(1)
assert hasattr(inspect, "isclass") == True; _ledger.append(1)


print(f"MAMBA_ASSERTION_PASS: test_typing_abc_contextlib_inspect_value_ops {sum(_ledger)} asserts")
