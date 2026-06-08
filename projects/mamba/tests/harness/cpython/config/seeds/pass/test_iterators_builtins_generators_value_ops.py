# Atomic 233 pass conformance — builtin functions / iterators / generators /
# comprehensions / magic methods / class machinery / exception handling /
# closures / walrus / format ops that match between CPython 3.12 and mamba.


class _Adder:
    def __init__(self, x):
        self.x = x

    def __add__(self, other):
        return _Adder(self.x + other.x)

    def __repr__(self):
        return f"_Adder({self.x})"

    def __eq__(self, other):
        return isinstance(other, _Adder) and self.x == other.x

    def __hash__(self):
        return hash(self.x)

    def __lt__(self, other):
        return self.x < other.x

    def __bool__(self):
        return self.x != 0

    def __len__(self):
        return self.x

    def __contains__(self, val):
        return val == self.x

    def __call__(self, y):
        return self.x + y


class _Base:
    def greet(self):
        return "base"


class _Child(_Base):
    def greet(self):  # type: ignore[override]
        return "child:" + super().greet()


class _C2:
    @classmethod
    def cm(cls):
        return cls.__name__

    @staticmethod
    def sm():
        return "static"

    @property
    def p(self):
        return 42


class _MyErr(Exception):
    pass


def _gen_simple():
    yield 1
    yield 2
    yield 3


def _gen_from():
    yield from [1, 2, 3]
    yield from "ab"


def _make_counter():
    count = 0

    def inc():
        nonlocal count
        count += 1
        return count

    return inc


_ledger: list[int] = []

# 1) builtins — sum/max/min/any/all/map/filter/zip/enumerate value ops
assert sum([1, 2, 3]) == 6; _ledger.append(1)
assert sum([1, 2, 3], 10) == 16; _ledger.append(1)
assert sum((1, 2, 3)) == 6; _ledger.append(1)
assert max([3, 1, 4]) == 4; _ledger.append(1)
assert max([-3, 1, -4], key=abs) == -4; _ledger.append(1)
assert min([3, 1, 4]) == 1; _ledger.append(1)
assert min([-3, 1, -4], key=abs) == 1; _ledger.append(1)
assert max([], default=99) == 99; _ledger.append(1)
assert min([], default=99) == 99; _ledger.append(1)
assert max(3, 1) == 3; _ledger.append(1)
assert min(3, 1) == 1; _ledger.append(1)
assert any([0, 0, 1]) == True; _ledger.append(1)
assert any([0, 0, 0]) == False; _ledger.append(1)
assert any([]) == False; _ledger.append(1)
assert all([1, 1, 1]) == True; _ledger.append(1)
assert all([1, 0, 1]) == False; _ledger.append(1)
assert all([]) == True; _ledger.append(1)
assert list(map(lambda x: x * 2, [1, 2, 3])) == [2, 4, 6]; _ledger.append(1)
assert list(filter(lambda x: x % 2 == 0, [1, 2, 3, 4])) == [2, 4]; _ledger.append(1)
assert list(zip([1, 2, 3], ["a", "b", "c"])) == [(1, "a"), (2, "b"), (3, "c")]; _ledger.append(1)
assert list(zip([1, 2], ["a", "b"], strict=True)) == [(1, "a"), (2, "b")]; _ledger.append(1)
assert list(enumerate(["a", "b", "c"])) == [(0, "a"), (1, "b"), (2, "c")]; _ledger.append(1)
assert list(enumerate(["a", "b"], start=10)) == [(10, "a"), (11, "b")]; _ledger.append(1)
assert sorted([-3, 1, -4, 2], key=abs) == [1, 2, -3, -4]; _ledger.append(1)
assert sorted([1, 2, 3], reverse=True) == [3, 2, 1]; _ledger.append(1)

# 2) type ctors + chr/ord/hash/repr/bool
assert chr(97) == "a"; _ledger.append(1)
assert ord("a") == 97; _ledger.append(1)
assert hash(42) == 42; _ledger.append(1)
assert repr([1, 2]) == "[1, 2]"; _ledger.append(1)
assert bool([]) == False; _ledger.append(1)
assert bool([1]) == True; _ledger.append(1)
assert bool("") == False; _ledger.append(1)
assert bool("x") == True; _ledger.append(1)
assert bool(0) == False; _ledger.append(1)
assert bool(1) == True; _ledger.append(1)
assert bool(None) == False; _ledger.append(1)
assert int("42") == 42; _ledger.append(1)
assert float("3.14") == 3.14; _ledger.append(1)
assert complex(3, 4) == (3 + 4j); _ledger.append(1)
assert bytes([65, 66]) == b"AB"; _ledger.append(1)
assert list((1, 2, 3)) == [1, 2, 3]; _ledger.append(1)
assert tuple([1, 2, 3]) == (1, 2, 3); _ledger.append(1)
assert set([1, 2, 2, 3]) == {1, 2, 3}; _ledger.append(1)
assert frozenset([1, 2, 3]) == frozenset({1, 2, 3}); _ledger.append(1)

