# RUN: parse
# Extracted from CPython 3.12 Lib/test/test_syntax.py — valid syntax boundary constructs only.


# --- Valid assignment targets ---

a = 1
a, b = 1, 2
(a, b) = (1, 2)
[a, b] = [1, 2]
a, *b = [1, 2, 3]
*a, b = [1, 2, 3]
a, *b, c = [1, 2, 3, 4]
[a, *b, c] = [1, 2, 3, 4]
(a, (b, c)) = (1, (2, 3))
a.b = 1 if True else 2
a[0] = 1
a[0:1] = [1]


# --- Valid chained assignment ---

a = b = 1
a = b = c = 42
x = y = z = []


# --- Valid augmented assignment ---

a = 0
a += 1
a -= 1
a *= 2
a //= 2
a **= 2
a %= 3
a &= 0xFF
a |= 0x01
a ^= 0x10
a >>= 1
a <<= 1
a /= 2.0
a @= [[1]]  # matrix multiply augmented assign


# --- Valid del targets ---

x = 1
del x

class _Obj:
    attr = 1

obj = _Obj()
del obj.attr

d = {0: 'a', 1: 'b'}
del d[0]

a, b = 1, 2
del a, b

lst = [1, 2, 3]
del lst[:]


# NOTE: Annotation targets (x: int, x: int = 5) not yet supported by parser
# x: int
# x: int = 5
# x: str = "hello"
#
# class Ann:
#     x: int
#     y: str = "abc"
#     z: list = []


# --- Valid function definitions: positional-only (PEP 570) ---

def f1(x, /): pass
def f2(x, /, y): pass
def f3(x, /, y, z): pass
def f4(x, y, /, z): pass
def f5(x=1, /): pass
def f6(x=1, /, y=2): pass
def f7(x, /, y=1, *, z=2): pass
def f8(x, /, y=1, *, z): pass


# --- Valid function definitions: keyword-only (PEP 3102) ---

def g1(*, x): pass
def g2(*, x, y): pass
def g3(*, x=1, y=2): pass
def g4(a, *, x): pass
def g5(a, b=1, *, x, y=2): pass


# --- Valid function definitions: mixed parameter kinds ---

def h1(a, b, /, c, *, d): pass
def h2(a, b=1, /, c=2, *, d=3): pass
def h3(a, /, b, *args, c, **kwargs): pass
def h4(a, b, /, *args, **kwargs): pass
def h5(*args, **kwargs): pass
def h6(a, *args): pass
def h7(**kwargs): pass


# --- Valid lambda with various params ---

f = lambda: None
f = lambda x: x
f = lambda x, y: x + y
f = lambda *args: args
f = lambda **kwargs: kwargs
f = lambda *args, **kwargs: (args, kwargs)
# NOTE: Positional-only params in lambda not yet supported by parser
# f = lambda x, /, y: (x, y)
# f = lambda x, /, y, *, z: (x, y, z)
# f = lambda x=1, /, y=2: (x, y)
# f = lambda x, y=1, *, z=2: (x, y, z)


# --- Valid for loop targets ---

for a in [1, 2, 3]: pass
for a, b in [(1, 2)]: pass
# NOTE: Parenthesized/bracket for-targets not yet supported by parser
# for (a, b) in [(1, 2)]: pass
# for [a, b] in [[1, 2]]: pass
# for *a, b in [[1, 2, 3]]: pass
# for a, *b in [[1, 2, 3]]: pass
# for (a, (b, c)) in [(1, (2, 3))]: pass

# Nested for loops
for i in range(3):
    for j in range(3):
        pass


# --- Valid with statement targets ---

class _CM:
    def __enter__(self): return self
    def __exit__(self, *args): pass

with _CM() as x: pass
# NOTE: Parenthesized with-targets and multiple with-items not yet supported
# with _CM() as (a, b): pass
# with _CM(), _CM(): pass
# with _CM() as x, _CM() as y: pass


# --- Valid walrus operator (PEP 572) ---

(x := 5)
if (n := 10) > 5: pass
while (line := "done") == "done": break
result = [(y := x + 1) for x in range(3)]
filtered = [y for x in range(10) if (y := x * 2) > 5]


# --- Valid yield/return in function context ---

def gen():
    yield 1
    yield from [2, 3]
    x = yield 4

def func():
    return
    return 42
    return (1, 2, 3)


# --- Valid break/continue in loop context ---

for i in range(10):
    if i == 5:
        break
    if i % 2 == 0:
        continue

while True:
    break


# NOTE: Soft keywords (match, case, type) as identifiers not yet supported
# match = 42; case = "hello"; type = int

# NOTE: match/case statements are covered by parse/match_stmt.py fixture

# NOTE: Type aliases (PEP 695) and type parameter syntax not yet supported
# type Vector = list[float]
# def first[T](lst: list[T]) -> T: ...
# class Container[T]: ...


# --- Valid line continuation ---

x = (1 +
     2 +
     3)

result = [1, 2,
          3, 4]

y = {'a': 1,
     'b': 2}

if (True and
    True):
    pass


# --- Implicit line continuation in brackets ---

data = [
    1,
    2,
    3,
]

params = dict(
    a=1,
    b=2,
    c=3,
)

tup = (
    "first",
    "second",
    "third",
)


# --- Valid star expressions ---

a, *b = [1, 2, 3]
# NOTE: Bracket/paren star unpack targets not yet supported
# [a, *b] = [1, 2, 3]
# (*a, b) = [1, 2, 3]

# Star in function calls
def f(*args, **kwargs): pass
f(*[1, 2], **{"a": 3})
f(*[1], *[2], **{"a": 3}, **{"b": 4})

# NOTE: PEP 646 star in subscript not yet supported
# idx[*t,]; idx[0, *t]


# --- Complex expressions that are valid ---

# Ternary in various contexts
a = 1 if True else 2
b = [1 if True else 2]
c = (1 if True else 2,)
d = {1 if True else 2: "val"}

# Nested ternary
e = 1 if True else (2 if False else 3)

# Walrus in comprehension
sums = [s for x in range(5) if (s := x + 1) > 2]

# Yield expression in generator
def gen_walrus():
    x = yield 1
    yield x


# --- Multiple statements on one line ---

# NOTE: Semicolons not yet supported
# a = 1; b = 2; c = 3
if True: pass
if True: a = 1
for i in [1]: pass
while False: pass


# --- Empty constructs ---

class Empty: pass

class EmptySlots:
    __slots__ = ()

def empty(): pass

async def async_empty(): pass

def empty_return():
    return


# --- Ellipsis as body ---

class EllipsisBody: ...

def ellipsis_func(): ...

async def async_ellipsis(): ...


# --- Nested functions and classes ---

def outer():
    x = 1
    def inner():
        nonlocal x
        x = 2
    inner()
    return x

class Outer:
    class Inner:
        pass
    def method(self):
        class LocalClass:
            pass
        return LocalClass()


# --- Global and nonlocal declarations ---

g = 0

def use_global():
    global g
    g = 42

def outer_nonlocal():
    x = 0
    def inner():
        nonlocal x
        x = 1
    inner()
    return x


# --- Assert statement ---

assert True
assert 1 == 1
assert True, "message"


# --- Raise statement variants ---

try:
    raise ValueError
except ValueError:
    pass

try:
    raise ValueError("msg")
except ValueError:
    pass

try:
    raise ValueError("msg") from None
except ValueError:
    pass

try:
    try:
        raise ValueError
    except ValueError:
        raise
except ValueError:
    pass
