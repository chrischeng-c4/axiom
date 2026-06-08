# RUN: parse
# Extracted from CPython 3.12 Lib/test/test_call.py — syntax constructs only.


# --- Simple function calls ---

def noop():
    pass

def identity(x):
    return x

def add(a, b):
    return a + b

noop()
identity(42)
add(1, 2)


# --- Positional arguments ---

def pos3(a, b, c):
    return (a, b, c)

pos3(1, 2, 3)
pos3(1, 2, 3)
pos3("x", "y", "z")
pos3([], {}, ())


# --- Keyword arguments ---

def kw_func(name, value):
    return (name, value)

kw_func(name="x", value=42)
kw_func(value=42, name="x")
kw_func("x", value=42)


# --- Default arguments ---

def with_defaults(a, b=10, c=20):
    return a + b + c

with_defaults(1)
with_defaults(1, 2)
with_defaults(1, 2, 3)
with_defaults(1, c=30)
with_defaults(1, b=5, c=15)


# --- *args unpacking ---

def var_pos(*args):
    return args

var_pos()
var_pos(1)
var_pos(1, 2, 3)
var_pos(1, "two", 3.0, None)

items = [1, 2, 3]
var_pos(*items)
var_pos(*[10, 20])
var_pos(0, *items)
var_pos(*items, 4)
var_pos(*items, *[4, 5])


# --- **kwargs unpacking ---

def var_kw(**kwargs):
    return kwargs

var_kw()
var_kw(a=1)
var_kw(a=1, b=2, c=3)

opts = {"x": 10, "y": 20}
var_kw(**opts)
var_kw(**{"a": 1})
var_kw(z=30, **opts)
var_kw(**opts, z=30)
var_kw(**opts, **{"z": 30})


# --- Mixed positional, keyword, *args, **kwargs ---

def mixed(a, b, *args, **kwargs):
    return (a, b, args, kwargs)

mixed(1, 2)
mixed(1, 2, 3, 4)
mixed(1, 2, key="val")
mixed(1, 2, 3, 4, key="val")
mixed(*[1, 2, 3], **{"key": "val"})


def full_sig(a, b, /, c, *args, d, e=5, **kw):
    pass

full_sig(1, 2, 3, d=4)
full_sig(1, 2, 3, 4, 5, d=6, e=7)
full_sig(1, 2, c=3, d=4, extra=99)
full_sig(1, 2, 3, *[4, 5], d=6, **{"e": 7})


# --- Keyword-only arguments ---

def kw_only(*, key):
    return key

def kw_only_default(*, key=42):
    return key

def kw_only_multi(*, a, b, c=10):
    return (a, b, c)

kw_only(key=1)
kw_only_default()
kw_only_default(key=99)
kw_only_multi(a=1, b=2)
kw_only_multi(a=1, b=2, c=3)


# --- Positional-only arguments ---

def pos_only(a, b, /):
    return a + b

def pos_and_kw(a, b, /, c, d):
    return (a, b, c, d)

pos_only(1, 2)
pos_and_kw(1, 2, 3, 4)
pos_and_kw(1, 2, c=3, d=4)


# --- Nested calls ---

def double(x):
    return x * 2

def square(x):
    return x ** 2

double(square(3))
square(double(5))
double(double(double(1)))
add(double(1), square(2))
identity(identity(identity(42)))


# --- Method calls ---

class Calc:
    def __init__(self, val=0):
        self.val = val

    def add(self, n):
        return Calc(self.val + n)

    def sub(self, n):
        return Calc(self.val - n)

    def mul(self, n):
        return Calc(self.val * n)

    def get(self):
        return self.val

c = Calc(10)
c.add(5)
c.sub(3)
c.mul(2)
c.get()
c.add(5).sub(3).mul(2).get()


# --- Method calls on built-in types ---

"hello world".split()
"hello world".split(" ")
"hello".upper()
"HELLO".lower()
"hello world".replace("world", "python")
" hello ".strip()
",".join(["a", "b", "c"])

