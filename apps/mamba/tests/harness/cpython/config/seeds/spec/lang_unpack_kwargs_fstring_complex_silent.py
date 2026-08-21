# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `[*x, *y]` and `(*x, *y)` literal
# unpacking (the documented "starred expressions inside a list/tuple
# display unpack the iterable items in place" — mamba produces an
# empty list/tuple instead of the concatenated items), `f(**kw)` call
# unpacking (the documented "double-starred argument unpacking passes
# each (key, value) pair as a keyword argument" — mamba drops the
# kwargs so the receiver sees an empty dict), `@classmethod`
# decorator (the documented "@classmethod binds the class as the
# first argument so `cls.__name__` resolves to the class name string"
# — mamba returns None from the classmethod call), `f"{...!a}"`
# ascii-conversion (the documented "the !a conversion calls ascii()
# producing a backslash-escaped representation of non-ASCII chars" —
# mamba does not escape, e.g. 'café' remains literal), nested
# f-string format spec (the documented "expressions inside the format
# spec are evaluated and substituted before applying the spec" —
# mamba ignores the substituted width spec), scientific-notation
# float literal `1.5e3` (the documented "scientific notation produces
# a float" — mamba's `1.5e3` evaluates to a boxed-handle int), the
# `.real` attribute on a complex literal (the documented "complex
# objects expose `.real` returning a float" — mamba returns a
# boxed-handle int), keyword-only-marker function arity (the
# documented "after `*` only keyword arguments are permitted" — mamba
# silently accepts a positional argument), and `__init_subclass__`
# hook (the documented "subclasses invoke the parent's
# `__init_subclass__` at class-definition time" — mamba never invokes
# the hook, so a sibling counter stays at 0).
# Ten-pack pinned to atomic 251.
#
# Behavioral edges that CONFORM on mamba (15 dunder operator methods
# __add__/__eq__/__lt__/__le__/__gt__/__ge__/__hash__/__getitem__/
# __setitem__/__contains__/__call__/__enter__/__exit__/__len__/
# __bool__; walrus in if/while/comprehension; starred unpacking in
# tuple-assignment a,*b,c / *a,b / a,*b / *args call expansion /
# {**a,**b} dict literal; match statement literal/sequence-rest/
# mapping/class-via-__match_args__; __slots__ basic + blocks extra
# attr; descriptor __get__; multiple-inheritance MRO left-first +
# 4-class __mro__ length; super() chain through 3 levels; function
# decorator + parameterized decorator + @staticmethod; f-string
# basic interp + !s + format spec + width fill; literal forms
# hex/octal/binary/underscore int + complex literal; nested-with
# enter/exit ordering + single-line multi-context; kwonly happy +
# posonly happy + default-arg both modes; chained comparison
# 1<2<3 / 1<3>2 / 4-chain / ==-chain; lambda no-arg/one-arg/two-arg/
# default-arg) are covered in the matching pass fixture
# `test_dunder_walrus_match_decorator_value_ops`.
from typing import Any


class _CDec:
    @classmethod
    def c(cls):
        return cls.__name__


class _Counter:
    count = 0
    def __init_subclass__(cls, **kw):
        super().__init_subclass__(**kw)
        _Counter.count += 1


class _Child1(_Counter):
    pass


class _Child2(_Counter):
    pass


def _kwonly_func(a, *, b):
    return (a, b)


_ledger: list[int] = []

# 1) `[*x, *y]` list-literal unpacking — items must concatenate
#    (mamba: returns []; the inline starred items are dropped)
assert [*[1, 2], *[3, 4]] == [1, 2, 3, 4]; _ledger.append(1)

# 2) `(*x, *y)` tuple-literal unpacking — items must concatenate
#    (mamba: returns (); the inline starred items are dropped)
assert (*[1, 2], *[3, 4]) == (1, 2, 3, 4); _ledger.append(1)

# 3) `f(**kw)` call unpacking — receiver sees keyword pairs
#    (mamba: kwargs are dropped, receiver sees empty mapping)
def _dstar_call(**kw) -> list:
    return sorted(kw.items())
assert _dstar_call(**{"a": 1, "b": 2}) == [("a", 1), ("b", 2)]; _ledger.append(1)

# 4) @classmethod bound class — `cls.__name__` returns class name
#    (mamba: returns None from the classmethod call)
assert _CDec.c() == "_CDec"; _ledger.append(1)

# 5) f-string `!a` conversion — non-ASCII must be backslash-escaped
#    (mamba: leaves non-ASCII characters literal)
assert f"x={'café'!a}" == "x='caf\\xe9'"; _ledger.append(1)

# 6) Nested {} inside f-string format spec — width spec applied
#    (mamba: ignores the substituted width spec, produces just '42')
assert f"{42:{'>'}{5}}" == "   42"; _ledger.append(1)

# 7) Scientific notation float literal — evaluates to float
#    (mamba: returns a boxed-handle int)
def _scientific() -> Any:
    return 1.5e3
assert _scientific() == 1500.0; _ledger.append(1)

# 8) complex(3+4j).real — must return float 3.0
#    (mamba: returns a boxed-handle int)
def _complex_real() -> Any:
    return (3 + 4j).real
assert _complex_real() == 3.0; _ledger.append(1)

# 9) `def f(a, *, b)` — b is keyword-only, positional must fail
#    (mamba: silently accepts positional, no TypeError)
def _kwonly_blocks_positional() -> str:
    try:
        _kwonly_func(1, 2)  # type: ignore[misc]
        return "allowed"
    except TypeError:
        return "blocked"
assert _kwonly_blocks_positional() == "blocked"; _ledger.append(1)

# 10) `__init_subclass__` hook — invoked per subclass at class creation
#     (mamba: never invokes the hook, so count stays at 0)
assert _Counter.count == 2; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_unpack_kwargs_fstring_complex_silent {sum(_ledger)} asserts")
