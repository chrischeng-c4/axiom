# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(typing, 'NewType')` (the
# documented "typing exposes the NewType helper" — mamba returns
# False), `hasattr(typing, 'get_origin')` (the documented "typing
# exposes the get_origin introspector" — mamba returns False),
# `hasattr(typing, 'get_args')` (the documented "typing exposes
# the get_args introspector" — mamba returns False), `hasattr
# (typing, 'Annotated')` (the documented "typing exposes the
# Annotated special form" — mamba returns False), `hasattr(typing,
# 'Self')` (the documented "typing exposes the Self special form"
# — mamba returns False), `type(typing.TypeVar('T')).__name__` (the
# documented "TypeVar() returns a TypeVar instance" — mamba returns
# 'NoneType' — TypeVar factory returns None), `hasattr(inspect,
# 'Parameter')` (the documented "inspect exposes the Parameter
# class" — mamba returns False), `hasattr(inspect, 'Signature')`
# (the documented "inspect exposes the Signature class" — mamba
# returns False), `inspect.isfunction(myfn)` (the documented
# "isfunction recognises def-functions" — mamba returns False —
# predicate never positive), and `inspect.isclass(int)` (the
# documented "isclass recognises builtin classes" — mamba returns
# False — predicate never positive).
# Ten-pack pinned to atomic 291.
#
# Behavioral edges that CONFORM on mamba (typing — hasattr List/
# Dict/Set/Tuple/Optional/Union/Callable/Any/TypeVar/Generic/
# Iterator/ClassVar/Final/Literal/Protocol/TypedDict/NamedTuple/
# cast/get_type_hints + cast value passthrough. inspect — hasattr
# signature/isfunction/ismethod/isclass/getmembers) are covered in
# the matching pass fixture `test_typing_inspect_value_ops`.
import typing
import inspect


def _myfn():
    pass


_ledger: list[int] = []

# 1) hasattr(typing, 'NewType') — NewType helper
#    (mamba: returns False)
assert hasattr(typing, "NewType") == True; _ledger.append(1)

# 2) hasattr(typing, 'get_origin') — get_origin introspector
#    (mamba: returns False)
assert hasattr(typing, "get_origin") == True; _ledger.append(1)

# 3) hasattr(typing, 'get_args') — get_args introspector
#    (mamba: returns False)
assert hasattr(typing, "get_args") == True; _ledger.append(1)

# 4) hasattr(typing, 'Annotated') — Annotated special form
#    (mamba: returns False)
assert hasattr(typing, "Annotated") == True; _ledger.append(1)

# 5) hasattr(typing, 'Self') — Self special form
#    (mamba: returns False)
assert hasattr(typing, "Self") == True; _ledger.append(1)

# 6) type(typing.TypeVar('T')).__name__ == 'TypeVar' — TypeVar instance
#    (mamba: returns 'NoneType' — TypeVar factory returns None)
assert type(typing.TypeVar("T")).__name__ == "TypeVar"; _ledger.append(1)

# 7) hasattr(inspect, 'Parameter') — Parameter class
#    (mamba: returns False)
assert hasattr(inspect, "Parameter") == True; _ledger.append(1)

# 8) hasattr(inspect, 'Signature') — Signature class
#    (mamba: returns False)
assert hasattr(inspect, "Signature") == True; _ledger.append(1)

# 9) inspect.isfunction(_myfn) — isfunction recognises def-functions
#    (mamba: returns False — predicate never positive)
assert inspect.isfunction(_myfn) == True; _ledger.append(1)

# 10) inspect.isclass(int) — isclass recognises builtin classes
#     (mamba: returns False — predicate never positive)
assert inspect.isclass(int) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typing_inspect_silent {sum(_ledger)} asserts")
