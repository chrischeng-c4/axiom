# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# decorator patterns broad

# basic decorator
def track(fn):
    def wrapper(*args, **kw):
        return fn(*args, **kw)
    return wrapper

@track
def add(a, b):
    return a + b

print(add(1, 2))
print(add(10, 20))

# decorator with transformation
def double_result(fn):
    def wrapper(*args):
        return fn(*args) * 2
    return wrapper

@double_result
def square(x):
    return x * x

print(square(3))
print(square(5))

# stacked decorators
def add_one(fn):
    def wrapper(*args):
        return fn(*args) + 1
    return wrapper

def mul_two(fn):
    def wrapper(*args):
        return fn(*args) * 2
    return wrapper

@add_one
@mul_two
def base(x):
    return x

print(base(5))
print(base(10))

@mul_two
@add_one
def other(x):
    return x

print(other(5))
print(other(10))

# decorator returning a result
def log_call(fn):
    def wrapper(*args):
        result = fn(*args)
        return result
    return wrapper

@log_call
def greet(name):
    return "Hello, " + name

print(greet("Alice"))
print(greet("Bob"))

# decorator with arg (factory)
def times(n):
    def deco(fn):
        def wrapper(*args):
            return fn(*args) * n
        return wrapper
    return deco

@times(3)
def identity(x):
    return x

print(identity(5))
print(identity(10))

# classmethod / staticmethod
class Holder:
    val = 42
    @classmethod
    def from_cls(cls):
        return cls.val
    @staticmethod
    def doubled(x):
        return x * 2

print(Holder.from_cls())
print(Holder.doubled(50))

h = Holder()
print(h.from_cls())
print(h.doubled(7))

# property decorator
class Temp:
    def __init__(self, c):
        self._c = c
    @property
    def c(self):
        return self._c
    @property
    def f(self):
        return self._c * 9 // 5 + 32

t = Temp(25)
print(t.c)
print(t.f)
t2 = Temp(0)
print(t2.c)
print(t2.f)
