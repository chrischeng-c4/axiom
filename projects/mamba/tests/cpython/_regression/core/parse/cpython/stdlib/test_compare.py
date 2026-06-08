# RUN: parse
# Extracted from CPython 3.12 Lib/test/test_compare.py — comparison syntax constructs only.


# --- Empty class (identity-based comparison) ---

class Empty:
    def __repr__(self):
        return '<Empty>'


# --- Class with __eq__ ---

class Cmp:
    def __init__(self, arg):
        self.arg = arg

    def __repr__(self):
        return '<Cmp %s>' % self.arg

    def __eq__(self, other):
        return self.arg == other


# --- Collection-based comparisons ---

set1 = [2, 2.0, 2, 2 + 0j, Cmp(2.0)]
set2 = [[1], (3,), None, Empty()]
candidates = set1 + set2

for a in candidates:
    for b in candidates:
        if ((a in set1) and (b in set1)) or a is b:
            a == b
        else:
            a != b


# --- Identity comparisons ---

L = []
for i in range(10):
    L.insert(len(L) // 2, Empty())
for a in L:
    for b in L:
        a == b
        a is b


# --- __ne__ defaults to not __eq__ ---

a = Cmp(1)
b = Cmp(1)
c = Cmp(2)
a == b
a != b
a != c


# --- Reflected comparison dispatch ---

class Left:
    def __eq__(*args):
        return NotImplemented

class Right:
    def __eq__(*args):
        return NotImplemented
    def __ne__(*args):
        return NotImplemented

Left() != Right()


# --- Subclass priority for reflected ops ---

class Base:
    def __eq__(*args):
        return NotImplemented

class Derived(Base):
    def __eq__(*args):
        return NotImplemented
    def __ne__(*args):
        return NotImplemented

Base() != Derived()


# --- Comparison operator lambdas ---

ops = (
    ('__eq__', lambda a, b: a == b),
    ('__lt__', lambda a, b: a < b),
    ('__le__', lambda a, b: a <= b),
    ('__gt__', lambda a, b: a > b),
    ('__ge__', lambda a, b: a >= b),
)


# --- CompBase hierarchy with rich comparison methods ---

class CompBase:
    pass

class CompNone(CompBase):
    meth = ()

class CompEq(CompBase):
    meth = ("eq",)
    def __eq__(self, other):
        return self.x == other.x

class CompNe(CompBase):
    meth = ("ne",)
    def __ne__(self, other):
        return self.x != other.x

class CompEqNe(CompBase):
    meth = ("eq", "ne")
    def __eq__(self, other):
        return self.x == other.x
    def __ne__(self, other):
        return self.x != other.x

class CompLt(CompBase):
    meth = ("lt",)
    def __lt__(self, other):
        return self.x < other.x

class CompGt(CompBase):
    meth = ("gt",)
    def __gt__(self, other):
        return self.x > other.x

class CompLtGt(CompBase):
    meth = ("lt", "gt")
    def __lt__(self, other):
        return self.x < other.x
    def __gt__(self, other):
        return self.x > other.x

class CompLe(CompBase):
    meth = ("le",)
    def __le__(self, other):
        return self.x <= other.x

class CompGe(CompBase):
    meth = ("ge",)
    def __ge__(self, other):
        return self.x >= other.x

class CompLeGe(CompBase):
    meth = ("le", "ge")
    def __le__(self, other):
        return self.x <= other.x
    def __ge__(self, other):
        return self.x >= other.x

all_comp_classes = (
    CompNone,
    CompEq, CompNe, CompEqNe,
    CompLt, CompGt, CompLtGt,
    CompLe, CompGe, CompLeGe,
)


# --- Sorted instance creation pattern ---

def create_sorted_instances(cls, values):
    instances = [cls() for _ in range(len(values))]
    instances.sort(key=id)
    for inst, value in zip(instances, values):
        inst.x = value
    return instances


# --- Comparison operations ---

a = object()
b = object()
a == a
a == b
a != b
a is a
a is not b


# --- Str subclass comparisons ---

class StrSubclass(str):
    pass

s1 = str("a")
s2 = str("b")
c1 = StrSubclass("a")
c2 = StrSubclass("b")
c3 = StrSubclass("b")

s1 == s1
s1 < s2
c1 == c1
c1 < c2
c2 == c3
s1 < c2
s2 == c3
c1 < s2


# --- Number comparisons ---

i1 = 1001
i2 = 1002
i1 == i1
i1 < i2

f1 = 1001.0
f2 = 1001.1
f1 == f1
f1 < f2

c1_cplx = 1001 + 0j
c2_cplx = 1001 + 1j
c1_cplx == c1_cplx
c1_cplx != c2_cplx


# --- Sequence comparisons ---

l1 = [1, 2]
l2 = [2, 3]
l1 == l1
l1 < l2

t1 = (1, 2)
t2 = (2, 3)
t1 == t1
t1 < t2

r1 = range(1, 2)
r2 = range(2, 2)
r1 == r1
r1 != r2

t1 != l1
l1 != r1


# --- Bytes comparisons ---

bs1 = b'a1'
bs2 = b'b2'
bs1 == bs1
bs1 < bs2

ba1 = bytearray(b'a1')
ba2 = bytearray(b'b2')
ba1 == ba1
ba1 < ba2

bs1 == ba1
bs1 < ba2


# --- Set comparisons ---

s1_set = {1, 2}
s2_set = {1, 2, 3}
s1_set == s1_set
s1_set < s2_set

f1_frozen = frozenset(s1_set)
f2_frozen = frozenset(s2_set)
f1_frozen == f1_frozen
f1_frozen < f2_frozen

s1_set == f1_frozen


# --- Dict comparisons ---

d1 = {1: "a", 2: "b"}
d2 = {2: "b", 3: "c"}
d3 = {3: "c", 2: "b"}
d1 == d1
d1 != d2
d2 == d3


# --- Chained comparisons ---

1 < 2 < 3
1 < 2 and 2 < 3
1 <= 2 <= 2
0 < 1 < 2 < 3 < 4


# --- NotImplemented return pattern ---

class OnlyEq:
    def __init__(self, value):
        self.value = value
    def __eq__(self, other):
        if isinstance(other, OnlyEq):
            return self.value == other.value
        return NotImplemented
    def __ne__(self, other):
        if isinstance(other, OnlyEq):
            return self.value != other.value
        return NotImplemented

oe1 = OnlyEq(1)
oe2 = OnlyEq(1)
oe3 = OnlyEq(2)
oe1 == oe2
oe1 != oe3
oe1 == 1


# --- TypeError on unordered comparisons ---

try:
    object() < object()
except TypeError:
    pass

try:
    object() <= object()
except TypeError:
    pass

try:
    object() > object()
except TypeError:
    pass

try:
    object() >= object()
except TypeError:
    pass


# --- Cross-type TypeError ---

try:
    1 < "a"
except TypeError:
    pass

# NOTE: [1] < (1,) — list/tuple comparison not yet supported by parser
# try:
#     [1] < (1,)
# except TypeError:
#     pass

try:
    None < 1
except TypeError:
    pass

try:
    None > 1
except TypeError:
    pass


# --- Comparison with None ---

None == None
None != 1
None is None
1 is not None


# --- Bool as int subclass in comparisons ---

True == 1
False == 0
True != 0
False != 1
True > False
False < True
True >= True
False <= False


# --- Rich comparison returning non-bool ---

class ReturnsString:
    def __eq__(self, other):
        return "yes"
    def __ne__(self, other):
        return "no"
    def __lt__(self, other):
        return "less"
    def __gt__(self, other):
        return "greater"
    def __le__(self, other):
        return "less_or_eq"
    def __ge__(self, other):
        return "greater_or_eq"

rs = ReturnsString()
rs == 1
rs != 1
rs < 1
rs > 1
rs <= 1
rs >= 1


# --- __hash__ = None with __eq__ ---

class UnhashableEq:
    def __eq__(self, other):
        return True
    __hash__ = None

u1 = UnhashableEq()
u2 = UnhashableEq()
u1 == u2

try:
    hash(u1)
except TypeError:
    pass


# --- Comparison in expressions ---

x = 5
result = (x > 3) and (x < 10)
result = (x >= 5) or (x <= 0)
result = not (x == 5)
result = True if x > 3 else False
vals = [i for i in range(10) if i > 5]
gen = (i for i in range(10) if i < 3)


# --- Mixed comparisons with arithmetic ---

(1 + 2) == 3
(2 * 3) > 5
(10 - 3) < 8
(10 // 3) == 3
(10 % 3) != 0
(2 ** 3) >= 8


# --- Comparison method lookup with subclass ---

class LHS:
    def __eq__(self, other):
        return "lhs"
    def __hash__(self):
        return 0

class RHS:
    def __eq__(self, other):
        return "rhs"
    def __hash__(self):
        return 0

class RHSSub(LHS):
    def __eq__(self, other):
        return "rhssub"
    def __hash__(self):
        return 0

lhs = LHS()
rhs = RHS()
rhssub = RHSSub()
lhs == rhs
lhs == rhssub


# --- Identity vs equality ---

class NeverEqual:
    def __eq__(self, other):
        return False
    def __hash__(self):
        return id(self)

obj = NeverEqual()
obj is obj
obj == obj
