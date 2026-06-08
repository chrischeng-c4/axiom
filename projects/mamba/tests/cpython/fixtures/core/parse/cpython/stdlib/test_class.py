# RUN: parse
# Extracted from CPython 3.12 Lib/test/test_class.py — class syntax constructs only.


# --- Operator method names ---

testmeths = [
    # Binary operations
    "add", "radd", "sub", "rsub", "mul", "rmul",
    "matmul", "rmatmul", "truediv", "rtruediv",
    "floordiv", "rfloordiv", "mod", "rmod",
    "divmod", "rdivmod", "pow", "rpow",
    "rshift", "rrshift", "lshift", "rlshift",
    "and", "rand", "or", "ror", "xor", "rxor",
    # List/dict operations
    "contains", "getitem", "setitem", "delitem",
    # Unary operations
    "neg", "pos", "abs",
    # Generic operations
    "init",
]


# --- Decorator function ---

callLst = []

def trackCall(f):
    def track(*args, **kwargs):
        callLst.append((f.__name__, args))
        return f(*args, **kwargs)
    return track


# --- Class with all operator methods via decorator ---

class AllTests:
    @trackCall
    def __hash__(self, *args):
        return hash(id(self))

    @trackCall
    def __str__(self, *args):
        return "AllTests"

    @trackCall
    def __repr__(self, *args):
        return "AllTests"

    @trackCall
    def __int__(self, *args):
        return 1

    @trackCall
    def __index__(self, *args):
        return 1

    @trackCall
    def __float__(self, *args):
        return 1.0

    @trackCall
    def __eq__(self, *args):
        return True

    @trackCall
    def __ne__(self, *args):
        return False

    @trackCall
    def __lt__(self, *args):
        return False

    @trackCall
    def __le__(self, *args):
        return True

    @trackCall
    def __gt__(self, *args):
        return False

    @trackCall
    def __ge__(self, *args):
        return True

    @trackCall
    def __add__(self, *args):
        pass

    @trackCall
    def __radd__(self, *args):
        pass

    @trackCall
    def __sub__(self, *args):
        pass

    @trackCall
    def __rsub__(self, *args):
        pass

    @trackCall
    def __mul__(self, *args):
        pass

    @trackCall
    def __rmul__(self, *args):
        pass

    @trackCall
    def __matmul__(self, *args):
        pass

    @trackCall
    def __rmatmul__(self, *args):
        pass

    @trackCall
    def __truediv__(self, *args):
        pass

    @trackCall
    def __rtruediv__(self, *args):
        pass

    @trackCall
    def __floordiv__(self, *args):
        pass

    @trackCall
    def __rfloordiv__(self, *args):
        pass

    @trackCall
    def __mod__(self, *args):
        pass

    @trackCall
    def __rmod__(self, *args):
        pass

    @trackCall
    def __pow__(self, *args):
        pass

    @trackCall
    def __rpow__(self, *args):
        pass

    @trackCall
    def __rshift__(self, *args):
        pass

    @trackCall
    def __rrshift__(self, *args):
        pass

    @trackCall
    def __lshift__(self, *args):
        pass

    @trackCall
    def __rlshift__(self, *args):
        pass

    @trackCall
    def __and__(self, *args):
        pass

    @trackCall
    def __rand__(self, *args):
        pass

    @trackCall
    def __or__(self, *args):
        pass

    @trackCall
    def __ror__(self, *args):
        pass

    @trackCall
    def __xor__(self, *args):
        pass

    @trackCall
    def __rxor__(self, *args):
        pass

    @trackCall
    def __contains__(self, *args):
        pass

    @trackCall
    def __getitem__(self, *args):
        pass

    @trackCall
    def __setitem__(self, *args):
        pass

    @trackCall
    def __delitem__(self, *args):
        pass

    @trackCall
    def __neg__(self, *args):
        pass

    @trackCall
    def __pos__(self, *args):
        pass

    @trackCall
    def __abs__(self, *args):
        pass

    @trackCall
    def __init__(self, *args):
        pass


# --- Binary operator usage ---

