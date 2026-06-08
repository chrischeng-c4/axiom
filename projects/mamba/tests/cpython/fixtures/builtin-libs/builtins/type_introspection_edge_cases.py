# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///

# Type introspection edge cases
print(isinstance(True, int))
print(isinstance(1, (str, float, int)))
print(issubclass(bool, int))
print(issubclass(int, object))

class C:
    x = 10

obj = C()
print(getattr(obj, 'x'))
setattr(obj, 'y', 20)
print(obj.y)
delattr(obj, 'y')
print(hasattr(obj, 'y'))
print(getattr(object(), 'missing', 'default_val'))
print(callable(len))
print(callable(42))
