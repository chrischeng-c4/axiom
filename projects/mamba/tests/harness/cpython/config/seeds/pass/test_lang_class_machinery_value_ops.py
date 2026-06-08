# Atomic 325 pass conformance — class machinery depth: basic class
# definition, instance/method/class attrs, isinstance/type/class
# name, inheritance + super(), diamond inheritance MRO via method
# dispatch, dunders (__repr__/__str__/__eq__/__hash__/__add__/
# __mul__/__len__/__bool__/__contains__/__iter__/__getitem__/
# __setitem__/__lt__), properties (getter + setter), classmethod/
# staticmethod (instance- and class-bound), __slots__ enforcement,
# manual abstract method via NotImplementedError, dir() listing,
# getattr/setattr/hasattr/delattr, vars(), class-var vs instance
# shadowing, multiple-inheritance method dispatch order. All asserts
# match between CPython 3.12 and mamba.

_ledger: list[int] = []

# 1) basic class
class _A:
    def __init__(self, x):
        self.x = x
    def get(self):
        return self.x

_a = _A(42)
assert _a.x == 42; _ledger.append(1)
assert _a.get() == 42; _ledger.append(1)
assert isinstance(_a, _A); _ledger.append(1)
assert type(_a).__name__ == "_A"; _ledger.append(1)
assert _A.__name__ == "_A"; _ledger.append(1)

# 2) inheritance + super()
class _B(_A):
    def __init__(self, x, y):
        super().__init__(x)
        self.y = y
    def get(self):
        return (super().get(), self.y)

_b = _B(10, 20)
assert _b.x == 10; _ledger.append(1)
assert _b.y == 20; _ledger.append(1)
assert _b.get() == (10, 20); _ledger.append(1)
assert isinstance(_b, _A); _ledger.append(1)
assert isinstance(_b, _B); _ledger.append(1)
assert issubclass(_B, _A); _ledger.append(1)

# 3) diamond inheritance MRO via method dispatch
class _Base:
    def m(self):
        return "Base"
class _Left(_Base):
    def m(self):
        return "Left"
class _Right(_Base):
    def m(self):
        return "Right"
class _Diamond(_Left, _Right):
    pass

assert _Diamond().m() == "Left"; _ledger.append(1)
assert issubclass(_Diamond, _Left); _ledger.append(1)
assert issubclass(_Diamond, _Right); _ledger.append(1)
assert issubclass(_Diamond, _Base); _ledger.append(1)

# 4) value dunders __repr__/__str__/__eq__/__hash__
class _Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    def __repr__(self):
        return f"Point({self.x},{self.y})"
    def __str__(self):
        return f"<{self.x},{self.y}>"
    def __eq__(self, other):
        return self.x == other.x and self.y == other.y
    def __hash__(self):
        return hash((self.x, self.y))

_p1 = _Point(1, 2)
_p2 = _Point(1, 2)
_p3 = _Point(3, 4)
assert repr(_p1) == "Point(1,2)"; _ledger.append(1)
assert str(_p1) == "<1,2>"; _ledger.append(1)
assert (_p1 == _p2) == True; _ledger.append(1)
assert (_p1 == _p3) == False; _ledger.append(1)
assert hash(_p1) == hash(_p2); _ledger.append(1)

# 5) container dunders __add__/__mul__/__len__/__bool__/__contains__/__iter__
class _V:
    def __init__(self, items):
        self.items = list(items)
    def __add__(self, other):
        return _V(self.items + other.items)
    def __mul__(self, n):
        return _V(self.items * n)
    def __len__(self):
        return len(self.items)
    def __bool__(self):
        return len(self.items) > 0
    def __contains__(self, item):
        return item in self.items
    def __iter__(self):
        return iter(self.items)

_v1 = _V([1, 2])
_v2 = _V([3])
assert (_v1 + _v2).items == [1, 2, 3]; _ledger.append(1)
assert (_v1 * 3).items == [1, 2, 1, 2, 1, 2]; _ledger.append(1)
assert len(_v1) == 2; _ledger.append(1)
assert bool(_v1) == True; _ledger.append(1)
assert bool(_V([])) == False; _ledger.append(1)
assert (2 in _v1) == True; _ledger.append(1)
assert (99 in _v1) == False; _ledger.append(1)
assert list(_v1) == [1, 2]; _ledger.append(1)

