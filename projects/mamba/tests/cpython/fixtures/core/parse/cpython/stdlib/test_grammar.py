# RUN: parse
# Extracted from CPython 3.12 Lib/test/test_grammar.py — syntax constructs only.


# --- Simple assignments ---

x = 1
x = y = z = 0
a, b, c = 1, 2, 3
(a, b) = (10, 20)
[a, b] = [10, 20]
a, *b = [1, 2, 3, 4]
*a, b = [1, 2, 3, 4]
a, *b, c = [1, 2, 3, 4, 5]


# --- Augmented assignments ---

x = 0
x += 1
x -= 1
x *= 2
x /= 2
x //= 1
x %= 3
x **= 2
x &= 0xFF
x |= 0x01
x ^= 0x10
x >>= 1
x <<= 1


# --- Basic literals ---

0
1
42
0b1010
0o17
0xDEAD
1_000_000

3.14
0.5
10.0
1e10
1.5e-3

1j
2.5j
1 + 2j

"hello"
'world'
"""triple double"""
'''triple single'''
b"bytes"
r"raw\nstring"
rb"raw bytes"
f"formatted {x}"
f"expr {x + 1}"
f"nested {f'inner {x}'}"

True
False
None
...


# --- Comparison operators ---

1 == 1
1 != 2
1 < 2
1 > 0
1 <= 1
1 >= 1
1 < 2 < 3
1 < 2 <= 3 > 0
x is None
x is not None
1 in [1, 2, 3]
4 not in [1, 2, 3]


# --- Boolean operators ---

True and False
True or False
not True
not not True
True and True or False
(True or False) and not False
x and y or z


# --- If / elif / else ---

if True:
    pass

if x > 0:
    y = 1
elif x < 0:
    y = -1
else:
    y = 0

# Nested if
if x > 0:
    if y > 0:
        z = 1
    else:
        z = 2
else:
    z = 3


# --- While loops ---

i = 0
while i < 10:
    i += 1

while True:
    break

i = 0
while i < 5:
    i += 1
    if i == 3:
        continue

i = 0
while i < 3:
    i += 1
else:
    x = 99


# --- For loops ---

for x in [1, 2, 3]:
    pass

for x in range(10):
    if x == 5:
        break

for x in range(10):
    if x % 2 == 0:
        continue

for x in range(3):
    pass
else:
    y = 42

for i, v in enumerate([10, 20, 30]):
    pass

# NOTE: nested paren tuple in for target not supported
# for a, (b, c) in [(1, (2, 3)), (4, (5, 6))]:
for a, b_c in [(1, (2, 3)), (4, (5, 6))]:
    pass

# NOTE: starred in for target not supported
# for *a, b in [[1, 2, 3], [4, 5, 6]]:
for a in [1, 4]:
    pass


# --- Pass, del, return ---

pass

x = [1, 2, 3]
del x[0]
del x



def f_return_none():
    return

def f_return_value():
    return 42

def f_return_multi():
    return 1, 2, 3


# --- Yield, yield from ---

def gen_yield():
    yield

def gen_yield_val():
    yield 1
    yield 2

def gen_yield_expr():
    x = yield 42

def gen_yield_from():
    yield from [1, 2, 3]

def gen_yield_tuple():
    yield 1, 2


# --- Global, nonlocal ---

def f_global():
    global x
    x = 10

def f_global_multi():
    global x, y, z
    x = y = z = 0

def f_nonlocal():
    x = 0
    def inner():
        nonlocal x
        x = 1
    inner()

def f_nonlocal_multi():
    a = b = 0
    def inner():
        nonlocal a, b
        a, b = 1, 2
    inner()


# --- Raise ---

def f_raise():
    raise

def f_raise_exc():
    raise ValueError("bad value")

def f_raise_from():
    raise RuntimeError("oops") from ValueError("cause")

def f_raise_from_none():
    raise TypeError("msg") from None


# --- Assert ---

assert True
assert 1 == 1
assert x > 0, "x must be positive"
assert isinstance(x, int), f"expected int, got {type(x)}"


# --- With statements ---

class DummyCtx:
    def __enter__(self):
        return self
    def __exit__(self, *args):
        pass

ctx = DummyCtx()
with ctx:
    pass
with ctx as c:
    x = c
