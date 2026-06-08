# RUN: parse
# Extracted from CPython 3.12 Lib/test/test_augassign.py — syntax constructs only.


# --- Augmented assignment on simple names ---

x = 0
x += 10
x -= 3
x *= 4
x /= 2
x //= 3
x %= 7
x **= 2
x &= 0xFF
x |= 0x10
x ^= 0x01
x <<= 2
x >>= 1


# --- Augmented assignment with float ---

f = 1.0
f += 0.5
f -= 0.1
f *= 3.0
f /= 2.0
f //= 1.5
f %= 0.3
f **= 2.0


# --- Augmented assignment with string (concatenation) ---

s = "hello"
s += " world"
s *= 3


# --- Augmented assignment with lists ---

lst = [1, 2, 3]
lst += [4, 5]
lst *= 2


# --- Augmented assignment on attributes ---

class Obj:
    def __init__(self):
        self.x = 0
        self.y = 1.0
        self.name = "obj"
        self.items = [1, 2]

o = Obj()
o.x += 10
o.x -= 3
o.x *= 4
o.x /= 2
o.x //= 3
o.x %= 7
o.x **= 2
o.x &= 0xFF
o.x |= 0x10
o.x ^= 0x01
o.x <<= 2
o.x >>= 1
o.y += 0.5
o.name += "_suffix"
o.items += [3, 4]


# --- Augmented assignment on subscripts ---

data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
data[0] += 100
data[1] -= 1
data[2] *= 10
data[3] /= 2
data[4] //= 3
data[5] %= 4
data[6] **= 2
data[7] &= 0x0F
data[8] |= 0xF0
data[9] ^= 0xFF

d = {"a": 1, "b": 2, "c": 3}
d["a"] += 10
d["b"] -= 1
d["c"] *= 5


# --- Augmented assignment on nested subscripts ---

nested = [[1, 2], [3, 4]]
nested[0][1] += 10
nested[1][0] -= 1


# --- Augmented assignment on nested attributes ---

class Inner:
    def __init__(self):
        self.val = 0

class Outer:
    def __init__(self):
        self.inner = Inner()

obj = Outer()
obj.inner.val += 42
obj.inner.val -= 10
obj.inner.val *= 2


# --- Augmented assignment in function body ---

def update_locals():
    a = 0
    a += 1
    a -= 1
    a *= 2
    a /= 2
    a //= 1
    a %= 3
    a **= 2
    a &= 0xFF
    a |= 0x01
    a ^= 0x10
    a <<= 1
    a >>= 1
    return a


# --- Augmented assignment in class body ---

class Counter:
    count = 0

    @classmethod
    def increment(cls):
        cls.count += 1

    @classmethod
    def decrement(cls):
        cls.count -= 1

    @classmethod
    def multiply(cls, factor):
        cls.count *= factor

    @classmethod
    def reset(cls):
        cls.count &= 0


# --- Augmented assignment with global ---

total = 0

def accumulate(val):
    global total
    total += val

def scale(factor):
    global total
    total *= factor


# --- Augmented assignment with nonlocal ---

def make_counter():
    count = 0
    def increment():
        nonlocal count
        count += 1
    def decrement():
        nonlocal count
        count -= 1
    return increment, decrement


# --- Augmented assignment in loops ---

def sum_range(n):
    result = 0
    for i in range(n):
        result += i
    return result

def factorial(n):
    result = 1
    for i in range(1, n + 1):
        result *= i
    return result

def bitwise_accumulate(values):
    result = 0
    for v in values:
        result |= v
    return result


# --- Augmented assignment with conditional ---

def clamp_add(x, delta, maximum):
    x += delta
    if x > maximum:
        x -= (x - maximum)
    return x


# --- Augmented assignment with complex expressions ---

x = 10
x += 1 + 2 * 3
x -= (4 - 1)
x *= 2 ** 3
x //= 1 + 1
x %= 3 + 4
x **= 1 + 1

lst = [1, 2, 3]
lst += [i * 2 for i in range(3)]

s = "base"
s += "_" + "suffix"


# --- Augmented assignment with walrus ---

x = 0
x += (y := 5)


# --- Dunder method classes for augmented assign ---

class AugTest:
    def __init__(self, val):
        self.val = val

    def __iadd__(self, other):
        self.val += other
        return self

    def __isub__(self, other):
        self.val -= other
        return self

    def __imul__(self, other):
        self.val *= other
        return self

    def __itruediv__(self, other):
        self.val /= other
        return self

    def __ifloordiv__(self, other):
        self.val //= other
        return self

    def __imod__(self, other):
        self.val %= other
        return self

    def __ipow__(self, other):
        self.val **= other
        return self

    def __iand__(self, other):
        self.val &= other
        return self

    def __ior__(self, other):
        self.val |= other
        return self

    def __ixor__(self, other):
        self.val ^= other
        return self

    def __ilshift__(self, other):
        self.val <<= other
        return self

    def __irshift__(self, other):
        self.val >>= other
        return self

t = AugTest(100)
t += 10
t -= 5
t *= 2
t /= 3
t //= 2
t %= 7
t **= 2
t &= 0xFF
t |= 0x01
t ^= 0x10
t <<= 1
t >>= 1