# 3) callable/hasattr/getattr/setattr/type/isinstance/issubclass
class _Foo:
    x = 10
    def m(self):
        return 1


_inst = _Foo()
assert callable(_Foo) == True; _ledger.append(1)
assert callable(_inst) == False; _ledger.append(1)
assert callable(_inst.m) == True; _ledger.append(1)
assert hasattr(_inst, "x") == True; _ledger.append(1)
assert hasattr(_inst, "zzz") == False; _ledger.append(1)
assert getattr(_inst, "x") == 10; _ledger.append(1)
assert getattr(_inst, "zzz", 99) == 99; _ledger.append(1)
_inst2 = _Foo()
setattr(_inst2, "y", 42)
assert getattr(_inst2, "y") == 42; _ledger.append(1)
assert type(_inst).__name__ == "_Foo"; _ledger.append(1)
assert isinstance(1, int) == True; _ledger.append(1)
assert isinstance("a", str) == True; _ledger.append(1)
assert isinstance([], list) == True; _ledger.append(1)
assert isinstance({}, dict) == True; _ledger.append(1)
assert isinstance((), tuple) == True; _ledger.append(1)
assert isinstance(1, (int, float)) == True; _ledger.append(1)
assert issubclass(bool, int) == True; _ledger.append(1)
assert type(1).__name__ == "int"; _ledger.append(1)
assert type("a").__name__ == "str"; _ledger.append(1)
assert type([]).__name__ == "list"; _ledger.append(1)
assert type({}).__name__ == "dict"; _ledger.append(1)
assert type((1,)).__name__ == "tuple"; _ledger.append(1)
assert type(set()).__name__ == "set"; _ledger.append(1)
assert type(None).__name__ == "NoneType"; _ledger.append(1)