[3, 1, 2].sort()
[1, 2, 3].append(4)
[1, 2, 3].extend([4, 5])
[1, 2, 3, 2].count(2)
[1, 2, 3].index(2)

{"a": 1}.get("a")
{"a": 1}.get("b", 0)
{"a": 1, "b": 2}.keys()
{"a": 1, "b": 2}.values()
{"a": 1, "b": 2}.items()
{"a": 1}.update({"b": 2})
{"a": 1}.pop("a")


# --- Chained calls ---

def make_adder(n):
    def adder(x):
        return x + n
    return adder

make_adder(5)(10)
make_adder(1)(make_adder(2)(3))

def returns_callable():
    def inner():
        def innermost():
            return 42
        return innermost
    return inner

returns_callable()()()


# --- Lambda calls ---

(lambda: 42)()
(lambda x: x * 2)(21)
(lambda x, y: x + y)(10, 20)
(lambda *args: sum(args))(1, 2, 3, 4)
(lambda **kw: kw)(**{"a": 1, "b": 2})
(lambda x, y=10: x + y)(5)
# NOTE: pos-only params (/) not supported in lambda
# (lambda x, /, y, *, z: x + y + z)(1, 2, z=3)
(lambda x, y, z: x + y + z)(1, 2, 3)


# --- Calls with complex argument expressions ---

add(1 + 2, 3 * 4)
identity([x ** 2 for x in range(5)])
identity({k: v for k, v in [("a", 1)]})
identity((x for x in range(3)))
identity(lambda x: x + 1)
add(*[i for i in [1, 2]])
var_kw(**{k: v for k, v in [("a", 1), ("b", 2)]})


# --- Calls with starred expressions ---

def f5(a, b, c, d, e):
    pass

args1 = (1, 2)
args2 = (3, 4, 5)
f5(*args1, *args2)

first = {"a": 1}
second = {"b": 2}
var_kw(**first, **second)


# --- Built-in function calls ---

len([1, 2, 3])
range(10)
range(1, 10)
range(1, 10, 2)
sorted([3, 1, 2])
sorted([3, 1, 2], reverse=True)
sorted([3, 1, 2], key=lambda x: -x)
list(range(5))
tuple(range(5))
set(range(5))
dict(a=1, b=2)
dict([("a", 1), ("b", 2)])
int("42")
float("3.14")
str(42)
bool(1)
repr([1, 2, 3])
type(42)
isinstance(42, int)
issubclass(bool, int)
hasattr([], "append")
getattr([], "append")
setattr(Calc, "x", 10)
callable(lambda: None)
map(str, [1, 2, 3])
filter(None, [0, 1, 2])
zip([1, 2], [3, 4])
enumerate([10, 20, 30])
enumerate([10, 20, 30], start=1)
min(1, 2, 3)
max(1, 2, 3)
sum([1, 2, 3])
sum([1, 2, 3], 10)
abs(-42)
round(3.14159, 2)
pow(2, 10)
pow(2, 10, 1000)
divmod(17, 5)
hash("hello")
id(42)
print("hello")
print("a", "b", sep=", ", end="\n")
print(*[1, 2, 3], sep=" ")


# --- Calls in different contexts ---

# In assignment
result = add(1, 2)

# In condition
if identity(True):
    pass

# In loop
for x in sorted([3, 1, 2]):
    pass

# In comprehension
result = [double(x) for x in range(5)]

# In return
def wrapper():
    return add(1, 2)

# In assert
assert identity(True)
assert add(1, 2) == 3, f"Expected 3, got {add(1, 2)}"

# In ternary
val = add(1, 2) if True else add(3, 4)

# In f-string
msg = f"result is {add(1, 2)}"

# As default argument
def with_call_default(x=list()):
    pass

# As decorator argument
def decorator(n):
    def wrapper(f):
        return f
    return wrapper

@decorator(add(1, 2))
def decorated():
    pass