testme = AllTests()
testme + 1
1 + testme
testme - 1
1 - testme
testme * 1
1 * testme
testme @ 1
1 @ testme
testme / 1
1 / testme
testme // 1
1 // testme
testme % 1
1 % testme
divmod(testme, 1)
divmod(1, testme)
testme ** 1
1 ** testme
testme >> 1
1 >> testme
testme << 1
1 << testme
testme & 1
1 & testme
testme | 1
1 | testme
testme ^ 1
1 ^ testme


# --- List/dict operations ---

class EmptyClass:
    pass

try:
    1 in EmptyClass()
except TypeError:
    pass

1 in testme
testme[1]
testme[1] = 1
del testme[1]


# --- Slice operations ---

testme[:42]
testme[:42] = "The Answer"
del testme[:42]
testme[2:1024:10]
testme[2:1024:10] = "A lot"
del testme[2:1024:10]
# NOTE: Multi-dimensional Ellipsis slices not yet supported by parser
# testme[:42, ..., :24:, 24, 100]
# testme[:42, ..., :24:, 24, 100] = "Strange"
# del testme[:42, ..., :24:, 24, 100]


# --- Unary operators ---

-testme
+testme
abs(testme)
int(testme)
float(testme)


# --- Hash, repr, str ---

hash(testme)
repr(testme)
str(testme)


# --- Comparison operators ---

testme == 1
testme < 1
testme > 1
testme != 1
1 == testme
1 < testme
1 > testme
1 != testme


# --- Subclass with __getattr__, __setattr__, __delattr__ ---

class ExtraTests(AllTests):
    @trackCall
    def __getattr__(self, *args):
        return "SomeVal"

    @trackCall
    def __setattr__(self, *args):
        pass

    @trackCall
    def __delattr__(self, *args):
        pass

extra = ExtraTests()
extra.spam
extra.eggs = "spam, spam, spam and ham"
del extra.cardinal


# --- __del__ ---

x = []

class DelTest:
    def __del__(self):
        x.append("crab people, crab people")

dt = DelTest()
del dt


# --- Bad return type methods ---

class BadTypeClass:
    def __int__(self):
        return None
    __float__ = __int__
    __complex__ = __int__
    __str__ = __int__
    __repr__ = __int__
    __bytes__ = __int__
    __bool__ = __int__
    __index__ = __int__


# --- Hash without __hash__ ---

class C0:
    pass

hash(C0())

class C2:
    def __eq__(self, other):
        return 1


# --- Recursive __call__ ---

class A:
    pass

A.__call__ = A()
a = A()

try:
    a()
except RecursionError:
    pass


# --- Property descriptor ---

def booh(self):
    raise AttributeError("booh")

class PropA:
    a = property(booh)

try:
    PropA().a
except AttributeError as x:
    pass

class PropE:
    __eq__ = property(booh)

try:
    PropE() == PropE()
except AttributeError:
    pass


# --- Method comparison ---

class MethodCmp:
    def __init__(self, x):
        self.x = x
    def f(self):
        pass
    def g(self):
        pass
    def __eq__(self, other):
        return True
    def __hash__(self):
        raise TypeError

class MethodCmpChild(MethodCmp):
    pass


# --- __slots__ ---

class SlotB:
    y = 0
    __slots__ = ('z',)


# --- Constructor patterns ---

class PlainC:
    pass

class InitNewD:
    def __new__(cls, *args, **kwargs):
        super().__new__(cls, *args, **kwargs)
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

class InitOnlyE:
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)


# --- Metaclass with extended call syntax ---

class Meta(int):
    def __init__(*args, **kwargs):
        pass

    def __new__(cls, name, bases, attrs, **kwargs):
        return bases, kwargs

d = {'metaclass': Meta}

# NOTE: **kwargs in class bases not yet supported by parser
# class MA(**d):
#     pass
#
# class MB(0, 1, 2, 3, 4, 5, 6, 7, **d):
#     pass
#
# class MC(0, *range(1, 8), **d, foo='bar'):
#     pass


# --- __init__ as property descriptor ---

class PropInit:
    __init__ = property(booh)

try:
    PropInit()
except AttributeError:
    pass


# --- Dynamic attribute class ---

class DynAttr:
    def __init__(self):
        self.attr = 1


# --- Dynamic type manipulation ---

def add(self, other):
    return 'summa'

name = str(b'__add__', 'ascii')
type.__setattr__(DynAttr, name, add)
type.__delattr__(DynAttr, name)
