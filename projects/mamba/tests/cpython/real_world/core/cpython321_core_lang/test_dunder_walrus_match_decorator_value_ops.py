# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "test_dunder_walrus_match_decorator_value_ops"
# subject = "cpython321.test_dunder_walrus_match_decorator_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_dunder_walrus_match_decorator_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_dunder_walrus_match_decorator_value_ops: execute CPython 3.12 seed test_dunder_walrus_match_decorator_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 251 pass conformance — dunder operator methods (__add__/__eq__/
# __lt__/__le__/__gt__/__ge__/__hash__/__getitem__/__setitem__/
# __contains__/__call__/__enter__/__exit__/__len__/__bool__) +
# walrus operator (in if, while, comprehension) + starred unpacking
# (a,*b,c / *a,b / a,*b / *args expansion / {**a,**b} dict literal) +
# match statement (literal int, sequence pattern with rest, mapping
# pattern, class pattern via __match_args__) + class advanced features
# (__slots__ blocking extra attr, descriptor __get__, multiple-
# inheritance MRO left-first, super() chain through 3 levels) +
# decorators (function decorator, parameterized decorator,
# @staticmethod) + f-string (basic interp, !s, format spec, width
# fill) + literal forms (hex/octal/binary/underscore int, complex
# literal) + multi-context with (nested, single-line) + arg modes
# (kwonly happy, posonly happy, default arg) + chained comparison
# (1<2<3, 1<3>2, ==-chain) + lambda forms (no-arg, one-arg, two-arg,
# default). All asserts match between CPython 3.12 and mamba.


class _Add:
    def __init__(self, v):
        self.v = v
    def __add__(self, o):
        return _Add(self.v + o.v)
    def __eq__(self, o):
        return isinstance(o, _Add) and self.v == o.v
    def __repr__(self):
        return f"_Add({self.v})"


class _Cmp:
    def __init__(self, v):
        self.v = v
    def __lt__(self, o):
        return self.v < o.v
    def __le__(self, o):
        return self.v <= o.v
    def __gt__(self, o):
        return self.v > o.v
    def __ge__(self, o):
        return self.v >= o.v


class _Hash:
    def __init__(self, v):
        self.v = v
    def __hash__(self):
        return self.v * 7
    def __eq__(self, o):
        return isinstance(o, _Hash) and self.v == o.v


class _GetItem:
    def __getitem__(self, k):
        return f"got-{k}"


class _SetItem:
    def __init__(self):
        self.store = {}
    def __setitem__(self, k, v):
        self.store[k] = v


class _Contains:
    def __init__(self, items):
        self.items = items
    def __contains__(self, k):
        return k in self.items


class _Call:
    def __call__(self, x, y):
        return x + y


class _CM:
    def __enter__(self):
        return "in"
    def __exit__(self, *a):
        return False


class _Len:
    def __len__(self):
        return 42


class _Bool:
    def __init__(self, v):
        self.v = v
    def __bool__(self):
        return self.v


class _Pt:
    __match_args__ = ("x", "y")
    def __init__(self, x, y):
        self.x = x
        self.y = y


class _SlotCls:
    __slots__ = ("x", "y")
    def __init__(self, x, y):
        self.x = x
        self.y = y


class _Desc:
    def __get__(self, obj, objtype=None):
        return "desc-value"


class _UsesDesc:
    d = _Desc()


class _MIA:
    def f(self):
        return "A"


class _MIB:
    def f(self):
        return "B"


class _MIC(_MIA, _MIB):
    pass


class _SuperBase:
    def f(self):
        return "Base"


class _SuperMid(_SuperBase):
    def f(self):
        return "Mid->" + super().f()


class _SuperSub(_SuperMid):
    def f(self):
        return "Sub->" + super().f()


class _CDec:
    @staticmethod
    def s():
        return "sm"


_ledger: list[int] = []