# 4) iterators — iter/next/StopIteration/reversed
assert list(iter([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
_it = iter([1, 2, 3])
assert next(_it) == 1; _ledger.append(1)
assert next(_it) == 2; _ledger.append(1)
assert next(_it) == 3; _ledger.append(1)
_it_empty = iter([])
assert next(_it_empty, "default") == "default"; _ledger.append(1)
try:
    _it_stop = iter([1])
    next(_it_stop)
    next(_it_stop)
    _ok = False
except StopIteration:
    _ok = True
assert _ok == True; _ledger.append(1)
assert list(iter("abc")) == ["a", "b", "c"]; _ledger.append(1)
assert list(iter((1, 2, 3))) == [1, 2, 3]; _ledger.append(1)
assert sorted(iter({"a": 1, "b": 2})) == ["a", "b"]; _ledger.append(1)
assert sorted(iter({1, 2, 3})) == [1, 2, 3]; _ledger.append(1)
assert list(reversed([1, 2, 3])) == [3, 2, 1]; _ledger.append(1)
assert list(reversed("abc")) == ["c", "b", "a"]; _ledger.append(1)
assert list(reversed((1, 2, 3))) == [3, 2, 1]; _ledger.append(1)
assert list(reversed(range(3))) == [2, 1, 0]; _ledger.append(1)

# 5) generators — simple/yield-from/genexpr
assert list(_gen_simple()) == [1, 2, 3]; _ledger.append(1)
assert list(_gen_from()) == [1, 2, 3, "a", "b"]; _ledger.append(1)
assert list(x * 2 for x in range(5)) == [0, 2, 4, 6, 8]; _ledger.append(1)

# 6) comprehensions — list/dict/set/nested/flat/conditional
assert [x * 2 for x in range(5)] == [0, 2, 4, 6, 8]; _ledger.append(1)
assert [x for x in range(10) if x % 2 == 0] == [0, 2, 4, 6, 8]; _ledger.append(1)
assert {x: x * 2 for x in range(3)} == {0: 0, 1: 2, 2: 4}; _ledger.append(1)
assert sorted({x % 3 for x in range(10)}) == [0, 1, 2]; _ledger.append(1)
assert [[y for y in range(x)] for x in range(4)] == [[], [0], [0, 1], [0, 1, 2]]; _ledger.append(1)
assert [y for x in [[1, 2], [3, 4]] for y in x] == [1, 2, 3, 4]; _ledger.append(1)
assert ["even" if x % 2 == 0 else "odd" for x in range(4)] == ["even", "odd", "even", "odd"]; _ledger.append(1)

# 7) magic methods on user class
assert (_Adder(1) + _Adder(2)).x == 3; _ledger.append(1)
assert repr(_Adder(5)) == "_Adder(5)"; _ledger.append(1)
assert (_Adder(1) == _Adder(1)) == True; _ledger.append(1)
assert (_Adder(1) != _Adder(2)) == True; _ledger.append(1)
assert hash(_Adder(7)) == hash(7); _ledger.append(1)
assert (_Adder(1) < _Adder(2)) == True; _ledger.append(1)
assert bool(_Adder(0)) == False; _ledger.append(1)
assert bool(_Adder(5)) == True; _ledger.append(1)
assert len(_Adder(5)) == 5; _ledger.append(1)
assert (5 in _Adder(5)) == True; _ledger.append(1)
assert _Adder(10)(5) == 15; _ledger.append(1)

# 8) class machinery — super/MRO/classmethod/staticmethod/property
assert _Child().greet() == "child:base"; _ledger.append(1)
assert [c.__name__ for c in _Child.__mro__] == ["_Child", "_Base", "object"]; _ledger.append(1)
assert _C2.cm() == "_C2"; _ledger.append(1)
assert _C2.sm() == "static"; _ledger.append(1)
assert _C2().p == 42; _ledger.append(1)

# 9) exception handling — try/except/finally + custom subclass
try:
    _ = 1 / 0
    _ok = False
except ZeroDivisionError:
    _ok = True
assert _ok == True; _ledger.append(1)
try:
    _ = {}["x"]
    _ok = False
except KeyError:
    _ok = True
assert _ok == True; _ledger.append(1)
try:
    _ = int("abc")
    _ok = False
except ValueError:
    _ok = True
assert _ok == True; _ledger.append(1)
try:
    raise _MyErr("custom")
    _ok = False
except _MyErr as _e:
    _ok = str(_e) == "custom"
assert _ok; _ledger.append(1)
_log: list[str] = []
try:
    _log.append("try")
    raise ValueError("x")
except ValueError:
    _log.append("except")
finally:
    _log.append("finally")
assert _log == ["try", "except", "finally"]; _ledger.append(1)

# 10) closure / nonlocal
_counter = _make_counter()
assert _counter() == 1; _ledger.append(1)
assert _counter() == 2; _ledger.append(1)
assert _counter() == 3; _ledger.append(1)

# 11) walrus / assignment expressions
_nums = [1, 2, 3, 4, 5]
assert [y for x in _nums if (y := x * 2) > 4] == [6, 8, 10]; _ledger.append(1)
if (_n := 10) > 5:
    _walrus_val = _n
else:
    _walrus_val = -1
assert _walrus_val == 10; _ledger.append(1)

# 12) len of containers
assert len({}) == 0; _ledger.append(1)
assert len([]) == 0; _ledger.append(1)
assert len("") == 0; _ledger.append(1)
assert len(set()) == 0; _ledger.append(1)
assert len(tuple()) == 0; _ledger.append(1)
assert len(range(5)) == 5; _ledger.append(1)
assert len({"a": 1, "b": 2}) == 2; _ledger.append(1)
assert len([1, 2, 3]) == 3; _ledger.append(1)
assert len("abc") == 3; _ledger.append(1)

# 13) format() / .format() / %-format / f-string
assert format(255, "x") == "ff"; _ledger.append(1)
assert format(255, "04x") == "00ff"; _ledger.append(1)
assert format(3.14, ".2f") == "3.14"; _ledger.append(1)
assert "{}+{}".format(1, 2) == "1+2"; _ledger.append(1)
assert "{0}".format("x") == "x"; _ledger.append(1)
assert "{0}+{1}".format(1, 2) == "1+2"; _ledger.append(1)
assert "%d" % 5 == "5"; _ledger.append(1)
assert "%s" % "x" == "x"; _ledger.append(1)
assert "%d-%s" % (5, "x") == "5-x"; _ledger.append(1)
assert f"v={1 + 1}" == "v=2"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_iterators_builtins_generators_value_ops {sum(_ledger)} asserts")
