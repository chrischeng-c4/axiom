# RUN: parse
# CPython 3.12 test_builtins: basic builtin coverage

# abs
x = abs(-5)
x = abs(3.14)

# bool
b = bool(0)
b = bool("")
b = bool(None)
b = bool(1)

# chr / ord
c = chr(65)
n = ord(chr(65))

# divmod
q, r = divmod(10, 3)

# enumerate
for i, v in enumerate([1, 2, 3]):
    pass

for i, v in enumerate([1, 2, 3], start=1):
    pass

# filter
evens = list(filter(lambda x: x % 2 == 0, range(10)))

# hasattr / getattr / setattr / delattr
class Obj:
    x = 1

obj = Obj()
has = hasattr(obj, "x")
val = getattr(obj, "x")
val = getattr(obj, "y", None)
setattr(obj, "x", 2)

# isinstance / issubclass
b = isinstance(1, int)
b = isinstance(1, (int, float))
b = issubclass(bool, int)

# iter / next
it = iter([1, 2, 3])
v = next(it)
v = next(it, None)

# len
n = len([1, 2, 3])
n = len("hello")

# map
doubled = list(map(lambda x: x * 2, range(5)))
combined = list(map(lambda x, y: x + y, [1, 2], [3, 4]))

# max / min / sum
mx = max(1, 2, 3)
mx = max([1, 2, 3])
mx = max([1, 2, 3], key=lambda x: -x)
mn = min(1, 2, 3)
s = sum([1, 2, 3])
s = sum([1, 2, 3], 10)

# print
print("hello")
print("a", "b", "c", sep=", ")
print("no newline", end="")

# range
for i in range(5):
    pass
for i in range(1, 10):
    pass
for i in range(0, 10, 2):
    pass

# repr / str / format
s = repr(42)
s = str(42)
s = format(3.14, ".2f")

# reversed / sorted / zip
lst = list(reversed([1, 2, 3]))
lst = sorted([3, 1, 2])
lst = sorted([3, 1, 2], reverse=True)
lst = sorted(["b", "a"], key=lambda x: x.lower())
pairs = list(zip([1, 2], ["a", "b"]))

# type / id
t = type(42)
n = id(42)

# round
r = round(3.14)
r = round(3.14159, 2)

# pow
p = pow(2, 10)
p = pow(2, 10, 1000)

# any / all
b = any([True, False, True])
b = all([True, True, True])

# vars / dir
d = vars()
attrs = dir(42)

# hex / oct / bin
s = hex(255)
s = oct(8)
s = bin(10)

# bytes / bytearray / memoryview
b = bytes(b"hello")
b = bytes([72, 101, 108, 108, 111])
ba = bytearray(b"hello")
ba = bytearray(5)

# complex
c = complex(1, 2)
c = complex("1+2j")

# frozenset
fs = frozenset([1, 2, 3])
fs = frozenset()

# slice
sl = slice(1, 5, 2)