# 1) Dunder operator methods
assert (_Add(1) + _Add(2)).v == 3; _ledger.append(1)
assert (_Add(3) == _Add(3)) == True; _ledger.append(1)
assert (_Add(3) == _Add(4)) == False; _ledger.append(1)
assert (_Cmp(1) < _Cmp(2)) == True; _ledger.append(1)
assert (_Cmp(2) <= _Cmp(2)) == True; _ledger.append(1)
assert (_Cmp(3) > _Cmp(1)) == True; _ledger.append(1)
assert (_Cmp(3) >= _Cmp(3)) == True; _ledger.append(1)
assert hash(_Hash(3)) == 21; _ledger.append(1)
assert (_Hash(5) in {_Hash(5), _Hash(6)}) == True; _ledger.append(1)
assert _GetItem()[42] == "got-42"; _ledger.append(1)
assert _GetItem()["k"] == "got-k"; _ledger.append(1)

def _setitem() -> dict:
    o = _SetItem()
    o["a"] = 1
    return o.store
assert _setitem() == {"a": 1}; _ledger.append(1)

assert (2 in _Contains([1, 2, 3])) == True; _ledger.append(1)
assert (9 in _Contains([1, 2, 3])) == False; _ledger.append(1)
assert _Call()(1, 2) == 3; _ledger.append(1)

def _ctx_use() -> str:
    with _CM() as r:
        return r
assert _ctx_use() == "in"; _ledger.append(1)

assert len(_Len()) == 42; _ledger.append(1)
assert bool(_Bool(True)) == True; _ledger.append(1)
assert bool(_Bool(False)) == False; _ledger.append(1)

# 2) Walrus operator
def _walrus_if() -> int:
    if (n := 10) > 5:
        return n
    return -1
assert _walrus_if() == 10; _ledger.append(1)

def _walrus_while() -> list:
    out: list = []
    data = iter([1, 2, 3])
    while (x := next(data, None)) is not None:
        out.append(x)
    return out
assert _walrus_while() == [1, 2, 3]; _ledger.append(1)

def _walrus_compr() -> list:
    nums = [1, 2, 3, 4]
    return [y for x in nums if (y := x * 2) > 4]
assert _walrus_compr() == [6, 8]; _ledger.append(1)

# 3) Starred unpacking
def _star_mid():
    a, *b, c = [1, 2, 3, 4, 5]
    return (a, b, c)
assert _star_mid() == (1, [2, 3, 4], 5); _ledger.append(1)

def _star_front():
    *a, b = [1, 2, 3]
    return (a, b)
assert _star_front() == ([1, 2], 3); _ledger.append(1)

def _star_back():
    a, *b = [1, 2, 3]
    return (a, b)
assert _star_back() == (1, [2, 3]); _ledger.append(1)

def _star_args(*args) -> int:
    return sum(args)
assert _star_args(*[1, 2, 3]) == 6; _ledger.append(1)

assert {**{"a": 1}, **{"b": 2}} == {"a": 1, "b": 2}; _ledger.append(1)

# 4) Match statement
def _match_literal(x: int) -> str:
    match x:
        case 1:
            return "one"
        case 2:
            return "two"
        case _:
            return "other"
assert _match_literal(1) == "one"; _ledger.append(1)
assert _match_literal(2) == "two"; _ledger.append(1)
assert _match_literal(99) == "other"; _ledger.append(1)

def _match_seq(x):
    match x:
        case [1, 2, 3]:
            return "exact"
        case [1, *rest]:
            return ("rest", rest)
        case _:
            return "other"
assert _match_seq([1, 2, 3]) == "exact"; _ledger.append(1)
assert _match_seq([1, 9, 8]) == ("rest", [9, 8]); _ledger.append(1)

def _match_mapping(x):
    match x:
        case {"a": v}:
            return ("a-key", v)
        case _:
            return "miss"
assert _match_mapping({"a": 42}) == ("a-key", 42); _ledger.append(1)

def _match_class(p):
    match p:
        case _Pt(0, 0):
            return "origin"
        case _Pt(x, 0):
            return ("x-axis", x)
        case _Pt(x, y):
            return (x, y)
        case _:
            return "?"
assert _match_class(_Pt(0, 0)) == "origin"; _ledger.append(1)
assert _match_class(_Pt(5, 0)) == ("x-axis", 5); _ledger.append(1)
assert _match_class(_Pt(3, 4)) == (3, 4); _ledger.append(1)

# 5) Class advanced — slots, descriptor, MRO, super
def _slots_basic():
    s = _SlotCls(1, 2)
    return (s.x, s.y)
