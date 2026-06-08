# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# chr/ord/id/hash/isinstance/type broad

# ord
print(ord("a"))
print(ord("A"))
print(ord("z"))
print(ord("Z"))
print(ord("0"))
print(ord("9"))
print(ord(" "))
print(ord("!"))
print(ord("~"))

# chr
print(chr(65))
print(chr(90))
print(chr(97))
print(chr(122))
print(chr(48))
print(chr(57))
print(chr(32))
print(chr(33))

# chr(ord(c)) round trip
for c in "Hello!":
    print(chr(ord(c)))

# ord(chr(i)) round trip
for i in [65, 66, 67, 97, 98, 99]:
    print(ord(chr(i)))

# hash consistency
print(hash(42) == hash(42))
print(hash("hello") == hash("hello"))
print(hash((1, 2)) == hash((1, 2)))
print(hash(1.5) == hash(1.5))

# hash of different values
a = hash("abc")
b = hash("abd")
print(a != b)

# type
print(type(42).__name__)
print(type(3.14).__name__)
print(type("x").__name__)
print(type([]).__name__)
print(type({}).__name__)
print(type(()).__name__)
print(type(None).__name__)
print(type(True).__name__)
print(type(set()).__name__)

# type on class instance
class Foo:
    pass

f = Foo()
print(type(f).__name__)

# isinstance int
print(isinstance(42, int))
print(isinstance(42, float))
print(isinstance(3.14, float))
print(isinstance(3.14, int))
print(isinstance("x", str))
print(isinstance("x", int))

# isinstance with tuple of types
print(isinstance(42, (int, float)))
print(isinstance(3.14, (int, float)))
print(isinstance("x", (int, float)))
print(isinstance("x", (int, float, str)))

# callable
def fn():
    pass

print(callable(fn))
print(callable(42))
print(callable("x"))
print(callable(print))