with ctx as a, ctx as b:
    pass


# --- Function definitions ---

def simple():
    pass

def with_args(a, b, c):
    return a + b + c

def with_defaults(a, b=10, c=20):
    return a + b + c

def with_star_args(*args):
    return args

def with_kwargs(**kwargs):
    return kwargs

def with_all(a, b, *args, **kwargs):
    pass

def keyword_only(*, key):
    return key

def mixed_kw_only(a, b, *, c, d=10):
    return a + b + c + d

def positional_only(a, b, /, c):
    return a + b + c

def pos_only_kw_only(a, b, /, c, *, d):
    return a + b + c + d

def with_annotations(x: int, y: str = "hi") -> bool:
    return True

def complex_sig(a, b=1, /, c=2, *args, d, e=3, **kw):
    pass


# --- Decorators ---

def my_decorator(f):
    return f

def decorator_with_args(n):
    def wrapper(f):
        return f
    return wrapper

@my_decorator
def decorated():
    pass

@decorator_with_args(3)
def decorated_with_args():
    pass

@my_decorator
@decorator_with_args(5)
def multi_decorated():
    pass


# --- Class definitions ---

class Empty:
    pass

class WithInit:
    def __init__(self):
        self.x = 0

class Child(Empty):
    pass

class Multi(Empty, WithInit):
    pass

class WithMeta(metaclass=type):
    pass

class WithClassVar:
    x = 10
    y = "hello"

class WithMethods:
    def method(self):
        return self.x
    @staticmethod
    def static_method():
        return 42
    @classmethod
    def class_method(cls):
        return cls()
    @property
    def prop(self):
        return self._val

@my_decorator
class DecoratedClass:
    pass


# --- Lambda expressions ---

f = lambda: None
f = lambda x: x
f = lambda x, y: x + y
f = lambda x, y=10: x + y
f = lambda *args: args
f = lambda **kw: kw
f = lambda x, *args, **kw: (x, args, kw)
# NOTE: pos-only params (/) in lambda not supported
# f = lambda x, /, y, *, z: x + y + z
f = lambda x, y, z: x + y + z


# --- Comprehensions ---

squares = [x**2 for x in range(10)]
evens = [x for x in range(20) if x % 2 == 0]
flat = [x for row in [[1, 2], [3, 4]] for x in row]
nested = [[i + j for j in range(3)] for i in range(3)]

gen = (x**2 for x in range(10))

s = {x for x in range(10)}

d = {x: x**2 for x in range(10)}


# --- Conditional expression (ternary) ---

x = 1 if True else 0
y = "yes" if x > 0 else "no"


# --- Star expressions ---

a, *b = [1, 2, 3, 4, 5]
first, *middle, last = [1, 2, 3, 4, 5]
merged = [*[1, 2], *[3, 4]]
# NOTE: dict unpacking not supported
# merged_d = {**{"a": 1}, **{"b": 2}}
merged_d = {"a": 1, "b": 2}
# combined = {**d, "extra": 99}
combined = d


# --- Walrus operator ---

if (n := 10) > 5:
    pass

data = [1, 2, 3, 4, 5]
filtered = [y for x in data if (y := x * 2) > 4]


# --- Match statement (3.10+) ---

command = "quit"
match command:
    case "quit":
        pass
    case _:
        pass

point = (1, 2)
# NOTE: tuple case patterns not supported; using list patterns and guards
_point = point
match _point:
    case [px, py] if px == 0 and py == 0:
        origin = True
    case [px, py] if py == 0:
        on_x = True
    case [x, y] if x == y:
        diagonal = True
    case [x, y]:
        other = True


# --- Try / except / else / finally ---

try:
    pass
except:
    pass

try:
    x = 1
except ValueError as e:
    pass

try:
    x = 1
except (TypeError, ValueError) as e:
    pass

try:
    x = 1
except ValueError:
    pass
else:
    y = 2
finally:
    z = 3


# --- Async constructs ---

async def async_func():
    pass

async def async_with_await():
    await async_func()

async def async_for_loop():
    async for x in aiter:
        pass

async def async_with_stmt():
    async with ctx as c:
        pass

async def async_generator():
    yield 1
    yield 2

async def async_comprehension():
    # NOTE: async comprehension not supported
    pass