assert _slots_basic() == (1, 2); _ledger.append(1)

def _slots_blocked() -> str:
    s = _SlotCls(1, 2)
    try:
        s.z = 3  # type: ignore
        return "allowed"
    except AttributeError:
        return "blocked"
assert _slots_blocked() == "blocked"; _ledger.append(1)

assert _UsesDesc().d == "desc-value"; _ledger.append(1)
assert _MIC().f() == "A"; _ledger.append(1)
assert len(_MIC.__mro__) == 4; _ledger.append(1)
assert _SuperSub().f() == "Sub->Mid->Base"; _ledger.append(1)

# 6) Decorators
def _dec(f):
    def wrapper(*a, **k):
        return "wrapped-" + str(f(*a, **k))
    return wrapper

@_dec
def _greet():
    return "hi"
assert _greet() == "wrapped-hi"; _ledger.append(1)

def _param_dec(prefix):
    def actual(f):
        def w(*a, **k):
            return prefix + str(f(*a, **k))
        return w
    return actual

@_param_dec("X-")
def _hello():
    return "hello"
assert _hello() == "X-hello"; _ledger.append(1)

assert _CDec.s() == "sm"; _ledger.append(1)

# 7) f-string basic forms
_v = 42
assert f"x={_v}" == "x=42"; _ledger.append(1)
assert f"x={'hi'!s}" == "x=hi"; _ledger.append(1)
assert f"{3.14159:.2f}" == "3.14"; _ledger.append(1)
assert f"{42:*>5}" == "***42"; _ledger.append(1)

# 8) Literal forms
assert 0xFF == 255; _ledger.append(1)
assert 0o17 == 15; _ledger.append(1)
assert 0b1010 == 10; _ledger.append(1)
assert 1_000_000 == 1000000; _ledger.append(1)
assert (1 + 2j) == complex(1, 2); _ledger.append(1)

# 9) Multi-context with
class _CtxR:
    def __init__(self, name, log):
        self.name = name
        self.log = log
    def __enter__(self):
        self.log.append(f"enter-{self.name}")
        return self.name
    def __exit__(self, *a):
        self.log.append(f"exit-{self.name}")
        return False

def _nested_with() -> list:
    log: list = []
    with _CtxR("a", log) as a:
        with _CtxR("b", log) as b:
            log.append(f"in-{a}-{b}")
    return log
assert _nested_with() == ["enter-a", "enter-b", "in-a-b", "exit-b", "exit-a"]; _ledger.append(1)

def _single_line_with() -> list:
    log: list = []
    with _CtxR("a", log) as a, _CtxR("b", log) as b:
        log.append(f"in-{a}-{b}")
    return log
assert _single_line_with() == ["enter-a", "enter-b", "in-a-b", "exit-b", "exit-a"]; _ledger.append(1)

# 10) Keyword-only / positional-only / default arg happy path
def _kwonly(a, *, b):
    return (a, b)
assert _kwonly(1, b=2) == (1, 2); _ledger.append(1)

def _posonly(a, b, /):
    return (a, b)
assert _posonly(1, 2) == (1, 2); _ledger.append(1)

def _default_arg(a, b=10):
    return (a, b)
assert _default_arg(1) == (1, 10); _ledger.append(1)
assert _default_arg(1, 2) == (1, 2); _ledger.append(1)

# 11) Chained comparison
assert (1 < 2 < 3) == True; _ledger.append(1)
assert (1 < 3 > 2) == True; _ledger.append(1)
assert (1 < 2 < 3 < 4) == True; _ledger.append(1)
assert (5 > 4 > 3 > 2) == True; _ledger.append(1)
assert (1 == 1 == 1) == True; _ledger.append(1)
assert (1 == 1 == 2) == False; _ledger.append(1)

# 12) Lambda forms
assert (lambda: 5)() == 5; _ledger.append(1)
assert (lambda x: x * 2)(3) == 6; _ledger.append(1)
assert (lambda x, y: x + y)(10, 20) == 30; _ledger.append(1)
assert (lambda x=7: x)() == 7; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_dunder_walrus_match_decorator_value_ops {sum(_ledger)} asserts")
