# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(typing, 'Iterable')` (the
# documented "typing exposes the Iterable generic alias" — mamba
# returns False), `hasattr(typing, 'get_args')` (the documented
# "typing exposes the get_args introspection helper" — mamba returns
# False), `hasattr(typing, 'get_origin')` (the documented "typing
# exposes the get_origin introspection helper" — mamba returns
# False), `typing.Optional[int]` (the documented "Optional[T]
# constructs a typing.Optional[T] alias" — mamba returns None),
# `typing.Union[int, str]` (the documented "Union[T1, T2] constructs
# a typing.Union[T1, T2] alias" — mamba returns None), `type
# (abc.ABC).__name__` (the documented "abc.ABC's metaclass is
# ABCMeta" — mamba returns 'function'), `hasattr(contextlib,
# 'closing')` (the documented "contextlib exposes the closing
# context manager" — mamba returns False), `hasattr(contextlib,
# 'redirect_stdout')` (the documented "contextlib exposes the
# redirect_stdout context manager" — mamba returns False), `hasattr
# (contextlib, 'ExitStack')` (the documented "contextlib exposes the
# ExitStack class" — mamba returns False), and `type(contextlib.
# nullcontext()).__name__` (the documented "nullcontext() returns a
# nullcontext instance" — mamba returns 'NoneType').
# Ten-pack pinned to atomic 268.
#
# Behavioral edges that CONFORM on mamba (typing — hasattr Any/List/
# Dict/Tuple/Set/Optional/Union/Callable/Type/Iterator/Generator/
# TYPE_CHECKING/cast/get_type_hints/Protocol/Final/Literal/Named
# Tuple/TypedDict/ClassVar/TypeVar + TYPE_CHECKING==False + cast
# (int, 5)==5. abc — hasattr ABC/ABCMeta/abstractmethod/abstract
# property/abstractclassmethod/abstractstaticmethod/update_abstract
# methods. weakref — hasattr ref/proxy/WeakSet/WeakValueDictionary/
# WeakKeyDictionary/finalize/getweakrefcount/getweakrefs/Reference
# Type. contextlib — hasattr contextmanager/suppress/nullcontext +
# suppress callable) are covered in the matching pass fixture
# `test_typing_abc_weakref_contextlib_value_ops`.
import typing
import abc
import contextlib


_ledger: list[int] = []

# 1) hasattr(typing, 'Iterable') — Iterable generic alias
#    (mamba: returns False)
assert hasattr(typing, "Iterable") == True; _ledger.append(1)

# 2) hasattr(typing, 'get_args') — introspection helper
#    (mamba: returns False)
assert hasattr(typing, "get_args") == True; _ledger.append(1)

# 3) hasattr(typing, 'get_origin') — introspection helper
#    (mamba: returns False)
assert hasattr(typing, "get_origin") == True; _ledger.append(1)

# 4) typing.Optional[int] is not None — yields Optional alias
#    (mamba: returns None)
assert (typing.Optional[int] is None) == False; _ledger.append(1)

# 5) typing.Union[int, str] is not None — yields Union alias
#    (mamba: returns None)
assert (typing.Union[int, str] is None) == False; _ledger.append(1)

# 6) type(abc.ABC).__name__ == 'ABCMeta' — metaclass identity
#    (mamba: returns 'function')
assert type(abc.ABC).__name__ == "ABCMeta"; _ledger.append(1)

# 7) hasattr(contextlib, 'closing') — closing context manager
#    (mamba: returns False)
assert hasattr(contextlib, "closing") == True; _ledger.append(1)

# 8) hasattr(contextlib, 'redirect_stdout') — redirect_stdout cm
#    (mamba: returns False)
assert hasattr(contextlib, "redirect_stdout") == True; _ledger.append(1)

# 9) hasattr(contextlib, 'ExitStack') — ExitStack class
#    (mamba: returns False)
assert hasattr(contextlib, "ExitStack") == True; _ledger.append(1)

# 10) type(contextlib.nullcontext()).__name__ == 'nullcontext'
#     (mamba: returns 'NoneType' — nullcontext() returns None)
assert type(contextlib.nullcontext()).__name__ == "nullcontext"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typing_abc_weakref_contextlib_silent {sum(_ledger)} asserts")
