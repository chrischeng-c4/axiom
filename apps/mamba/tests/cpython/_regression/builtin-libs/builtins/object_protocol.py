# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Builtins conformance: object protocol (R1.6).
# Tests isinstance, type, id, hash — avoids getattr/setattr/delattr (codegen crash).

# isinstance checks
print(isinstance(42, int))
print(isinstance("hello", str))
print(isinstance(3.14, float))
print(isinstance(True, bool))
print(isinstance([], list))
print(isinstance({}, dict))

# type() returns the type
print(type(42).__name__)
print(type("hello").__name__)
print(type(3.14).__name__)
print(type(True).__name__)
print(type([]).__name__)
print(type({}).__name__)

# id returns unique integer
x = [1, 2, 3]
y = x
z = [1, 2, 3]
print(id(x) == id(y))
print(id(x) == id(z))

# hash works for immutable types
print(isinstance(hash(42), int))
print(isinstance(hash("hello"), int))
print(hash(42) == hash(42))

# repr and str
print(repr(42))
print(repr("hello"))
print(str(42))
print(str(3.14))
