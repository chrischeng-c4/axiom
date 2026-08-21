# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Type introspection builtins: isinstance with tuple-of-types, getattr with default
# Type introspection builtins conformance (S3-S5)
# isinstance, issubclass, getattr, setattr, delattr, hasattr

# S3: isinstance/issubclass chains
print(isinstance(True, int))
print(isinstance(True, (str, int)))
print(issubclass(bool, int))
print(issubclass(int, object))

# isinstance with various types
print(isinstance(42, int))
print(isinstance(3.14, float))
print(isinstance('hello', str))
print(isinstance([], list))
print(isinstance({}, dict))
print(isinstance(None, type(None)))

# issubclass chains
print(issubclass(bool, object))
print(issubclass(str, object))
print(issubclass(list, object))

# S4: getattr/setattr/delattr
class C:
    x = 10

obj = C()
print(getattr(obj, 'x'))
setattr(obj, 'y', 20)
print(obj.y)
delattr(obj, 'y')
print(hasattr(obj, 'y'))

# S5: getattr with default
print(getattr(object(), 'missing', 'default_val'))
print(getattr(C(), 'nonexistent', 42))
print(getattr(C(), 'x', 99))

# hasattr edge cases
print(hasattr(C(), 'x'))
print(hasattr(C(), 'nonexistent'))
print(hasattr([], 'append'))
print(hasattr([], 'nonexistent'))

# type() introspection
print(type(42).__name__)
print(type('hello').__name__)
print(type(True).__name__)
print(type(None).__name__)
print(type([]).__name__)