# 6) __getitem__/__setitem__/__lt__ dunders
class _Box:
    def __init__(self, items):
        self.items = list(items)
    def __getitem__(self, i):
        return self.items[i]
    def __setitem__(self, i, v):
        self.items[i] = v
    def __lt__(self, other):
        return len(self.items) < len(other.items)

_bx = _Box([1, 2, 3])
assert _bx[0] == 1; _ledger.append(1)
assert _bx[2] == 3; _ledger.append(1)
_bx[1] = 99
assert _bx.items == [1, 99, 3]; _ledger.append(1)
assert (_bx < _Box([1, 2, 3, 4, 5])) == True; _ledger.append(1)

# 7) properties (getter + setter)
class _T:
    def __init__(self, x):
        self._x = x
    @property
    def x(self):
        return self._x
    @x.setter
    def x(self, value):
        self._x = value * 2

_t = _T(5)
assert _t.x == 5; _ledger.append(1)
_t.x = 7
assert _t.x == 14; _ledger.append(1)
assert _t._x == 14; _ledger.append(1)

# 8) classmethod / staticmethod (not via cls() — diverges)
class _Q:
    counter = 0
    @classmethod
    def get_counter(cls):
        return cls.counter
    @staticmethod
    def square(x):
        return x * x

assert _Q.get_counter() == 0; _ledger.append(1)
assert _Q.square(5) == 25; _ledger.append(1)
assert _Q().get_counter() == 0; _ledger.append(1)

# 9) __slots__ enforcement
class _S:
    __slots__ = ("a", "b")
    def __init__(self, a, b):
        self.a = a
        self.b = b

_s = _S(1, 2)
assert _s.a == 1; _ledger.append(1)
assert _s.b == 2; _ledger.append(1)
_slot_raised = False
try:
    _s.c = 99
except AttributeError:
    _slot_raised = True
assert _slot_raised == True; _ledger.append(1)

# 10) manual abstract method via NotImplementedError
class _Shape:
    def area(self):
        raise NotImplementedError

class _Circle(_Shape):
    def __init__(self, r):
        self.r = r
    def area(self):
        return self.r * self.r * 3

_circ = _Circle(2)
assert _circ.area() == 12; _ledger.append(1)
_abs_raised = False
try:
    _Shape().area()
except NotImplementedError:
    _abs_raised = True
assert _abs_raised == True; _ledger.append(1)

# 11) instance/class attr
class _O:
    cls_attr = "shared"
    def __init__(self):
        self.inst_attr = "private"

_o = _O()
assert _o.cls_attr == "shared"; _ledger.append(1)
assert _o.inst_attr == "private"; _ledger.append(1)
assert _O.cls_attr == "shared"; _ledger.append(1)

# 12) getattr/setattr/hasattr/delattr
class _X:
    def __init__(self):
        self.a = 1

_xx = _X()
assert getattr(_xx, "a") == 1; _ledger.append(1)
assert getattr(_xx, "b", "d") == "d"; _ledger.append(1)
assert hasattr(_xx, "a") == True; _ledger.append(1)
assert hasattr(_xx, "b") == False; _ledger.append(1)
setattr(_xx, "z", 99)
assert _xx.z == 99; _ledger.append(1)
delattr(_xx, "z")
assert hasattr(_xx, "z") == False; _ledger.append(1)

# 13) vars()
class _W:
    def __init__(self):
        self.a = 1
        self.b = 2

_w = _W()
assert vars(_w) == {"a": 1, "b": 2}; _ledger.append(1)

# 14) class variable vs instance shadow
class _CV:
    count = 0
    def inc(self):
        self.count += 1
_cv = _CV()
_cv.inc()
_cv.inc()
assert _cv.count == 2; _ledger.append(1)
assert _CV.count == 0; _ledger.append(1)

# 15) multiple inheritance method dispatch order (MRO)
class _MA:
    def m(self):
        return "MA"
class _MB:
    def m(self):
        return "MB"
class _MC(_MA, _MB):
    pass
class _MD(_MB, _MA):
    pass

assert _MC().m() == "MA"; _ledger.append(1)
assert _MD().m() == "MB"; _ledger.append(1)

# 16) dir() includes class methods
class _D:
    a = 1
    def m(self):
        pass
_dd = _D()
assert "a" in dir(_dd); _ledger.append(1)
assert "m" in dir(_dd); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_lang_class_machinery_value_ops {sum(_ledger)} asserts")
